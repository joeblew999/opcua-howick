//! # opcua-howick
//!
//! OPC UA edge agent for Howick FRAMA roll-forming machines.
//! Runs on a small compute module (Raspberry Pi / NUC / Mac Mini) on factory LAN.
//!
//! Three concurrent services:
//!   - OPC UA server (port 4840) — machine state for any OPC UA client
//!   - HTTP status server (port 4841) — JSON API for plat-trunk / Tauri
//!   - Job file watcher — CSV drop to machine input directory
//!
//! Topology-agnostic: plat_trunk.url in config.toml points to CF or localhost.

mod config;
mod http_server;
mod machine;
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
        topology        = config.topology(),
        plat_trunk_url  = %config.plat_trunk.url,
        opcua_port      = config.opcua.port,
        http_port       = config.http.port,
        machine         = %config.machine.machine_name,
        "Configuration loaded"
    );

    // Shared machine state — updated by watcher, read by OPC UA + HTTP servers
    let state = machine::new_shared_state();
    {
        let mut s = state.write().await;
        s.status = machine::MachineStatus::Idle;
    }

    tracing::info!(
        "Running — OPC UA: opc.tcp://{}:{}/  HTTP: http://{}:{}/status",
        config.opcua.host, config.opcua.port,
        config.http.host,  config.http.port,
    );

    // Run all three services concurrently — if any exits (error or ctrl-c), stop all
    tokio::select! {
        result = watcher::run_job_watcher(config.machine.clone(), state.clone()) => {
            if let Err(e) = result { tracing::error!("Job watcher: {e}"); }
        }
        result = server::run_server(&config, state.clone()) => {
            if let Err(e) = result { tracing::error!("OPC UA server: {e}"); }
        }
        result = http_server::run_http_server(&config, state.clone()) => {
            if let Err(e) = result { tracing::error!("HTTP server: {e}"); }
        }
    }

    tracing::info!("opcua-howick stopped");
    Ok(())
}
