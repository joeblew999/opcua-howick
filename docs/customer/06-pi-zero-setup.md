# Document 6 of 7 — Pi Zero 2W Setup
## Howick FRAMA — USB Gadget Provisioning

**Prepared by:** Gerard Webb, ubuntu Software
**Date:** March 2026
**Audience:** Gerard — internal provisioning guide for Option B

---

## How it works

The Raspberry Pi Zero 2W plugs permanently into the Howick FRAMA's USB port via
a 3m cable. It runs `howick-agent` — polls the Pi 5 for pending jobs and writes
each CSV to a virtual USB drive. The FRAMA sees a normal USB stick. Nothing on
the machine side changes.

```
[Factory WiFi]
      │
[Pi Zero 2W] ←── WiFi ──── Pi 5 (job queue + dashboard)
      │        └── Tailscale ──── MacBook (remote access)
[USB cable, 3m]
      │
[Howick FRAMA USB port]
      │
[Machine reads CSV, produces steel]
```

---

## USB gadget mode

The Pi Zero 2W's USB port operates in device mode — it presents itself as USB
mass storage to whatever it is plugged into. The Linux `g_mass_storage` kernel
module exposes a 512MB FAT32 disk image (`/piusb.bin`) as USB storage.
`howick-agent` writes CSVs into that image, then signals the kernel to
re-present the storage — same effect as ejecting and reinserting a USB stick.

---

## Hardware (see Document 3 — Hardware Quote)

| Item | Role |
|------|------|
| Raspberry Pi Zero 2W | USB gadget + WiFi agent |
| Micro-USB 3m cable | Permanent connection to Howick FRAMA |
| microSD 32GB | OS + USB image storage |
| USB-A charger 5V/2.5A | Power near machine |

---

## Setup steps

Set `ZERO_HOST` before starting:

```bash
export ZERO_HOST=pi@howick-pi-zero.local
```

### Step 1 — Flash the Pi

Raspberry Pi Imager (MacBook):
- OS: Raspberry Pi OS Lite (64-bit)
- Hostname: `howick-pi-zero`
- Enable SSH
- Set factory WiFi SSID and password

### Step 2 — Install Tailscale

```bash
mise run tailscale:install:pi-zero
```

Note the `100.x.x.x` Tailscale IP. Update `ZERO_HOST`:

```bash
export ZERO_HOST=pi@100.x.x.x
```

From this point SSH works from anywhere — no need to be on factory WiFi.

### Step 3 — USB gadget setup (reboots Pi)

```bash
mise run setup:usb-gadget:pi-zero
```

Does in one shot:
- Creates 512MB FAT32 image at `/piusb.bin` labelled `HOWICK`
- Enables `dwc2` overlay in `/boot/firmware/config.txt`
- Adds `dwc2,g_mass_storage` to `/boot/firmware/cmdline.txt`
- Creates `/etc/rc.local` (mount + load gadget on boot)
- Installs `/usr/local/bin/usb-refresh.sh` (USB re-present after write)
- Reboots

### Step 4 — Secrets (Doppler)

```bash
mise run doppler:setup:pi-zero
```

Links to `opcua-howick / pi-zero` config. Set `PLAT_TRUNK_API_KEY` and
`PLAT_TRUNK_URL` in the Doppler dashboard. Secrets are never written to disk.

### Step 5 — Deploy howick-agent

```bash
mise run deploy:pi-zero
```

Cross-compiles, copies binary, installs systemd service, starts it.

### Step 6 — Verify

```bash
mise run status:pi-zero    # service running?
mise run logs:pi-zero      # stream live logs
```

---

## Config on Pi Zero

Copy `config.pi-zero.toml` to `~/config.toml`. Key values:

```toml
[machine]
usb_gadget_mode   = true
machine_input_dir = "/mnt/usb_share"   # (**) confirm path with Prin's operator

[plat_trunk]
url = "http://howick-pi5.local:4841"   # Pi 5 address on factory WiFi

[sensor]
enabled        = false          # set true after Phase 2 hardware is wired
empty_spool_kg = 18.0           # (**) weigh empty spool and enter here
low_alert_m    = 50.0           # alert threshold in metres
```

---

## Phase 2 — Coil sensor wiring

When the load cell + HX711 arrives (see Document 3):

| HX711 | Pi Zero GPIO |
|-------|-------------|
| VCC | 3.3V (pin 1) |
| GND | GND (pin 6) |
| DAT | GPIO 5 (pin 29) |
| CLK | GPIO 6 (pin 31) |

Run 5m cable from coil spool to Pi Zero. Set `sensor.enabled = true`.

Test without hardware (dev):
```bash
echo "23.5" > /tmp/coil_weight_kg
```

---

## Ongoing

```bash
ZERO_HOST=pi@100.x.x.x mise run deploy:pi-zero   # push new version
mise run logs:pi-zero                              # stream logs
mise run ssh:pi-zero                               # SSH in
```

Auto-update runs hourly via systemd timer — see Document 7 (Operations Runbook).

---

→ **Next: Document 7 — Operations Runbook** (full lifecycle: deploy, update, secrets)

---

**Gerard Webb**
ubuntu Software
gerard@ubuntu-software.com
