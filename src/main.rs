//! # opcua-howick
//!
//! Full OPC UA edge agent for Howick FRAMA machines.
//! For Raspberry Pi 5, NUC, Mac Mini, or Windows PC.
//!
//! For Raspberry Pi Zero 2W (USB gadget mode), use `howick-agent` instead —
//! a minimal binary with no OPC UA or HTTP server (~3MB vs ~15MB).
//!
//! Four concurrent services:
//!   - OPC UA server  (port 4840) — machine state for any OPC UA client
//!   - HTTP server    (port 4841) — JSON API for plat-trunk / Tauri
//!   - Job poller                 — polls plat-trunk API for R2-queued jobs
//!   - File watcher               — picks up CSV files dropped locally

use opcua_howick::{config, http_server, machine, poller, server, watcher, VERSION};

use std::net::SocketAddr;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("opcua-howick {VERSION}");
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("opcua_howick=info".parse().unwrap()),
        )
        .init();

    tracing::info!(version = VERSION, "opcua-howick starting");

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
        config.opcua.host,
        config.opcua.port,
        config.http.host,
        config.http.port,
        config.plat_trunk.url,
    );

    let http_addr: SocketAddr = format!("{}:{}", config.http.host, config.http.port).parse()?;
    let http_listener = tokio::net::TcpListener::bind(http_addr).await?;

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
        r = http_server::run_http_server(http_listener, &config, state.clone()) => {
            if let Err(e) = r { tracing::error!("HTTP server: {e}"); }
        }
    }

    tracing::info!("opcua-howick stopped");
    Ok(())
}
