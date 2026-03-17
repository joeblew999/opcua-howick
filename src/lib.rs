// Shared library — exposes all modules for both binaries and integration tests.

/// Full version string embedded at compile time: `"0.1.0 (abc1234)"`.
///
/// - The semver part comes from `Cargo.toml [package] version` — single source of truth.
/// - The hash comes from `build.rs` via `GIT_COMMIT_HASH` — pinpoints the exact commit.
///
/// Use this in startup logs, `--version` output, and the HTTP `/status` response
/// so any deployed binary can be identified without SSH access.
pub const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("GIT_COMMIT_HASH"),
    ")"
);

pub mod config;
pub mod http_server;
pub mod machine;
pub mod opcua_client;
pub mod poller;
pub mod sensor;
pub mod server;
pub mod usb_gadget;
pub mod watcher;
