use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration loaded from config.toml.
///
/// Supports all three deployment topologies:
/// - Cloud:  plat_trunk_url = "https://your-worker.workers.dev"
/// - LAN:    plat_trunk_url = "http://localhost:3000"
/// - Hybrid: plat_trunk_url = "http://localhost:3000" (syncs to cloud separately)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub opcua:      OpcUaConfig,
    pub machine:    MachineConfig,
    pub plat_trunk: PlatTrunkConfig,
    pub http:       HttpConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpcUaConfig {
    pub host:             String,
    pub port:             u16,
    pub application_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfig {
    pub machine_name:       String,
    pub job_input_dir:      PathBuf,
    pub machine_input_dir:  PathBuf,
    pub machine_output_dir: PathBuf,
    /// USB gadget mode — set true when running on Pi Zero 2W acting as USB mass storage.
    /// When true, after each CSV write the USB storage is re-presented to the host machine.
    /// Set false for all other deployments (Pi 5, NUC, Windows, Mac).
    #[serde(default)]
    pub usb_gadget_mode: bool,
}

/// HTTP status server — the CF Worker / Tauri backend calls this to get
/// real machine state for the plugin UI status panel.
///
/// Topology A (Cloud):  CF Worker → HTTP → this server (via Tailscale or VPN)
/// Topology B/C (LAN):  Tauri local server → HTTP → localhost (same machine)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    /// Host to bind HTTP status server on
    pub host: String,
    /// Port for HTTP status API (default 4841 — one above OPC UA standard port)
    pub port: u16,
}

/// Connection back to plat-trunk — same HTTP API regardless of topology.
///
/// Topology A (Cloud):  url = "https://your-worker.workers.dev"
/// Topology B (LAN):    url = "http://localhost:3000"
/// Topology C (Hybrid): url = "http://localhost:3000"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatTrunkConfig {
    pub url:                        String,
    pub api_key:                    String,
    pub status_push_interval_secs:  u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            opcua: OpcUaConfig {
                host:             "0.0.0.0".to_string(),
                port:             4840,
                application_name: "Howick Edge Agent".to_string(),
            },
            machine: MachineConfig {
                machine_name:       "Howick FRAMA".to_string(),
                job_input_dir:      PathBuf::from("./jobs/input"),
                machine_input_dir:  PathBuf::from("./jobs/machine"),
                machine_output_dir: PathBuf::from("./jobs/output"),
                usb_gadget_mode: false,
            },
            http: HttpConfig {
                host: "0.0.0.0".to_string(),
                port: 4841,
            },
            plat_trunk: PlatTrunkConfig {
                url:                       "http://localhost:3000".to_string(),
                api_key:                   String::new(),
                status_push_interval_secs: 5,
            },
        }
    }
}

impl Config {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_or_default(path: &std::path::Path) -> Self {
        match Self::load(path) {
            Ok(c) => {
                tracing::info!("Loaded config from {}", path.display());
                c
            }
            Err(e) => {
                tracing::warn!(
                    "Could not load config from {}: {e} — using defaults",
                    path.display()
                );
                Self::default()
            }
        }
    }

    pub fn topology(&self) -> &'static str {
        if self.plat_trunk.url.contains("localhost") || self.plat_trunk.url.contains("127.0.0.1") {
            "LAN"
        } else {
            "Cloud"
        }
    }
}
