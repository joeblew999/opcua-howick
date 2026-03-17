/// HTTP server exposing machine state over a simple JSON API.
///
/// The CF Worker (or Tauri local server) calls this to get machine status
/// for the plugin UI. This is the bridge from OPC UA state → HTTP → browser.
///
/// Endpoints:
///   GET  /status          Machine status JSON
///   POST /jobs            Submit a CSV job (alternative to file drop)
///   GET  /jobs            List recent jobs
use std::net::SocketAddr;

use tokio::net::TcpListener;

use crate::config::Config;
use crate::machine::SharedState;

/// Run the HTTP status server on config.http_port.
pub async fn run_http_server(config: &Config, state: SharedState) -> anyhow::Result<()> {
    let addr: SocketAddr = format!("{}:{}", config.http.host, config.http.port).parse()?;

    let listener = TcpListener::bind(addr).await?;
    tracing::info!("HTTP status server on http://{}/", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, state).await {
                tracing::warn!("HTTP connection error: {e}");
            }
        });
    }
}

async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    state: SharedState,
) -> anyhow::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).await?;
    let req = std::str::from_utf8(&buf[..n]).unwrap_or("");

    // Parse method and path from first line
    let first_line = req.lines().next().unwrap_or("");
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/");

    let (status_code, body) = match (method, path) {
        ("GET", "/status") | ("GET", "/status?") => {
            let s = state.read().await;
            let body = format!(
                r#"{{"status":"{status}","current_job":{current_job},"pieces_produced":{pieces},"queue_depth":{queue},"coil_remaining":{coil}}}"#,
                status = s.status.as_str(),
                current_job = s
                    .current_job
                    .as_deref()
                    .map(|j| format!("\"{j}\""))
                    .unwrap_or("null".into()),
                pieces = s.pieces_produced,
                queue = s.job_queue.len(),
                coil = s.coil_remaining_m,
            );
            (200, body)
        }

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
            (200, body)
        }

        ("POST", "/jobs") => {
            // Read body — extract CSV and frameset_name from JSON
            // For now just acknowledge — the file watcher handles actual CSV ingestion
            let body_start = req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(req.len());
            let json_body = &req[body_start..];
            if let Some(frameset_name) = extract_json_string(json_body, "frameset_name") {
                let job_id = format!("{}-{}", frameset_name, timestamp_secs());
                let body = format!(
                    r#"{{"job_id":"{job_id}","frameset_name":"{frameset_name}","status":"queued"}}"#
                );
                (200, body)
            } else {
                (400, r#"{"error":"missing frameset_name"}"#.into())
            }
        }

        ("GET", "/health") => (200, r#"{"ok":true}"#.into()),

        _ => (404, r#"{"error":"not found"}"#.into()),
    };

    let response = format!(
        "HTTP/1.1 {status_code} {reason}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}",
        status_code = status_code,
        reason      = if status_code == 200 { "OK" } else if status_code == 400 { "Bad Request" } else { "Not Found" },
        len         = body.len(),
        body        = body,
    );

    stream.write_all(response.as_bytes()).await?;
    Ok(())
}

/// Very simple JSON string extractor — avoids pulling in serde for this tiny server.
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let start = json.find(&needle)? + needle.len();
    let rest = json[start..].trim_start();
    let rest = rest.strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn timestamp_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
