# Howick FRAMA — Automated Job Delivery Setup

## What this does

Instead of copying files to a USB stick and walking to the machine, jobs are
sent automatically from the browser to the machine over WiFi.

The existing SketchUp and FrameBuilderMRD workflow is **not changed**. Both
methods work side-by-side.

---

## Hardware to order

All items available from **raspberrypithailand.com** — official Raspberry Pi reseller,
free nationwide shipping, 3-day delivery, full warranty.

| Item | Est. Cost |
|------|-----------|
| Raspberry Pi Zero 2W | ~500 THB |
| Raspberry Pi 5 4GB | ~2,000 THB |
| Official Raspberry Pi 27W USB-C power supply | ~400 THB |
| SanDisk Ultra microSD 32GB × 2 | ~500 THB |
| USB-A to Micro-USB cable 3m | ~300 THB |
| **Total** | **~3,700 THB** |

---

## What gets installed

Two small computers are placed near the machine:

**Pi Zero 2W** — plugs into the machine's USB port via a 3m cable.
The machine sees it as a USB stick. New job files appear on it automatically.

**Pi 5** — sits on the factory WiFi. Provides a status dashboard showing
machine state, job queue, and (Phase 2) coil material remaining.

---

## Phase 2 — Coil inventory sensor

A small weight sensor is placed under the coil spool. A 5m cable runs back
to the Pi Zero 2W. The system then shows how many metres of material remain,
so you know when to order more coil before running out mid-job.

| Item | Where | Est. Cost |
|------|-------|-----------|
| Load cell (50kg) + HX711 board | Lazada | ~450 THB |
| 5m sensor cable | Lazada | ~150 THB |
| **Total** | | **~600 THB** |

---

## Network requirements

- Factory WiFi (existing)
- No firewall changes needed
- No fixed IP addresses needed

---

## Support and remote access

The system uses **Tailscale** — a secure tunnel that allows remote diagnosis
and software updates from anywhere without accessing the factory network
directly. All updates are pushed remotely.
