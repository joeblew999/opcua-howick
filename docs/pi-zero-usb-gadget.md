# Pi Zero 2W as USB Mass Storage — Replacing the USB Stick

## The Idea

Prin's factory currently transfers CSV files to the Howick FRAMA machine
via a USB stick. An operator runs FrameBuilderMRD, copies the CSV to a
stick, walks to the machine, plugs it in.

A Raspberry Pi Zero 2W (~$15) can **pretend to be a USB stick** while
simultaneously connecting to the factory WiFi. howick-agent runs on it,
polls plat-trunk for new jobs, and writes CSVs to the fake USB partition.
The Howick machine sees a new file appear on its "USB stick" and runs it
automatically.

From the machine's perspective: nothing changes. It still reads from USB.
From the operator's perspective: no walking, no swapping, no USB sticks.

```
plat-trunk (browser, anywhere)
    ↓ design wall → Generate CSV → Send to Machine
CF Worker → R2 (or Tauri local)
    ↓ howick-agent polls every 5s
Pi Zero 2W (WiFi to factory LAN, Tailscale for remote access)
    ↓ USB cable (looks like USB mass storage to machine)
Howick FRAMA USB port
    ↓ reads new CSV automatically
Steel members come out
```

---

## Hardware

See [customer/03-bom.md](customer/03-bom.md) for full BOM and where to order in Thailand.

| Item | Cost | Notes |
|------|------|-------|
| Raspberry Pi Zero 2W | ~$15 | USB OTG — can act as USB device |
| Anker Micro-USB 10ft cable | ~$10 | Long enough to reach machine USB port |
| microSD 32GB | ~$8 | OS + storage partition |
| USB-A charger (5V/2.5A) | ~$8 | Use extension cable if outlet is far |
| **Total** | **~$41** | One-time, permanent replacement for USB sticks |

---

## How USB Gadget Mode Works

The Pi Zero 2W's USB port operates in **device/gadget mode** — it appears
as a USB mass storage device (USB stick) to whatever it's plugged into.

We use `g_mass_storage` — a Linux kernel module that exposes a disk image
file (`/piusb.bin`) as USB storage. howick-agent writes CSVs into that
image, then signals the kernel to re-present the storage to the machine.

---

## Setup — everything is a mise task

All setup steps run from your MacBook via SSH. Set `ZERO_HOST` to the Pi's
local hostname first, then Tailscale IP once Tailscale is installed.

```bash
export ZERO_HOST=pi@howick-pi-zero.local
```

### Step 1 — Flash the Pi

Use **Raspberry Pi Imager** on your MacBook:
- OS: Raspberry Pi OS Lite (64-bit)
- Hostname: `howick-pi-zero`
- Enable SSH
- Set factory WiFi credentials

### Step 2 — Install Tailscale (do this first)

```bash
mise run tailscale:install:pi-zero
```

Note the `100.x.x.x` Tailscale IP printed at the end. Update `ZERO_HOST`:

```bash
export ZERO_HOST=pi@100.x.x.x
```

From this point you can SSH from anywhere — no need to be on factory WiFi.

### Step 3 — USB gadget setup

```bash
mise run setup:usb-gadget:pi-zero
```

This does in one shot:
- Creates the 512MB FAT32 disk image at `/piusb.bin` labelled `HOWICK`
- Enables `dwc2` overlay in `/boot/firmware/config.txt`
- Adds `dwc2,g_mass_storage` to `/boot/firmware/cmdline.txt`
- Creates `/etc/rc.local` to mount + load gadget on boot
- Installs `/usr/local/bin/usb-refresh.sh` for post-write USB re-presentation
- Reboots the Pi

### Step 4 — Set up secrets with Doppler

```bash
mise run doppler:setup:pi-zero
```

This installs the Doppler CLI on the Pi and links it to the `opcua-howick / pi-zero` config.
Set `PLAT_TRUNK_API_KEY` and `PLAT_TRUNK_URL` in the Doppler dashboard —
they are injected at runtime, never written to disk.

### Step 5 — Deploy howick-agent

```bash
mise run deploy:pi-zero
```

Builds, copies binary, installs systemd service, starts it. Done.

### Step 6 — Verify

```bash
mise run status:pi-zero    # check service is running
mise run logs:pi-zero      # stream live logs
```

---

## Deployment via mise (ongoing)

```bash
# Push a new binary after code changes
ZERO_HOST=pi@100.x.x.x mise run deploy:pi-zero

# Stream logs
mise run logs:pi-zero

# SSH in
mise run ssh:pi-zero
```

---

## The Three Phases for Prin

### Phase 0 — Right now (no hardware change)
Designer downloads CSV from plat-trunk Machine tab → copies to USB stick manually.
Existing SketchUp + FrameBuilderMRD workflow untouched. Prin validates CSV output.

### Phase 1 — Pi Zero 2W + Pi 5 (~$121, ~2 hour setup)
Pi Zero plugged into machine USB port via long cable.
Pi 5 on factory LAN running full OPC UA + HTTP.
Jobs flow: browser → plat-trunk → Pi Zero → machine. No walking. No USB swapping.

### Phase 2 — OPC UA visibility (Pi 5, no extra hardware)
Full OPC UA server on Pi 5 (port 4840) exposing machine state:
job queue depth, pieces produced, coil remaining.
Any OPC UA client on factory LAN — or Prin's phone — sees live status.

---

## Physical Setup at the Factory

```
[Factory WiFi]
      |
[Pi Zero 2W] ←── WiFi ──── plat-trunk (cloud or Tauri)
      |        └── Tailscale ──── your MacBook (remote)
[USB cable, 3m]
      |
[Howick FRAMA USB port]
      |
[Machine reads CSV, produces steel]

[Pi 5] ←── WiFi / Ethernet ──── factory LAN
       └── OPC UA :4840, HTTP :4841
```

The Pi Zero sits behind or under the machine. The USB cable is the only
physical connection to the machine. From the machine's perspective it
has always had a USB stick plugged in — it just never runs out of jobs.
