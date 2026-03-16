use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration loaded from config.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub opcua: OpcUaConfig,
    pub machine: MachineConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpcUaConfig {
    /// OPC UA server endpoint host
    pub host: String,
    /// OPC UA server port (default 4840 is the OPC UA standard port)
    pub port: u16,
    /// Application name shown to OPC UA clients
    pub application_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfig {
    /// Folder to watch for new CSV jobs (from plat-trunk or manual drop)
    pub job_input_dir: PathBuf,
    /// Folder the Howick machine watches for its input files
    pub machine_input_dir: PathBuf,
    /// Folder where completed job signals appear
    pub machine_output_dir: PathBuf,
    /// Howick machine name/ID for labelling
    pub machine_name: String,
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
                job_input_dir: PathBuf::from("./jobs/input"),
                machine_input_dir: PathBuf::from("./jobs/machine"),
                machine_output_dir: PathBuf::from("./jobs/output"),
                machine_name: "Howick FRAMA".to_string(),
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
                tracing::warn!("Could not load config from {}: {e} — using defaults", path.display());
                Self::default()
            }
        }
    }
}
