# opcua-howick

<https://github.com/joeblew999/opcua-howick>

Automates job delivery to Howick FRAMA roll-forming machines — eliminates the USB-stick walk.

The operator drags a CSV file into a browser. It reaches the machine over WiFi automatically.
No changes to the Howick machine. No changes to SketchUp or FrameBuilderMRD.

---

## Web dashboard

The dashboard is the primary human interface. It shows the full pipeline live and lets
operators upload jobs.

**Running locally:** `http://localhost:4841/dashboard`
**On hardware (Pi 5):** `http://howick-pi5.local:4841/dashboard`

- Live pipeline: Design PC → opcua-server → Pi Zero → Howick FRAMA
- Job queue and completion history
- Drag-and-drop CSV upload
- Auto-refreshes every 2 seconds — leave it open in a browser tab

---

## Quick start (no hardware needed)

```bash
mise run daily          # build, test, start all daemons
```

Then open `http://localhost:4841/dashboard` and drag in a CSV.

What happens:
1. CSV lands in the dashboard upload — queued immediately
2. howick-frama (running locally) picks it up from the queue
3. CSV written to `./jobs/machine/` (simulates the USB gadget path)
4. Dashboard shows the job move from Queued → Done

```bash
mise run dev:job        # drops T1.csv (roof truss) into the queue
mise run help           # show all available commands
```

---

## On hardware (Pi 5 + Pi Zero 2W)

Two computers on the factory WiFi:

| Device | Binary | Role |
|--------|--------|------|
| Pi 5 | `opcua-server` | Dashboard, job queue, OPC UA server |
| Pi Zero 2W | `howick-frama` | Subscribes to Pi 5 via OPC UA, writes CSV to virtual USB |

Deploy:
```bash
mise run device:deploy -- pi5
mise run device:deploy -- pi-zero
mise run device:deploy -- windows
```

First-time provisioning:
```bash
mise run device:provision -- pi5
mise run device:provision -- pi-zero
```

See `docs/customer/06-pi-zero-setup.md` for full provisioning guide.

---

## Two binaries

```
opcua-server   Pi 5 / Mac / NUC / Windows    OPC UA server + HTTP + job queue + file watcher
howick-frama   Pi Zero 2W                    OPC UA client — subscribes to Pi 5, writes CSV to USB
```

`howick-frama` uses OPC UA subscriptions — Pi 5 pushes instantly when a job is queued. No polling.
Falls back to HTTP polling when `plat_trunk.url` is an HTTP address (dev and cloud modes).

---

## OPC UA server (port 4840)

Connect any OPC UA client to `opc.tcp://<pi5>:4840/` (namespace `urn:howick-frama`):

```
/Howick/Machine/   Status, CurrentJob, PiecesProduced, CoilRemaining, LastError
/Howick/Jobs/      QueueDepth, CompletedCount, PendingJobId, PendingJobName, PendingJobCsv
                   CompleteJob(job_id)   ← method
```

Free browser: **UaExpert** (Windows/Mac/Linux) from unified-automation.com.

---

## Dev tools

This project uses the [jdx](https://github.com/jdx) ecosystem — same tools from dev laptop to production hardware:

| Tool | Role | Config |
|------|------|--------|
| [mise](https://mise.jdx.dev) | Task runner + tool manager | `.mise.toml` + `.mise/tasks/*.toml` |
| [pitchfork](https://pitchfork.jdx.dev) | Daemon/process manager | `pitchfork.toml` |
| [hk](https://hk.jdx.dev) | Git hook manager | `hk.pkl` |
| [fnox](https://github.com/jdx/fnox) | Secret management | `fnox.toml` |
| [cargo-make](https://github.com/sagiegurari/cargo-make) | Cross-platform scripting (Duckscript) | `Makefile.toml` |

```bash
mise run help             # show all commands
mise run start            # start daemons
mise run stop             # stop daemons
mise run logs             # stream live logs
mise run status           # daemon health
```

---

## Config files

App configs live in `config/`, named `<binary>.<env>.toml`:

| File | Binary | Where used |
|------|--------|------------|
| `config/opcua-server.dev.toml` | opcua-server | Dev laptop (loaded by pitchfork) |
| `config/opcua-server.pi5.toml` | opcua-server | Pi 5 production |
| `config/opcua-server.windows.toml` | opcua-server | Windows Design PC |
| `config/howick-frama.dev.toml` | howick-frama | Dev laptop, OPC UA mode |
| `config/howick-frama.pi-zero.toml` | howick-frama | Pi Zero 2W production |
| `config/howick-frama.windows.toml` | howick-frama | Windows Design PC |

---

## Run tests

```bash
mise run test             # all Rust tests
mise run ci               # full CI gate (check + fmt + test)
mise run check            # clippy + check only
```

---

## Customer docs

See `docs/customer/` — seven documents covering proposal, system overview, hardware quote,
setup guide, roadmap, Pi Zero provisioning, and ops runbook.

Export to PDF: `mise run docs:pdf` (or `mise run docs:pdf -- proposal`)

---

## Related

- [async-opcua](https://github.com/FreeOpcUa/async-opcua) — OPC UA library
- [howick-rs](https://github.com/joeblew999/howick-rs) — Howick CSV parser

---

MIT OR Apache-2.0
