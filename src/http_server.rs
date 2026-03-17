/// HTTP server — pipeline dashboard, operator upload UI, JSON API, and
/// plat-trunk job endpoints for howick-agent (Pi Zero).
///
/// Endpoints:
///   GET  /                              → redirect to /dashboard
///   GET  /dashboard                     → full pipeline status UI (auto-refreshes)
///   POST /upload                        → accept raw CSV; X-Filename header names the job
///   GET  /status                        → machine state JSON
///   GET  /jobs                          → queued + completed jobs JSON
///   GET  /health                        → health check
///
///   — plat-trunk API (called by howick-agent on Pi Zero) —
///   GET  /api/jobs/howick/pending       → next queued job for the agent
///   POST /api/jobs/howick/:id/complete  → agent marks job delivered to USB

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::SystemTime;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::config::Config;
use crate::machine::SharedState;

pub async fn run_http_server(config: &Config, state: SharedState) -> anyhow::Result<()> {
    let addr: SocketAddr = format!("{}:{}", config.http.host, config.http.port).parse()?;
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(
        "HTTP server on http://{}/ — dashboard at http://{}/dashboard",
        addr,
        addr
    );

    let job_input_dir = config.machine.job_input_dir.clone();

    loop {
        let (stream, _) = listener.accept().await?;
        let state = state.clone();
        let job_input_dir = job_input_dir.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, state, job_input_dir).await {
                tracing::warn!("HTTP connection error: {e}");
            }
        });
    }
}

