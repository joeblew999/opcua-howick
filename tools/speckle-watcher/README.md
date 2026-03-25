# speckle-watcher

Bridges SketchUp → Speckle → plat-trunk Framing Extractor → opcua-howick.

See [ADR 0004](../../docs/adr/0004-speckle-sketchup-bridge.md) for the full
architecture and rationale.

## What this does

1. **inspect_skp** — uploads a `.skp` file to Speckle and dumps the geometry
   tree to see what meshes, layers, and groups come through
2. **watcher** (planned) — polls Speckle for new model versions, runs the
   Framing Extractor, and pushes machine files to opcua-howick

## Prerequisites

- Python 3.12+ (managed via mise — see `.mise.toml`)
- A [Speckle account](https://app.speckle.systems) (free tier is fine)
  **Note:** Gmail/free email providers may not work for signup — use a
  business domain email (e.g. ubuntusoftware.com).
- A personal access token from your Speckle profile settings

## Setup

```bash
# Install Python 3.12 via mise (one-time)
mise install python@3.12

# Install dependencies
pip3 install specklepy
```

## Quick start — inspect a SketchUp file

### Step 1: Upload to Speckle

1. Go to https://app.speckle.systems
2. Create a new project in the **gerard-test** workspace
3. Drag and drop `dev/fixtures/Gerard_25032026/25062026.skp` into the project
4. Copy the **stream ID** from the URL (the long string after `/streams/`)

### Step 2: Get your token

1. Go to https://app.speckle.systems/settings/user/developer
2. Create a personal access token (all scopes)
3. Copy it

### Step 3: Run the inspector

```bash
cd /path/to/opcua-howick

export SPECKLE_TOKEN="your-token-here"
export SPECKLE_SERVER="https://app.speckle.systems"

PYTHONPATH=tools/speckle-watcher/src \
  python3 -m speckle_watcher.inspect_skp <stream_id>
```

This prints the full object tree — meshes, layers, groups, materials — so
you can see what Speckle extracted from the SketchUp model.

## What we're looking for

The inspector output tells us whether wall detection is feasible:

- **Meshes** — are wall faces preserved as flat rectangular meshes?
- **Layers/Tags** — did Prin use layer names like "Walls", "Openings"?
- **Groups/Components** — are wall panels grouped separately?
- **Dimensions** — can we extract wall length, height, opening positions?

If the geometry is clean enough, the Framing Extractor can detect walls
and generate machine files (Howick CSV, FrameCad RFY) automatically —
no manual re-entry, no FrameBuilderMRD.

## Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SPECKLE_TOKEN` | (required) | Personal access token from Speckle profile |
| `SPECKLE_SERVER` | `https://app.speckle.systems` | Speckle server URL (cloud or self-hosted) |

## Project structure

```
tools/speckle-watcher/
  .mise.toml                    — Python 3.12
  pyproject.toml                — dependencies (specklepy)
  README.md                     — this file
  src/speckle_watcher/
    __init__.py
    inspect_skp.py              — upload + inspect tool
    watcher.py                  — (planned) model-change watcher
    extractor.py                — (planned) wall detection + framing generator
    csv_writer.py               — (planned) Howick CSV output
    rfy_writer.py               — (planned) FrameCad RFY output
```
