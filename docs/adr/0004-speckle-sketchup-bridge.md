# ADR 0004 — Speckle as the SketchUp-to-Machine Bridge

**Status:** Proposed
**Date:** 2026-03-25

---

## Context

### The problem

Prin (EMH, Si Racha) designs buildings in SketchUp. To get a job to the
Howick FRAMA machine, the operator manually re-enters wall frame parameters
into FrameBuilderMRD (a SketchUp plugin, 20,000 THB/year). There is no
automatic link between the 3D SketchUp model and the machine file.

plat-trunk's Framing Extractor can generate machine files (Howick CSV,
FrameCad RFY) from a 3D model — but it works with STEP format, not SketchUp.
Asking Prin to switch from SketchUp to a STEP-based CAD is unrealistic.

### The discovery

[Speckle](https://speckle.systems/) is an open-source AEC data platform that
acts as a hub between CAD tools. It has:

- A **SketchUp connector** that extracts geometry, materials, layers, and
  component/group structure from .skp models
- A **Python SDK** (`specklepy`) for programmatic access to model data
- **Speckle Server** — self-hostable via Docker (Apache 2.0 license)
- Upload/export of IFC, STEP, SKP, and 20+ other formats

### Self-hosting reality

| Component | Self-hostable? | Notes |
|-----------|---------------|-------|
| **Speckle Server** | **Yes** | Docker Compose, Apache 2.0, 6 services + Postgres + Redis + MinIO |
| **Speckle Automate** | **No — cloud only** | Not open source, runs on Speckle infrastructure only |
| **SketchUp connector** | Yes | Ruby extension, installs on Design PC |
| **specklepy SDK** | Yes | `pip install specklepy`, Python 3.10+ |

**Speckle Automate is NOT self-hostable.** It is cloud-only and not open source.
We cannot depend on it. Instead, we build our own model-change watcher using
the `specklepy` SDK — the same polling pattern already used by opcua-howick's
`http_poller`.

### Speckle Server Docker services

```
speckle-ingress       — reverse proxy (port 80)
speckle-frontend-2    — web UI (Nuxt)
speckle-server        — backend API (port 3000)
preview-service       — 3D model previews (3GB RAM limit)
webhook-service       — event hooks
ifc-import-server     — IFC file processing
postgres              — database
redis                 — cache / sessions
minio                 — S3-compatible object storage
```

This can run on a Pi 5 / NUC / any Linux box on the factory LAN, or on a
cloud VPS. The entire stack is open source and Docker-based.

---

## Decision

**Use self-hosted Speckle Server as the bridge between SketchUp and plat-trunk.
Build our own model-change watcher instead of depending on Speckle Automate.**

Prin keeps SketchUp. The Speckle SketchUp connector syncs his model to our
self-hosted Speckle server. Our watcher detects new model versions and runs
the Framing Extractor. No manual re-entry. No cloud dependency.

---

## Architecture

### End-to-end flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  SketchUp    │     │  Speckle     │     │  plat-trunk  │     │  opcua-howick│
│              │     │  Server      │     │  Framing     │     │              │
│  Prin designs│────▶│  (self-      │────▶│  Extractor   │────▶│  Delivers    │
│  3D model    │     │  hosted,     │     │  + model     │     │  CSV/RFY to  │
│              │     │  Docker)     │     │  watcher     │     │  machine via │
│  (unchanged) │     │              │     │              │     │  USB gadget  │
│              │     │  Receives    │     │  Polls for   │     │              │
│              │     │  geometry    │     │  new versions│     │              │
│              │     │  via SU      │     │  detects     │     │              │
│              │     │  connector   │     │  walls,      │     │              │
│              │     │              │     │  generates   │     │              │
│              │     │              │     │  CSV + RFY   │     │              │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
  Design PC           Pi 5 / NUC /         Pi 5 / NUC /         Pi 5 + Pi Zero
                      cloud VPS             same host
```

### Model-change watcher (replaces Speckle Automate)

```python
# Our own watcher — polls Speckle for new model versions
# Same pattern as opcua-howick's http_poller
from specklepy.api.client import SpeckleClient
from specklepy.api import operations
from specklepy.transports.server import ServerTransport

client = SpeckleClient(host="http://speckle.local")
client.authenticate_with_token(token)

last_version = None

while True:
    # Poll for latest version on the model
    versions = client.version.get_versions(model_id=model_id, project_id=project_id)
    latest = versions.items[0].id

    if latest != last_version:
        # New version detected — run Framing Extractor
        ref_obj = versions.items[0].referenced_object
        transport = ServerTransport(stream_id=project_id, client=client)
        model = operations.receive(obj_id=ref_obj, remote_transport=transport)

        # Read FrameBuilderMRD layers, extract members, generate CSV
        frameset = extract_frameset(model)
        howick_csv = frameset.to_csv()
        submit_to_opcua_howick(howick_csv)

        last_version = latest

    sleep(10)  # poll every 10 seconds
```

This runs alongside opcua-howick on the same Pi 5 / NUC. No cloud dependency.
No Speckle Automate subscription. Fully self-hosted.

### What Speckle extracts from SketchUp

The SketchUp connector converts to generic Speckle primitives:

| SketchUp entity | Speckle representation |
|----------------|----------------------|
| Faces | Meshes |
| Edges | Lines |
| Groups / Components | Block Instances |
| Materials | Opacity, metalness, roughness, diffuse, emissive |
| Layers / Tags | Preserved |
| Attributes | Basic only — dynamic components may not transfer |

**Important:** Speckle does NOT extract BIM concepts (walls, openings, etc.)
from SketchUp. The SketchUp connector sends raw geometry. The Framing
Extractor must **detect** structural elements from the geometry itself.

### What plat-trunk's Framing Extractor must do

1. **Receive** geometry from Speckle via `specklepy` Python SDK
2. **Detect wall planes** from mesh faces — flat rectangular surfaces,
   grouped by layer/tag
3. **Find openings** — holes in wall planes (doors, windows)
4. **Extract dimensions** — wall length, height, opening positions and sizes
5. **Generate framing layout** — apply framing rules:
   - Stud spacing (e.g. 600mm centres)
   - Top plate, bottom plate
   - Noggings at mid-height and around openings
   - Header and sill for openings
   - End studs, king studs, jack studs
6. **Output machine files:**
   - Howick CSV (for Howick FRAMA machines)
   - FrameCad RFY (for FrameCad machines — format TBC, need samples)
7. **Generate assembly layout** with per-piece legend (replaces FrameBuilderMRD output)

---

## What this replaces

| Current step | Replaced by | Savings |
|-------------|------------|---------|
| Manual re-entry into FrameBuilderMRD | Automatic wall detection from geometry | Time + errors |
| FrameBuilderMRD license | Not needed | 20,000 THB/year |
| USB stick walk | opcua-howick Pi Zero delivery | Time |
| No job queue | opcua-howick dashboard | Visibility |
| FrameCad software license (for FrameCad machines) | Direct RFY generation | $1,000–$3,000/year |
| Speckle Automate (cloud, not open source) | Our own model-change watcher | Independence |

---

## What this does NOT replace

- **SketchUp** — Prin keeps it. That's the whole point.
- **Operator judgment** — the Framing Extractor applies rules, but the
  operator may need to review/approve the framing layout before sending
  to the machine.
- **SketchUp layout with legend** — plat-trunk needs to generate an
  equivalent assembly document (could push back to Speckle for 3D viewing,
  or generate a PDF).

---

## Deployment options

### Option A — Pi 5 runs everything

```
Pi 5:
  - Speckle Server (Docker)
  - Model watcher + Framing Extractor
  - opcua-howick (opcua-server)

Pi Zero:
  - howick-frama (USB gadget delivery)
```

Lightweight but the Pi 5 runs a lot. Speckle's preview-service wants 3GB RAM
— may need to disable it or run without previews on a 4GB Pi 5.

### Option B — Separate NUC / VPS for Speckle

```
NUC / VPS:
  - Speckle Server (Docker)
  - Model watcher + Framing Extractor

Pi 5:
  - opcua-howick (opcua-server)

Pi Zero:
  - howick-frama (USB gadget delivery)
```

Better separation. Speckle gets its own resources. Could be a $5/month VPS.

### Option C — Speckle cloud free tier + self-hosted watcher

```
Speckle Cloud (app.speckle.systems):
  - Speckle Server (free tier)

Pi 5 / NUC:
  - Model watcher + Framing Extractor (polls Speckle cloud)
  - opcua-howick

Pi Zero:
  - howick-frama
```

Simplest to start with. No Docker setup. Free tier may have limits.
Still fully independent — we poll their API, we don't use Automate.

---

## Risks and unknowns

| Risk | Mitigation |
|------|-----------|
| SketchUp geometry too ambiguous — can't reliably detect walls | Require Prin to use consistent layer naming (e.g. "Walls", "Openings") |
| Framing rules vary by building code / region | Make rules configurable, start with Thai/NZ standards |
| FrameCad RFY format unknown | Blocked until we get samples from Steel Solutions or Maxxi Factory |
| Speckle SketchUp connector is "early development" | Monitor development, contribute fixes upstream if needed |
| Speckle Server too heavy for Pi 5 | Use Option B (NUC) or Option C (cloud free tier) |
| Speckle changes API or licensing | We only depend on the open-source server + SDK, both Apache 2.0 |

---

## Alternatives considered

**1. Build a SketchUp plugin that generates CSV directly**
Rejected — this is what FrameBuilderMRD already does, and it requires manual
entry. We'd be rebuilding their product.

**2. Require Prin to switch to STEP-based CAD (plat-trunk native)**
Rejected — Prin won't switch. This would kill the engagement.

**3. Export SketchUp to IFC, import into plat-trunk**
Possible fallback — SketchUp has native IFC export. IFC carries more
structural semantics than raw Speckle geometry. But it's file-based (no
auto-trigger) and SketchUp's IFC export is limited.

**4. Use Speckle Automate (cloud)**
Rejected — Automate is not open source and not self-hostable. We don't want
a cloud dependency for core functionality.

**5. Use self-hosted Speckle Server + our own watcher (this ADR)**
Best option — Prin changes nothing, we control the full stack, open source,
and the Python SDK makes the Framing Extractor straightforward.

---

## Next steps

1. [ ] Try Option C first — use Speckle cloud free tier, send Prin's .skp
   through it, inspect output via `specklepy`
2. [ ] Prototype wall detection from Speckle meshes in Python
3. [ ] If cloud free tier works, deploy watcher on Pi 5 alongside opcua-howick
4. [ ] If we need self-hosted, set up Speckle Server Docker on NUC
5. [ ] Get RFY samples from Maxxi Factory / Steel Solutions
6. [ ] Build Framing Extractor
7. [ ] Connect output to opcua-howick job queue
8. [ ] Install Speckle SketchUp connector on Prin's Design PC for live testing

---

## Related

- [ADR 0002 — Two design workflows](0002-two-design-workflows.md) — SketchUp + plat-trunk both supported
- [ADR 0003 — Multi-agent node managers](0003-multi-agent-node-managers.md) — per-machine namespace URIs
- [ADR 0005 — Speckle geometry findings](0005-speckle-geometry-findings.md) — confirms framing already in SketchUp model (FrameBuilderMRD layers)
- [Prin workflow](../customer/prin/03-current-workflow.md) — current manual process
- [Howick FRAMA machine profile](../machines/howick-frama.md) — CSV format spec
- [FrameCad machine profile](../machines/framecad.md) — RFY/XML formats, file ecosystem

---

## References

- [Speckle](https://speckle.systems/)
- [Speckle Server GitHub](https://github.com/specklesystems/speckle-server) — Apache 2.0
- [Speckle Server Docker Compose](https://github.com/specklesystems/speckle-server/blob/main/docker-compose-speckle.yml)
- [Speckle SketchUp connector](https://speckle.systems/integrations/sketchup/)
- [speckle-sketchup GitHub](https://github.com/specklesystems/speckle-sketchup)
- [specklepy Python SDK](https://docs.speckle.systems/developers/sdks/python/introduction)
- [Speckle Automate FAQ — not self-hostable](https://docs.speckle.systems/developers/automate/faq)
- [Speckle Cloud vs Self-Hosting](https://speckle.systems/blog/speckle-cloud-vs-self-hosting/)