async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    state: SharedState,
    job_input_dir: PathBuf,
) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 2 * 1024 * 1024];
    let n = stream.read(&mut buf).await?;
    buf.truncate(n);

    let raw = String::from_utf8_lossy(&buf);
    let header_end = raw.find("\r\n\r\n").unwrap_or(n);
    let headers_str = &raw[..header_end];
    let body_bytes = &buf[(header_end + 4).min(n)..];

    let first_line = headers_str.lines().next().unwrap_or("");
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/");

    let header = |name: &str| -> Option<&str> {
        let needle = name.to_ascii_lowercase();
        headers_str.lines().skip(1).find_map(|line| {
            let colon = line.find(':')?;
            if line[..colon].trim().to_ascii_lowercase() == needle {
                Some(line[colon + 1..].trim())
            } else {
                None
            }
        })
    };

    let (status_code, content_type, body) = match (method, path) {
        // ── Dashboard (full pipeline view) ────────────────────────────────────
        ("GET", "/") => ("301", "text/plain", String::new()),

        ("GET", "/dashboard") | ("GET", "/upload") => {
            let html = dashboard_page();
            ("200", "text/html; charset=utf-8", html)
        }

        // ── CSV upload from browser ────────────────────────────────────────────
        ("POST", "/upload") => {
            let filename = header("x-filename")
                .map(sanitise_filename)
                .unwrap_or_else(|| "upload.csv".into());
            let filename = if filename.ends_with(".csv") {
                filename
            } else {
                format!("{filename}.csv")
            };

            let csv = std::str::from_utf8(body_bytes)
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            if csv.is_empty() || !csv.starts_with("UNIT") {
                tracing::warn!("Upload rejected — not a valid Howick CSV ({})", filename);
                (
                    "400",
                    "application/json",
                    r#"{"error":"not a valid Howick CSV — file must start with UNIT,MILLIMETRE"}"#
                        .into(),
                )
            } else {
                tokio::fs::create_dir_all(&job_input_dir).await?;
                let dest = job_input_dir.join(&filename);
                tokio::fs::write(&dest, csv.as_bytes()).await?;
                tracing::info!("Uploaded: {} → {}", filename, dest.display());

                {
                    let mut s = state.write().await;
                    s.last_upload_at = Some(SystemTime::now());
                }

                let frameset = filename.trim_end_matches(".csv");
                let body =
                    format!(r#"{{"ok":true,"frameset_name":"{frameset}","queued":true}}"#);
                ("200", "application/json", body)
            }
        }

        // ── Status JSON ────────────────────────────────────────────────────────
        ("GET", "/status") | ("GET", "/status?") => {
            let s = state.read().await;
            let upload_secs = ago_secs(s.last_upload_at);
            let agent_secs = ago_secs(s.agent_last_seen_at);
            let body = format!(
                concat!(
                    r#"{{"status":"{status}","current_job":{current_job},"#,
                    r#""pieces_produced":{pieces},"queue_depth":{queue},"#,
                    r#""coil_remaining":{coil},"last_error":"{error}","#,
                    r#""last_upload_secs_ago":{upload},"completed_count":{completed},"#,
                    r#""agent_last_seen_secs_ago":{agent},"agent_last_error":"{agent_err}"}}"#,
                ),
                status      = s.status.as_str(),
                current_job = s.current_job.as_deref()
                    .map(|j| format!("\"{j}\""))
                    .unwrap_or("null".into()),
                pieces    = s.pieces_produced,
                queue     = s.job_queue.len(),
                coil      = s.coil_remaining_m,
                error     = s.last_error,
                upload    = upload_secs.map(|v| v.to_string()).unwrap_or("null".into()),
                completed = s.completed_jobs.len(),
                agent     = agent_secs.map(|v| v.to_string()).unwrap_or("null".into()),
                agent_err = s.agent_last_error,
            );
            ("200", "application/json", body)
        }

        // ── Jobs JSON ──────────────────────────────────────────────────────────
        ("GET", "/jobs") => {
            let s = state.read().await;
            let completed: Vec<String> = s
                .completed_jobs
                .iter()
                .rev()
                .take(10)
                .map(|j| {
                    format!(
                        r#"{{"id":"{}","frameset_name":"{}"}}"#,
                        j.id, j.frameset_name
                    )
                })
                .collect();
            let queued: Vec<String> = s
                .job_queue
                .iter()
                .map(|j| {
                    format!(
                        r#"{{"id":"{}","frameset_name":"{}"}}"#,
                        j.id, j.frameset_name
                    )
                })
                .collect();
            let body = format!(
                r#"{{"queued":[{}],"completed":[{}]}}"#,
                queued.join(","),
                completed.join(","),
            );
            ("200", "application/json", body)
        }

        // ── Local job queue — Path 1 Topology B/C ────────────────────────────
        // howick-agent (Pi Zero) polls these when delivery_mode=queue and
        // plat_trunk.url points at this machine instead of plat-trunk.
        // Same API shape as plat-trunk so howick-agent works unchanged.
        ("GET", "/api/jobs/howick/pending") => {
            let mut s = state.write().await;
            s.agent_last_seen_at = Some(SystemTime::now());

            if let Some(job) = s.job_queue.first() {
                let id = job.id.clone();
                let frameset = job.frameset_name.clone();
                let csv_path = job.csv_path.clone();
                drop(s);
                match tokio::fs::read_to_string(&csv_path).await {
                    Ok(csv) => {
                        let escaped = csv
                            .replace('\\', "\\\\")
                            .replace('"', "\\\"")
                            .replace('\r', "")
                            .replace('\n', "\\n");
                        tracing::info!("Agent polled — serving job {id} ({frameset})");
                        let body = format!(
                            r#"{{"jobs":[{{"job_id":"{id}","frameset_name":"{frameset}","csv":"{escaped}"}}]}}"#
                        );
                        ("200", "application/json", body)
                    }
                    Err(e) => {
                        tracing::error!("Could not read CSV for job {id}: {e}");
                        ("500", "application/json", r#"{"error":"csv read failed"}"#.into())
                    }
                }
            } else {
                drop(s);
                tracing::debug!("Agent polled — queue empty");
                ("200", "application/json", r#"{"jobs":[]}"#.into())
            }
        }

        ("POST", p) if p.starts_with("/api/jobs/howick/") && p.ends_with("/complete") => {
            let job_id = p
                .trim_start_matches("/api/jobs/howick/")
                .trim_end_matches("/complete")
                .to_string();
            let mut s = state.write().await;
            s.agent_last_seen_at = Some(SystemTime::now());
            if let Some(pos) = s.job_queue.iter().position(|j| j.id == job_id) {
                let job = s.job_queue.remove(pos);
                tracing::info!("Agent confirmed delivery: {} ({})", job_id, job.frameset_name);
                s.completed_jobs.push(job);
                s.status = crate::machine::MachineStatus::Idle;
                s.current_job = None;
            }
            drop(s);
            ("200", "application/json", r#"{"ok":true}"#.into())
        }

        ("POST", p) if p.starts_with("/api/jobs/howick/") && p.ends_with("/error") => {
            let err = std::str::from_utf8(body_bytes).unwrap_or("unknown").trim().to_string();
            let mut s = state.write().await;
            s.agent_last_seen_at = Some(SystemTime::now());
            s.agent_last_error = err.clone();
            drop(s);
            tracing::warn!("Agent reported error: {err}");
            ("200", "application/json", r#"{"ok":true}"#.into())
        }

        ("GET", "/health") => ("200", "application/json", r#"{"ok":true}"#.into()),

        _ => ("404", "application/json", r#"{"error":"not found"}"#.into()),
    };

    let (numeric_code, reason) = match status_code {
        "200" => (200u16, "OK"),
        "301" => (301, "Moved Permanently"),
        "400" => (400, "Bad Request"),
        "500" => (500, "Internal Server Error"),
        _ => (404, "Not Found"),
    };

    let mut response = format!(
        "HTTP/1.1 {numeric_code} {reason}\r\nContent-Type: {content_type}\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {len}\r\nConnection: close\r\n",
        len = body.len(),
    );
    if numeric_code == 301 {
        response.push_str("Location: /dashboard\r\n");
    }
    response.push_str("\r\n");
    response.push_str(&body);

    stream.write_all(response.as_bytes()).await?;
    Ok(())
}

/// Seconds since a SystemTime, or None if not set.
fn ago_secs(t: Option<SystemTime>) -> Option<u64> {
    t.and_then(|t| t.elapsed().ok()).map(|d| d.as_secs())
}

fn sanitise_filename(raw: &str) -> String {
    raw.chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.'))
        .collect()
}


fn dashboard_page() -> String {
    // The page fetches /status every 2s and re-renders the pipeline nodes in JS.
    r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Howick Pipeline</title>
<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: system-ui, sans-serif; background: #0f172a; color: #e2e8f0; min-height: 100vh; padding: 1.5rem 1rem 3rem; }
h1 { font-size: 1.3rem; font-weight: 700; margin-bottom: 0.2rem; }
.subtitle { color: #64748b; font-size: 0.8rem; margin-bottom: 2rem; }

/* Pipeline */
.pipeline { display: flex; align-items: stretch; gap: 0; margin-bottom: 2rem; flex-wrap: wrap; gap: 0.5rem; }
.node { flex: 1; min-width: 140px; background: #1e293b; border-radius: 0.75rem; padding: 1rem; border: 2px solid #334155; position: relative; transition: border-color 0.3s; }
.node.ok     { border-color: #22c55e33; }
.node.warn   { border-color: #f59e0b33; }
.node.error  { border-color: #ef444433; }
.node-icon   { font-size: 1.6rem; margin-bottom: 0.4rem; }
.node-name   { font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.06em; color: #64748b; margin-bottom: 0.2rem; }
.node-status { font-size: 1rem; font-weight: 700; margin-bottom: 0.5rem; }
.node-detail { font-size: 0.78rem; color: #94a3b8; line-height: 1.5; }
.node-error  { font-size: 0.75rem; color: #f87171; margin-top: 0.4rem; background: #7f1d1d33; padding: 0.3rem 0.5rem; border-radius: 0.3rem; word-break: break-word; }
.arrow { display: flex; align-items: center; font-size: 1.4rem; color: #334155; padding: 0 0.25rem; flex-shrink: 0; align-self: center; }

/* Jobs */
.section { margin-bottom: 1.5rem; }
.section h2 { font-size: 0.85rem; text-transform: uppercase; letter-spacing: 0.06em; color: #64748b; margin-bottom: 0.75rem; }
.job-list { display: flex; flex-direction: column; gap: 0.4rem; }
.job-row { display: flex; align-items: center; gap: 0.75rem; background: #1e293b; border-radius: 0.5rem; padding: 0.6rem 0.85rem; font-size: 0.85rem; }
.job-badge { font-size: 0.7rem; padding: 0.15rem 0.5rem; border-radius: 999px; font-weight: 600; }
.badge-queued    { background: #1d4ed8; color: #bfdbfe; }
.badge-done      { background: #14532d; color: #86efac; }
.empty-state { color: #475569; font-size: 0.85rem; padding: 0.5rem 0; }

/* Upload */
.upload-section { margin-top: 2rem; }
.drop-zone { border: 2px dashed #334155; border-radius: 1rem; padding: 2.5rem 2rem; text-align: center; cursor: pointer; transition: border-color 0.15s, background 0.15s; background: #1e293b; }
.drop-zone.drag-over { border-color: #3b82f6; background: #1e3a5f; }
.drop-zone.success   { border-color: #22c55e; background: #14532d22; }
.drop-zone.errstate  { border-color: #ef4444; background: #7f1d1d22; }
.drop-icon { font-size: 2.5rem; margin-bottom: 0.75rem; }
.drop-text { color: #94a3b8; font-size: 0.9rem; line-height: 1.6; }
.drop-text strong { color: #e2e8f0; }
#file-input { display: none; }
.pick-btn { display: inline-block; margin-top: 1rem; padding: 0.55rem 1.4rem; background: #3b82f6; color: white; border-radius: 0.5rem; font-size: 0.875rem; font-weight: 600; cursor: pointer; border: none; }
.pick-btn:hover { background: #2563eb; }
#upload-msg { margin-top: 1rem; font-size: 0.9rem; min-height: 1.4rem; text-align: center; color: #94a3b8; }
#upload-msg.ok  { color: #22c55e; }
#upload-msg.err { color: #ef4444; }

.refresh-note { text-align: right; font-size: 0.7rem; color: #334155; margin-bottom: 0.5rem; }
</style>
</head>
<body>

<h1>Howick Pipeline</h1>
<p class="subtitle">ubuntu Software — live job delivery status</p>

<p class="refresh-note" id="refresh-note">Connecting…</p>

<div class="pipeline" id="pipeline">
  <!-- rendered by JS -->
</div>

<div class="section">
  <h2>Job Queue</h2>
  <div class="job-list" id="queued-list"><p class="empty-state">Loading…</p></div>
</div>

<div class="section">
  <h2>Recent Completions</h2>
  <div class="job-list" id="completed-list"><p class="empty-state">Loading…</p></div>
</div>

<div class="upload-section">
  <div class="section"><h2>Upload Job</h2></div>
  <div class="drop-zone" id="drop-zone">
    <div class="drop-icon">📂</div>
    <p class="drop-text"><strong>Drag CSV here</strong><br>or click to pick from FrameBuilderMRD</p>
    <button class="pick-btn" onclick="document.getElementById('file-input').click()">Choose CSV file</button>
    <input type="file" id="file-input" accept=".csv,text/csv,text/plain">
  </div>
  <div id="upload-msg"></div>
</div>

<script>
// ── Pipeline rendering ─────────────────────────────────────────────────────

function nodeClass(status) {
  if (!status || status === 'Offline' || status === 'Never seen') return '';
  if (status === 'Online' || status === 'Idle' || status === 'Running') return 'ok';
  if (status === 'Warn' || status === 'Stale') return 'warn';
  return 'error';
}

function colour(status) {
  const map = {
    'Online': '#22c55e', 'Idle': '#3b82f6', 'Running': '#22c55e',
    'Stale': '#f59e0b', 'Offline': '#ef4444', 'Never seen': '#6b7280',
    'Error': '#ef4444',
  };
  return map[status] || '#94a3b8';
}

function agentStatus(secsAgo) {
  if (secsAgo === null) return 'Never seen';
  if (secsAgo < 30)   return 'Online';
  if (secsAgo < 120)  return 'Stale';
  return 'Offline';
}

function ago(secs) {
  if (secs === null) return '—';
  if (secs < 60)  return secs + 's ago';
  if (secs < 3600) return Math.floor(secs / 60) + 'm ago';
  return Math.floor(secs / 3600) + 'h ago';
}

function renderNode(icon, name, status, details, error) {
  const cls = nodeClass(status);
  const col = colour(status);
  const errHtml = error ? `<div class="node-error">⚠ ${error}</div>` : '';
  const detailHtml = details.map(d => `<div>${d}</div>`).join('');
  return `<div class="node ${cls}">
    <div class="node-icon">${icon}</div>
    <div class="node-name">${name}</div>
    <div class="node-status" style="color:${col}">${status}</div>
    <div class="node-detail">${detailHtml}</div>
    ${errHtml}
  </div>`;
}

function renderPipeline(s, jobs) {
  const uploadAgo = ago(s.last_upload_secs_ago);
  const agentSt   = agentStatus(s.agent_last_seen_secs_ago);
  const agentAgo  = ago(s.agent_last_seen_secs_ago);
  const lastDone  = jobs.completed.length > 0 ? jobs.completed[0].frameset_name : '—';

  const nodes = [
    renderNode('💻', 'Design PC', s.last_upload_secs_ago !== null ? 'Online' : 'Waiting',
      [`Last upload: ${uploadAgo}`], ''),
    '<div class="arrow">→</div>',
    renderNode('🖥️', 'opcua-howick', s.status,
      [`Queue: ${s.queue_depth} job${s.queue_depth !== 1 ? 's' : ''}`,
       `Done: ${s.completed_count}`],
      s.last_error || ''),
    '<div class="arrow">→</div>',
    renderNode('🔌', 'Pi Zero / USB', agentSt,
      [`Last seen: ${agentAgo}`],
      s.agent_last_error || ''),
    '<div class="arrow">→</div>',
    renderNode('🏭', 'Howick FRAMA', lastDone !== '—' ? 'Active' : 'Waiting',
      [`Last job: ${lastDone}`,
       s.current_job ? `Running: ${s.current_job}` : 'Idle'],
      ''),
  ];

  document.getElementById('pipeline').innerHTML = nodes.join('');
}

function renderJobs(jobs) {
  const ql = document.getElementById('queued-list');
  const cl = document.getElementById('completed-list');

  if (jobs.queued.length === 0) {
    ql.innerHTML = '<p class="empty-state">No jobs queued</p>';
  } else {
    ql.innerHTML = jobs.queued.map(j =>
      `<div class="job-row"><span class="job-badge badge-queued">Queued</span>${j.frameset_name} <span style="color:#475569;font-size:0.75rem">${j.id}</span></div>`
    ).join('');
  }

  if (jobs.completed.length === 0) {
    cl.innerHTML = '<p class="empty-state">No completed jobs yet</p>';
  } else {
    cl.innerHTML = jobs.completed.map(j =>
      `<div class="job-row"><span class="job-badge badge-done">Done</span>${j.frameset_name} <span style="color:#475569;font-size:0.75rem">${j.id}</span></div>`
    ).join('');
  }
}

async function refresh() {
  try {
    const [statusRes, jobsRes] = await Promise.all([
      fetch('/status'),
      fetch('/jobs'),
    ]);
    const s    = await statusRes.json();
    const jobs = await jobsRes.json();
    renderPipeline(s, jobs);
    renderJobs(jobs);
    const now = new Date().toLocaleTimeString();
    document.getElementById('refresh-note').textContent = 'Updated ' + now;
  } catch (e) {
    document.getElementById('refresh-note').textContent = 'Connection error — retrying…';
  }
}

refresh();
setInterval(refresh, 2000);

// ── Upload ─────────────────────────────────────────────────────────────────

const zone  = document.getElementById('drop-zone');
const input = document.getElementById('file-input');
const msg   = document.getElementById('upload-msg');

zone.addEventListener('dragover',  e => { e.preventDefault(); zone.classList.add('drag-over'); });
zone.addEventListener('dragleave', ()  => zone.classList.remove('drag-over'));
zone.addEventListener('drop', e => {
  e.preventDefault();
  zone.classList.remove('drag-over');
  if (e.dataTransfer.files[0]) upload(e.dataTransfer.files[0]);
});
input.addEventListener('change', () => { if (input.files[0]) upload(input.files[0]); });

async function upload(file) {
  msg.className = '';
  msg.textContent = 'Uploading ' + file.name + '…';
  zone.className = 'drop-zone';
  try {
    const text = await file.text();
    const res  = await fetch('/upload', {
      method: 'POST',
      headers: { 'Content-Type': 'text/plain', 'X-Filename': file.name },
      body: text,
    });
    const json = await res.json();
    if (json.ok) {
      zone.classList.add('success');
      msg.className = 'ok';
      msg.textContent = '✓ ' + file.name + ' queued — the machine will pick it up shortly.';
      refresh();
    } else {
      zone.classList.add('errstate');
      msg.className = 'err';
      msg.textContent = '✗ ' + (json.error || 'Upload failed');
    }
  } catch (e) {
    zone.classList.add('errstate');
    msg.className = 'err';
    msg.textContent = '✗ Network error: ' + e.message;
  }
  input.value = '';
}
</script>
</body>
</html>"#.to_string()
}

#[allow(dead_code)]
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let start = json.find(&needle)? + needle.len();
    let rest = json[start..].trim_start();
    let rest = rest.strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

