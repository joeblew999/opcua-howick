use std::sync::Arc;
use std::time::Duration;

use opcua::server::address_space::Variable;
use opcua::server::diagnostics::NamespaceMetadata;
use opcua::server::node_manager::memory::{
    simple_node_manager, InMemoryNodeManager, SimpleNodeManager, SimpleNodeManagerImpl,
};
use opcua::server::{ServerBuilder, SubscriptionCache};
use opcua::types::{BuildInfo, DataValue, DateTime, NodeId, UAString};

use crate::config::Config;
use crate::machine::SharedState;

const NS_URI: &str = "urn:howick-edge-agent";

/// Node ID helpers for our namespace
fn node(ns: u16, name: &str) -> NodeId {
    NodeId::new(ns, name)
}

/// Build and run the OPC UA server.
/// Returns when the server is shut down (ctrl-c).
pub async fn run_server(config: &Config, state: SharedState) -> anyhow::Result<()> {
    tracing::info!(
        "Starting OPC UA server on {}:{}",
        config.opcua.host,
        config.opcua.port
    );

    let (server, handle) = ServerBuilder::new()
        .application_name(&config.opcua.application_name)
        .application_uri(NS_URI)
        .product_uri("https://github.com/joeblew999/opcua-howick")
        .host(config.opcua.host.clone())
        .port(config.opcua.port)
        .build_info(BuildInfo {
            product_uri: "https://github.com/joeblew999/opcua-howick".into(),
            manufacturer_name: "Ubuntu Software Pty Ltd".into(),
            product_name: "opcua-howick".into(),
            software_version: env!("CARGO_PKG_VERSION").into(),
            build_number: "1".into(),
            build_date: DateTime::now(),
        })
        .with_node_manager(simple_node_manager(
            NamespaceMetadata {
                namespace_uri: NS_URI.to_owned(),
                ..Default::default()
            },
            "howick",
        ))
        .trust_client_certs(true)
        .diagnostics_enabled(false)
        .build()
        .unwrap();

    let node_manager: Arc<InMemoryNodeManager<SimpleNodeManagerImpl>> = handle
        .node_managers()
        .get_of_type::<SimpleNodeManager>()
        .unwrap();
    let ns = handle.get_namespace_index(NS_URI).unwrap();
    let subscriptions = handle.subscriptions().clone();

    // Build the address space
    build_address_space(ns, &node_manager, &config.machine.machine_name);

    // Spawn state sync task — pushes machine state into OPC UA nodes
    let state_clone = state.clone();
    let node_manager_clone = node_manager.clone();
    let subscriptions_clone = subscriptions.clone();
    tokio::spawn(async move {
        sync_state_to_nodes(ns, state_clone, node_manager_clone, subscriptions_clone).await;
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

    tracing::info!("OPC UA server running at opc.tcp://{}:{}/", config.opcua.host, config.opcua.port);
    server.run().await.map_err(|e| anyhow::anyhow!("Server error: {e:?}"))?;

    Ok(())
}

/// Populate the OPC UA address space with Howick machine nodes.
///
/// Address space layout:
/// ```
/// /Howick/
///   Machine/
///     Status           String
///     CurrentJob       String
///     PiecesProduced   UInt32
///     CoilRemaining    Double  (metres)
///     LastError        String
///   Jobs/
///     QueueDepth       UInt32  (number of jobs waiting)
///     CompletedCount   UInt32
/// ```
fn build_address_space(
    ns: u16,
    manager: &Arc<InMemoryNodeManager<SimpleNodeManagerImpl>>,
    machine_name: &str,
) {
    let address_space = manager.address_space();
    let mut address_space = address_space.write();

    // Root folder: /Howick
    let howick_folder = node(ns, "Howick");
    address_space.add_folder(&howick_folder, "Howick", "Howick", &NodeId::objects_folder_id());

    // /Howick/Machine folder
    let machine_folder = node(ns, "Machine");
    address_space.add_folder(&machine_folder, "Machine", machine_name, &howick_folder);

    // Machine variables
    address_space.add_variables(
        vec![
            Variable::new(&node(ns, "Machine/Status"), "Status", "Status", UAString::from("Offline")),
            Variable::new(&node(ns, "Machine/CurrentJob"), "CurrentJob", "Current Job", UAString::from("")),
            Variable::new(&node(ns, "Machine/PiecesProduced"), "PiecesProduced", "Pieces Produced", 0u32),
            Variable::new(&node(ns, "Machine/CoilRemaining"), "CoilRemaining", "Coil Remaining (m)", 0f64),
            Variable::new(&node(ns, "Machine/LastError"), "LastError", "Last Error", UAString::from("")),
        ],
        &machine_folder,
    );

    // /Howick/Jobs folder
    let jobs_folder = node(ns, "Jobs");
    address_space.add_folder(&jobs_folder, "Jobs", "Jobs", &howick_folder);

    address_space.add_variables(
        vec![
            Variable::new(&node(ns, "Jobs/QueueDepth"), "QueueDepth", "Queue Depth", 0u32),
            Variable::new(&node(ns, "Jobs/CompletedCount"), "CompletedCount", "Completed Count", 0u32),
        ],
        &jobs_folder,
    );

    tracing::info!("OPC UA address space built — {} nodes under /Howick", 7);
}

/// Continuously sync shared MachineState into the OPC UA node values.
/// OPC UA subscriptions will push updates to connected clients automatically.
async fn sync_state_to_nodes(
    ns: u16,
    state: SharedState,
    manager: Arc<InMemoryNodeManager<SimpleNodeManagerImpl>>,
    subscriptions: Arc<SubscriptionCache>,
) {
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    loop {
        interval.tick().await;
        let s = state.read().await;

        let _ = manager.set_values(
            &subscriptions,
            [
                (
                    &node(ns, "Machine/Status"),
                    None,
                    DataValue::new_now(UAString::from(s.status.as_str())),
                ),
                (
                    &node(ns, "Machine/CurrentJob"),
                    None,
                    DataValue::new_now(UAString::from(
                        s.current_job.as_deref().unwrap_or(""),
                    )),
                ),
                (
                    &node(ns, "Machine/PiecesProduced"),
                    None,
                    DataValue::new_now(s.pieces_produced),
                ),
                (
                    &node(ns, "Machine/CoilRemaining"),
                    None,
                    DataValue::new_now(s.coil_remaining_m),
                ),
                (
                    &node(ns, "Machine/LastError"),
                    None,
                    DataValue::new_now(UAString::from(s.last_error.as_str())),
                ),
                (
                    &node(ns, "Jobs/QueueDepth"),
                    None,
                    DataValue::new_now(s.job_queue.len() as u32),
                ),
                (
                    &node(ns, "Jobs/CompletedCount"),
                    None,
                    DataValue::new_now(s.completed_jobs.len() as u32),
                ),
            ]
            .into_iter(),
        );
    }
}
