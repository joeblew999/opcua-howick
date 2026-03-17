/// Job poller — polls the plat-trunk backend for pending Howick jobs.
///
/// This is the cloud-topology counterpart to the file watcher.
/// In all topologies, opcua-howick calls the same HTTP API:
///
/// ```text
/// Topology A (Cloud):  polls https://your-worker.workers.dev/api/jobs/howick/pending
/// Topology B/C (LAN):  polls http://localhost:3000/api/jobs/howick/pending
/// ```
///
/// The CF Worker stores jobs in R2 at `jobs/howick/{job_id}.csv`.
/// This poller fetches pending jobs, writes CSV to the machine input directory,
/// then marks them as completed via the API.
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::config::{Config, MachineConfig, PlatTrunkConfig};
use crate::machine::{Job, MachineStatus, SharedState};

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct PendingJobsResponse {
    jobs: Vec<PendingJob>,
}

#[derive(Debug, Deserialize)]
struct PendingJob {
    job_id: String,
    frameset_name: String,
    csv: String,
}

#[derive(Debug, Serialize)]
struct CompleteJobRequest {
    job_id: String,
    status: String,
}

// ── Poller ────────────────────────────────────────────────────────────────────

/// Poll the plat-trunk API for pending jobs and process them.
///
/// Runs continuously, polling every `config.plat_trunk.status_push_interval_secs`.
/// Each pending job is:
///   1. Written as a CSV to `machine_input_dir` (picked up by Howick machine)
///   2. Marked complete via POST /api/jobs/howick/{job_id}/complete
///   3. Added to SharedState completed_jobs
pub async fn run_job_poller(config: Config, state: SharedState) -> anyhow::Result<()> {
    let interval = Duration::from_secs(config.plat_trunk.status_push_interval_secs);
    let client = build_client(&config.plat_trunk)?;
    let base_url = config.plat_trunk.url.trim_end_matches('/').to_string();

    tracing::info!(
        url      = %base_url,
        interval = ?interval,
        "Job poller started"
    );

    loop {
        if let Err(e) = poll_once(
            &client,
            &base_url,
            &config.plat_trunk,
            &config.machine,
            &state,
        )
        .await
        {
            tracing::warn!("Poll error (will retry): {e}");
        }
        tokio::time::sleep(interval).await;
    }
}

async fn poll_once(
    client: &reqwest::Client,
    base_url: &str,
    pt_config: &PlatTrunkConfig,
    mc_config: &MachineConfig,
    state: &SharedState,
) -> anyhow::Result<()> {
    let url = format!("{base_url}/api/jobs/howick/pending");

    let mut req = client.get(&url);
    if !pt_config.api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", pt_config.api_key));
    }

    let resp = req.send().await?;
    if !resp.status().is_success() {
        // Backend may not be up yet — not an error worth logging loudly
        tracing::debug!(status = %resp.status(), "Pending jobs endpoint returned non-200");
        return Ok(());
    }

    let pending: PendingJobsResponse = resp.json().await?;

    if pending.jobs.is_empty() {
        return Ok(());
    }

    tracing::info!(count = pending.jobs.len(), "Fetched pending jobs");

    for job in pending.jobs {
        if let Err(e) = process_job(&job, mc_config, state).await {
            tracing::error!(job_id = %job.job_id, "Failed to process job: {e}");
            continue;
        }
        // Mark complete on the backend
        let complete_url = format!("{base_url}/api/jobs/howick/{}/complete", job.job_id);
        let mut req = client.post(&complete_url).json(&CompleteJobRequest {
            job_id: job.job_id.clone(),
            status: "completed".into(),
        });
        if !pt_config.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", pt_config.api_key));
        }
        if let Err(e) = req.send().await {
            tracing::warn!(job_id = %job.job_id, "Failed to mark job complete: {e}");
        } else {
            tracing::info!(job_id = %job.job_id, "Job marked complete");
        }
    }

    Ok(())
}

async fn process_job(
    job: &PendingJob,
    mc_config: &MachineConfig,
    state: &SharedState,
) -> anyhow::Result<()> {
    // Write CSV to machine input directory (handles USB gadget refresh if configured)
    let filename = format!("{}.csv", job.frameset_name);
    crate::usb_gadget::write_job(mc_config, &filename, &job.csv).await?;

    let dest = mc_config.machine_input_dir.join(&filename);
    tracing::info!(
        job_id        = %job.job_id,
        frameset_name = %job.frameset_name,
        dest          = %dest.display(),
        "CSV written to machine input"
    );

    // Update shared state
    {
        let mut s = state.write().await;
        s.status = MachineStatus::Running;
        s.current_job = Some(job.frameset_name.clone());
        s.completed_jobs.push(Job {
            id: job.job_id.clone(),
            frameset_name: job.frameset_name.clone(),
            csv_path: dest,
            submitted_at: std::time::SystemTime::now(),
        });
        // Reset to idle — real status tracking needs machine output monitoring (Phase 2)
        s.status = MachineStatus::Idle;
        s.current_job = None;
    }

    Ok(())
}

fn build_client(_config: &PlatTrunkConfig) -> anyhow::Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(concat!("opcua-howick/", env!("CARGO_PKG_VERSION")));

    // Use rustls (no OpenSSL dep) — important for clean cross-compilation to Pi
    builder = builder.use_rustls_tls();

    Ok(builder.build()?)
}
