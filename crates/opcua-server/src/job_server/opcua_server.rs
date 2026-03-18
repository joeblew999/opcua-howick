use std::sync::{Arc, Mutex};
use std::time::Duration;

use opcua::server::address_space::{MethodBuilder, Variable};
use opcua::server::diagnostics::NamespaceMetadata;
use opcua::server::node_manager::memory::{
    simple_node_manager, InMemoryNodeManager, SimpleNodeManager, SimpleNodeManagerImpl,
};
use opcua::server::{ServerBuilder, SubscriptionCache};
use opcua::types::{
    BuildInfo, DataTypeId, DataValue, DateTime, NodeId, StatusCode, UAString, Variant,
};

use opcua_howick::config::Config;
use opcua_howick::machine::SharedState;

/// Node ID helpers for our namespace
fn node(ns: u16, name: &str) -> NodeId {
    NodeId::new(ns, name)
}

/// Build and run the OPC UA server (production — binds to config host:port).
pub async fn run_server(config: &Config, state: SharedState) -> anyhow::Result<()> {
    tracing::info!(
        "Starting OPC UA server on {}:{}",
        config.opcua.host,
        config.opcua.port
    );

    let completions: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let (server, handle) = build_server_builder(
        &config.opcua.application_name,
        &config.opcua.namespace_uri,
        &config.opcua.host,
        config.opcua.port,
        None,
        std::path::PathBuf::from(format!("./pki-server-{}", config.opcua.port)),
    )
    .build()
    .unwrap();

    let node_manager: Arc<InMemoryNodeManager<SimpleNodeManagerImpl>> = handle
        .node_managers()
        .get_of_type::<SimpleNodeManager>()
        .unwrap();
    let ns = handle
        .get_namespace_index(&config.opcua.namespace_uri)
        .unwrap();
    let subscriptions = handle.subscriptions().clone();

    build_address_space(
        ns,
        &node_manager,
        &config.machine.machine_name,
        completions.clone(),
    );

    let state_clone = state.clone();
    let nm_clone = node_manager.clone();
    let subs_clone = subscriptions.clone();
    let completions_clone = completions.clone();
    tokio::spawn(async move {
        sync_state_to_nodes(ns, state_clone, nm_clone, subs_clone, completions_clone).await;
    });

    // Graceful shutdown on ctrl-c
    let handle_c = handle.clone();
    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::warn!("Failed to register ctrl-c handler: {e}");
            return;
        }
        tracing::info!("Shutting down OPC UA server...");
        handle_c.cancel();
    });

    tracing::info!(
        "OPC UA server running at opc.tcp://{}:{}/",
        config.opcua.host,
        config.opcua.port
    );
    server
        .run()
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {e:?}"))?;

    Ok(())
}

/// Build and run the OPC UA server on a pre-bound listener (used in integration tests).
///
/// # Pattern from async-opcua integration tests
/// ```rust,ignore
/// let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
/// let addr = listener.local_addr().unwrap();
/// tokio::spawn(run_server_with(listener, &config, state));
/// // connect client to opc.tcp://127.0.0.1:{addr.port()}/
/// ```
pub async fn run_server_with(
    listener: tokio::net::TcpListener,
    config: &Config,
    state: SharedState,
) -> anyhow::Result<()> {
    let addr = listener.local_addr()?;
    let completions: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let (server, handle) = build_server_builder(
        &config.opcua.application_name,
        &config.opcua.namespace_uri,
        "127.0.0.1",
        addr.port(),
        Some(format!("opc.tcp://127.0.0.1:{}/", addr.port())),
        std::path::PathBuf::from(format!("target/tmp/pki-server-{}", addr.port())),
    )
    .build()
    .unwrap();

    let node_manager: Arc<InMemoryNodeManager<SimpleNodeManagerImpl>> = handle
        .node_managers()
        .get_of_type::<SimpleNodeManager>()
        .unwrap();
    let ns = handle
        .get_namespace_index(&config.opcua.namespace_uri)
        .unwrap();
    let subscriptions = handle.subscriptions().clone();

    build_address_space(
        ns,
        &node_manager,
        &config.machine.machine_name,
        completions.clone(),
    );

    let state_clone = state.clone();
    let nm_clone = node_manager.clone();
    let subs_clone = subscriptions.clone();
    let completions_clone = completions.clone();
    tokio::spawn(async move {
        sync_state_to_nodes(ns, state_clone, nm_clone, subs_clone, completions_clone).await;
    });

    server
        .run_with(listener)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {e:?}"))?;

    Ok(())
}

