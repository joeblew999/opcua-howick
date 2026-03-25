# FrameCad — Machine Profile

**Manufacturer:** [FRAMECAD](https://framecad.com/), New Zealand
**Type:** Light gauge steel roll-forming machine (frame, truss & panel)

---

## Model range

| Model | Gauge range | Notes |
|-------|-------------|-------|
| F325iT | Up to 1.20 mm (18 ga) | 12 hydraulic/punch functions, popular residential profiles |
| F325iT-L | Up to 1.20 mm (18 ga) | Long-bed variant of F325iT |
| F450iT | Up to 1.55 mm (16 ga) | 1–4 storey structures, 11 tooling punches |
| ST950H | Up to 2.50 mm (12 ga) | Heavy duty — wide-span trusses, long-span floor joists |

Steel Solutions (Bangkok) and Maxxi Factory (Hua Hin) each operate a **FrameCad** machine (exact models TBC).

Machine cost: ~**$400,000 USD** (F325iT-L, based on used machinery listings).

---

## Software ecosystem and pricing

| Product | Role | Cost |
|---------|------|------|
| FRAMECAD Structure | 3D structural design | Annual subscription (price not public) |
| FRAMECAD Steelwise | Detailing & engineering — 4 à la carte modules (Detail, Import, Engineer, CAM) | Annual subscription per module (price not public) |
| FRAMECAD Nexa | Cloud production management — upload jobs, manage queue | $0–$3,000/yr per manufacturing line |

### Nexa subscription tiers (per manufacturing line, per year)

| Tier | Cost/year | Includes |
|------|-----------|----------|
| Nexa Base | **Free** | Job uploads, version control, scheduling, 3D status views |
| Nexa Builder | **$1,000** | + coil management, mobile app, unlimited active projects |
| Nexa Podium | **$2,000** | + ICC compliance, quality control |
| Nexa Enterprise | **$3,000** | + panel tracking design-to-install, logistics planning |

### What this means for customers

Customers who have already invested ~$400k in a FrameCad machine are locked into
FrameCad's software ecosystem with ongoing annual subscription fees — Steelwise
for design, Nexa for production management. These fees apply per machine, per year.

**plat-trunk + opcua-howick replaces the FrameCad software layer entirely:**
- plat-trunk handles STEP CAD design → CSV generation (replaces Steelwise)
- opcua-howick handles job delivery, queue management, and machine monitoring (replaces Nexa)
- The FrameCad machine itself is unchanged — it still reads the same job files

The customer keeps their machine investment. The software subscription goes to zero.

---

## File formats — the full picture

FrameCad has **5 different file formats** across their ecosystem. Understanding
which format goes where is critical for bypassing their software.

### Format overview

| Format | What it is | Producer | Consumer | We have samples? |
|--------|-----------|----------|----------|-----------------|
| **RFY** | Native machine job file. The format the FrameCad machine actually runs. | FrameCad Steelwise / Detailer | FrameCad machine (Factory 2 controller) | **No — need from site** |
| **XML** (`<framecad_import>`) | 3D structural model for import into FrameCad software. Sticks, profiles, tool actions. | FrameBuilderMRD, 3rd party tools | FrameCad Detailer / Steelwise → converts to RFY | **Yes** — `dev/fixtures/Gerard_25032026/` |
| **FIM** | 3D exchange format for Tekla Structures. Contains assembly GUIDs. | Tekla Structures | FrameCad Detailer | No |
| **CSV** | Howick-format flat file. Nexa can also import these. | FrameBuilderMRD, Howick partners | Howick machines, FrameCad Nexa | **Yes** — `dev/fixtures/` |
| **.nexa** | FrameCad's cloud format — 3D + 2D data combined. | FrameCad Steelwise via Nexa | FrameCad Nexa → machine | No |

### How files flow to the machine

```
Design software (Steelwise, FrameBuilderMRD, Tekla, etc.)
  │
  ├── XML / FIM ──▶ FrameCad Detailer/Steelwise ──▶ RFY ──┐
  │                 (requires FrameCad license)              │
  ├── RFY ──────────────────────────────────────────────────┤
  │                                                          ▼
  │                                              ┌──────────────────┐
  │                                              │  FrameCad machine │
  │                                              │  (Factory 2)      │
  │                                              └──────────────────┘
  │                                                          ▲
  └── CSV / RFY / .nexa ──▶ Nexa (cloud) ──────────────────┘
                            ($0–$3k/yr)
```

**The machine reads RFY.** Everything else gets converted to RFY either by
FrameCad software (license required) or by Nexa (subscription required).

### RFY format (the target)

RFY is FrameCad's native machine format. Key features:
- Can span tool operations across sticks
- Produces short web notches/flange cuts with scrap pieces
- Allows per-tool customisation of scrap pieces
- Simpler to program manual parts at machine level

**We do not have RFY samples yet.** This is the #1 item on the site checklists
for Steel Solutions and Maxxi Factory. Without RFY samples we cannot bypass
FrameCad software.

### XML format (`<framecad_import>`) — we have this

The XML we have from FrameBuilderMRD is an **import format** for FrameCad
software — it does NOT go directly to the machine. FrameCad Detailer reads
this XML and converts it to RFY.

```xml
<framecad_import name="FileNAME">
  <jobnum> " " </jobnum>
  <client> " " </client>
  <drawing_info units="Metric" envelope_ref="Centre" stick_ref="Centre">
    <datedrawn>"25-3-2026"</datedrawn>
  </drawing_info>
  <plan name="wall1_25062026">
    <elevation>0.000</elevation>
    <frame name="wall1" type="ExternalWall">
      <envelope>
        <vertex>0.0,44.45,0.0</vertex>
        ...
      </envelope>
      <stick name="S17" type="Stud" gauge="0.95" yield="495"
             tensile="495" coating="Z275" usage="Stud">
        <start>1132.7,44.45,2.0</start>
        <end>1132.7,44.45,2548.0</end>
        <profile web="88.9" l_flange="41.3" r_flange="41.3"
                 l_lip="10.0" r_lip="10.0" shape="C" />
        <flipped> false </flipped>
      </stick>
      <tool_action name="Service">
        <start>5854.45,44.45,400.018</start>
        <end>5745.55,44.45,400.018</end>
      </tool_action>
    </frame>
  </plan>
</framecad_import>
```

#### XML elements

| Element | Description |
|---------|-------------|
| `<plan>` | One plan per wall/panel |
| `<frame>` | Wall frame — `type` = ExternalWall, InternalWall, etc. |
| `<envelope>` | Wall boundary polygon (3D vertices) |
| `<stick>` | Individual member — stud, plate, nogging, sill, head plate |
| `<profile>` | C-section dimensions: web, flanges, lips |
| `<tool_action>` | Machine operations (e.g. service holes) with 3D coords |

#### Stick attributes

| Attribute | Example | Description |
|-----------|---------|-------------|
| `name` | S17, T3, B1, N8 | Member ID (S=stud, T=top, B=bottom, N=nog, L=sill, H=head) |
| `type` | Stud, Plate | Member type |
| `gauge` | 0.95 | Steel thickness in mm |
| `yield` | 495 | Yield strength MPa |
| `tensile` | 495 | Tensile strength MPa |
| `coating` | Z275 | Zinc coating class |
| `usage` | Stud, TopPlate, BottomPlate, Nog, Sill, HeadPlate | Structural role |
| `<flipped>` | true/false | Orientation |

### Comparison: all formats

| | Howick CSV | FrameCad XML | RFY (target) |
|-|-----------|-------------|-------------|
| Format | Flat CSV | Structured XML | Binary/proprietary? TBC |
| Geometry | Linear punch positions | Full 3D coordinates | TBC |
| Material data | Profile code only | gauge, yield, tensile, coating | TBC |
| Structural semantics | None | type, usage | TBC |
| Machine operations | DIMPLE, LIP_CUT, SWAGE, etc. | tool_action with 3D coords | spans across sticks, scrap control |
| Machine reads directly? | **Yes** (Howick) | **No** — needs conversion | **Yes** (FrameCad) |

### Paths to drive a FrameCad machine without FrameCad software

| Option | How | Needs | Status |
|--------|-----|-------|--------|
| **1. Generate RFY directly** | plat-trunk → RFY → USB gadget → machine | RFY format spec (reverse engineer from samples) | **Blocked — need RFY samples** |
| **2. Generate CSV via Nexa** | plat-trunk → CSV → Nexa (free tier) → machine | Nexa Base account (free) | Possible but still uses FrameCad cloud |
| **3. Generate XML** | plat-trunk → XML → FrameCad Detailer → RFY → machine | FrameCad license | **Defeats the purpose** |
| **4. Generate RFY + USB gadget** | Same as #1 but delivered via Pi Zero | RFY spec + Pi Zero hardware | **The goal — fully independent** |

Option 1/4 is the goal. Option 2 is the fallback.

### Fixture files

- [dev/fixtures/Gerard_25032026/](../../dev/fixtures/Gerard_25032026/) — full sample set
  - `25062026.skp` — SketchUp model
  - `machinefiles/.../Howick_H400/` — Howick CSVs (wall1: 50 components, wall2: 7)
  - `machinefiles/.../FrameCADExport/` — FrameCad XMLs (wall1, wall2)
  - `Assembly_Documents_Skp_Layout/` — SketchUp layout with piece legend

### Critical TODO

- [ ] **Get RFY samples from Steel Solutions or Maxxi Factory** — this is the
  #1 blocker for FrameCad machine support without FrameCad software
- [ ] Reverse-engineer RFY format from samples
- [ ] Determine if RFY is binary or text-based
- [ ] Build RFY generator in plat-trunk

---

## OPC UA integration (planned)

- **Namespace URI:** `urn:framecad` (config-driven)
- **Delivery:** Same Pi Zero USB gadget approach as Howick, or via Nexa cloud API
- **See:** [ADR 0003 — Multi-agent node managers](../adr/0003-multi-agent-node-managers.md)

---

## Deployed machines

| Location | Company | Owner | Model | Status |
|----------|---------|-------|-------|--------|
| Bangkok | Steel Solutions | Mr Prisit | TBC | Not yet connected |
| Hua Hin | Maxxi Factory | — | TBC | Not yet connected |

---

## References

- [FRAMECAD — Steel framing machines](https://framecad.com/steel-framing-machines)
- [FRAMECAD F325iT](https://framecad.com/steel-framing-machines/f325it)
- [FRAMECAD F450iT](https://www.framecad.com/en/framecad-system/manufacturing-equipment/f450it/)
- [FRAMECAD Steelwise](https://framecad.com/steel-framing-software/steelwise)
- [FRAMECAD Nexa](https://framecad.com/nexa)
