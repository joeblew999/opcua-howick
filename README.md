# opcua-howick

Bridges [plat-trunk](https://cad.ubuntusoftware.net) CAD to Howick FRAMA
roll-forming machines. Replaces the USB stick workflow.

---

## The Problem

Prin's factory in Si Racha, Thailand currently transfers CSV job files to
the Howick FRAMA machine via USB stick. An operator runs FrameBuilderMRD,
copies files to a stick, walks to the machine, plugs it in.

**opcua-howick eliminates that entirely.**

---

## Three Phases

### Phase 0 — Right now, no hardware (immediate)

Designer uses plat-trunk Machine tab → Generate CSV → Download.
Copies CSV to USB stick manually. Still eliminates SketchUp + FrameBuilderMRD.
Prin can validate CSV output against existing jobs immediately.

### Phase 1 — Pi Zero 2W (~$23, ~1 hour setup)

A Raspberry Pi Zero 2W plugged into the machine's USB port via a long cable.
It **pretends to be a USB stick** (USB gadget/mass storage mode) while
connecting to factory WiFi. opcua-howick runs on it, polls plat-trunk,
writes new CSVs to the fake USB partition. The machine sees files appear
on its "USB stick" automatically.

```
plat-trunk browser (anywhere)
    ↓ design → Generate CSV → Send to Machine
CF Worker / Tauri → R2
    ↓ opcua-howick polls every 5s
Pi Zero 2W (WiFi, hidden behind machine)
    ↓ USB cable — appears as USB mass storage
Howick FRAMA USB port
    ↓ reads CSV, runs job
Steel members come out
```

No walking. No USB swapping. No FrameBuilderMRD. No SketchUp.

See [docs/pi-zero-usb-gadget.md](docs/pi-zero-usb-gadget.md) for full setup guide.

### Phase 2 — OPC UA visibility (same Pi, no extra hardware)

The Pi already runs an OPC UA server (port 4840) exposing machine state:
job queue depth, pieces produced, coil remaining. Any OPC UA client on the
factory LAN — or Prin's phone — can see live machine status.

---

## Deployment Options

| Option | Hardware | Setup | Best for |
|--------|----------|-------|----------|
| Pi Zero 2W (USB gadget) | $23 | 1hr | Permanent install, Phase 1 |
| Raspberry Pi 5 | $80 | 30min | Pi with network share to machine PC |
| Windows PC (.exe) | $0 | 15min | If machine PC is accessible |
| Mac Mini | $600 | 20min | Factory office, Topology B/C |

---

## Quick Start (Pi Zero 2W)

```bash
# 1. Build for Pi Zero 2W
mise run build:pi-zero

# 2. Deploy
PI_HOST=pi@howick-pi.local mise run deploy:pi-zero

# 3. Configure on the Pi
ssh pi@howick-pi.local
nano ~/config.toml
# Set: usb_gadget_mode = true
# Set: machine_input_dir = "/mnt/usb_share"
# Set: plat_trunk.url = "https://your-worker.workers.dev"

# 4. See docs/pi-zero-usb-gadget.md for USB gadget setup
```

---

## Services

Three concurrent services on the Pi:

| Service | Port | Purpose |
|---------|------|---------|
| OPC UA server | 4840 | Machine state for any OPC UA client |
| HTTP status API | 4841 | JSON API for plat-trunk UI status panel |
| Job poller | — | Polls plat-trunk every 5s for new jobs |
| File watcher | — | Picks up CSVs dropped locally |

---

## mise tasks

```bash
mise run build:pi-zero     # Cross-compile for Pi Zero 2W
mise run deploy:pi-zero    # Build + deploy to Pi Zero 2W
mise run deploy:pi         # Build + deploy to Pi 5
mise run status:pi         # Check machine status via HTTP
mise run test:submit       # Submit test job to local instance
mise run logs:pi           # Stream live logs from Pi
```

---

## Related

- [howick-rs](https://github.com/joeblew999/howick-rs) — CSV parser/serialiser
- [plat-trunk](https://cad.ubuntusoftware.net) — browser-native B-Rep CAD
- [async-opcua](https://github.com/FreeOpcUa/async-opcua) — OPC UA library
- [docs/pi-zero-usb-gadget.md](docs/pi-zero-usb-gadget.md) — Pi Zero setup guide
- [docs/architecture.md](docs/architecture.md) — full system architecture

---

## License

MIT OR Apache-2.0
