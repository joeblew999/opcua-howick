/// Self-update mechanism — checks GitHub Releases API for a newer version,
/// downloads the asset, and atomically replaces the binary.
///
/// Uses only `reqwest` and `serde` (already in Cargo.toml).
///
/// In production:
///   - `api_base` = "https://api.github.com"
///   - `install_path` = None  (replaces the running binary via current_exe())
///
/// In tests:
///   - `api_base` = "http://127.0.0.1:{port}" (mock server)
///   - `install_path` = Some(&temp_path)       (avoids touching the real binary)
use serde::Deserialize;

/// Returns the Rust target triple for the current compile target.
/// This is used to select the correct asset from a GitHub release.
pub fn target_triple() -> &'static str {
    #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
    {
        "aarch64-unknown-linux-gnu"
    }
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    {
        "x86_64-unknown-linux-gnu"
    }
    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    {
        "aarch64-apple-darwin"
    }
    #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
    {
        "x86_64-apple-darwin"
    }
    #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
    {
        "x86_64-pc-windows-msvc"
    }
    #[cfg(not(any(
        all(target_arch = "aarch64", target_os = "linux"),
        all(target_arch = "x86_64", target_os = "linux"),
        all(target_arch = "aarch64", target_os = "macos"),
        all(target_arch = "x86_64", target_os = "macos"),
        all(target_arch = "x86_64", target_os = "windows"),
    )))]
    {
        "unknown"
    }
}

/// Parse a semver tag like "v1.2.3" or "1.2.3" into a (major, minor, patch) tuple.
/// Returns None if the string cannot be parsed.
fn parse_version(tag: &str) -> Option<(u64, u64, u64)> {
    let s = tag.trim_start_matches('v');
    let mut parts = s.splitn(3, '.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    // patch may have a suffix like "-beta" — only take the numeric prefix
    let patch_raw = parts.next().unwrap_or("0");
    let patch_numeric: String = patch_raw
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let patch = patch_numeric.parse::<u64>().unwrap_or(0);
    Some((major, minor, patch))
}

#[derive(Deserialize)]
struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Deserialize)]
struct LatestRelease {
    pub tag_name: String,
    pub assets: Vec<ReleaseAsset>,
}

/// Check GitHub Releases for a newer version.  Download and replace the binary
/// when a newer version is found.
///
/// # Arguments
/// - `client`          — shared `reqwest::Client`
/// - `bin_name`        — binary name, e.g. `"howick-frama"` or `"opcua-server"`
/// - `current_version` — currently running version string, e.g. `"0.1.0 (abc1234)"`
/// - `api_base`        — GitHub API base URL; `"https://api.github.com"` in production
/// - `install_path`    — `None` replaces the running executable; `Some(p)` writes to `p` (tests)
///
/// # Returns
/// - `Ok(true)`  — a newer version was found and the binary was replaced
/// - `Ok(false)` — already at the latest version; no action taken
/// - `Err(_)`    — network error, parse error, or I/O error
pub async fn check_and_update(
    client: &reqwest::Client,
    bin_name: &str,
    current_version: &str,
    api_base: &str,
    install_path: Option<&std::path::Path>,
) -> anyhow::Result<bool> {
    let url = format!("{api_base}/repos/joeblew999/opcua-howick/releases/latest");

    let release: LatestRelease = client
        .get(&url)
        .header("User-Agent", "opcua-howick-updater/1")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    // Strip any git hash suffix from current_version, e.g. "0.1.0 (abc1234)" → "0.1.0"
    let semver_current = current_version
        .split_whitespace()
        .next()
        .unwrap_or(current_version);

    let current_tuple = parse_version(semver_current)
        .ok_or_else(|| anyhow::anyhow!("Cannot parse current version: {semver_current}"))?;
    let remote_tuple = parse_version(&release.tag_name)
        .ok_or_else(|| anyhow::anyhow!("Cannot parse remote tag: {}", release.tag_name))?;

    if remote_tuple <= current_tuple {
        tracing::debug!(
            current = semver_current,
            remote = %release.tag_name,
            "Already at latest version — no update needed"
        );
        return Ok(false);
    }

    // Find the matching asset: {bin_name}-{target_triple()}
    let asset_name = format!("{bin_name}-{}", target_triple());
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No asset named '{asset_name}' in release {}",
                release.tag_name
            )
        })?;

    tracing::info!(
        current = semver_current,
        remote = %release.tag_name,
        asset = %asset.name,
        "Newer version available — downloading"
    );

    let bytes = client
        .get(&asset.browser_download_url)
        .header("User-Agent", "opcua-howick-updater/1")
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    // Determine where to write the new binary
    let dest = match install_path {
        Some(p) => p.to_path_buf(),
        None => std::env::current_exe()
            .map_err(|e| anyhow::anyhow!("Cannot determine current exe path: {e}"))?,
    };

    // Write to a temp file first, then atomically rename — avoids a broken binary
    // if we are interrupted mid-write.
    let tmp = dest.with_extension("tmp-update");
    tokio::fs::write(&tmp, &bytes).await?;

    // Make the new binary executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&tmp).await?.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&tmp, perms).await?;
    }

    tokio::fs::rename(&tmp, &dest).await?;

    tracing::info!(
        path = %dest.display(),
        version = %release.tag_name,
        "Binary replaced — exit 0 so systemd can restart with new version"
    );

    Ok(true)
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_basic() {
        assert_eq!(parse_version("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("0.1.0"), Some((0, 1, 0)));
        assert_eq!(parse_version("v99.0.0"), Some((99, 0, 0)));
    }

    #[test]
    fn parse_version_with_suffix() {
        // patch suffix is stripped, only numeric prefix kept
        assert_eq!(parse_version("v1.2.3-beta"), Some((1, 2, 3)));
    }

    #[test]
    fn parse_version_with_hash() {
        // "0.1.0 (abc1234)" → strip space → "0.1.0"
        let raw = "0.1.0 (abc1234)";
        let semver = raw.split_whitespace().next().unwrap();
        assert_eq!(parse_version(semver), Some((0, 1, 0)));
    }

    #[test]
    fn target_triple_is_not_empty() {
        assert!(!target_triple().is_empty());
    }

    #[test]
    fn version_comparison_semantics() {
        let current = parse_version("0.1.0").unwrap();
        let newer = parse_version("v99.0.0").unwrap();
        let same = parse_version("0.1.0").unwrap();
        assert!(newer > current);
        assert!(same == current);
        assert!(current <= same);
    }
}
