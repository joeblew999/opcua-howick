use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
pub enum MachineStatus {
    Offline,
    Idle,
    Running,
    #[allow(dead_code)]
    Error(String),
}

impl MachineStatus {
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
pub struct MachineState {
    pub status: MachineStatus,
    pub current_job: Option<String>,
    pub pieces_produced: u32,
    pub coil_remaining_m: f64,
    pub last_error: String,
    pub job_queue: Vec<Job>,
    pub completed_jobs: Vec<Job>,
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
