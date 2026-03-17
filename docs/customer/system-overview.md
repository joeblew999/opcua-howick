# System Overview
## Howick FRAMA — Automated Job Delivery

**Prepared for:** Prin — Si Racha Factory, Thailand
**Prepared by:** Gerard Webb, ubuntu Software
**Date:** March 2026

---

## Option A — Design PC only

Simplest start. No hardware purchase. `opcua-howick.exe` runs on the Design PC
alongside SketchUp and FrameBuilderMRD. Operator still carries USB stick to the
machine, but gets the dashboard and drag-and-drop upload instead of manually copying files.

```
┌─────────────────────────────────────────────────────┐
│  Design PC (Windows)                                │
│                                                     │
│  SketchUp + FrameBuilderMRD  — generates CSV        │
│  opcua-howick.exe            — dashboard + watcher  │
│  Browser                     — Job Dashboard        │
│                                http://localhost:4841│
└──────────────────────┬──────────────────────────────┘
                       │
                  USB stick
                  (still manual walk)
                       │
┌──────────────────────▼──────────────────────────────┐
│  Howick FRAMA                                       │
│  USB port only — no network, no attached PC         │
│  reads CSV from USB stick                           │
└─────────────────────────────────────────────────────┘
```

`config.toml` key settings — see `config.option-a.toml`:
```toml
delivery_mode     = "direct"
usb_gadget_mode   = false
machine_input_dir = "C:\\Howick\\Jobs\\"   # TBC with Prin's operator
```

---

## Option B — Pi 5 + Pi Zero (full automation, no USB stick)

Dedicated hardware on the factory WiFi. Pi Zero plugs permanently into the FRAMA
USB port and replaces the USB stick. Operator uses a browser on any device.

```
┌─────────────────────────────────────────────────────┐
│  Design PC (Windows)                                │
│                                                     │
│  SketchUp + FrameBuilderMRD  — generates CSV        │
│  Browser                     — opens Job Dashboard  │
│                                http://pi5.local:4841│
└──────────────────────┬──────────────────────────────┘
                       │ WiFi — drag CSV into dashboard
┌──────────────────────▼──────────────────────────────┐
│  Pi 5                                               │
│                                                     │
│  opcua-howick   — Job Dashboard  (:4841)            │
│                   OPC UA server  (:4840)            │
│                   job queue                         │
└──────────────────────┬──────────────────────────────┘
                       │ WiFi — polls /api/jobs/howick/pending
┌──────────────────────▼──────────────────────────────┐
│  Pi Zero                                            │
│                                                     │
│  howick-agent   — polls Pi 5 for pending jobs       │
│                   writes CSV to virtual USB         │
│                   (Phase 2) reads coil weight       │
└──────────────────────┬──────────────────────────────┘
                       │ USB cable 3m
┌──────────────────────▼──────────────────────────────┐
│  Howick FRAMA                                       │
│  sees Pi Zero as a normal USB stick                 │
│  reads CSV — no changes to machine                  │
└─────────────────────────────────────────────────────┘
```

`config.toml` key settings — see `config.option-b-pi5.toml` and `config.pi-zero.toml`:
```toml
# Pi 5
delivery_mode     = "queue"
usb_gadget_mode   = false

# Pi Zero
usb_gadget_mode   = true
machine_input_dir = "/mnt/usb_share"   # TBC with Prin's operator

[plat_trunk]
url = "http://pi5.local:4841"          # polls Pi 5, not plat-trunk
```

---

## plat-trunk path (future — Phase 3)

When plat-trunk's CAD platform is ready. The Pi Zero polls plat-trunk directly
instead of Pi 5. Everything else is identical to Option B.

```
┌─────────────────────────────────────────────────────┐
│  Anywhere (cloud or LAN)                            │
│                                                     │
│  plat-trunk     — STEP CAD + Framing Extractor      │
│                   generates CSV, queues job         │
│                   serves /api/jobs/howick/pending   │
└──────────────────────┬──────────────────────────────┘
                       │ WiFi / internet
┌──────────────────────▼──────────────────────────────┐
│  Pi Zero (same hardware as Option B)                │
│                                                     │
│  howick-agent   — polls plat-trunk (not Pi 5)       │
│                   writes CSV to virtual USB         │
└──────────────────────┬──────────────────────────────┘
                       │ USB cable 3m
┌──────────────────────▼──────────────────────────────┐
│  Howick FRAMA                                       │
└─────────────────────────────────────────────────────┘
```

