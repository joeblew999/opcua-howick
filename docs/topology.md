# System Topology

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

`config.toml` key settings:
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
└──────────────────────┬──────────────────────────────┘
                       │ USB cable 3m
┌──────────────────────▼──────────────────────────────┐
│  Howick FRAMA                                       │
│  sees Pi Zero as a normal USB stick                 │
│  reads CSV — no changes to machine                  │
└─────────────────────────────────────────────────────┘
```

`config.toml` key settings (Pi 5):
```toml
delivery_mode     = "queue"
usb_gadget_mode   = false
machine_input_dir = "./jobs/input"
```

`config.toml` key settings (Pi Zero):
```toml
usb_gadget_mode   = true
machine_input_dir = "/mnt/usb_share"   # TBC with Prin's operator

[plat_trunk]
url = "http://pi5.local:4841"          # polls Pi 5, not plat-trunk
```

---

## plat-trunk path (future)

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

`config.toml` key settings (Pi Zero):
```toml
[plat_trunk]
url = "https://your-plat-trunk.workers.dev"   # points at plat-trunk, not Pi 5
```

---

## Systems and where they run

| Software | Runs on | Role |
|----------|---------|------|
| SketchUp | Design PC | Prin's 3D design tool |
| FrameBuilderMRD | Design PC | Howick's CSV generator |
| opcua-howick | Design PC (Option A) or Pi 5 (Option B) | Dashboard, job queue, OPC UA server |
| howick-agent | Pi Zero only | Polls for jobs, writes CSV to virtual USB |
| mock-plat-trunk | Dev laptop only | Simulates plat-trunk for local testing |
| plat-trunk | Cloud / LAN (future) | ubuntu Software's STEP CAD platform |

---

## Two job creation paths — same delivery, different source

| Path | Who uses it | CSV comes from | Status |
|------|-------------|----------------|--------|
| SketchUp path | Prin | SketchUp → FrameBuilderMRD → drag into dashboard | Works today |
| plat-trunk path | plat-trunk users | plat-trunk Framing Extractor → job queue | Future |

Both paths produce identical CSV files. The delivery mechanism (howick-agent →
virtual USB → FRAMA) is the same for both.

---

## Key unknown

**What folder on the USB does the Howick FRAMA look for CSV files?**
One question for Prin's operator. Sets `machine_input_dir` on the Pi Zero.

---

## Comparison

| | Option A | Option B |
|--|----------|----------|
| Hardware cost | None | ~3,700 THB |
| USB stick needed | Yes (still manual) | No — retired permanently |
| Dashboard access | Design PC only | Any browser on factory WiFi |
| Job delivery | Manual (drag in + walk stick) | Automatic over WiFi |
| Start today | Yes | After hardware arrives |
| Upgrade path | Add Pi hardware any time | Already there |
