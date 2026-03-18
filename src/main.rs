//! # opcua-howick
//!
//! Full OPC UA edge agent for Howick FRAMA machines.
//! For Raspberry Pi 5, NUC, Mac Mini, or Windows PC.
//!
//! For Raspberry Pi Zero 2W (USB gadget mode), use `howick-frama` instead —
//! a minimal binary with no OPC UA or HTTP server (~3MB vs ~15MB).
//!
//! Four concurrent services:
//!   - OPC UA server  (port 4840) — machine state for any OPC UA client
//!   - HTTP server    (port 4841) — JSON API for plat-trunk / Tauri
//!   - Job poller                 — polls plat-trunk API for R2-queued jobs
//!   - File watcher               — picks up CSV files dropped locally

use opcua_howick::{
    config,
    job_server::{http, opcua_server, watcher},
    machine, updater, VERSION,
};

use std::net::SocketAddr;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("opcua-server {VERSION}");
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("opcua_howick=info".parse().unwrap()),
        )
        .init();

    tracing::info!(version = VERSION, "opcua-server starting");

    // Background self-update check — runs once on startup.
    // On update: exit(0) so systemd restarts the new binary automatically.
    // On failure (offline, no asset, etc.): logged at debug level and ignored.
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        match updater::check_and_update(
            &client,
            "opcua-server",
            VERSION,
            "https://api.github.com",
            None,
        )
        .await
        {
            Ok(true) => {
                tracing::info!("Self-update complete — restarting");
                std::process::exit(0);
            }
            Ok(false) => {}
            Err(e) => tracing::debug!("Update check failed (offline?): {e}"),
        }
    });

    let config_path = std::env::args()
        .skip_while(|a| a != "--config")
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("opcua-server.dev.toml"));
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
        r = opcua_howick::http_poller::run_job_poller(config.clone(), state.clone()) => {
            if let Err(e) = r { tracing::error!("Job poller: {e}"); }
        }
        r = opcua_server::run_server(&config, state.clone()) => {
            if let Err(e) = r { tracing::error!("OPC UA server: {e}"); }
        }
        r = http::run_http_server(http_listener, &config, state.clone()) => {
            if let Err(e) = r { tracing::error!("HTTP server: {e}"); }
        }
    }

    tracing::info!("opcua-server stopped");
    Ok(())
}
