use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum MachineStatus {
    Offline,
    Idle,
    Running,
    #[allow(dead_code)]
    Error(String),
}

impl MachineStatus {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            MachineStatus::Offline => "Offline",
            MachineStatus::Idle => "Idle",
            MachineStatus::Running => "Running",
            MachineStatus::Error(_) => "Error",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Job {
    pub id: String,
    pub frameset_name: String,
    pub csv_path: std::path::PathBuf,
    pub submitted_at: std::time::SystemTime,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct MachineState {
    pub status: MachineStatus,
    pub current_job: Option<String>,
    pub pieces_produced: u32,
    pub coil_remaining_m: f64,
    pub last_error: String,
    pub job_queue: Vec<Job>,
    pub completed_jobs: Vec<Job>,

    /// When a CSV was last uploaded via the web UI
    pub last_upload_at: Option<std::time::SystemTime>,
    /// When howick-agent last polled the local job queue (delivery_mode=queue)
    pub agent_last_seen_at: Option<std::time::SystemTime>,
    /// Last error reported by howick-agent via POST /api/jobs/howick/:id/error
    pub agent_last_error: String,

    // ── Phase 2: coil sensor ──────────────────────────────────────────────────
    /// When Pi Zero last pushed a coil weight reading (None = sensor not fitted)
    pub sensor_last_read_at: Option<std::time::SystemTime>,
    /// True when coil_remaining_m < sensor.low_alert_m
    pub coil_low_alert: bool,
}

impl MachineState {
    pub fn new() -> Self {
        Self {
            status: MachineStatus::Offline,
            current_job: None,
            pieces_produced: 0,
            coil_remaining_m: 0.0,
            last_error: String::new(),
            job_queue: Vec::new(),
            completed_jobs: Vec::new(),
            last_upload_at: None,
            agent_last_seen_at: None,
            agent_last_error: String::new(),
            sensor_last_read_at: None,
            coil_low_alert: false,
        }
    }
}

impl Default for MachineState {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedState = Arc<RwLock<MachineState>>;

pub fn new_shared_state() -> SharedState {
    Arc::new(RwLock::new(MachineState::new()))
}
