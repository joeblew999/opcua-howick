# speckle-watcher

Bridges SketchUp → Speckle → Howick CSV machine files.

See [ADR 0004](../../docs/adr/0004-speckle-sketchup-bridge.md) for architecture
and [ADR 0005](../../docs/adr/0005-speckle-geometry-findings.md) for the geometry
analysis that proves this works.

## What this does

1. **inspect_skp** — connects to Speckle, dumps the geometry tree from an
   uploaded SketchUp model (layers, meshes, groups, materials)
2. **converter** — reads FrameBuilderMRD layer structure from Speckle (Stud,
   Nog, BottomPlate, TopPlate, etc.), extracts member positions and punch
   operation positions from mesh vertices, outputs Howick CSV
3. **watcher** (planned) — polls Speckle for new model versions, auto-runs
   the converter

## Prerequisites

- Python 3.12+ (managed via mise — see `.mise.toml`)
- A [Speckle account](https://app.speckle.systems) (free tier is fine)
  **Note:** Gmail/free email providers may not work for signup — use a
  business domain email (e.g. ubuntusoftware.net).
- A personal access token from Speckle developer settings

## Setup

```bash
# Install Python 3.12 via mise (one-time)
mise install python@3.12

# Install dependencies
pip3 install specklepy

# Copy and edit .env
cp .env.example .env
# Edit .env with your token
```

## Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SPECKLE_TOKEN` | (required) | Personal access token |
| `SPECKLE_SERVER` | `https://app.speckle.systems` | Speckle server URL |

Token URL: https://app.speckle.systems/settings/user/developer

## Speckle project (already set up)

| Item | Value |
|------|-------|
| Workspace | gerard-test (ID: `8d4725a853`) |
| Project | opcua-howick-test (ID: `f3318660fc`) |
| Model | wall-frames (ID: `c6539853c0`) |
| Source file | `dev/fixtures/Gerard_25032026/25062026.skp` |
| Console | https://app.speckle.systems/projects/f3318660fc |

## Usage

### Inspect a model

```bash
SPECKLE_TOKEN=xxx PYTHONPATH=tools/speckle-watcher/src \
  python3 -m speckle_watcher.inspect_skp

# Or with specific project/model IDs:
SPECKLE_TOKEN=xxx PYTHONPATH=tools/speckle-watcher/src \
  python3 -m speckle_watcher.inspect_skp f3318660fc c6539853c0
```

### Convert to Howick CSV

```bash
# Print CSV to stdout
SPECKLE_TOKEN=xxx PYTHONPATH=tools/speckle-watcher/src \
  python3 -m speckle_watcher.converter f3318660fc c6539853c0

# Write to file
SPECKLE_TOKEN=xxx PYTHONPATH=tools/speckle-watcher/src \
  python3 -m speckle_watcher.converter f3318660fc c6539853c0 output/
```

## How the converter works

FrameBuilderMRD (the SketchUp plugin) creates a standard layer structure that
Speckle preserves:

```
wall/
  Stud          — 139 elements → 11 real members
  Nog           — 122 elements → 10 real members
  BottomPlate   —  70 elements →  6 real members
  TopPlate      — 107 elements →  6 real members
  window        —   5 elements → 23 real members
  generic_frame —   3 elements →  4 real members
  lateralbrace  —   1 element  →  7 real members
```

The converter:
1. Filters real members (20+ vertices) from tiny punch markers (4 vertices)
2. Recursively extracts meshes from nested collections
3. Detects member axis and length from bounding box
4. Extracts punch operation positions from vertex clustering patterns
5. Classifies operations (DIMPLE, SWAGE, SERVICE_HOLE, LIP_CUT)
6. Outputs Howick CSV with COMPONENT records

### Current status (v0.2)

| Metric | Generated | Original | Notes |
|--------|-----------|----------|-------|
| Components | 67 | 50 | Need to filter duplicates |
| Key positions | Within 0.5mm | Exact | Binning resolution |
| DIMPLE | ✅ | ✅ | Working |
| SWAGE | ✅ | ✅ | Working |
| SERVICE_HOLE | ✅ | ✅ | Working |
| LIP_CUT | ⚠️ | ✅ | Needs tuning |
| NOTCH/WEB | ❌ | ✅ | Not yet classified |

## Project structure

```
tools/speckle-watcher/
  .mise.toml                    — Python 3.12
  .env                          — your token (gitignored)
  .env.example                  — template
  .gitignore
  pyproject.toml                — dependencies (specklepy)
  README.md                     — this file
  src/speckle_watcher/
    __init__.py
    inspect_skp.py              — inspect Speckle model geometry
    converter.py                — Speckle → Howick CSV converter
```