---

## Systems and where they run

| Software | Runs on | Role |
|----------|---------|------|
| SketchUp | Design PC | Prin's 3D design tool |
| FrameBuilderMRD | Design PC | Howick's CSV generator |
| opcua-howick | Design PC (Option A) or Pi 5 (Option B) | Dashboard, job queue, OPC UA server |
| howick-agent | Pi Zero only | Polls for jobs, writes CSV to virtual USB |
| plat-trunk | Cloud / LAN (future) | ubuntu Software's STEP CAD platform |

---

## Two design workflows — both supported permanently

| Path | Who uses it | CSV comes from | Status |
|------|-------------|----------------|--------|
| SketchUp path | Prin | SketchUp → FrameBuilderMRD → drag into dashboard | Works today |
| plat-trunk path | plat-trunk users | plat-trunk Framing Extractor → job queue | Future (Phase 3) |

Both paths produce identical CSV files. The delivery mechanism (howick-agent →
virtual USB → FRAMA) is the same for both. Prin does not need to change his
SketchUp workflow for Phase 3 — both paths run alongside each other permanently.

---

## Software internals

### opcua-howick (Pi 5 or Design PC)

Four concurrent services:

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
│    POST /api/sensor/coil  — Phase 2 weight push     │
│                                                     │
│  File watcher                                       │
│    watches job_input_dir for new .csv files         │
│                                                     │
│  Job poller                                         │
│    polls plat-trunk API for pending jobs            │
│    (Phase 3 — plat-trunk path)                      │
└─────────────────────────────────────────────────────┘
```

### howick-agent (Pi Zero 2W)

Minimal binary — polls job queue, writes CSV to virtual USB, pushes coil weight:

```
┌─────────────────────────────────────────────────────┐
│  howick-agent                                       │
│                                                     │
│  Job poller    — GET /api/jobs/howick/pending       │
│                  writes CSV to /mnt/usb_share       │
│                  POST /api/jobs/howick/:id/complete │
│                                                     │
│  Sensor push   — reads HX711 load cell (Phase 2)   │
│                  POST /api/sensor/coil to Pi 5      │
│                                                     │
│  No OPC UA server.  No HTTP server.                 │
│  Binary: ~3MB   RAM: ~16MB                          │
└─────────────────────────────────────────────────────┘
```

---

## OPC UA address space

Any OPC UA client on the factory LAN can connect to `opc.tcp://howick-pi5.local:4840/`
and read live machine state:

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

Updated every 500ms via subscription push to connected clients.

---

## CSV format

The Howick FRAMA reads a specific CSV format. Both SketchUp+FrameBuilderMRD
and the plat-trunk Framing Extractor produce this same format. The delivery
software (opcua-howick, howick-agent) treats the CSV as opaque — it delivers
it, it does not parse it.

Reference jobs from Prin's machine:
- **T1** — roof truss, 22 components, S8908 profile, 3945mm chords
- **W1** — wall frame, 42 components, S8908 profile, 4740mm plates

Operations seen in Prin's jobs: `DIMPLE`, `LIP_CUT`, `SWAGE`, `WEB`,
`END_TRUSS`, `SERVICE_HOLE`, `NOTCH`, `LABEL_NRM`, `LABEL_INV`

---

## Key unknown

**What folder on the USB does the Howick FRAMA look for CSV files?**
One question for Prin's operator. Sets `machine_input_dir` in `config.pi-zero.toml`.
Could be the root of the USB or a named subfolder.

---

## Comparison

| | Option A | Option B |
|--|----------|----------|
| Hardware cost | None | ~3,700 THB |
| USB stick needed | Yes (still manual) | No — retired permanently |
| Dashboard access | Design PC only | Any browser on factory WiFi |
| Job delivery | Manual (drag in + walk stick) | Automatic over WiFi |
| Coil sensor (Phase 2) | No (no Pi Zero GPIO) | Yes |
| Start today | Yes | After hardware arrives |
| Upgrade path | Add Pi hardware any time | Already there |

---

**Gerard Webb**
ubuntu Software
gerard@ubuntu-software.com
