# opcua-howick

Automates job delivery to Howick FRAMA roll-forming machines — eliminates the USB-stick walk.

The operator drags a CSV file into a browser. It reaches the machine over WiFi automatically.
No changes to the Howick machine. No changes to SketchUp or FrameBuilderMRD.

---

## Web dashboard

The dashboard is the primary human interface. It shows the full pipeline live and lets
operators upload jobs.

**Running locally:** `http://localhost:4841/dashboard`
**On hardware (Pi 5):** `http://howick-pi5.local:4841/dashboard`

- Live pipeline: Design PC → opcua-howick → Pi Zero → Howick FRAMA
- Job queue and completion history
- Drag-and-drop CSV upload
- Auto-refreshes every 2 seconds — leave it open in a browser tab

---

## Demo from a laptop (no hardware needed)

```bash
mise run dev:all    # starts opcua-howick + howick-agent on this machine
```

Then open `http://localhost:4841/dashboard` and drag in a CSV.

What happens:
1. CSV lands in the dashboard upload — queued immediately
2. howick-agent (running locally) picks it up from the queue
3. CSV written to `./jobs/machine/` (simulates the USB gadget path)
4. Dashboard shows the job move from Queued → Done

To submit a fixture job from the command line:
```bash
mise run dev:job       # drops T1.csv (roof truss) into the queue
mise run dev:status    # check machine state as JSON
```

---

## On hardware (Pi 5 + Pi Zero 2W)

Two computers on the factory WiFi:

| Device | Binary | Role |
|--------|--------|------|
| Pi 5 | `opcua-howick` | Dashboard, job queue, OPC UA server |
| Pi Zero 2W | `howick-agent` | Polls Pi 5, writes CSV to virtual USB (replaces USB stick) |

Deploy:
```bash
PI5_HOST=pi@howick-pi5.local   mise run deploy:pi5
ZERO_HOST=pi@howick-pi-zero.local  mise run deploy:pi-zero
```

First-time provisioning (includes USB gadget setup):
```bash
ZERO_HOST=pi@howick-pi-zero.local  mise run setup:first-boot:pi-zero
# Pi reboots — wait 30s, then update ZERO_HOST to Tailscale IP
ZERO_HOST=pi@100.x.x.x  mise run setup:post-reboot:pi-zero

PI5_HOST=pi@howick-pi5.local  mise run setup:first-boot:pi5
```

See `docs/customer/06-pi-zero-setup.md` for full provisioning guide.

---

## Two binaries

```
opcua-howick   Pi 5 / Mac / NUC / Windows    OPC UA server + HTTP + job queue + file watcher
howick-agent   Pi Zero 2W                    OPC UA client — subscribes to Pi 5, writes CSV to USB
```

`howick-agent` uses OPC UA subscriptions — Pi 5 pushes instantly when a job is queued. No polling.
Falls back to HTTP polling when `plat_trunk.url` is an HTTP address (dev and cloud modes).

---

## OPC UA server (port 4840)

Connect any OPC UA client to `opc.tcp://<pi5>:4840/` (namespace `urn:howick-edge-agent`):

```
/Howick/Machine/   Status, CurrentJob, PiecesProduced, CoilRemaining, LastError
/Howick/Jobs/      QueueDepth, CompletedCount, PendingJobId, PendingJobName, PendingJobCsv
                   CompleteJob(job_id)   ← method
```

Free browser: **UaExpert** (Windows/Mac/Linux) from unified-automation.com.

---

## Config files

Naming: `<binary>.<env>.toml` — binary name first, environment second.

| File | Binary | Where used |
|------|--------|------------|
| `opcua-server.dev.toml` | opcua-server | Dev laptop (default) |
| `opcua-server.pi5.toml` | opcua-server | Pi 5 production |
| `opcua-server.windows.toml` | opcua-server | Windows Design PC |
| `howick-agent.dev.toml` | howick-agent | Dev laptop, OPC UA mode (default) |
| `howick-agent.dev-mock.toml` | howick-agent | Dev laptop, mock-plat-trunk HTTP mode |
| `howick-agent.dev-http.toml` | howick-agent | Dev laptop, local opcua-server HTTP mode |
| `howick-agent.pi-zero.toml` | howick-agent | Pi Zero 2W production |
| `howick-agent.windows.toml` | howick-agent | Windows Design PC |

---

## Run tests

```bash
cargo test                    # all (5 HTTP pipeline + 3 OPC UA integration)
cargo test --test opcua       # OPC UA only
cargo test --test pipeline    # HTTP pipeline only
RUST_LOG=debug cargo test     # verbose
```

---

## Customer docs

See `docs/customer/` — seven documents covering proposal, system overview, hardware quote,
setup guide, roadmap, Pi Zero provisioning, and ops runbook.

---

## Related

- [async-opcua](https://github.com/FreeOpcUa/async-opcua) — OPC UA library
- [howick-rs](https://github.com/joeblew999/howick-rs) — Howick CSV parser

---

MIT OR Apache-2.0
