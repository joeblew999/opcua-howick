# opcua-howick

Automates job delivery to Howick FRAMA roll-forming machines.
Eliminates the USB-stick walk — jobs arrive over WiFi.

---

## The problem

Prin's factory in Si Racha, Thailand transfers CSV job files to the Howick FRAMA
via USB stick. An operator runs FrameBuilderMRD, copies files to a stick, walks
to the machine, plugs it in. Every job, every time.

**opcua-howick eliminates that walk.**

---

## How it works

A Raspberry Pi Zero 2W plugs permanently into the machine's USB port via a 3m cable.
It pretends to be a USB stick while connecting to factory WiFi. Jobs are dragged into
a browser dashboard and arrive at the machine automatically.

From the machine's perspective: nothing changed. It still reads from USB.

See [docs/customer/02-system-overview.md](docs/customer/02-system-overview.md) for full
topology diagrams.

---

## Two binaries

| Binary | For | OPC UA | HTTP | RAM | Size |
|--------|-----|--------|------|-----|------|
| `opcua-howick` | Pi 5, NUC, Mac, Windows | ✅ | ✅ | ~64MB | ~15MB |
| `howick-agent` | Pi Zero 2W (USB gadget) | ❌ | ❌ | ~16MB | ~3MB |

The Pi Zero only needs to poll and write. `howick-agent` does exactly that.

---

## Two design workflows — both permanent

| Path | CSV from | Status |
|------|----------|--------|
| SketchUp + FrameBuilderMRD | Prin's existing tools — drag into dashboard | Works today |
| plat-trunk Framing Extractor | ubuntu Software STEP CAD | Future (Phase 3) |

---

## Customer docs

All documentation lives in [docs/customer/](docs/customer/).

| Doc | Purpose |
|-----|---------|
| [proposal.md](docs/customer/01-proposal.md) | What the system is, what it costs, recommendation |
| [system-overview.md](docs/customer/02-system-overview.md) | Topology diagrams, where things run, OPC UA |
| [hardware-quote.md](docs/customer/03-hardware-quote.md) | What to order and where in Thailand |
| [setup-guide.md](docs/customer/04-setup-guide.md) | What Prin does vs what Gerard does |
| [roadmap.md](docs/customer/05-roadmap.md) | Phases 1–4 with status and open questions |
| [pi-zero-setup.md](docs/customer/06-pi-zero-setup.md) | Pi Zero 2W USB gadget provisioning (Gerard) |
| [ops-runbook.md](docs/customer/07-ops-runbook.md) | Full lifecycle: deploy, update, secrets (Gerard) |

PDFs in [docs/dist/](docs/dist/) — rebuild with `mise run docs:pdf:all`.

---

## Config files

One config per machine — deploy tasks copy it automatically on first deploy:

| File | Machine |
|------|---------|
| [config.windows.toml](config.windows.toml) | Windows Design PC |
| [config.pi5.toml](config.pi5.toml) | Pi 5 |
| [config.pi-zero.toml](config.pi-zero.toml) | Pi Zero 2W |

---

## mise tasks

```bash
# Local dev
mise run dev               # Start opcua-howick locally
mise run dev:mock          # Start mock plat-trunk on :3000
mise run dev:job           # Drop T1.csv into pipeline  (JOB=W1 for wall frame)
mise run dev:agent:local   # Run howick-agent polling local opcua-howick
mise run dev:sensor        # Simulate coil weight push  (COIL_KG=23.5)
mise run dev:status        # Check HTTP status

# Build
mise run build:agent:pi-zero   # Cross-compile howick-agent for Pi Zero 2W
mise run build:pi5             # Cross-compile opcua-howick for Pi 5

# Deploy (set ZERO_HOST / PI5_HOST to Tailscale IP)
mise run deploy:pi-zero        # Build + deploy howick-agent to Pi Zero 2W
mise run deploy:pi5            # Build + deploy opcua-howick to Pi 5
mise run deploy:windows        # Build + deploy opcua-howick.exe to Windows PC

# SSH / logs / status
mise run ssh:pi-zero           # SSH into Pi Zero 2W
mise run ssh:pi5               # SSH into Pi 5
mise run logs:pi-zero          # Stream live logs from Pi Zero 2W
mise run logs:pi5              # Stream live logs from Pi 5
mise run status:pi5            # Service status + HTTP API

# Docs
mise run docs:pdf:all          # Rebuild all PDFs into docs/dist/

# CI
mise run ci                    # check + fmt + test
```

---

## Related

- [howick-rs](https://github.com/joeblew999/howick-rs) — Howick CSV parser/serialiser
- [plat-trunk](https://cad.ubuntusoftware.net) — ubuntu Software's STEP CAD platform
- [async-opcua](https://github.com/FreeOpcUa/async-opcua) — OPC UA library used

---

## License

MIT OR Apache-2.0
