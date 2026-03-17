//! # opcua-howick
//!
//! OPC UA edge agent for Howick FRAMA roll-forming machines.
//! Runs on a small compute module (Raspberry Pi / NUC / Mac Mini) on factory LAN.
//!
//! Four concurrent services:
//!   - OPC UA server  (port 4840) — machine state for any OPC UA client
//!   - HTTP server    (port 4841) — JSON API for plat-trunk / Tauri
//!   - File watcher              — picks up CSV files dropped locally
//!   - Job poller                — polls plat-trunk API for R2-queued jobs
//!
//! Topology-agnostic: plat_trunk.url in config.toml points to CF or localhost.
//! All topologies use the same HTTP API — the poller just polls a different URL.

mod config;
mod http_server;
mod machine;
mod poller;
mod server;
mod watcher;

use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("opcua_howick=info".parse().unwrap()),
        )
        .init();

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "opcua-howick starting");

    let config_path = PathBuf::from("config.toml");
    let config = config::Config::load_or_default(&config_path);

    tracing::info!(
        topology       = config.topology(),
        plat_trunk_url = %config.plat_trunk.url,
        opcua_port     = config.opcua.port,
        http_port      = config.http.port,
        machine        = %config.machine.machine_name,
        "Configuration loaded"
    );

    let state = machine::new_shared_state();
    {
        let mut s = state.write().await;
        s.status = machine::MachineStatus::Idle;
    }

    tracing::info!(
        "Services: OPC UA opc.tcp://{}:{}/ | HTTP http://{}:{}/ | poller→{}",
        config.opcua.host, config.opcua.port,
        config.http.host,  config.http.port,
        config.plat_trunk.url,
    );

    // Run all four services concurrently
    tokio::select! {
        r = watcher::run_job_watcher(config.machine.clone(), state.clone()) => {
            if let Err(e) = r { tracing::error!("File watcher: {e}"); }
        }
        r = poller::run_job_poller(config.clone(), state.clone()) => {
            if let Err(e) = r { tracing::error!("Job poller: {e}"); }
        }
        r = server::run_server(&config, state.clone()) => {
            if let Err(e) = r { tracing::error!("OPC UA server: {e}"); }
        }
        r = http_server::run_http_server(&config, state.clone()) => {
            if let Err(e) = r { tracing::error!("HTTP server: {e}"); }
        }
    }

    tracing::info!("opcua-howick stopped");
    Ok(())
}
