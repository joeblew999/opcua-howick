/// USB gadget support for Pi Zero 2W deployment.
///
/// When opcua-server runs on a Pi Zero 2W acting as USB mass storage,
/// writing a file to the mounted image is not enough — the host machine
/// (Howick FRAMA) needs to be notified that the storage has changed.
///
/// This module handles:
/// 1. Writing the CSV to the mounted USB image path
/// 2. Syncing the filesystem
/// 3. Re-presenting the USB storage to the host machine
///
/// If NOT running in USB gadget mode (e.g. Pi 5, NUC, Windows),
/// this module is a no-op — files are written directly to the path.
use std::path::Path;
use std::time::Duration;

use crate::config::MachineConfig;

/// Write a CSV file to the machine input directory.
///
/// In USB gadget mode: writes to the mounted image and refreshes USB.
/// In standard mode: writes directly to the configured path.
pub async fn write_job(config: &MachineConfig, filename: &str, csv: &str) -> anyhow::Result<()> {
    let dest = config.machine_input_dir.join(filename);

    tokio::fs::create_dir_all(&config.machine_input_dir).await?;
    tokio::fs::write(&dest, csv).await?;

    tracing::info!(
        path = %dest.display(),
        bytes = csv.len(),
        "CSV written to machine input"
    );

    // If USB gadget mode is configured, refresh the USB presentation
    if config.usb_gadget_mode {
        refresh_usb_gadget().await?;
    }

    Ok(())
}

/// Sync filesystem and re-present USB storage to the Howick machine.
///
/// This makes the newly written CSV visible to the host machine.
/// Only relevant on Pi Zero 2W with g_mass_storage gadget mode.
async fn refresh_usb_gadget() -> anyhow::Result<()> {
    // Sync all pending writes to the image
    tracing::debug!("USB gadget: syncing filesystem");

    // Run sync via shell — most portable approach
    let sync_result = tokio::process::Command::new("sync").status().await;
    if let Err(e) = sync_result {
        tracing::warn!("sync failed: {e}");
    }

    // Brief pause to ensure sync completes
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Re-present storage to host by toggling the gadget suspended state
    // This is equivalent to briefly unplugging and replugging the USB stick
    let gadget_path = "/sys/bus/platform/drivers/dwc2/dwc2/gadget/suspended";
    if Path::new(gadget_path).exists() {
        let _ = tokio::fs::write(gadget_path, "1").await;
        tokio::time::sleep(Duration::from_millis(300)).await;
        let _ = tokio::fs::write(gadget_path, "0").await;
        tracing::info!("USB gadget: storage re-presented to host machine");
    } else {
        // Try the refresh script if present (fallback)
        let script = "/usr/local/bin/usb-refresh.sh";
        if Path::new(script).exists() {
            let _ = tokio::process::Command::new("sh")
                .arg(script)
                .status()
                .await;
        }
    }

    Ok(())
}
