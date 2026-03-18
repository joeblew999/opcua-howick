# Document 2 of 7 — System Overview
## Howick FRAMA — Automated Job Delivery

**Prepared for:** Prin — Si Racha Factory, Thailand
**Prepared by:** Gerard Webb, ubuntu Software
**Date:** March 2026

---

## Option A — Design PC only

Simplest start. No hardware purchase. Both binaries run on the Design PC alongside
SketchUp and FrameBuilderMRD. Operator still carries USB stick to the machine, but
the file copying is automatic — drag into the browser, the USB stick has it.

```
┌─────────────────────────────────────────────────────┐
│  Design PC (Windows)                                │
│                                                     │
│  SketchUp + FrameBuilderMRD  — generates CSV        │
│  opcua-howick.exe            — dashboard + job queue│
│  howick-agent.exe            — picks up queued jobs │
│                                writes to USB stick  │
│  Browser                     — Job Dashboard        │
│                                http://localhost:4841│
└──────────────────────┬──────────────────────────────┘
                       │
                  USB stick
                  (still manual walk — file copied automatically)
                       │
┌──────────────────────▼──────────────────────────────┐
│  Howick FRAMA                                       │
│  USB port only — no network, no attached PC         │
│  reads CSV from USB stick                           │
└─────────────────────────────────────────────────────┘
```

Key config (see `config.windows.toml`):
```toml
usb_gadget_mode   = false
machine_input_dir = "D:\\"   # USB stick drive letter — TBC with Prin's operator
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
                       │ WiFi — polls job queue
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

Key config (see `config.pi5.toml` and `config.agent.pi-zero.toml`):
```toml
# Pi 5
usb_gadget_mode = false

# Pi Zero
usb_gadget_mode   = true
machine_input_dir = "/mnt/usb_share"   # TBC with Prin's operator

[plat_trunk]
url = "opc.tcp://howick-pi5.local:4840/"   # OPC UA subscription — Pi Zero subscribes to Pi 5
```

---

## Phase 3 — plat-trunk path (future)

When ubuntu Software's CAD platform is ready. Pi Zero polls plat-trunk directly
instead of the Pi 5. Everything else is identical to Option B. Prin's SketchUp
workflow continues unchanged alongside it.

```
┌─────────────────────────────────────────────────────┐
│  Anywhere (cloud or LAN)                            │
│                                                     │
│  plat-trunk     — STEP CAD + Framing Extractor      │
│                   generates CSV, queues job         │
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

## Where everything runs

| Software | Runs on | Role |
|----------|---------|------|
| SketchUp | Design PC | Prin's 3D design tool |
| FrameBuilderMRD | Design PC | Howick's CSV generator |
| opcua-howick | Design PC (Option A) or Pi 5 (Option B) | Dashboard, job queue, OPC UA server |
| howick-agent | Design PC (Option A) or Pi Zero 2W (Option B) | Polls for jobs, writes CSV to USB |
| plat-trunk | Cloud / LAN (Phase 3 — future) | ubuntu Software's STEP CAD platform |

---

## Two design workflows — both supported permanently

| Path | Who uses it | CSV comes from | Status |
|------|-------------|----------------|--------|
| SketchUp path | Prin | SketchUp → FrameBuilderMRD → drag into dashboard | Works today |
| plat-trunk path | plat-trunk users | plat-trunk Framing Extractor → job queue | Phase 3 (future) |

Both paths produce identical CSV files. The delivery mechanism is the same for
both. Prin does not need to change his SketchUp workflow for Phase 3.

---

## Dashboard and HTTP API (port 4841)

```
GET  /dashboard          — live pipeline status, drag-and-drop upload
POST /upload             — submit CSV job from browser
GET  /status             — machine state as JSON
GET  /jobs               — job queue + recent completions
POST /api/sensor/coil    — Phase 2: Pi Zero pushes coil weight here
```

---

## OPC UA server (port 4840)

### What is OPC UA?

OPC UA (Unified Architecture) is the international standard for industrial machine
communication — used by Siemens, ABB, Fanuc, and every major SCADA and MES vendor
worldwide. It is the same protocol used to monitor CNC machines, PLCs, and robots
in large factories.

opcua-howick runs a full, standards-compliant OPC UA server on the Pi 5. This is
not a cut-down version — it is the same `async-opcua` library used in production
industrial systems, exposing a proper address space with live subscriptions.

### What it gives you

Any OPC UA client on the factory WiFi can connect and subscribe to live machine data:

```
opc.tcp://howick-pi5.local:4840/

/Howick/
  Machine/
    Status           "Running" | "Idle" | "Error" | "Offline"
    CurrentJob       frameset name e.g. "T1", "W1"
    PiecesProduced   count (future)
    CoilRemaining    metres of steel remaining (Phase 2 — needs sensor)
    LastError        last error message
  Jobs/
    QueueDepth       jobs waiting to be delivered
    CompletedCount   jobs delivered to machine
```

All values update every 500ms automatically via OPC UA subscriptions — no polling needed.

### Connecting right now — free tool

**UaExpert** (free, Windows/Mac/Linux) — the standard OPC UA browser used by engineers
worldwide:

1. Download from unified-automation.com → Downloads → OPC UA Clients
2. Add server: `opc.tcp://howick-pi5.local:4840/`
3. Browse to `Objects → Howick → Machine`
4. Right-click any node → Add to subscription
5. Watch values update live every 500ms

### Why this matters

Any factory monitoring system, SCADA, or MES that speaks OPC UA can connect to the
Pi 5 and read machine state — with no changes to opcua-howick. This is the same
integration path used to connect Siemens S7 PLCs, Fanuc CNCs, and Beckhoff controllers
to factory dashboards. You are getting that capability on a Pi 5 for free.

Future integrations (energy monitoring, production reporting, ERP systems) connect
to the same OPC UA endpoint — no changes required.

---

## CSV format

The Howick FRAMA reads a specific CSV format. Both SketchUp+FrameBuilderMRD
and plat-trunk produce this same format. The software delivers it without
parsing it — the machine's existing logic is unchanged.

Reference jobs from Prin's machine used to develop and test this system:
- **T1** — roof truss, 22 components, S8908 profile, 3945mm chords
- **W1** — wall frame, 42 components, S8908 profile, 4740mm plates

---

## Comparison

| | Option A | Option B |
|--|----------|----------|
| Hardware cost | None | ~3,700 THB |
| USB stick needed | Yes (still manual) | No — retired permanently |
| Dashboard access | Design PC only | Any browser on factory WiFi |
| Job delivery | Manual (drag in + walk stick) | Automatic over WiFi |
| Coil sensor (Phase 2) | No | Yes |
| Start today | Yes | After hardware arrives (3 days) |
| Upgrade path | Add Pi hardware any time | Already there |

---

## Key unknown

**What folder on the USB does the Howick FRAMA look for CSV files?**
One question for Prin's operator. Sets `machine_input_dir` in `config.agent.pi-zero.toml`.

---

→ **Next: Document 3 — Hardware Quote** (what to order and where in Thailand)

---

**Gerard Webb**
ubuntu Software

