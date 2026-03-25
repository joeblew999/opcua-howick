# Howick FRAMA — Machine Profile

**Manufacturer:** [Howick Ltd](https://www.howickltd.com/), New Zealand
**Type:** Light gauge steel roll-forming machine (frame, truss & panel)

---

## Model range

| Model | Gauge range | Speed | Notes |
|-------|-------------|-------|-------|
| FRAMA 3200 | 0.75–1.15 mm (22–18 ga) | 900 m/hr | Residential & commercial frames + trusses |
| FRAMA 4200 | 0.75–0.95 mm (22–20 ga) | 700 m/hr | Dedicated truss, rivet jointing |
| FRAMA 5600 | 0.95–1.55 mm (20–16 ga) | 800 m/hr | Heavier/wider sections |
| FRAMA 6800 | — | — | Large commercial |
| FRAMA 7600 | — | — | Large commercial |
| FRAMA 7800 | — | — | Large commercial |

Prin's Si Racha factory has a **Howick FRAMA**. FrameBuilderMRD exports to the
**Howick_H400** format, which is compatible with the FRAMA machine — the H400
CSV format is what the FRAMA reads.

---

## Machine PC and software

The Howick FRAMA has a **built-in Windows PC** with a touchscreen panel and
physical controls (START, PAUSE, AUTO, RESET, PUNCH, E-STOP).

| Item | Value |
|------|-------|
| Software | Howick FRAMA (FramingMachine) |
| Version | **3.2.0.0** |
| Edition | Unrestricted Edition |
| Licensed to | Howick Limited |
| OS | Windows (embedded) |
| Remote access | **TeamViewer 13** installed |

The operator browses the USB drive from within the Howick software and selects
the job folder — there is no fixed folder path.

## OPEN: Can the Howick software be driven programmatically?

**This is the key unknown.** Right now the operator must manually browse and
select a job from the Howick FRAMA touchscreen UI. If we can automate this
step, job delivery becomes fully hands-free.

### What we need to investigate

- [ ] Does `FramingMachine.exe` accept **command-line arguments**?
  e.g. `FramingMachine.exe --load D:\jobs\T1.csv`
- [ ] Is there a **watched/auto-load folder** that the software picks up from?
- [ ] Is there a **config file** that sets a default job directory?
- [ ] Does the software expose any **API, OPC UA, or MQTT** endpoint?
  (some newer Howick firmware versions may have this)
- [ ] Can a job be loaded via **Windows automation** (SendKeys, UI Automation)?
  Last resort — fragile but possible since it's a Windows PC.

### How to find out

1. **Ask Howick directly** — contact Howick Ltd (NZ) and ask if v3.2.0.0
   supports any form of programmatic job loading or auto-start
2. **Investigate on the machine PC** — via TeamViewer:
   - Check `FramingMachine.exe /?` or `--help` from command prompt
   - Look for config/INI files in the install directory
   - Check if there's a newer firmware version with automation features
3. **Check Howick documentation** — any machine integration guide or API docs

### Impact

| Scenario | Result |
|----------|--------|
| **No automation** | Pi Zero presents CSV on USB, operator picks job from touchscreen. Eliminates the walk + USB stick but still needs manual selection. |
| **Watched folder** | Pi Zero writes CSV, machine auto-loads. Fully hands-free. |
| **Command-line / API** | Pi Zero or Pi 5 triggers job load remotely. Fully hands-free. |

Even without automation, opcua-howick still eliminates the USB stick walk.
With automation, the entire process is zero-touch from dashboard to machine.

### Research result (March 2026)

Checked Howick's website, software pages, and file converter docs. **No
programmatic job loading exists.** The FRAMA Machine Control is a closed
Windows UI — no API, no watched folder, no command line, no OPC UA/MQTT.
The operator must always select the job from the touchscreen. Current
software version is 3.6.0.0 (Prin has 3.2.0.0).

Sources checked:
- [FRAMA Machine Control Software](https://www.howickltd.com/software/frama-machine-control-software)
- [Howick Software Partners](https://www.howickltd.com/software)
- [Howick File Converter](https://www.howickltd.com/software/howick-file-converter)

### Future option: Windows UI automation

Since the FRAMA runs on a Windows PC with TeamViewer installed, a small
automation agent on the machine PC could drive the Howick UI to achieve
fully zero-touch job loading:

```
Pi Zero writes CSV to USB gadget
  → automation agent detects new file
  → opens Howick FRAMA software file dialog
  → navigates to USB drive
  → selects the job CSV
  → confirms load
  → screen capture verifies correct job loaded
```

**Implementation options:**

| Approach | Pros | Cons |
|----------|------|------|
| AutoHotKey | Simple, tiny, runs on old Windows | Pixel-based, fragile if UI changes |
| Python + pyautogui | Screen capture + mouse/keyboard | Needs Python on machine PC |
| Windows UI Automation API | Proper button IDs, not pixels | Harder, needs accessible UI |
| Rust + windows crate | Fits our stack, could be a new binary | More effort upfront |

**Priority:** Phase 3 — the immediate win (no USB stick, no walking) is
already delivered by opcua-howick. UI automation adds zero-touch on top,
but is fragile and needs testing on the actual machine.

**Risk:** Howick software updates could break the automation. Pin the
software version or detect UI changes via screen capture and alert.

---

## Job input

- **Transport:** USB mass storage device (USB stick or Pi Zero USB gadget)
- **File format:** CSV (`.csv`)
- **Design software:** SketchUp → FrameBuilderMRD plugin → CSV export

---

## CSV format (Howick frameset)

The CSV is a flat, comma-separated file with no header row. Each line is a keyword-prefixed record.

### Record types

| Keyword | Purpose | Example |
|---------|---------|---------|
| `UNIT` | Measurement unit | `UNIT,MILLIMETRE` |
| `PROFILE` | Steel profile code + description | `PROFILE,S8908,Standard Profile` |
| `FRAMESET` | Frameset name (= job name) | `FRAMESET,T1` |
| `COMPONENT` | One steel member to roll-form | see below |

### COMPONENT record

```
COMPONENT,<id>,<label_mode>,<qty>,<length_mm>,<op>,<pos>,<op>,<pos>,...
```

| Field | Description |
|-------|-------------|
| `id` | Component ID within frameset (e.g. `T1-1`) |
| `label_mode` | `LABEL_NRM` (normal) or `LABEL_INV` (inverted) |
| `qty` | Quantity (usually `1`) |
| `length_mm` | Total component length in mm |
| Operations | Pairs of `<operation>,<position_mm>` along the length |

### Operations

| Operation | Description |
|-----------|-------------|
| `DIMPLE` | Dimple punch at position |
| `LIP_CUT` | Lip cut at position |
| `SWAGE` | Swage at position |
| `WEB` | Web punch at position |
| `END_TRUSS` | End-of-truss marker at position |

### Example (T1 truss, first component)

```csv
UNIT,MILLIMETRE
PROFILE,S8908,Standard Profile
FRAMESET,T1
COMPONENT,T1-1,LABEL_INV,1,3945.0,DIMPLE,20.65,DIMPLE,212.52,...,WEB,2634.7,WEB,1755.6
```

### Fixture files

- [dev/fixtures/T1.csv](../../dev/fixtures/T1.csv) — truss frameset (22 components)
- [dev/fixtures/W1.csv](../../dev/fixtures/W1.csv) — wall frameset
- [dev/fixtures/Gerard_25032026/](../../dev/fixtures/Gerard_25032026/) — full sample set from Prin (March 2026):
  - `Howick_H400/.../25062026_wall1_walls.csv` — wall1 (50 components, profile 350S162-33)
  - `Howick_H400/.../25062026_wall2_walls.csv` — wall2 (7 components)
  - Includes FrameCad XML exports and SketchUp layout for comparison

---

## OPC UA integration

- **Namespace URI:** `urn:howick-frama` (config-driven)
- **Delivery:** Pi Zero 2W presents as USB mass storage gadget, CSV written to the virtual drive
- **See:** [ADR 0003 — Multi-agent node managers](../adr/0003-multi-agent-node-managers.md)

---

## References

- [Howick FRAMA 3200](https://www.howickltd.com/products/frama-3200)
- [Howick FRAMA 4200](https://www.howickltd.com/products/frama-4200)
- [Howick FRAMA 5600](https://www.howickltd.com/machines/frama-5600)
- [Howick Buyer Guide (PDF)](https://www.buildersshow.com/assets/docs/ibs/pressReleases/ExhibitorProductLit_41804_HowickFRAMAQuickFireMachineBuyersGuide_1.pdf)
- [howick-rs CSV parser](https://github.com/joeblew999/howick-rs)
