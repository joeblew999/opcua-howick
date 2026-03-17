//! OPC UA client agent — connects to the Pi 5 OPC UA server as an M2M client.
//!
//! This is used by howick-agent (Pi Zero) when `plat_trunk.url` is an OPC UA
//! endpoint (starts with `opc.tcp://`).
//!
//! ## Industry pattern: subscriptions, not polling
//!
//! Instead of polling an HTTP endpoint every N seconds, howick-agent:
//!   1. Connects to the Pi 5 OPC UA server
//!   2. **Subscribes** to `Jobs/PendingJobId`
//!   3. The Pi 5 **pushes** a notification the instant a new job is queued
//!   4. Agent reads `Jobs/PendingJobCsv` via OPC UA Read
//!   5. Writes CSV to the USB gadget mount point
//!   6. Calls OPC UA **Method** `Jobs/CompleteJob(job_id)` to mark done
//!
//! This is the same pattern used by SCADA systems connecting to Siemens PLCs,
//! Fanuc CNCs, and any OPC UA server. Zero custom protocol. Zero reinvention.
//!
//! ## Reference
//!
//! See /private/tmp/async-opcua/samples/simple-client/src/main.rs for the
//! async-opcua subscription pattern this code is based on.
use std::sync::{Arc, Mutex};
use std::time::Duration;

use opcua::{
    client::{ClientBuilder, DataChangeCallback, IdentityToken, MonitoredItem},
    crypto::SecurityPolicy,
    types::{
        AttributeId, DataValue, MessageSecurityMode, MonitoredItemCreateRequest, NodeId,
        ReadValueId, TimestampsToReturn, UserTokenPolicy, VariableId, Variant,
    },
};

use crate::config::Config;
use crate::machine::{Job, MachineStatus, SharedState};

const NS_URI: &str = "urn:howick-edge-agent";

