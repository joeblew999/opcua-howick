/// Integration tests for the self-update mechanism.
///
/// Runs the REAL updater code against a local mock HTTP server (raw tokio TCP,
/// same pattern as src/http_server.rs).
///
/// Tests:
///   - update downloads binary when a newer version is available
///   - update skips when already at the latest version
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use opcua_howick::updater::{check_and_update, target_triple};

// ── Counter for unique temp paths ─────────────────────────────────────────────

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn unique_install_path() -> std::path::PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("howick-update-test-{n}"))
}

// ── Mock server ───────────────────────────────────────────────────────────────

/// Start a mock GitHub Releases API server that serves:
///   GET /repos/joeblew999/opcua-howick/releases/latest
///       → JSON with tag_name and an asset pointing back at this server
///   GET /assets/{name}
///       → b"DUMMY" (simulated binary download)
///
/// Returns the bound port so the test can construct `api_base`.
async fn start_mock_server(tag_name: &'static str) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    // Build the asset name using the real target_triple() so the updater can find it.
    let asset_name = format!("howick-agent-{}", target_triple());
    let download_url = format!("http://127.0.0.1:{port}/assets/{asset_name}");

    // Build JSON response once; move it into the server task.
    let release_json = serde_json::json!({
        "tag_name": tag_name,
        "assets": [
            {
                "name": asset_name,
                "browser_download_url": download_url
            }
        ]
    })
    .to_string();

    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            let release_json = release_json.clone();
            let asset_name_clone = format!("howick-agent-{}", target_triple());

            tokio::spawn(async move {
                let mut buf = vec![0u8; 8192];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                buf.truncate(n);

                let raw = String::from_utf8_lossy(&buf);
                let first_line = raw.lines().next().unwrap_or("");
                let path = first_line.split_whitespace().nth(1).unwrap_or("/");

                let (status, content_type, body): (&str, &str, String) =
                    if path == "/repos/joeblew999/opcua-howick/releases/latest" {
                        ("200 OK", "application/json", release_json)
                    } else if path == format!("/assets/{asset_name_clone}") {
                        ("200 OK", "application/octet-stream", "DUMMY".to_string())
                    } else {
                        (
                            "404 Not Found",
                            "application/json",
                            r#"{"error":"not found"}"#.to_string(),
                        )
                    };

                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}",
                    len = body.len(),
                );
                let _ = stream.write_all(response.as_bytes()).await;
            });
        }
    });

    port
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// When the remote release has a higher version number (v99.0.0 > 0.1.0),
/// the updater should download the asset, write it to the install path, and
/// return Ok(true).
#[tokio::test]
async fn update_downloads_binary_when_newer_version_available() {
    let port = start_mock_server("v99.0.0").await;
    // Give the mock server a moment to start listening
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let install_path = unique_install_path();
    let api_base = format!("http://127.0.0.1:{port}");
    let client = reqwest::Client::new();

    let result = check_and_update(
        &client,
        "howick-agent",
        "0.1.0 (abc1234)",
        &api_base,
        Some(&install_path),
    )
    .await;

    assert!(
        result.is_ok(),
        "check_and_update returned error: {:?}",
        result
    );
    assert!(result.unwrap(), "expected Ok(true) — should have updated");

    // Verify the binary was written to the install path
    let written = tokio::fs::read(&install_path)
        .await
        .expect("install_path should exist after update");
    assert_eq!(
        written, b"DUMMY",
        "downloaded content should be DUMMY bytes"
    );

    // Cleanup
    let _ = tokio::fs::remove_file(&install_path).await;
}

/// When the remote release is at the same version (or lower), the updater
/// should do nothing and return Ok(false).
#[tokio::test]
async fn update_skips_when_already_at_latest_version() {
    // Mock server says "v0.1.0" — same as the current version we pass in
    let port = start_mock_server("v0.1.0").await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let install_path = unique_install_path();
    let api_base = format!("http://127.0.0.1:{port}");
    let client = reqwest::Client::new();

    let result = check_and_update(
        &client,
        "howick-agent",
        "0.1.0 (abc1234)",
        &api_base,
        Some(&install_path),
    )
    .await;

    assert!(
        result.is_ok(),
        "check_and_update returned error: {:?}",
        result
    );
    assert!(
        !result.unwrap(),
        "expected Ok(false) — should NOT have updated"
    );

    // install_path must NOT have been created
    assert!(
        !install_path.exists(),
        "install_path should not exist when no update was needed"
    );
}
