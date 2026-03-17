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
    pub http: HttpConfig,
    #[serde(default)]
    pub sensor: SensorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpcUaConfig {
    pub host: String,
    pub port: u16,
    pub application_name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryMode {
    #[default]
    /// File watcher writes CSV directly to machine_input_dir.
    /// Use for Topology A (Design PC only) — no Pi Zero.
    Direct,
    /// File watcher holds CSV in queue; howick-agent picks it up via HTTP.
    /// Use for Topology B/C (Pi Zero polls opcua-howick or plat-trunk).
    Queue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfig {
    pub machine_name: String,
    pub job_input_dir: PathBuf,
    pub machine_input_dir: PathBuf,
    pub machine_output_dir: PathBuf,
    /// USB gadget mode — set true when running on Pi Zero 2W acting as USB mass storage.
    /// When true, after each CSV write the USB storage is re-presented to the host machine.
    /// Set false for all other deployments (Pi 5, NUC, Windows, Mac).
    #[serde(default)]
    pub usb_gadget_mode: bool,
    /// How uploaded CSVs reach the Howick FRAMA machine:
    ///   "direct" — watcher writes immediately to machine_input_dir (Topology A, no Pi Zero)
    ///   "queue"  — watcher holds in queue; howick-agent picks up via HTTP (Topology B/C)
    #[serde(default)]
    pub delivery_mode: DeliveryMode,
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

/// Coil weight sensor — Phase 2.
///
/// A load cell + HX711 ADC sits under the coil spool on the Pi Zero GPIO.
/// The Pi Zero pushes raw weight readings to opcua-howick via POST /api/sensor/coil.
/// opcua-howick converts kg → metres remaining and updates the dashboard +
/// OPC UA Machine/CoilRemaining node.
///
/// Hardware (see docs/customer/hardware-quote.md — Phase 2):
///   - Load cell 20kg     × 1
///   - HX711 ADC module   × 1
///   - 5m ribbon cable    × 1
///   - Cost: ~680 THB from Lazada Thailand.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConfig {
    /// Set true when Pi Zero has the load cell wired up and calibrated.
    #[serde(default)]
    pub enabled: bool,
    /// Weight of the empty coil spool in kg.  Weigh it and enter here.
    #[serde(default = "default_empty_spool_kg")]
    pub empty_spool_kg: f64,
    /// Steel consumed per metre of S8908 profile in kg/m.  Default calibrated for S8908.
    #[serde(default = "default_steel_kg_per_m")]
    pub steel_kg_per_m: f64,
    /// Alert fires when coil drops below this many metres remaining.
    #[serde(default = "default_low_alert_m")]
    pub low_alert_m: f64,
    /// How often to push a sensor reading from Pi Zero to Pi 5 (seconds).
    #[serde(default = "default_sensor_poll_secs")]
    pub poll_interval_secs: u64,
}

fn default_empty_spool_kg() -> f64 {
    18.0
}
fn default_steel_kg_per_m() -> f64 {
    0.74
} // S8908 profile: ~740g/m
fn default_low_alert_m() -> f64 {
    50.0
}
fn default_sensor_poll_secs() -> u64 {
    30
}

impl Default for SensorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            empty_spool_kg: default_empty_spool_kg(),
            steel_kg_per_m: default_steel_kg_per_m(),
            low_alert_m: default_low_alert_m(),
            poll_interval_secs: default_sensor_poll_secs(),
        }
    }
}

/// Connection back to plat-trunk — same HTTP API regardless of topology.
///
/// Topology A (Cloud):  url = "https://your-worker.workers.dev"
/// Topology B (LAN):    url = "http://localhost:3000"
/// Topology C (Hybrid): url = "http://localhost:3000"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatTrunkConfig {
    pub url: String,
    pub api_key: String,
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
                usb_gadget_mode: false,
                delivery_mode: DeliveryMode::Direct,
            },
            http: HttpConfig {
                host: "0.0.0.0".to_string(),
                port: 4841,
            },
            plat_trunk: PlatTrunkConfig {
                url: "http://localhost:3000".to_string(),
                api_key: String::new(),
                status_push_interval_secs: 5,
            },
            sensor: SensorConfig::default(),
        }
    }
}

impl SensorConfig {
    /// Converts a raw weight reading (kg on load cell) to metres of steel remaining.
    ///
    /// Subtracts the empty spool weight then divides by the linear density.
    /// Returns 0.0 if the calculation is negative (coil exhausted or not fitted).
    #[allow(dead_code)]
    pub fn coil_metres(&self, raw_weight_kg: f64) -> f64 {
        let steel_kg = raw_weight_kg - self.empty_spool_kg;
        if steel_kg <= 0.0 {
            return 0.0;
        }
        (steel_kg / self.steel_kg_per_m).max(0.0)
    }
}

impl Config {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_or_default(path: &std::path::Path) -> Self {
        let mut config = match Self::load(path) {
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
        };
        // Allow env var overrides for common fields (useful in dev tasks)
        if let Ok(url) = std::env::var("PLAT_TRUNK_URL") {
            tracing::info!("PLAT_TRUNK_URL override: {url}");
            config.plat_trunk.url = url;
        }
        if let Ok(mode) = std::env::var("DELIVERY_MODE") {
            config.machine.delivery_mode = match mode.to_lowercase().as_str() {
                "queue" => DeliveryMode::Queue,
                "direct" => DeliveryMode::Direct,
                _ => config.machine.delivery_mode,
            };
            tracing::info!("DELIVERY_MODE override: {mode}");
        }
        config
    }

    pub fn topology(&self) -> &'static str {
        if self.plat_trunk.url.contains("localhost") || self.plat_trunk.url.contains("127.0.0.1") {
            "LAN"
        } else {
            "Cloud"
        }
    }
}
