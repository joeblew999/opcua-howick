# Document 5 of 7 — Project Roadmap
## Howick FRAMA — Automated Job Delivery

**Prepared for:** Prin — Si Racha Factory, Thailand
**Prepared by:** Gerard Webb, ubuntu Software
**Date:** March 2026

---

## Phase 1 — Job delivery and dashboard

**Goal:** jobs reach the Howick FRAMA over WiFi without a USB stick walk.
**Cost:** ~3,700 THB hardware (Option B) or free (Option A — Design PC only).

| Item | Status | Notes |
|------|--------|-------|
| Dashboard UI — drag-and-drop job upload | ✅ Done | `http://<host>:4841/dashboard` |
| Dashboard — live pipeline status, errors | ✅ Done | auto-refreshes every 2s |
| Option A — software on Design PC | ✅ Done | runs alongside SketchUp today |
| SketchUp + FrameBuilderMRD CSV format | ✅ Done | tested against Prin's real job files (T1, W1) |
| OPC UA server — machine state | ✅ Done | port 4840 |
| Option B — Pi 5 server + Pi Zero USB gadget | 🔧 In progress | needs Pi hardware on site |
| Confirm USB folder path on Howick FRAMA | ⏳ One question | ask Prin's operator |
| First live test at Si Racha factory | ⏳ Awaiting | after Option A or B is set up |

**What Prin needs to confirm:** what folder on the USB stick does the
Howick FRAMA machine look for job files? One answer from the operator unlocks
full end-to-end testing.

---

## Phase 2 — Coil inventory sensor

**Goal:** know how much steel material is on the coil spool before starting a job.
**Cost:** ~680 THB hardware (Lazada Thailand). Requires Option B (Pi Zero GPIO).

| Item | Status | Notes |
|------|--------|-------|
| Weight → metres remaining calculation | ✅ Done | calibrated against empty spool weight |
| CoilRemaining shown on dashboard | ✅ Done | live metres + low-coil alert |
| Low-coil alert before job starts | ✅ Done | amber warning when below threshold |
| OPC UA node: Machine/CoilRemaining | ✅ Done | in address space, updates from sensor |
| Physical load cell + HX711 wired to Pi Zero | ⏳ Awaiting hardware | 5m cable to coil spool |
| Calibrate with Prin's empty spool weight | ⏳ Awaiting hardware | one weight reading from Prin |

**Note:** the software is complete and tested. Adding the physical sensor is a
one-off wiring job — no downtime, no changes to anything else.

**When:** any time after Phase 1 is running.

---

## Phase 3 — plat-trunk integration

**Goal:** jobs flow directly from ubuntu Software's STEP CAD platform to the machine.
Designed for when plat-trunk's Framing Extractor is ready.

| Item | Status | Notes |
|------|--------|-------|
| howick-agent polls plat-trunk job queue | ✅ Done | same binary, different config URL |
| Machine state readable via OPC UA | ✅ Done | status, queue depth, coil remaining |
| plat-trunk Framing Extractor — STEP → Howick CSV | ⏳ plat-trunk work | produces same CSV as FrameBuilderMRD |

**Note:** SketchUp + FrameBuilderMRD continues to work alongside plat-trunk
permanently. Prin does not need to change his design workflow for this phase.

---

## Phase 4 — Additional machines

**Goal:** extend the same system to other roll-forming machines in the factory.

| Item | Status | Notes |
|------|--------|-------|
| Multi-machine dashboard | ⏳ Planned | one dashboard, all machines |
| Per-machine hardware (Pi 5 + Pi Zero) | ⏳ Planned | ~3,700 THB per additional machine |
| Machine-specific configuration | ⏳ Planned | separate config per machine |

**When:** after Phase 1 is proven. Scope and hardware confirmed per machine.

---

## Summary timeline

```
Now          Phase 1A — Option A running on Design PC
             (no hardware, one remote session)

+1–2 weeks   Phase 1B — Option B hardware ordered and delivered
             Pi 5 + Pi Zero on factory WiFi
             USB stick retired

Any time     Phase 2 — Coil sensor wired and calibrated
             (~680 THB hardware, one remote session)
             Software is already complete.

Later        Phase 3 — plat-trunk integration
             (when plat-trunk Framing Extractor is ready)

Later        Phase 4 — Additional machines
             (per machine, scope TBD)
```

---

## Open questions

| Question | Who answers |
|----------|-------------|
| What folder does the Howick FRAMA look for files on the USB? | Prin's operator |
| Is the Design PC on the factory WiFi, or on a separate network? | Prin |
| How many Howick FRAMA machines are in the factory? | Prin |
| Are there other roll-forming machines to add later (Phase 4)? | Prin |
| Empty coil spool weight in kg (for Phase 2 calibration)? | Prin's operator |

---

→ **Next: Document 6 — Pi Zero Setup** (Gerard's provisioning guide for Option B)

---

**Gerard Webb**
ubuntu Software

