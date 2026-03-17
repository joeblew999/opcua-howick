# Howick FRAMA — Automated Job Delivery

## What problem this solves

Right now, every job requires someone to:

1. Save the file on the design computer
2. Copy it to a USB stick
3. Walk to the machine and plug it in
4. Walk back

This takes time on every single job. If the USB stick is missing, or the wrong file is copied, production stops.

**This system eliminates all of that.** Jobs go from the design computer to the machine over WiFi — automatically, in seconds, with no USB stick involved.

The existing SketchUp and FrameBuilderMRD workflow is **not changed**. Both methods work side-by-side. The operator does not need to do anything differently.

---

## Hardware to order

Two small computers. One-time purchase. Everything from one local store.

Order from **raspberrypithailand.com** — official Raspberry Pi reseller,
free nationwide shipping, 3-day delivery, full warranty.

| Item | Est. Cost |
|------|-----------|
| Raspberry Pi Zero 2W | ~500 THB |
| Raspberry Pi 5 4GB | ~2,000 THB |
| Official Raspberry Pi 27W USB-C power supply | ~400 THB |
| SanDisk Ultra microSD 32GB × 2 | ~500 THB |
| USB-A to Micro-USB cable 3m | ~300 THB |
| **Total** | **~3,700 THB** |

This is a one-time cost. No monthly fees. No subscriptions.

---

## What gets installed

**Pi Zero 2W** — plugs into the machine's USB port via a 3m cable.
The machine sees it exactly like a USB stick. Job files appear on it automatically over WiFi.

**Pi 5** — sits near the machine on factory WiFi. Provides a status dashboard
showing machine state and job queue.

---

## Phase 2 — Coil inventory sensor (optional, future)

A small weight sensor placed under the coil spool measures how much steel
material remains. The system then shows metres remaining on the dashboard —
so you know when to order a new coil before the current one runs out mid-job.

A coil running out mid-job scraps the partially-formed members and stops production.
This sensor prevents that.

| Item | Where | Est. Cost |
|------|-------|-----------|
| Load cell (50kg) + HX711 board | Lazada | ~450 THB |
| 5m sensor cable | Lazada | ~150 THB |
| **Total** | | **~600 THB** |

---

## Network requirements

- Factory WiFi (existing) — no changes needed
- No firewall changes
- No fixed IP addresses

---

## Support and remote access

The system uses **Tailscale** — a secure tunnel that allows us to diagnose
and fix issues remotely without accessing the factory network directly.
Software updates are pushed automatically every hour. No action required.
