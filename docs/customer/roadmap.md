# Project Roadmap
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
| SketchUp + FrameBuilderMRD CSV format | ✅ Done | tested against Prin's real job files |
| OPC UA server — machine state for plat-trunk | ✅ Done | port 4840 |
| Option B — Pi 5 server + Pi Zero USB gadget | 🔧 In progress | needs Pi hardware on site |
| Confirm USB folder path on Howick FRAMA | ⏳ One question | ask Prin's operator |
| First live test at Si Racha factory | ⏳ Awaiting | after Option A or B is set up |

**What Prin needs to confirm:** what folder on the USB stick does the
Howick FRAMA machine look for job files? One answer from the operator unlocks
full end-to-end testing.

---

## Phase 2 — Coil inventory sensor

**Goal:** know how much steel material is on the coil spool before starting a job.
**Cost:** ~680 THB hardware, ordered from Lazada Thailand.

| Item | Status | Notes |
|------|--------|-------|
| Load cell + HX711 sensor on Pi Zero GPIO | ⏳ Planned | 5m cable to coil spool |
| Weight → metres remaining calculation | ⏳ Planned | calibrated against empty spool weight |
| CoilRemaining shown on dashboard | ⏳ Planned | live metres remaining |
| Low-coil alert before job starts | ⏳ Planned | prevents scrap and mid-job stops |
| OPC UA node: Machine/CoilRemaining | ⏳ Planned | already in address space, needs sensor |

**When:** any time after Phase 1 is running. Add the sensor without touching anything else.

---

## Phase 3 — plat-trunk integration

**Goal:** jobs flow directly from ubuntu Software's CAD platform to the machine
with no manual steps. Designed for when plat-trunk's Framing Extractor is ready.

| Item | Status | Notes |
|------|--------|-------|
| howick-agent polls plat-trunk job queue | ✅ Done | same binary, different config URL |
| Machine state pushed to plat-trunk via OPC UA | ✅ Done | status, queue depth, errors |
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

Any time     Phase 2 — Coil sensor added
             (~680 THB, one remote session)

Later        Phase 3 — plat-trunk integration
             (when plat-trunk Framing Extractor is ready)

Later        Phase 4 — Additional machines
             (per machine, TBD)
```

---

## Open questions

| Question | Who answers |
|----------|-------------|
| What folder does the Howick FRAMA look for files on the USB? | Prin's operator |
| Is the Design PC on the factory WiFi, or separate? | Prin |
| How many Howick FRAMA machines are in the factory? | Prin |
| Are there other roll-forming machines (Phase 4)? | Prin |

---

**Gerard Webb**
ubuntu Software
gerard@ubuntu-software.com
