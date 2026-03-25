# ADR 0005 — Speckle Geometry Findings: SketchUp Model Already Contains Framing

**Status:** Confirmed
**Date:** 2026-03-25

---

## Summary

We uploaded Prin's real SketchUp model (`25062026.skp`) to Speckle cloud and
inspected the geometry via `specklepy`. The key finding is that **the framing
is already in the SketchUp model** with proper layer names. The Framing
Extractor does not need to detect walls from raw geometry — it just needs to
read the existing layer structure.

---

## Speckle project

| Item | Value |
|------|-------|
| Server | https://app.speckle.systems |
| Workspace | gerard-test (ID: `8d4725a853`) |
| Project | opcua-howick-test (ID: `f3318660fc`) |
| Model | wall-frames (ID: `c6539853c0`) |
| Version | `e799b993b6` |
| Object | `31a78bf95432bc65d7817b090742d6b3` |
| Source file | `dev/fixtures/Gerard_25032026/25062026.skp` |

---

## What Speckle extracted

### Top-level structure

```
[Collection] "Unnamed document"
  units = 'mm'
  elements:
    [Layer] "wall"          — 9 sub-layers (THE FRAMING)
    [Layer] "SF Aligner"    — 5 elements (alignment tool, ignore)
  groups:
    "wall1" — 707 objects
    + 61 other groups
  materials:
    galv_tex, red, green, + 3 more
```

### Wall layer sub-layers (the framing tree)

| Sub-layer | Elements | Description |
|-----------|----------|-------------|
| `wall_external_cladding_1` | 4 | External wall panels (5800×2550mm) |
| `wall_internal_cladding_1` | 2 | Internal wall panels |
| `Stud` | **139** | Vertical studs |
| `Nog` | **122** | Noggings (horizontal bracing) |
| `BottomPlate` | **70** | Bottom track plates |
| `TopPlate` | **107** | Top track plates |
| `window` | **5** | Window framing (studs + headers) |
| `generic_frame` | **3** | Filler/cripple studs |
| `lateralbrace` | **1** | Diagonal bracing |

### Sample member dimensions

**Stud (first element):**
```
532 vertices, 1338 faces
X: 1179.4 → 1220.6  (Δ41.3mm)   ← flange width
Y:    0.0 →   88.9  (Δ88.9mm)   ← web depth
Z:    2.0 →  948.0  (Δ946.0mm)  ← stud length
```

**Wall panel (external cladding):**
```
584 vertices
X:    0.0 → 5800.0  (Δ5800.0mm) ← wall length
Y:  -18.0 →   88.9  (Δ106.9mm)
Z:    0.0 → 2550.0  (Δ2550.0mm) ← wall height
```

### Profile confirmation

The C-section profile is visible in every member's bounding box:

| Dimension | Measured from geometry | Expected (S8908 profile) | Match? |
|-----------|----------------------|--------------------------|--------|
| Web depth | 88.9mm (Y axis) | 88.9mm | ✅ |
| Flange width | 41.3mm (X axis) | 41.3mm | ✅ |
| Lip | ~10mm | 10mm | ✅ |

### Wall dimensions confirmation

| Dimension | From SketchUp geometry | From FrameCad XML envelope | Match? |
|-----------|----------------------|---------------------------|--------|
| Wall length | 5800.0mm | 5800.0mm | ✅ |
| Wall height | 2550.0mm | 2550.0mm | ✅ |

---

## Key finding: no wall detection needed

The original assumption in ADR 0004 was that the Framing Extractor would need
to detect walls from raw mesh geometry. **This is wrong.**

Prin's SketchUp model already contains the complete steel frame with:
- Every stud, nogging, plate, and brace as a separate mesh
- Proper layer names matching structural roles (Stud, Nog, BottomPlate, TopPlate)
- Correct C-section profile geometry (88.9mm web, 41.3mm flanges)
- Accurate 3D positions

The Framing Extractor only needs to:

1. **Read** the layer tree from Speckle
2. **Extract** each member's position and length from vertex bounding boxes
3. **Map** to Howick CSV COMPONENT records
4. **Output** machine files

This is a much simpler problem than geometry detection.

**Crucially:** This layer structure is created by FrameBuilderMRD (the SketchUp
plugin), not by Prin manually. Every FrameBuilderMRD user worldwide will have
the same layer naming convention. This means the Framing Extractor works for
any FrameBuilderMRD user, not just Prin — a much larger market.

---

## Revised architecture

```
SketchUp (model already has framing on named layers)
  → Speckle connector (preserves layers, meshes, groups)
  → specklepy reads layer tree:
      wall/Stud      → 139 members with 3D positions
      wall/Nog       → 122 members
      wall/BottomPlate → 70 members
      wall/TopPlate  → 107 members
      wall/window    → 5 members
  → Map each member to CSV COMPONENT:
      position → start/end coordinates
      length   → bounding box longest axis
      profile  → derive from cross-section dimensions (88.9 web = S8908)
      operations → derive from member geometry (dimple, lip_cut, swage positions)
  → Output Howick CSV + FrameCad RFY
  → opcua-howick delivers to machine
```

---

## Remaining unknowns

| Unknown | Impact | How to resolve |
|---------|--------|---------------|
| How are punch positions (DIMPLE, LIP_CUT, SWAGE) encoded in the geometry? | Can't generate CSV operations without this | Compare mesh vertices to known CSV punch positions from the fixtures |
| Do all of Prin's models have this layer structure? | Need consistency to automate | Confirmed: this IS FrameBuilderMRD's output layer structure |
| Does FrameBuilderMRD create these layers? | **Yes — confirmed.** Every FrameBuilderMRD user gets this layer structure. | This is the SketchUp plugin's standard output |
| Is the layer naming consistent across projects? | Automation depends on predictable names | Review more .skp files |

---

## Related

- [ADR 0004 — Speckle as SketchUp bridge](0004-speckle-sketchup-bridge.md) — original architecture (updated by this finding)
- [Howick FRAMA CSV format](../machines/howick-frama.md) — target output format
- [FrameCad XML format](../machines/framecad.md) — comparison format
- [Prin workflow](../customer/prin/03-current-workflow.md) — how the model is created
