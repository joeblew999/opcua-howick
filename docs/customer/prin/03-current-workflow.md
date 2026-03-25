# Eastern Mobile House (EMH) — Current Workflow

**For:** Prin — Si Racha Factory, Laem Chabang
**Date:** March 2026

---

## Overview

Prin designs buildings in SketchUp on a Windows Design PC. To produce a job
for the machine, the operator manually enters wall frame parameters into the
FrameBuilderMRD plugin (there is no link to the 3D building model). The plugin
generates a 3D wall frame, and from that frame it can export:

- A **SketchUp layout file** (with a per-piece legend for assembly)
- **Machine files** for multiple brands — Howick CSV, FrameCad, and others

Prin uses the Howick CSV at Si Racha. The FrameCad output exists but is
unusable without a paid FrameCad license. The CSV is then copied to a USB
stick, walked to the machine, and plugged in — for every single job.

---

## Current workflow: design to machine

```
┌──────────────┐      ┌──────────────┐      ┌──────────────┐      ┌──────────────┐
│  1. DESIGN   │      │  2. FRAME    │      │  3. CSV +    │      │  4. USB TO   │
│              │      │   ENTRY      │      │   LAYOUT     │      │   MACHINE    │
│  Prin builds │  ╳   │  Operator    │─────▶│  Plugin      │─────▶│  Copy CSV to │
│  3D model in │ no   │  manually    │      │  generates:  │      │  USB stick,  │
│  SketchUp    │ link │  enters wall │      │              │      │  walk to     │
│              │      │  frame data  │      │  • 3D layout │      │  machine,    │
│              │      │  into Frame- │      │    in SU     │      │  plug in     │
│              │      │  BuilderMRD  │      │  • CSV for   │      │              │
│              │      │              │      │    machine   │      │              │
└──────────────┘      └──────────────┘      └──────────────┘      └──────────────┘
                      20,000 THB/yr
```

---

## Step details

### 1. Design in SketchUp

- **Software:** SketchUp (permanent license — Prin's primary tool)
- **Platform:** Windows Design PC
- **Output:** 3D model of the building — used for visualisation and client
  approval, but **not connected to FrameBuilderMRD** in any way

### 2. Manually enter frame data into FrameBuilderMRD

- **Software:** FrameBuilderMRD — a SketchUp plugin by Howick (**20,000 THB/year**, ~$570 USD)
- The operator manually enters each wall frame into the plugin — dimensions,
  stud spacing, openings, etc. There is **no automatic extraction** from the
  SketchUp 3D model. The plugin cannot read the existing building geometry.

### 3. Plugin generates layout + CSV

From the manually entered data, FrameBuilderMRD produces two outputs:

1. **SketchUp layout file** — a 3D wall frame view generated back into SketchUp,
   with a legend identifying each piece (stud, track, nogging, etc.). Used for
   visual checking, documentation, and on-site assembly reference.

2. **Machine files** — generated from the 3D wall frame. FrameBuilderMRD can
   export to **multiple machine formats**:
   - **Howick CSV** (`.csv`) — used today at Si Racha (e.g. `T1.csv`, `W1.csv`)
   - **FrameCad** — supported but unusable without a paid FrameCad license
   - **Other brands** — FrameBuilderMRD supports additional roll-former formats

Prin uses Howick CSV. The FrameCad output exists but the FrameCad machine
requires a paid FrameCad software license to accept jobs. _(Assumption —
needs confirmation.)_

**CSV format:** `UNIT` → `PROFILE` → `FRAMESET` → `COMPONENT` records with
operations (DIMPLE, LIP_CUT, SWAGE, WEB, END_TRUSS) — see
[machines/howick-frama.md](../../machines/howick-frama.md) for full format spec.

### 4. USB stick to machine

- Operator copies the CSV to a USB stick (any folder)
- Walks the stick to the Howick FRAMA on the factory floor
- Plugs USB into the **machine's Windows PC** (the FRAMA has a built-in
  Windows computer running Howick's own software)
- From within the Howick software, the operator **browses the USB drive**
  and selects the job folder — there is no fixed folder path
- Machine runs the job
- Removes stick, walks back, repeats for every job

---

## Software on the Design PC

| Software | Purpose | Cost |
|----------|---------|------|
| SketchUp | 3D building design | Permanent license — Prin owns it |
| FrameBuilderMRD | SketchUp plugin — manual frame entry → CSV | **20,000 THB/year** (~$570 USD) |
| Windows | OS | — |

---

## Pain points

| Problem | Impact |
|---------|--------|
| **Manual re-entry** | Frame data is entered by hand — no link to 3D model. Slow and error-prone. |
| **USB stick walk** | Operator walks to the machine for every single job |
| **No job queue** | One job at a time — next job waits until operator walks back |
| **No visibility** | Nobody knows what's running, queued, or errored |
| **No coil tracking** | Operator guesses when steel coil is running low |
| **FrameCad lock-out** | Plugin can output FrameCad files, but machine won't accept them without a paid license |

---

## What we change

### opcua-howick (now)

| Before | Option A (Design PC) | Option B (Pi hardware) |
|--------|----------------------|------------------------|
| Copy CSV to USB stick | Drag into browser dashboard | Drag into browser dashboard |
| Walk USB to machine | Still needed | **Eliminated — WiFi** |
| No job queue | Dashboard shows queue | Dashboard shows queue |
| No visibility | Dashboard shows status | Dashboard shows status |
| No coil tracking | — | Phase 2: live sensor |

**Option A** — software only, no hardware cost. Eliminates manual file copy.
**Option B** — Pi Zero replaces the USB stick permanently. No more walking.

SketchUp + FrameBuilderMRD workflow is **completely unchanged** in both options.

### plat-trunk (future)

```
plat-trunk (STEP CAD) → Framing Extractor → CSV → opcua-howick → machine
```

Eliminates **both** remaining gaps:

1. **No manual re-entry** — reads 3D model, generates CSV automatically
2. **No FrameCad license needed** — delivers via USB gadget, bypassing FrameCad software entirely
3. **Saves 20,000 THB/year** — replaces FrameBuilderMRD subscription

Runs alongside SketchUp permanently. Prin does not need to change anything.
See [ADR-0002](../../adr/0002-two-design-workflows.md).

---

**Gerard Webb**
ubuntu Software