/// Shared server builder configuration.
///
/// `pki_dir`: where OPC UA certificates are stored.
/// - Production: `./pki-server-{port}` (persists across restarts on the Pi)
/// - Tests: `target/tmp/pki-server-{port}` (gitignored, wiped by `cargo clean`)
fn build_server_builder(
    app_name: &str,
    namespace_uri: &str,
    host: &str,
    port: u16,
    discovery_url: Option<String>,
    pki_dir: std::path::PathBuf,
) -> ServerBuilder {
    // application_uri must differ from namespace_uri — otherwise the namespace table
    // would register the same URI at index 1 (server URI) AND index 2 (our node manager),
    // and clients would resolve to index 1 where no nodes are registered.
    // Always set discovery_urls explicitly so clients connect to 127.0.0.1,
    // not the bind address (0.0.0.0). On macOS, localhost resolves to [::1]
    // (IPv6) but the server binds IPv4-only → connection refused.
    let endpoint_url = discovery_url.unwrap_or_else(|| format!("opc.tcp://127.0.0.1:{port}/"));
    // application_uri is the server's own identity — distinct from the machine namespace URI
    let application_uri = format!("{namespace_uri}-server");

    ServerBuilder::new_anonymous(app_name)
        .application_uri(application_uri)
        .product_uri("https://github.com/joeblew999/opcua-howick")
        .host(host.to_owned())
        .port(port)
        .pki_dir(pki_dir)
        .build_info(BuildInfo {
            product_uri: "https://github.com/joeblew999/opcua-howick".into(),
            manufacturer_name: "Ubuntu Software Pty Ltd".into(),
            product_name: "opcua-server".into(),
            software_version: opcua_howick::VERSION.into(),
            build_number: "1".into(),
            build_date: DateTime::now(),
        })
        .with_node_manager(simple_node_manager(
            NamespaceMetadata {
                namespace_uri: namespace_uri.to_owned(),
                ..Default::default()
            },
            "howick",
        ))
        .trust_client_certs(true)
        .diagnostics_enabled(false)
        .discovery_urls(vec![endpoint_url])
}

/// Populate the OPC UA address space with Howick machine nodes and methods.
///
/// Address space layout:
/// ```text
/// /Howick/
///   Machine/
///     Status           String  — "Running" | "Idle" | "Error" | "Offline"
///     CurrentJob       String  — frameset name e.g. "T1"
///     PiecesProduced   UInt32
///     CoilRemaining    Double  (metres)
///     LastError        String
///   Jobs/
///     QueueDepth       UInt32
///     CompletedCount   UInt32
///     PendingJobId     String  — job_id of next pending job ("" = none)
///     PendingJobName   String  — frameset name of pending job
///     PendingJobCsv    String  — full CSV content (howick-frama reads this)
///     CompleteJob      Method  — call with job_id to mark delivered
/// ```
fn build_address_space(
    ns: u16,
    manager: &Arc<InMemoryNodeManager<SimpleNodeManagerImpl>>,
    machine_name: &str,
    completions: Arc<Mutex<Vec<String>>>,
) {
    let address_space = manager.address_space();
    let mut address_space = address_space.write();

    // Root folder: /Howick
    let howick_folder = node(ns, "Howick");
    address_space.add_folder(
        &howick_folder,
        "Howick",
        "Howick",
        &NodeId::objects_folder_id(),
    );

    // /Howick/Machine folder
    let machine_folder = node(ns, "Machine");
    address_space.add_folder(&machine_folder, "Machine", machine_name, &howick_folder);

    address_space.add_variables(
        vec![
            Variable::new(
                &node(ns, "Machine/Status"),
                "Status",
                "Status",
                UAString::from("Offline"),
            ),
            Variable::new(
                &node(ns, "Machine/CurrentJob"),
                "CurrentJob",
                "Current Job",
                UAString::from(""),
            ),
            Variable::new(
                &node(ns, "Machine/PiecesProduced"),
                "PiecesProduced",
                "Pieces Produced",
                0u32,
            ),
            Variable::new(
                &node(ns, "Machine/CoilRemaining"),
                "CoilRemaining",
                "Coil Remaining (m)",
                0f64,
            ),
            Variable::new(
                &node(ns, "Machine/LastError"),
                "LastError",
                "Last Error",
                UAString::from(""),
            ),
        ],
        &machine_folder,
    );

    // /Howick/Jobs folder
    let jobs_folder = node(ns, "Jobs");
    address_space.add_folder(&jobs_folder, "Jobs", "Jobs", &howick_folder);

    address_space.add_variables(
        vec![
            Variable::new(
                &node(ns, "Jobs/QueueDepth"),
                "QueueDepth",
                "Queue Depth",
                0u32,
            ),
            Variable::new(
                &node(ns, "Jobs/CompletedCount"),
                "CompletedCount",
                "Completed Count",
                0u32,
            ),
            // M2M job delivery nodes — howick-frama reads these via OPC UA subscription
            Variable::new(
                &node(ns, "Jobs/PendingJobId"),
                "PendingJobId",
                "Pending Job ID",
                UAString::from(""),
            ),
            Variable::new(
                &node(ns, "Jobs/PendingJobName"),
                "PendingJobName",
                "Pending Job Name",
                UAString::from(""),
            ),
            Variable::new(
                &node(ns, "Jobs/PendingJobCsv"),
                "PendingJobCsv",
                "Pending Job CSV",
                UAString::from(""),
            ),
        ],
        &jobs_folder,
    );

    // CompleteJob method — howick-frama calls this after writing CSV to USB
    // Signature: CompleteJob(job_id: String) -> ()
    let method_node = node(ns, "Jobs/CompleteJob");
    MethodBuilder::new(&method_node, "CompleteJob", "CompleteJob")
        .component_of(jobs_folder.clone())
        .input_args(
            &mut *address_space,
            &node(ns, "Jobs/CompleteJob/InputArgs"),
            &[("JobId", DataTypeId::String).into()],
        )
        .insert(&mut *address_space);

    // Method callback: queue job_id for async processing by sync_state_to_nodes
    manager
        .inner()
        .add_method_callback(method_node, move |args| {
            let Some(Variant::String(s)) = args.first() else {
                return Err(StatusCode::BadTypeMismatch);
            };
            let job_id = s.value().clone().unwrap_or_default();
            if job_id.is_empty() {
                return Err(StatusCode::BadInvalidArgument);
            }
            tracing::info!(job_id = %job_id, "OPC UA CompleteJob called");
            completions.lock().unwrap().push(job_id);
            Ok(Vec::new())
        });

    tracing::info!(
        "OPC UA address space built — Howick/Machine + Howick/Jobs + CompleteJob method"
    );
}

