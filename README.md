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

No walking. No USB swapping. Existing SketchUp + FrameBuilderMRD workflow untouched.

See [docs/pi-zero-usb-gadget.md](docs/pi-zero-usb-gadget.md) for full setup guide.

### Phase 2 — OPC UA visibility (same Pi, no extra hardware)

The Pi already runs an OPC UA server (port 4840) exposing machine state:
job queue depth, pieces produced, coil remaining. Any OPC UA client on the
factory LAN — or Prin's phone — can see live machine status.

---

## Deployment Options

| Option | Hardware | Setup | Best for |
|--------|----------|-------|----------|
| Pi Zero 2W + `howick-agent` | $41 | 1hr | Permanent install, USB gadget mode |
| Raspberry Pi 5 + `opcua-howick` | $80 | 30min | Full OPC UA + HTTP, factory LAN |
| Windows PC (.exe) | $0 | 15min | If machine PC is accessible |
| Mac Mini | $600 | 20min | Factory office, Topology B/C |

---

## Quick Start (Pi Zero 2W)

```bash
# 1. Build + deploy howick-agent to Pi Zero 2W
ZERO_HOST=pi@100.x.x.x mise run deploy:pi-zero

# 2. Build + deploy full OPC UA binary to Pi 5
PI5_HOST=pi@100.x.x.x mise run deploy:pi5

# 3. Configure on Pi Zero
mise run ssh:pi-zero
nano ~/config.toml
# Set: usb_gadget_mode = true
# Set: machine_input_dir = "/mnt/usb_share"
# Set: plat_trunk.url = "https://your-worker.workers.dev"

# 4. See docs/pi-zero-usb-gadget.md for USB gadget + Tailscale setup
```

---

## Two Binaries

| Binary | For | OPC UA | HTTP | RAM | Size |
|--------|-----|--------|------|-----|------|
| `howick-agent` | Pi Zero 2W (USB gadget) | ❌ | ❌ | ~16MB | ~3MB |
| `opcua-howick` | Pi 5, NUC, Mac, Windows | ✅ | ✅ | ~64MB | ~15MB |

The Pi Zero only needs to poll and write. `howick-agent` does exactly that.

## Services (opcua-howick full binary)

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
# Build
mise run build:agent:pi-zero   # Cross-compile howick-agent for Pi Zero 2W
mise run build:pi5             # Cross-compile opcua-howick for Pi 5

# Deploy (use Tailscale IPs via ZERO_HOST / PI5_HOST env vars)
mise run deploy:pi-zero        # Build + deploy howick-agent to Pi Zero 2W
mise run deploy:pi5            # Build + deploy opcua-howick to Pi 5
mise run deploy:windows        # Build + deploy opcua-howick.exe to Windows PC

# SSH
mise run ssh:pi-zero           # SSH into Pi Zero 2W
mise run ssh:pi5               # SSH into Pi 5

# Logs + status
mise run logs:pi-zero          # Stream live logs from Pi Zero 2W
mise run logs:pi5              # Stream live logs from Pi 5
mise run status:pi-zero        # Check Pi Zero service status
mise run status:pi5            # Check Pi 5 service status + HTTP API

# Tailscale (run once per Pi on first setup)
mise run tailscale:install:pi5      # Install + auth Tailscale on Pi 5
mise run tailscale:install:pi-zero  # Install + auth Tailscale on Pi Zero
mise run tailscale:status:pi5       # Check Tailscale is connected

# Doppler (secrets — replaces plaintext config.toml api keys)
mise run doppler:setup:pi5          # Install Doppler CLI on Pi 5 + link project
mise run doppler:setup:pi-zero      # Install Doppler CLI on Pi Zero + link project
mise run doppler:secrets            # List secrets locally

# Monitoring
mise run netdata:install:pi5        # Install Netdata on Pi 5

# Local dev
mise run test:submit           # Submit test job to local instance
mise run test:status           # Check local HTTP status
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
