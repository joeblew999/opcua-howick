# Pi Zero 2W Setup — USB Gadget Mode
## Howick FRAMA — Replacing the USB Stick Permanently

**Prepared by:** Gerard Webb, ubuntu Software
**Date:** March 2026

---

## How it works

The Raspberry Pi Zero 2W plugs permanently into the Howick FRAMA's USB port via
a 3m cable. It runs `howick-agent` — a small program that polls the Pi 5 for
pending jobs and writes each CSV to a virtual USB drive.

The Howick FRAMA sees the Pi Zero as a normal USB stick and reads from it exactly
as it always has. Nothing changes on the machine side.

```
[Factory WiFi]
      |
[Pi Zero 2W] ←── WiFi ──── Pi 5 (job queue + dashboard)
      |        └── Tailscale ──── MacBook (remote access)
[USB cable, 3m]
      |
[Howick FRAMA USB port]
      |
[Machine reads CSV, produces steel]
```

---

## How USB gadget mode works

The Pi Zero 2W's USB port operates in **device/gadget mode** — it presents itself
as a USB mass storage device to whatever it is plugged into.

The Linux `g_mass_storage` kernel module exposes a disk image file (`/piusb.bin`)
as USB storage. `howick-agent` writes CSVs into that image, then signals the kernel
to re-present the storage to the machine — the same effect as ejecting and re-inserting
a USB stick.

---

## Hardware needed

See `docs/customer/hardware-quote.md` for the full order and where to buy in Thailand.

| Item | Role |
|------|------|
| Raspberry Pi Zero 2W | USB gadget + WiFi agent |
| Micro-USB 3m cable | Permanent connection to Howick FRAMA |
| microSD 32GB | OS + USB image storage |
| USB-A charger 5V/2.5A | Power for Pi Zero (near machine or extension lead) |

---

## Setup — everything is a mise task

All steps run from a MacBook via SSH. Set `ZERO_HOST` before starting:

```bash
export ZERO_HOST=pi@howick-pi-zero.local
```

### Step 1 — Flash the Pi

Use **Raspberry Pi Imager** on the MacBook:
- OS: Raspberry Pi OS Lite (64-bit)
- Hostname: `howick-pi-zero`
- Enable SSH
- Set factory WiFi name and password

### Step 2 — Install Tailscale

```bash
mise run tailscale:install:pi-zero
```

Note the `100.x.x.x` Tailscale IP printed at the end. Update `ZERO_HOST`:

```bash
export ZERO_HOST=pi@100.x.x.x
```

From this point SSH works from anywhere — no need to be on factory WiFi.

### Step 3 — USB gadget setup (reboots Pi)

```bash
mise run setup:usb-gadget:pi-zero
```

This does in one shot:
- Creates 512MB FAT32 disk image at `/piusb.bin` labelled `HOWICK`
- Enables `dwc2` overlay in `/boot/firmware/config.txt`
- Adds `dwc2,g_mass_storage` to `/boot/firmware/cmdline.txt`
- Creates `/etc/rc.local` to mount + load gadget on boot
- Installs `/usr/local/bin/usb-refresh.sh` for post-write USB re-presentation
- Reboots the Pi

### Step 4 — Secrets (Doppler)

```bash
mise run doppler:setup:pi-zero
```

Installs Doppler CLI and links to `opcua-howick / pi-zero` config. Set
`PLAT_TRUNK_API_KEY` and `PLAT_TRUNK_URL` in the Doppler dashboard.

### Step 5 — Deploy howick-agent

```bash
mise run deploy:pi-zero
```

Builds, copies binary, installs systemd service, starts it.

### Step 6 — Verify

```bash
mise run status:pi-zero    # service running?
mise run logs:pi-zero      # stream live logs
```

---

## Config on Pi Zero

Copy `config.pi-zero.toml` to `~/config.toml` on the Pi Zero.
Key settings to confirm with Prin's operator:

```toml
[machine]
usb_gadget_mode   = true
machine_input_dir = "/mnt/usb_share"   # (**) confirm subfolder with operator

[plat_trunk]
url = "http://howick-pi5.local:4841"   # Pi 5 address on factory WiFi

[sensor]
enabled        = false   # set true after Phase 2 load cell is wired + calibrated
empty_spool_kg = 18.0    # (**) weigh empty spool and enter here
```

---

## Phase 2 — Coil sensor wiring

When the load cell + HX711 ADC arrives (see `hardware-quote.md`):

| HX711 pin | Pi Zero GPIO |
|-----------|-------------|
| VCC | 3.3V (pin 1) |
| GND | GND (pin 6) |
| DAT | GPIO 5 (pin 29) |
| CLK | GPIO 6 (pin 31) |

Run the 5m cable from the coil spool down to the Pi Zero.
Set `sensor.enabled = true` in `config.pi-zero.toml` and calibrate with a known weight.

Test without hardware:
```bash
echo "23.5" > /tmp/coil_weight_kg   # simulates 23.5 kg on spool
```

---

## Ongoing deployment

```bash
# Push a new version after code changes
ZERO_HOST=pi@100.x.x.x mise run deploy:pi-zero

# Stream logs
mise run logs:pi-zero

# SSH in
mise run ssh:pi-zero
```

Software updates also install automatically every hour via a systemd timer.
See `docs/customer/ops-runbook.md` for full lifecycle details.
