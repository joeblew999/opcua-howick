# Factory Automation Proposal
## Howick FRAMA — Automated Job Delivery

**Prepared for:** Prin — Si Racha Factory, Thailand
**Prepared by:** Gerard Webb, ubuntu Software
**Date:** March 2026

---

## Summary

Gerard Webb at ubuntu Software is offering to connect your Howick FRAMA machine
to the factory network at no charge. You pay only for the hardware. Everything
else — software, setup, and ongoing management — is provided free of charge.

Your existing workflow does not change. SketchUp, FrameBuilderMRD, and USB sticks
all continue to work exactly as they do today. This system adds a second, faster
path for getting jobs to the machine — running alongside what you already have.

---

## The problem this solves

Every job requires someone to copy a file to a USB stick, walk it to the Howick
FRAMA, and plug it in. This happens for every single job, every day.

This system eliminates that walk. A job sent from the design computer reaches
the machine over WiFi in seconds, automatically.

**Nothing is removed. Nothing is replaced. Both methods work side by side.**

---

## How it works — two setup options

### Option A — Design PC only (simplest, start here)

The software runs directly on the design computer alongside SketchUp and
FrameBuilderMRD. No extra hardware required.

```
Design PC
├── SketchUp + FrameBuilderMRD   generates job file as normal
├── opcua-howick (this software)  dashboard + job queue
└── Browser                       open http://localhost:4841/dashboard
      drag job file in
            │
            │ (operator still carries USB stick to machine)
            ▼
      Howick FRAMA reads CSV — no changes to machine
```

The operator drags the job file into the browser instead of copying it to a USB
stick. The USB stick stays as a backup — nothing is removed.

**Hardware cost: none.** Software runs on the existing Design PC.

---

### Option B — Dedicated hardware (no USB stick, full automation)

Two small computers on the factory WiFi handle everything. The design computer,
phones, and tablets can all reach the Job Dashboard from anywhere on the network.

```
Design PC
├── SketchUp + FrameBuilderMRD   generates job file as normal
└── Browser                       open http://pi5.local:4841/dashboard
      drag job file in
            │ WiFi
            ▼
      Pi 5 (credit-card sized computer on factory WiFi)
      └── opcua-howick   dashboard, job queue, OPC UA server
            │ WiFi
            ▼
      Pi Zero (smaller than a USB stick, plugged into FRAMA permanently)
      └── howick-agent   receives job over WiFi, writes to virtual USB
            │ USB cable 3m
            ▼
      Howick FRAMA reads CSV — no changes to machine
```

The USB stick is retired permanently. The machine sees the Pi Zero as a normal
USB stick — it never knows the difference.

**Hardware cost: ~3,700 THB** — see Hardware Quote document.

---

## Recommendation

Start with **Option A** — no hardware cost, running in one session. Once
you are comfortable with how it works, upgrade to Option B for the full
hands-free experience.

Both options use identical software. Moving from A to B requires only
plugging in two small computers and changing one configuration value.

---

## Phase 2 — Coil inventory sensor (optional, either option)

A weight sensor under the coil spool displays remaining material in metres
on the Job Dashboard. An alert fires before the coil runs out mid-job —
preventing scrap members and a full restart.

**Hardware cost: ~680 THB additional** — see Hardware Quote document.

---

## Phase 3 — Additional machines

The same system can be extended to other roll-forming machines in the factory.
Scope and hardware discussed per machine when the time comes.

---

## Why this is free

Ubuntu Software is building a platform for connected factory machines. Having
a real installation running against real equipment and real processes is
enormously valuable for development. In exchange, you get a working system
and ongoing support at no cost.

---

## What this means in practice

- **For the operator:** the dashboard replaces the USB walk. Everything else is identical.
- **For you:** jobs can be sent from any browser on the factory network the moment a design is ready.
- **For maintenance:** Gerard manages everything remotely. No site visits needed.
- **For updates:** software updates automatically. No action required.

---

## How to proceed

**Option A (no hardware):**
Contact Gerard Webb to schedule a setup session. Takes one hour on a remote call.

**Option B (with hardware):**
1. Review the Hardware Quote document and place the order at raspberrypithailand.com
2. Contact Gerard Webb to schedule the setup session after hardware arrives

---

**Gerard Webb**
ubuntu Software
gerard@ubuntu-software.com
