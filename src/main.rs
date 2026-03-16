//! # opcua-howick
//!
//! OPC UA edge agent for Howick FRAMA roll-forming machines.
//! Runs on a small compute module on the factory LAN.
//!
//! Usage:
//!   opcua-howick [--config config.toml]

mod config;
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
        host = %config.opcua.host,
        port = config.opcua.port,
        machine = %config.machine.machine_name,
        "Configuration loaded"
    );

    let state = machine::new_shared_state();
    {
        let mut s = state.write().await;
        s.status = machine::MachineStatus::Idle;
    }

    let watcher_state = state.clone();
    let watcher_config = config.machine.clone();
    let server_state = state.clone();
    let server_config = config.clone();

    tokio::select! {
        result = watcher::run_job_watcher(watcher_config, watcher_state) => {
            if let Err(e) = result {
                tracing::error!("Job watcher error: {e}");
            }
        }
        result = server::run_server(&server_config, server_state) => {
            if let Err(e) = result {
                tracing::error!("OPC UA server error: {e}");
            }
        }
    }

    tracing::info!("opcua-howick stopped");
    Ok(())
}
