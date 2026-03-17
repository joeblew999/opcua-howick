# opcua-howick Architecture

## What this system does

Automates job delivery to a Howick FRAMA roll-forming machine.

Currently Prin's operator generates a CSV job file on the Design PC, copies it
to a USB thumb drive, walks it to the machine, and plugs it in. Every job, every time.

This system eliminates the walk. A Raspberry Pi Zero 2W plugs permanently into the
FRAMA's USB port and appears to the machine as a normal USB stick. Jobs arrive over
WiFi. The machine never knows anything changed.

---

## Two design workflows — both supported permanently

See [topology.md](topology.md) for full detail. Summary:

### Workflow 1 — SketchUp + FrameBuilderMRD (primary, works today)

Prin's existing tools. Nothing changes for him.

```
SketchUp (Prin's design tool)
  + FrameBuilderMRD (Howick's CSV generator)
  → CSV file

operator drags CSV into dashboard at http://<host>:4841/dashboard
  → opcua-howick queues it
  → howick-agent delivers to FRAMA via virtual USB
```

### Workflow 2 — plat-trunk (future, when plat-trunk CAD is ready)

ubuntu Software's own 3D CAD platform, based on STEP files (ISO 10303).
The Framing Extractor in plat-trunk reads a STEP model and produces the same
Howick FRAMA CSV format. No SketchUp, no FrameBuilderMRD needed.

```
plat-trunk STEP CAD + Framing Extractor
  → CSV queued in plat-trunk backend
  → howick-agent polls plat-trunk API
  → delivers to FRAMA via virtual USB
```

**plat-trunk's CAD has a long way to go.** Workflow 1 is the primary path for
the foreseeable future. Both are permanent — neither replaces the other.

---

## Physical hardware

```
[Design PC — Windows]
  SketchUp + FrameBuilderMRD
  Browser → http://howick-pi5.local:4841/dashboard
  (opcua-howick.exe can also run here directly — see Topology A)

[Raspberry Pi 5]                          [optional — see topologies]
  opcua-howick
  OPC UA server :4840
  Dashboard + HTTP API :4841
  Job queue

[Raspberry Pi Zero 2W]  ←── 3m USB cable ──→  [Howick FRAMA]
  howick-agent                                   USB port only
  USB gadget mode                                no network
  appears as USB mass storage                    no attached PC
  polls job queue, writes CSV
```

The Howick FRAMA has a USB port only — no network, no PC. The Pi Zero is the
only way to automate job delivery without modifying the machine.

---

## Deployment options

See [topology.md](topology.md) for full diagrams. Summary:

### Option A — Design PC only

```
Design PC: SketchUp + FrameBuilderMRD + opcua-howick.exe + Browser
  → drag CSV into dashboard → watcher writes to machine_input_dir
  → operator still carries USB stick to FRAMA
```

### Option B — Pi 5 + Pi Zero (full automation)

```
Design PC: SketchUp + FrameBuilderMRD + Browser
  → drag CSV into dashboard on Pi 5

Pi 5: opcua-howick
  → job queue, dashboard, OPC UA server

Pi Zero: howick-agent
  → polls Pi 5 → writes CSV to virtual USB

Howick FRAMA: sees Pi Zero as USB stick → reads CSV
```

Three config values control which option is active:
- `delivery_mode` — `"direct"` (Option A) or `"queue"` (Option B)
- `machine_input_dir` — where the CSV lands
- `usb_gadget_mode` — `true` only on Pi Zero

---

## Software components

| Binary | Runs on | Role |
|--------|---------|------|
| `opcua-howick` | Pi 5 or Design PC | OPC UA server, dashboard, file watcher, job queue |
| `howick-agent` | Pi Zero 2W | Polls job queue, writes CSV to virtual USB |
| `mock-plat-trunk` | Dev only | Simulates plat-trunk API for local testing |

### opcua-howick internals

Four concurrent async services:

```
┌─────────────────────────────────────────────────────┐
│  opcua-howick                                       │
│                                                     │
│  OPC UA server (port 4840)                          │
│    Machine/Status, CurrentJob, PiecesProduced,      │
│    CoilRemaining, LastError                         │
│    Jobs/QueueDepth, CompletedCount                  │
│                                                     │
│  HTTP server (port 4841)                            │
│    GET  /dashboard   — live pipeline UI             │
│    POST /upload      — CSV upload from browser      │
│    GET  /status      — machine state JSON           │
│    GET  /jobs        — queue + history JSON         │
│    GET  /health      — health check                 │
│                                                     │
│  File watcher                                       │
│    watches job_input_dir for new .csv files         │
│    queues job → writes to machine_input_dir         │
│                                                     │
│  Job poller                                         │
│    polls plat-trunk API for pending jobs            │
│    (Workflow 2 — plat-trunk path)                   │
└─────────────────────────────────────────────────────┘
```

---

## CSV format — the contract

The Howick FRAMA CSV format is the interface between both design workflows and
the machine. Both FrameBuilderMRD and the plat-trunk Framing Extractor produce
this format. opcua-howick and howick-agent treat it as opaque — they deliver
it, they don't parse it.

Golden reference files in `dev/fixtures/`:
- `T1.csv` — roof truss, 22 components, S8908 profile, 3945mm chords
- `W1.csv` — wall frame, 42 components, S8908 profile, 4740mm plates

Operations seen in the wild: `DIMPLE`, `LIP_CUT`, `SWAGE`, `WEB`,
`END_TRUSS`, `SERVICE_HOLE`, `NOTCH`, `LABEL_NRM`, `LABEL_INV`

---

## OPC UA address space

```
/Howick/
  Machine/
    Status           String    "Running" | "Idle" | "Error" | "Offline"
    CurrentJob       String    frameset name e.g. "W1"
    PiecesProduced   UInt32
    CoilRemaining    Double    metres remaining (Phase 2 — needs sensor)
    LastError        String
  Jobs/
    QueueDepth       UInt32
    CompletedCount   UInt32
```

Updated every 500ms via subscription push to connected OPC UA clients.

---

## Key unknown

**What folder (or subfolder) on the USB does the Howick FRAMA look for CSVs?**

Could be the root or a subdirectory. One question for Prin's operator.
Sets `machine_input_dir` in Pi Zero `config.toml`.

---

## Related

- `docs/topology.md` — full topology and workflow detail
- `docs/adr/` — architecture decisions
- `dev/fixtures/` — golden CSV files from Prin's machine
- plat-trunk repo — Framing Extractor, job queue API, STEP CAD platform
