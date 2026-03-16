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
    pub opcua: OpcUaConfig,
    pub machine: MachineConfig,
    pub plat_trunk: PlatTrunkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpcUaConfig {
    /// OPC UA server host (0.0.0.0 = all interfaces)
    pub host: String,
    /// OPC UA standard port is 4840
    pub port: u16,
    /// Application name shown to OPC UA clients
    pub application_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfig {
    /// Human-readable machine name
    pub machine_name: String,
    /// Folder opcua-howick watches for incoming CSV jobs
    /// (plat-trunk drops files here, or operators copy manually)
    pub job_input_dir: PathBuf,
    /// Folder the Howick machine watches for its input files
    /// TODO: confirm exact path with Prin
    pub machine_input_dir: PathBuf,
    /// Folder where the machine signals job completion
    /// TODO: confirm exact path with Prin
    pub machine_output_dir: PathBuf,
}

/// Connection back to plat-trunk — same API regardless of topology.
///
/// Topology A (Cloud):  url = "https://your-worker.workers.dev"
/// Topology B (LAN):    url = "http://localhost:3000"
/// Topology C (Hybrid): url = "http://localhost:3000"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatTrunkConfig {
    /// Base URL of the plat-trunk Hono backend
    /// Cloud or localhost — opcua-howick doesn't care which
    pub url: String,
    /// API key for authenticating with plat-trunk
    /// (empty string = no auth, for local LAN deployments)
    pub api_key: String,
    /// Push machine status updates to plat-trunk every N seconds
    pub status_push_interval_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            opcua: OpcUaConfig {
                host: "0.0.0.0".to_string(),
                port: 4840,
                application_name: "Howick Edge Agent".to_string(),
            },
            machine: MachineConfig {
                machine_name: "Howick FRAMA".to_string(),
                job_input_dir: PathBuf::from("./jobs/input"),
                machine_input_dir: PathBuf::from("./jobs/machine"),
                machine_output_dir: PathBuf::from("./jobs/output"),
            },
            plat_trunk: PlatTrunkConfig {
                url: "http://localhost:3000".to_string(),
                api_key: String::new(),
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
