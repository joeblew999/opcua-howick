use std::path::Path;
use std::time::SystemTime;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use crate::config::MachineConfig;
use crate::machine::{Job, MachineStatus, SharedState};

/// Watch the job input directory for new CSV files.
/// When a CSV arrives, copy it to the machine input directory and update state.
pub async fn run_job_watcher(config: MachineConfig, state: SharedState) -> anyhow::Result<()> {
    // Ensure directories exist
    tokio::fs::create_dir_all(&config.job_input_dir).await?;
    tokio::fs::create_dir_all(&config.machine_input_dir).await?;
    tokio::fs::create_dir_all(&config.machine_output_dir).await?;

    tracing::info!("Watching for jobs in: {}", config.job_input_dir.display());
    tracing::info!("Machine input dir:    {}", config.machine_input_dir.display());

    let (tx, mut rx) = mpsc::channel(32);

    // Set up file system watcher
    let mut watcher = RecommendedWatcher::new(
        move |result: notify::Result<Event>| {
            if let Ok(event) = result {
                let _ = tx.blocking_send(event);
            }
        },
        notify::Config::default(),
    )?;

    watcher.watch(&config.job_input_dir, RecursiveMode::NonRecursive)?;

    // Process file system events
    while let Some(event) = rx.recv().await {
        if let EventKind::Create(_) = event.kind {
            for path in event.paths {
                if is_csv(&path) {
                    tracing::info!("New job file detected: {}", path.display());
                    if let Err(e) = handle_new_job(&path, &config, &state).await {
                        tracing::error!("Failed to process job {}: {e}", path.display());
                    }
                }
            }
        }
    }

    Ok(())
}

fn is_csv(path: &Path) -> bool {
    path.extension()
        .map(|e| e.eq_ignore_ascii_case("csv"))
        .unwrap_or(false)
}

async fn handle_new_job(
    csv_path: &Path,
    config: &MachineConfig,
    state: &SharedState,
) -> anyhow::Result<()> {
    let filename = csv_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.csv");

    // Derive job ID and frameset name from filename (e.g. "W1.csv" → "W1")
    let frameset_name = csv_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    let job_id = format!("{}-{}", frameset_name, timestamp_id());

    let job = Job {
        id: job_id.clone(),
        frameset_name: frameset_name.clone(),
        csv_path: csv_path.to_path_buf(),
        submitted_at: SystemTime::now(),
    };

    // Add to queue in shared state
    {
        let mut s = state.write().await;
        s.job_queue.push(job);
        tracing::info!("Job {} queued (queue depth: {})", job_id, s.job_queue.len());
    }

    // Copy CSV to machine input directory
    let dest = config.machine_input_dir.join(filename);
    tokio::fs::copy(csv_path, &dest).await?;
    tracing::info!("CSV copied to machine input: {}", dest.display());

    // Update state to running
    {
        let mut s = state.write().await;
        s.status = MachineStatus::Running;
        s.current_job = Some(frameset_name.clone());
        // Move from queue to in-progress (simplified: pop from queue)
        if let Some(pos) = s.job_queue.iter().position(|j| j.id == job_id) {
            let job = s.job_queue.remove(pos);
            s.completed_jobs.push(job);
        }
        s.status = MachineStatus::Idle; // Will be set properly once output monitoring exists
        tracing::info!("Job {} submitted to machine", job_id);
    }

    Ok(())
}

fn timestamp_id() -> String {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
