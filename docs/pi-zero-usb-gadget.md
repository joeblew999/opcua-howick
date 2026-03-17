# Pi Zero 2W as USB Mass Storage — Replacing the USB Stick

## The Idea

Prin's factory currently transfers CSV files to the Howick FRAMA machine
via a USB stick. An operator runs FrameBuilderMRD, copies the CSV to a
stick, walks to the machine, plugs it in.

A Raspberry Pi Zero 2W (~$15) can **pretend to be a USB stick** while
simultaneously connecting to the factory WiFi. opcua-howick runs on it,
polls plat-trunk for new jobs, and writes CSVs to the fake USB partition.
The Howick machine sees a new file appear on its "USB stick" and runs it
automatically.

From the machine's perspective: nothing changes. It still reads from USB.
From the operator's perspective: no walking, no swapping, no USB sticks.

```
plat-trunk (browser, anywhere)
    ↓ design wall → Generate CSV → Send to Machine
CF Worker → R2 (or Tauri local)
    ↓ opcua-howick polls every 5s
Pi Zero 2W (WiFi to factory LAN)
    ↓ USB cable (looks like USB mass storage to machine)
Howick FRAMA USB port
    ↓ reads new CSV automatically
Steel members come out
```

---

## Hardware

| Item | Cost | Notes |
|------|------|-------|
| Raspberry Pi Zero 2W | ~$15 | Has USB OTG (can act as USB device) |
| USB-A to micro-USB cable | ~$3 | Long (2-3m) to reach machine USB port |
| microSD card (8GB+) | ~$5 | OS + storage partition |
| **Total** | **~$23** | One-time, permanent replacement for USB sticks |

**Why Pi Zero 2W specifically:**
- USB OTG port — can act as USB *device* (not just host)
- Built-in WiFi — no USB WiFi dongle needed
- Small enough to hide behind the machine
- Runs full Linux — opcua-howick binary runs natively

---

## How USB Gadget Mode Works

The Pi Zero 2W's USB port can operate in two modes:
- **Host mode** (normal) — Pi controls USB devices (keyboard, drives, etc.)
- **Device/gadget mode** — Pi IS the USB device (appears as drive, keyboard, etc.)

We use `g_mass_storage` — a Linux kernel module that makes the Pi appear
as a USB mass storage device (USB stick) to whatever it's plugged into.

The Pi exposes a disk image file as the USB storage. opcua-howick mounts
that image, writes CSVs to it, then signals the kernel to "re-present" the
updated storage to the host machine.

---

## Setup Guide

### 1. Flash Pi Zero 2W

```bash
# Use Raspberry Pi Imager
# OS: Raspberry Pi OS Lite (64-bit)
# Enable SSH, set WiFi credentials for factory network
# Hostname: howick-pi
```

### 2. Create the USB mass storage image

```bash
ssh pi@howick-pi.local

# Create a 512MB FAT32 disk image (adjust size as needed)
sudo dd if=/dev/zero of=/piusb.bin bs=1M count=512
sudo mkdosfs /piusb.bin -F 32 -n "HOWICK"

# Create mount point
sudo mkdir -p /mnt/usb_share
```

### 3. Enable USB gadget mode

```bash
# Add to /boot/config.txt
echo "dtoverlay=dwc2" | sudo tee -a /boot/config.txt

# Add to /boot/cmdline.txt (after rootwait, on same line)
# modules-load=dwc2,g_mass_storage

# Create gadget config on boot
sudo tee /etc/rc.local << 'EOF'
#!/bin/bash
# Mount the USB image
mount -o loop,sync,noatime /piusb.bin /mnt/usb_share

# Load mass storage gadget pointing at the image
modprobe g_mass_storage file=/piusb.bin stall=0 ro=0

exit 0
EOF
sudo chmod +x /etc/rc.local
```

### 4. Install opcua-howick

```bash
# Download the arm64 binary from GitHub Releases
wget https://github.com/joeblew999/opcua-howick/releases/latest/download/opcua-howick-aarch64-unknown-linux-gnu
chmod +x opcua-howick-aarch64-unknown-linux-gnu
mv opcua-howick-aarch64-unknown-linux-gnu opcua-howick

# Or use mise run deploy:pi from your MacBook
```

### 5. Configure opcua-howick

```bash
cat > ~/config.toml << 'EOF'
[opcua]
host             = "0.0.0.0"
port             = 4840
application_name = "Howick Edge Agent - Si Racha Factory"

[http]
host = "0.0.0.0"
port = 4841

[machine]
machine_name      = "Howick FRAMA"
job_input_dir     = "/home/pi/jobs/input"
# This is the mounted USB image — Howick machine reads from here
machine_input_dir = "/mnt/usb_share"
machine_output_dir = "/home/pi/jobs/output"

[plat_trunk]
# Cloud topology:
url = "https://your-worker.workers.dev"
# LAN topology (Tauri on factory LAN):
# url = "http://tauri-machine.local:3000"
api_key                   = ""
status_push_interval_secs = 5
EOF
```

### 6. Install as systemd service

```bash
sudo tee /etc/systemd/system/opcua-howick.service << 'EOF'
[Unit]
Description=Howick Edge Agent
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=pi
WorkingDirectory=/home/pi
ExecStartPre=/bin/sleep 5
ExecStart=/home/pi/opcua-howick
Restart=always
RestartSec=5
Environment=RUST_LOG=opcua_howick=info

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable opcua-howick
sudo systemctl start opcua-howick
```

### 7. Handle USB re-presentation after write

When opcua-howick writes a new CSV, the Howick machine needs to see the
updated storage. Add a script that opcua-howick calls after each write:

```bash
sudo tee /usr/local/bin/usb-refresh.sh << 'EOF'
#!/bin/bash
# Sync filesystem and re-present USB storage to host
sync
echo 1 > /sys/bus/platform/drivers/dwc2/dwc2/gadget/suspended 2>/dev/null || true
sleep 0.5
echo 0 > /sys/bus/platform/drivers/dwc2/dwc2/gadget/suspended 2>/dev/null || true
EOF
sudo chmod +x /usr/local/bin/usb-refresh.sh
```

---

## Deployment via mise

From your MacBook:

```bash
# Build for Pi Zero 2W (arm64)
mise run build:pi5    # aarch64-unknown-linux-gnu

# Deploy (SSH key auth required)
PI_HOST=pi@howick-pi.local mise run deploy:pi
```

---

## The Three Phases for Prin

### Phase 0 — Right now (no hardware change)
Designer downloads CSV from plat-trunk Machine tab → copies to USB stick manually.
Still eliminates SketchUp + FrameBuilderMRD. Prin can validate the CSV output.

### Phase 1 — Pi Zero 2W (~$23, ~1 hour setup)
Pi plugged into machine USB port via long cable.
Jobs flow: browser → plat-trunk → Pi → machine. No walking. No USB swapping.

### Phase 2 — OPC UA visibility (optional, same Pi)
The Pi already runs an OPC UA server (port 4840).
Any OPC UA client on the factory LAN can see machine status, job queue,
pieces produced. Future: connect to factory MES, ERP, or Prin's phone.

---

## Physical Setup at the Factory

```
[Factory LAN / WiFi]
        |
   [Pi Zero 2W]  ←── WiFi ──── plat-trunk (cloud or Tauri)
        |
   [USB cable, 2m]
        |
   [Howick FRAMA USB port]
        |
   [Machine reads CSV, produces steel]
```

The Pi sits behind or under the machine. The USB cable is the only
physical connection to the machine. From the machine's perspective it
has always had a USB stick plugged in — it just never runs out of jobs.
