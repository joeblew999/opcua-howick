//! # howick-agent
//!
//! Minimal Howick edge agent for Raspberry Pi Zero 2W.
//!
//! The Pi Zero 2W (~$15) plugs into the Howick FRAMA machine's USB port
//! via a long cable and acts as a USB mass storage device (fake USB stick)
//! using Linux USB gadget mode (g_mass_storage kernel module).
//!
//! This binary does exactly two things:
//!   1. Poll plat-trunk for pending CSV jobs (HTTP GET every 5s)
//!   2. Write CSV to the USB gadget mount point + refresh USB presentation
//!
//! No OPC UA. No HTTP server. No file watcher.
//! Binary size: ~3MB. RAM: ~16MB. Fits comfortably on Pi Zero 2W (512MB).
//!
//! For the full agent with OPC UA + HTTP status server, use opcua-howick
//! on a Pi 5, NUC, or Mac Mini.
//!
//! ## Setup
//!
//! See docs/customer/06-pi-zero-setup.md for Pi Zero 2W setup guide.
//!
//! ## Config (config.toml)
//!
//! ```toml
//! [machine]
//! machine_input_dir = "/mnt/usb_share"   # mounted USB image
//! usb_gadget_mode   = true               # trigger USB refresh after write
//!
//! [plat_trunk]
//! url = "https://your-worker.workers.dev"
//! status_push_interval_secs = 5
//! ```

// Shared modules — only the ones we need
// (server.rs and http_server.rs are NOT included here)
#[path = "../config.rs"]
mod config;
#[path = "../machine.rs"]
mod machine;
#[path = "../poller.rs"]
mod poller;
#[path = "../sensor.rs"]
mod sensor;
#[path = "../usb_gadget.rs"]
mod usb_gadget;

use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("howick_agent=info".parse().unwrap())
                .add_directive("opcua_howick=info".parse().unwrap()),
        )
        .compact() // compact format — saves log space on Pi Zero's SD card
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "howick-agent starting (Pi Zero 2W minimal mode)"
    );

    let config_path = PathBuf::from("config.toml");
    let config = config::Config::load_or_default(&config_path);

    // Validate USB gadget mode is configured
    if !config.machine.usb_gadget_mode {
        tracing::warn!(
            "usb_gadget_mode = false in config.toml\n\
             Set usb_gadget_mode = true and machine_input_dir = /mnt/usb_share\n\
             See docs/customer/06-pi-zero-setup.md for Pi Zero 2W setup guide"
        );
    }

    tracing::info!(
        topology       = config.topology(),
        plat_trunk_url = %config.plat_trunk.url,
        machine_dir    = %config.machine.machine_input_dir.display(),
        usb_gadget     = config.machine.usb_gadget_mode,
        poll_interval  = config.plat_trunk.status_push_interval_secs,
        "Configuration loaded — polling for jobs"
    );

    // Shared machine state (lightweight — tracks job history only)
    let state = machine::new_shared_state();
    {
        let mut s = state.write().await;
        s.status = machine::MachineStatus::Idle;
    }

    tracing::info!(
        "Running — polling {} every {}s",
        config.plat_trunk.url,
        config.plat_trunk.status_push_interval_secs,
    );

    // Phase 2: coil sensor push loop (only when sensor.enabled = true in config)
    if config.sensor.enabled {
        tracing::info!(
            poll_interval = config.sensor.poll_interval_secs,
            "Coil sensor enabled — pushing weight to {}",
            config.plat_trunk.url
        );
        let sensor_url = config.plat_trunk.url.clone();
        let sensor_interval = config.sensor.poll_interval_secs;
        tokio::select! {
            r = poller::run_job_poller(config, state) => {
                if let Err(e) = r { tracing::error!("Job poller failed: {e}"); }
            }
            r = sensor::run_sensor_push(sensor_url, sensor_interval) => {
                if let Err(e) = r { tracing::error!("Sensor push failed: {e}"); }
            }
        }
    } else {
        // Sensor not fitted — just run the job poller
        if let Err(e) = poller::run_job_poller(config, state).await {
            tracing::error!("Job poller failed: {e}");
            std::process::exit(1);
        }
    }

    Ok(())
}