/// Snapshot of machine state for the sync task (avoids holding lock during async I/O).
struct StateSnapshot {
    status: String,
    current_job: String,
    pieces_produced: u32,
    coil_remaining_m: f64,
    last_error: String,
    queue_depth: u32,
    completed_count: u32,
    pending: Option<(String, String, std::path::PathBuf)>, // (id, name, csv_path)
}

/// Continuously sync SharedState → OPC UA nodes every 500ms.
/// Also processes pending CompleteJob method calls.
async fn sync_state_to_nodes(
    ns: u16,
    state: SharedState,
    manager: Arc<InMemoryNodeManager<SimpleNodeManagerImpl>>,
    subscriptions: Arc<SubscriptionCache>,
    completions: Arc<Mutex<Vec<String>>>,
) {
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    loop {
        interval.tick().await;

        // Process any pending CompleteJob calls (from method callback — sync Mutex)
        let to_complete: Vec<String> = completions.lock().unwrap().drain(..).collect();
        if !to_complete.is_empty() {
            let mut s = state.write().await;
            for job_id in to_complete {
                if let Some(idx) = s.job_queue.iter().position(|j| j.id == job_id) {
                    let job = s.job_queue.remove(idx);
                    tracing::info!(job_id = %job.id, "Job marked complete via OPC UA");
                    s.completed_jobs.push(job);
                }
            }
        }

        // Capture state snapshot (drop async lock before file I/O)
        let snap = {
            let s = state.read().await;
            StateSnapshot {
                status: s.status.as_str().to_string(),
                current_job: s.current_job.clone().unwrap_or_default(),
                pieces_produced: s.pieces_produced,
                coil_remaining_m: s.coil_remaining_m,
                last_error: s.last_error.clone(),
                queue_depth: s.job_queue.len() as u32,
                completed_count: s.completed_jobs.len() as u32,
                pending: s
                    .job_queue
                    .first()
                    .map(|j| (j.id.clone(), j.frameset_name.clone(), j.csv_path.clone())),
            }
        }; // async lock dropped here — safe to do file I/O

        // Read pending job CSV from disk (outside the state lock)
        let (pending_id, pending_name, pending_csv) = if let Some((id, name, path)) = snap.pending {
            let csv = tokio::fs::read_to_string(&path).await.unwrap_or_default();
            (id, name, csv)
        } else {
            (String::new(), String::new(), String::new())
        };

        let _ = manager.set_values(
            &subscriptions,
            [
                (
                    &node(ns, "Machine/Status"),
                    None,
                    DataValue::new_now(UAString::from(snap.status)),
                ),
                (
                    &node(ns, "Machine/CurrentJob"),
                    None,
                    DataValue::new_now(UAString::from(snap.current_job)),
                ),
                (
                    &node(ns, "Machine/PiecesProduced"),
                    None,
                    DataValue::new_now(snap.pieces_produced),
                ),
                (
                    &node(ns, "Machine/CoilRemaining"),
                    None,
                    DataValue::new_now(snap.coil_remaining_m),
                ),
                (
                    &node(ns, "Machine/LastError"),
                    None,
                    DataValue::new_now(UAString::from(snap.last_error)),
                ),
                (
                    &node(ns, "Jobs/QueueDepth"),
                    None,
                    DataValue::new_now(snap.queue_depth),
                ),
                (
                    &node(ns, "Jobs/CompletedCount"),
                    None,
                    DataValue::new_now(snap.completed_count),
                ),
                (
                    &node(ns, "Jobs/PendingJobId"),
                    None,
                    DataValue::new_now(UAString::from(pending_id)),
                ),
                (
                    &node(ns, "Jobs/PendingJobName"),
                    None,
                    DataValue::new_now(UAString::from(pending_name)),
                ),
                (
                    &node(ns, "Jobs/PendingJobCsv"),
                    None,
                    DataValue::new_now(UAString::from(pending_csv)),
                ),
            ]
            .into_iter(),
        );
    }
}
