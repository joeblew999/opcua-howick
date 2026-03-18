# ADR-0002: Support Two Design Workflows Permanently

**Status:** Accepted
**Date:** March 2026
**Context:** opcua-server + plat-trunk factory integration

---

## Context

### Strategic context

This engagement with Prin's factory (Si Racha, Thailand) serves two purposes:

1. **Immediate value** — automates job delivery to the Howick FRAMA, saving the
   USB-stick walk for every job.

2. **R&D testbed** — provides real hardware, real process flows, and real production
   CSV files to build the plat-trunk Framing Extractor against. Getting plat-trunk's
   CAD to the feature completeness of SketchUp takes time. Building it against
   theoretical data produces theoretical results. Prin's factory provides:
   - Real job types: roof trusses (T1), wall frames (W1), with real operations
     (DIMPLE, LIP_CUT, SWAGE, WEB, END_TRUSS, SERVICE_HOLE, NOTCH)
   - Real hardware constraints: USB-only machine, no network, no attached PC
   - Real operator workflow to design UX around

The golden files in `dev/fixtures/` are not just test data — they are the
specification that the plat-trunk Framing Extractor must eventually match exactly.

### The two workflows

Two completely separate design tools can generate Howick FRAMA job CSVs:

1. **SketchUp + FrameBuilderMRD** — Prin's existing workflow on his Design PC.
   SketchUp is Prin's design tool. FrameBuilderMRD is Howick's own software
   that converts SketchUp designs into the Howick FRAMA CSV format.

2. **plat-trunk Framing Extractor** — ubuntu Software's 3D CAD platform,
   based on STEP files (ISO 10303 open format). The Framing Extractor reads
   a STEP model and generates the same Howick FRAMA CSV. No knowledge of
   SketchUp. No dependency on FrameBuilderMRD.

plat-trunk's CAD platform has a long way to go before it is production-ready.
Prin uses SketchUp today and will continue to do so for the foreseeable future.

Both tools produce identical CSV output — the format is the contract.

---

## Decision

**Support both workflows permanently and equally.**

Neither workflow is a stepping stone to the other. They are independent paths
for different users and contexts:

- Workflow 1 (SketchUp) is for Prin's existing operator workflow
- Workflow 2 (plat-trunk) is for users of ubuntu Software's CAD platform

opcua-server and howick-frama must never require one workflow over the other.
The CSV is the only interface that matters.

---

## Consequences

### For opcua-server

- The dashboard upload UI (`POST /upload`) is a first-class feature, not a
  temporary workaround. It is the primary entry point for Workflow 1.
- The job poller (`src/poller.rs`) handles Workflow 2 — polling plat-trunk
  for pending jobs. Both paths write to the same job queue and file watcher.
- Configuration (`opcua-server.dev.toml`) must support both paths without code changes.

### For howick-frama

- howick-frama polls whatever URL is configured in `plat_trunk.url`.
- In Workflow 1: points at opcua-server's local queue endpoint.
- In Workflow 2: points at plat-trunk's API.
- Same binary, same code, different config.

### For the CSV format

- The golden files in `dev/fixtures/T1.csv` and `W1.csv` are the authoritative
  reference for both workflows.
- Any Framing Extractor changes in plat-trunk must produce output compatible
  with these files.
- opcua-server and howick-frama treat CSV content as opaque — they deliver it,
  they do not validate or parse it.

### For testing

- All local dev tests use the real golden fixture files, not synthetic data.
- `mise run dev:job` drops a real fixture into the pipeline.
- `mock-plat-trunk` serves the real fixture files to simulate Workflow 2.

---

## Rejected alternatives

**Make SketchUp a transition path only** — rejected. plat-trunk CAD is not
ready and may never fully replace SketchUp for Prin's use case. Treating
Workflow 1 as temporary would leave Prin without a working system.

**Build a SketchUp plugin for plat-trunk** — out of scope. plat-trunk has no
knowledge of SketchUp and should not. The CSV format is the boundary.

---

## Related

- `docs/customer/system-overview.md` — full topology, workflow, and architecture detail
- ADR-0001 — deployment topologies (Cloud / LAN / Hybrid)
- plat-trunk repo — Framing Extractor, STEP CAD, job queue API
