# Document 3 of 7 — Hardware Quote
## Howick FRAMA — Automated Job Delivery

**Prepared for:** Prin — Si Racha Factory, Thailand
**Prepared by:** Gerard Webb, ubuntu Software
**Date:** March 2026

---

The hardware below is the only cost involved in this project.
Software, setup, and ongoing management are provided free of charge.

**Option A (Design PC only) requires no hardware purchase.**
This quote is for Option B — dedicated hardware on the factory network.

---

## Phase 1 — Option B hardware

Order from **raspberrypithailand.com** — official Raspberry Pi reseller.
Free nationwide shipping. 3-day delivery. Full warranty.

| # | Item | Role | Est. Cost |
|---|------|------|-----------|
| 1 | Raspberry Pi Zero 2W | Plugs into Howick FRAMA USB port — replaces USB stick permanently | ~500 THB |
| 2 | Raspberry Pi 5 4GB | Small server — runs Job Dashboard, handles job queue | ~2,000 THB |
| 3 | Official Raspberry Pi 27W USB-C power supply | Powers the Pi 5 | ~400 THB |
| 4 | SanDisk Ultra microSD 32GB × 2 | Storage for both computers | ~500 THB |
| 5 | USB-A to Micro-USB cable, 3m | Connects Pi Zero to Howick FRAMA USB port | ~300 THB |
| | **Phase 1 Option B total** | | **~3,700 THB** |

**Note on the 3m cable (item 5):** Measure the distance from the Howick FRAMA USB
port to the nearest power point. Use an extension lead if needed — the Pi Zero
must be close enough to reach the machine's USB port.

**Upgrading from Option A later:** if you start with Option A and decide to add
the Pi hardware later, items 1, 4 (one card), and 5 are all you need to add.
Item 2 and 3 (Pi 5 + power) are optional — the Pi Zero can poll the Design PC
directly instead.

---

## Phase 2 — Coil inventory sensor (optional, add any time)

**Requires Option B** — the sensor wires to the Pi Zero's GPIO pins.
Order from **Lazada Thailand** when you are ready to add this.

| # | Item | Role | Est. Cost |
|---|------|------|-----------|
| 1 | Load cell 50 kg | Sits under coil spool, measures remaining steel | ~250 THB |
| 2 | HX711 amplifier board | Reads the load cell signal, connects to Pi Zero GPIO | ~130 THB |
| 3 | 4-core shielded cable, 5m | Runs from spool to Pi Zero | ~150 THB |
| 4 | Steel mounting plate ~150×150mm | Mounts load cell under spool | ~150 THB (local hardware shop) |
| | **Phase 2 total** | | **~680 THB** |

Gerard calibrates the sensor remotely after you weigh the empty coil spool and
send him the weight in kg. No production downtime.

---

## Summary

| | Cost |
|-|------|
| Option A — Design PC only | **Free** |
| Option B — Phase 1 dedicated hardware | ~3,700 THB |
| Phase 2 — coil sensor (Option B only) | +~680 THB |
| Software, setup, management | **Free** |
| Monthly fees | **None** |

---

→ **Next: Document 4 — Setup Guide** (what you do vs what Gerard does)

---

**Gerard Webb**
ubuntu Software

