# opcua-howick

Automates job delivery to Howick FRAMA roll-forming machines — eliminates the USB-stick walk.

---

## Get running locally

```bash
mise run dev          # start opcua-howick (OPC UA :4840, HTTP :4841)
mise run dev:mock     # start mock plat-trunk on :3000
mise run dev:job      # drop T1.csv into the pipeline
mise run dev:status   # check machine status
```

Open `http://localhost:4841/dashboard` to see the pipeline UI.

---

## Run tests

```bash
cargo test                    # all (6 HTTP pipeline + 3 OPC UA integration)
cargo test --test opcua       # OPC UA only — real server + real client on random port
cargo test --test pipeline    # HTTP pipeline only
```

---

## Two binaries

| Binary | Runs on | Role |
|--------|---------|------|
| `opcua-howick` | Pi 5 / NUC / Mac | OPC UA server + HTTP dashboard + job poller + file watcher |
| `howick-agent` | Pi Zero 2W | OPC UA client — subscribes to Pi 5, writes CSV to USB gadget |

`howick-agent` subscribes to `Jobs/PendingJobId` — Pi 5 pushes instantly when a job is queued. No polling.

---

## OPC UA address space

Connect any OPC UA client to `opc.tcp://<pi5>:4840/` (namespace `urn:howick-edge-agent`):

```
/Howick/Machine/   Status, CurrentJob, PiecesProduced, CoilRemaining, LastError
/Howick/Jobs/      QueueDepth, CompletedCount, PendingJobId, PendingJobName, PendingJobCsv
                   CompleteJob(job_id)   ← method
```

---

## Config files

| File | Machine |
|------|---------|
| `config.toml` | Local dev |
| `config.pi5.toml` | Pi 5 |
| `config.pi-zero.toml` | Pi Zero 2W |

---

## Deploy

```bash
mise run build:agent:pi-zero   # cross-compile howick-agent → aarch64
mise run deploy:pi-zero        # build + deploy to Pi Zero (ZERO_HOST=pi@x.x.x.x)
mise run deploy:pi5            # build + deploy to Pi 5    (PI5_HOST=pi@x.x.x.x)
```

---

## Related

- [async-opcua](https://github.com/FreeOpcUa/async-opcua) — OPC UA library
- [howick-rs](https://github.com/joeblew999/howick-rs) — Howick CSV parser
- [docs/customer/](docs/customer/) — customer-facing docs (proposal, setup guide, ops runbook)

---

MIT OR Apache-2.0
