/// Capture git commit hash at build time so binaries report their exact source revision.
///
/// Injects `GIT_COMMIT_HASH` env var (e.g. "abc1234") available via `env!("GIT_COMMIT_HASH")`.
/// Falls back to "unknown" if git is not available (e.g. in a vendor/offline build).
fn main() {
    let hash = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=GIT_COMMIT_HASH={hash}");

    // Rebuild only when the git HEAD changes (commit or branch switch)
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");
}
