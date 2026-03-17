//! Mock plat-trunk backend for local development.
//!
//! Simulates the two HTTP endpoints that howick-agent polls:
//!
//!   GET  /api/jobs/howick/pending          → returns one test job, then empty
//!   POST /api/jobs/howick/{id}/complete    → acknowledges completion
//!
//! Usage (two terminals):
//!
//!   terminal 1:  mise run dev:mock    # this binary — listens on :3000
//!   terminal 2:  mise run dev:agent   # howick-agent polls :3000
//!
//! Then watch ./jobs/machine/ for the written CSV.

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

const TEST_JOB_ID: &str = "dev-001";
const TEST_FRAMESET: &str = "TEST-W1";
const TEST_CSV: &str = "UNIT,MILLIMETRE\n\
    PROFILE,S8908,Standard Profile\n\
    FRAMESET,TEST-W1\n\
    COMPONENT,TEST-W1-1,LABEL_NRM,1,2400.0,DIMPLE,20.65,DIMPLE,70.65\n\
    COMPONENT,TEST-W1-2,LABEL_NRM,1,1800.0,DIMPLE,20.65,DIMPLE,70.65\n";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().compact().init();

    let addr: SocketAddr = "0.0.0.0:3000".parse()?;
    let listener = TcpListener::bind(addr).await?;

    // Track whether the test job has been served and completed
    let job_served = Arc::new(Mutex::new(false));

    println!("Mock plat-trunk listening on http://localhost:3000");
    println!("  Test job: {TEST_JOB_ID} — {TEST_FRAMESET}");
    println!("  Start agent: mise run dev:agent");
    println!();

    loop {
        let (stream, peer) = listener.accept().await?;
        let job_served = job_served.clone();
        tokio::spawn(async move {
            if let Err(e) = handle(stream, peer, job_served).await {
                tracing::warn!("Connection error from {peer}: {e}");
            }
        });
    }
}

async fn handle(
    mut stream: tokio::net::TcpStream,
    _peer: SocketAddr,
    job_served: Arc<Mutex<bool>>,
) -> anyhow::Result<()> {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).await?;
    let req = std::str::from_utf8(&buf[..n]).unwrap_or("");

    let first_line = req.lines().next().unwrap_or("");
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/");

    let (code, body) = match (method, path) {
        ("GET", "/api/jobs/howick/pending") => {
            let served = *job_served.lock().await;
            if served {
                println!("  [mock] GET pending → empty (job already completed)");
                (200, r#"{"jobs":[]}"#.to_string())
            } else {
                println!("  [mock] GET pending → serving job {TEST_JOB_ID}");
                let csv_escaped = TEST_CSV.replace('\n', "\\n").replace('"', "\\\"");
                let body = format!(
                    r#"{{"jobs":[{{"job_id":"{TEST_JOB_ID}","frameset_name":"{TEST_FRAMESET}","csv":"{csv_escaped}"}}]}}"#
                );
                (200, body)
            }
        }

        ("POST", p) if p.contains("/complete") => {
            *job_served.lock().await = true;
            println!("  [mock] POST complete → acknowledged, queue now empty");
            (200, r#"{"ok":true}"#.to_string())
        }

        ("GET", "/health") => (200, r#"{"ok":true}"#.to_string()),

        _ => {
            tracing::debug!("404: {method} {path}");
            (404, r#"{"error":"not found"}"#.to_string())
        }
    };

    let reason = match code {
        200 => "OK",
        400 => "Bad Request",
        _ => "Not Found",
    };

    let response = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: application/json\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}",
        len = body.len()
    );

    stream.write_all(response.as_bytes()).await?;
    Ok(())
}