/// Run the OPC UA client agent.
///
/// Connects to the OPC UA server at `config.plat_trunk.url`, subscribes to
/// `Jobs/PendingJobId`. When a new job arrives the server pushes a notification
/// immediately — no polling loop needed. The agent then:
///   - Reads the CSV via OPC UA Read
///   - Writes it to the USB gadget mount point
///   - Calls OPC UA Method `Jobs/CompleteJob(job_id)`
pub async fn run_opcua_agent(config: Config, state: SharedState) -> anyhow::Result<()> {
    let server_url = config.plat_trunk.url.trim_end_matches('/').to_string();

    tracing::info!(url = %server_url, "OPC UA agent starting — subscription mode");

    // Build client — same pattern as async-opcua simple-client sample.
    // `trust_server_certs(true)` is fine on a private factory LAN.
    // `session_retry_limit(-1)` means retry forever (factory should always be up).
    let mut client = ClientBuilder::new()
        .application_name("howick-agent")
        .application_uri("urn:howick-agent")
        .trust_server_certs(true)
        .create_sample_keypair(true)
        .session_retry_limit(-1)
        .client()
        .map_err(|e| anyhow::anyhow!("OPC UA client build: {e:?}"))?;

    let (session, event_loop) = client
        .connect_to_matching_endpoint(
            (
                server_url.as_str(),
                SecurityPolicy::None.to_str(),
                MessageSecurityMode::None,
                UserTokenPolicy::anonymous(),
            ),
            IdentityToken::Anonymous,
        )
        .await
        .map_err(|e| anyhow::anyhow!("OPC UA connect to {server_url}: {e:?}"))?;

    // Spawn event loop in background — it maintains the connection and delivers
    // subscription notifications. If it exits, the session has been abandoned.
    let _handle = event_loop.spawn();

    tracing::info!("Waiting for OPC UA connection to Pi 5...");
    session.wait_for_connection().await;
    tracing::info!("OPC UA connected to Pi 5 ✓");

    // Resolve our namespace index from the server's namespace array
    let ns = get_namespace_index(&session, NS_URI).await.unwrap_or(2);
    tracing::info!(ns, "Namespace index resolved");

    // Shared signal: DataChangeCallback (sync) → main task (async)
    // pending_job stores the job_id we need to fetch and deliver.
    let pending_job: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let pending_clone = pending_job.clone();
    let notify = Arc::new(tokio::sync::Notify::new());
    let notify_clone = notify.clone();

    // Subscribe to Jobs/PendingJobId — 500ms publishing interval matches server sync rate.
    // DataChangeCallback fires synchronously on the session event loop thread.
    let sub_id = session
        .create_subscription(
            Duration::from_millis(500),
            10,
            30,
            0,
            0,
            true,
            DataChangeCallback::new(move |dv: DataValue, _item: &MonitoredItem| {
                if let Some(Variant::String(s)) = dv.value {
                    let job_id = s.value().clone().unwrap_or_default();
                    if !job_id.is_empty() {
                        *pending_clone.lock().unwrap() = Some(job_id);
                        notify_clone.notify_one();
                    }
                }
            }),
        )
        .await
        .map_err(|e| anyhow::anyhow!("create_subscription: {e:?}"))?;

    // Monitor Jobs/PendingJobId — server pushes immediately on change
    session
        .create_monitored_items(
            sub_id,
            TimestampsToReturn::Both,
            vec![MonitoredItemCreateRequest::from(NodeId::new(
                ns,
                "Jobs/PendingJobId",
            ))],
        )
        .await
        .map_err(|e| anyhow::anyhow!("create_monitored_items: {e:?}"))?;

    tracing::info!("Subscribed to Jobs/PendingJobId — waiting for jobs from Pi 5");

    // Pre-compute node IDs for method call
    let jobs_folder_node = NodeId::new(ns, "Jobs");
    let complete_job_node = NodeId::new(ns, "Jobs/CompleteJob");

    // Main loop — woken by the subscription callback, not by a timer
    loop {
        notify.notified().await;

        let Some(job_id) = pending_job.lock().unwrap().take() else {
            continue;
        };

        tracing::info!(job_id = %job_id, "Job arrived via OPC UA subscription");

        // Read job details from the Pi 5 OPC UA server
        let results = match session
            .read(
                &[
                    ReadValueId {
                        node_id: NodeId::new(ns, "Jobs/PendingJobName"),
                        attribute_id: AttributeId::Value as u32,
                        ..Default::default()
                    },
                    ReadValueId {
                        node_id: NodeId::new(ns, "Jobs/PendingJobCsv"),
                        attribute_id: AttributeId::Value as u32,
                        ..Default::default()
                    },
                ],
                TimestampsToReturn::Both,
                0.0,
            )
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(job_id = %job_id, "OPC UA read job data failed: {e:?}");
                continue;
            }
        };

        let frameset_name = extract_string(&results[0]).unwrap_or_default();
        let csv = extract_string(&results[1]).unwrap_or_default();

        if csv.is_empty() {
            tracing::warn!(job_id = %job_id, "CSV content empty — server sync not ready yet, will retry on next notification");
            continue;
        }

        // Write CSV to machine input directory (triggers USB gadget refresh if configured)
        let filename = format!("{frameset_name}.csv");
        if let Err(e) = crate::usb_gadget::write_job(&config.machine, &filename, &csv).await {
            tracing::error!(job_id = %job_id, "USB write failed: {e}");
            continue;
        }

        let dest = config.machine.machine_input_dir.join(&filename);
        tracing::info!(
            job_id        = %job_id,
            frameset_name = %frameset_name,
            dest          = %dest.display(),
            "CSV written to machine input via OPC UA"
        );

        // Mark Running while job is in flight
        {
            let mut s = state.write().await;
            s.status = MachineStatus::Running;
            s.current_job = Some(frameset_name.clone());
        }

        // Call Jobs/CompleteJob(job_id) on the Pi 5 — moves job from queue to completed.
        // Tuple syntax: (object_node_id, method_node_id, Option<Vec<Variant>>)
        match session
            .call_one((
                jobs_folder_node.clone(),
                complete_job_node.clone(),
                Some(vec![Variant::String(job_id.clone().into())]),
            ))
            .await
        {
            Ok(_) => tracing::info!(job_id = %job_id, "OPC UA CompleteJob ✓"),
            Err(e) => tracing::warn!(job_id = %job_id, "OPC UA CompleteJob failed: {e:?}"),
        }

        // Mark Idle and record in history
        {
            let mut s = state.write().await;
            s.status = MachineStatus::Idle;
            s.current_job = None;
            s.completed_jobs.push(Job {
                id: job_id.clone(),
                frameset_name,
                csv_path: dest,
                submitted_at: std::time::SystemTime::now(),
            });
        }
    }
}

/// Read the server's namespace array and find the index for `uri`.
/// Returns `None` if not found; caller should fall back to 2 (the typical
/// index for the first custom namespace after OPC UA and server namespaces).
async fn get_namespace_index(session: &opcua::client::Session, uri: &str) -> Option<u16> {
    let results = session
        .read(
            &[ReadValueId {
                node_id: VariableId::Server_NamespaceArray.into(),
                attribute_id: AttributeId::Value as u32,
                ..Default::default()
            }],
            TimestampsToReturn::Server,
            0.0,
        )
        .await
        .ok()?;

    if let Some(Variant::Array(arr)) = &results.first()?.value {
        arr.values.iter().enumerate().find_map(|(i, v)| {
            if let Variant::String(s) = v {
                if s.value().as_deref() == Some(uri) {
                    Some(i as u16)
                } else {
                    None
                }
            } else {
                None
            }
        })
    } else {
        None
    }
}

fn extract_string(dv: &DataValue) -> Option<String> {
    match &dv.value {
        Some(Variant::String(s)) => s.value().clone(),
        _ => None,
    }
}
