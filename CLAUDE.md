# CLAUDE.md — opcua-howick

## Rules

### Use MCP tools — never shell out

Two MCP servers (`.mcp.json`) expose tools for the jdx dev stack. Always use them instead of Bash.

| Tool | MCP call | Notes |
|------|----------|-------|
| **mise** (tasks) | `mcp__mise__run_task { task: "test" }` | Never run `mise run` via Bash |
| **mise** (tools) | `mcp__mise__install_tool { tool: "node", version: "22" }` | |
| **fnox** (secrets) | `mcp__fnox__get_secret { name: "..." }` | Never hardcode/echo secrets |
| **fnox** (exec) | `mcp__fnox__exec { command: ["cargo", "run"] }` | Injects secrets as env vars |
| **pitchfork** | `mcp__mise__run_task { task: "dev:ps" }` | Via mise wrapper tasks |
| **hk** | — | Runs automatically on git hooks |

See `.claude/skills/mise/SKILL.md` for the full task reference.

### Dogfood — verify everything you change

After ANY change to mise tasks, configs, or scripts:
1. **Run the task** via `mcp__mise__run_task`
2. **Check the logs** — see logging rules below
3. **Run the test** — confirm pass
4. Never say "it works" without reading logs.

### Logging — where to look and how

Two types of output, two tools:

| What | Tool | Lifetime | How to check |
|------|------|----------|-------------|
| **Daemon output** (opcua-server, howick-frama, speckle) | pitchfork | Persistent (`~/.local/state/pitchfork/logs/`) | `mise run logs` / `logs:recent` / `logs:errors` |
| **Task output** (build, test, check) | pitchfork via `pitchfork run` | Persistent (same log store as daemons) | `mise run logs:task -- build` / `logs:task -- test` |

**After starting daemons** → always check: `mise run logs:recent`
**After a failure** → always check: `mise run logs:errors`
**To debug** → `mise run logs:debug` (restarts with RUST_LOG=debug)
**Remote device** → `mise run device:logs -- pi5` or `http://<device>:19876`
**Speckle daemons** → `mise run speckle:logs:recent` / `speckle:logs:errors`

**As Claude, when verifying changes:**
```
mcp__mise__run_task { task: "logs:recent" }       # after starting daemons
mcp__mise__run_task { task: "logs:errors" }       # after any failure
mcp__mise__run_task { task: "logs:task -- build" } # review build output
mcp__mise__run_task { task: "logs:task -- test" }  # review test output
```
Never say "it works" without checking `logs:errors` returns "No errors found".

### Clone async-opcua before any OPC UA work

```bash
git clone https://github.com/FreeOpcUa/async-opcua/ /private/tmp/async-opcua
```

Key files: `docs/client.md`, `docs/server.md`, `samples/simple-client/src/main.rs`, `samples/demo-server/src/methods.rs`. API docs: https://docs.rs/async-opcua/latest/opcua. Never reinvent what's already in async-opcua.

### General rules

- `mise run X` is the ONLY entry point — never tell the user to run pitchfork/hk/fnox/makers directly.
- Never write bash scripts — use inline TOML `run`, structured `run = [{ task }]`, or Duckscript via cargo-make.
- Pitchfork for anything long-running. Mise tasks for everything else.
- fnox for secrets. Never hardcode secrets in config files.

### Cross-platform tasks (cargo-make + Duckscript)

Tasks that need cross-platform scripting (conditionals, file ops) delegate to cargo-make via `Makefile.toml`. Duckscript is the scripting engine — no bash, no PowerShell, works on all OS.

**Pattern:** mise task → `makers <task> args` → Duckscript in Makefile.toml

```toml
# .mise/tasks/build.toml — mise stays as entry point
["build:bin"]
run = "makers build-bin ${usage_bin} ${usage_target} ${usage_cross} ${usage_profile}"

# Makefile.toml — Duckscript handles cross-platform logic
[tasks.build-bin]
script_runner = "@duckscript"
script = '''
# ... cross-platform logic here
'''
```

**5 tasks use this pattern:** `build-bin`, `deploy-windows`, `ship`, `docs-pdf`, `test-submit`

**Minor tasks use `run_windows`** instead (simpler): `restart`, `logs:errors`, `logs:debug`, `check`, `dev:setup`, `dev:job`

**SSH/device tasks stay bash** — they only run on Mac/Linux targeting Linux devices.

**Rules:**
- cargo-make args do NOT use `--` separator — pass args directly: `makers task arg1 arg2`
- Args arrive in Duckscript via `CARGO_MAKE_TASK_ARGS` (semicolon-separated)
- Use `workspace = false` on all custom tasks (we manage our own workspace)
- Never call `makers` directly — always go through `mise run`

---

## Architecture

### Two binaries

| Binary | Target | Role |
|--------|--------|------|
| `opcua-server` | Pi 5 / NUC / Mac | OPC UA server + HTTP server + file watcher + job poller |
| `howick-frama` | Pi Zero 2W | Subscribes to Pi 5 OPC UA server, writes CSV to USB gadget |

### Cargo workspace

Four crates in `crates/`: `core` (shared), `opcua-server` (Pi 5), `howick-frama` (Pi Zero), `mock-plat-trunk` (dev mock). `howick-frama` only compiles async-opcua client features — no server code on Pi Zero.

### OPC UA

OPC UA is the **primary transport** between Pi Zero and Pi 5. Pi 5 exposes machine state + job queue as OPC UA nodes. Pi Zero subscribes via standard OPC UA subscriptions — no polling. HTTP API (`job_server/http.rs`) is for the browser dashboard only.

### jdx ecosystem + cargo-make

Four jdx tools (same author), plus cargo-make for cross-platform scripting:

| Tool | Role | Config |
|------|------|--------|
| [mise](https://mise.jdx.dev) | Task runner + tool manager | `.mise.toml` + `.mise/tasks/*.toml` |
| [pitchfork](https://pitchfork.jdx.dev) | Daemon/process manager | `pitchfork.toml` |
| [hk](https://hk.jdx.dev) | Git hook manager | `hk.pkl` |
| [fnox](https://github.com/jdx/fnox) | Secret management (MCP-connected) | `fnox.toml` |
| [cargo-make](https://github.com/sagiegurari/cargo-make) | Cross-platform scripting (Duckscript) | `Makefile.toml` |

Same tools on every device (Mac, Pi 5, Pi Zero, Windows) — only config files differ. Per-device configs live in `config/`. Secrets flow from Infisical → fnox (age-encrypted locally, cloud-pulled in prod/CI).

### Speckle sub-projects

Three sub-projects in `tools/`, each with their own `.mise.toml`. Wrapper tasks in `.mise/tasks/speckle-*.toml` use `mise run -C` for tool isolation.

- **speckle-server/** — local Speckle instance (node, postgres, redis, minio via pitchfork)
- **speckle-docker/** — all-Docker fallback (slow on ARM Mac)
- **speckle-watcher/** — Python: SketchUp → Speckle → Howick CSV converter

---

## Commercial context

- **Free product** for the customer (Prin). Cost is hardware only.
- OPC UA is a key selling point — document it prominently.
- Always present new capabilities as "runs alongside" existing workflow — never scary.

---

## Running tests

```
mcp__mise__run_task { task: "test" }    # all Rust tests
mcp__mise__run_task { task: "ci" }      # full CI gate (check + fmt + test)
mcp__mise__run_task { task: "check" }   # clippy + check only
```
