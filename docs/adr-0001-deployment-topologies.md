# ADR-0001: Deployment Topologies — Cloud, LAN, Hybrid

**Status:** Accepted  
**Date:** March 2026  
**Context:** plat-trunk + opcua-howick factory integration

---

## Context

Some factories (like Prin's in Si Racha) will want to run everything on their
local LAN — no cloud dependency for production. Others will want cloud
collaboration. Some will want both.

plat-trunk already runs on Cloudflare Workers. Tauri v2 is already integrated
in plat-trunk. Automerge CRDT is the sync layer. These three facts make
multi-topology deployment achievable without rewriting anything.

---

## The Three Topologies

### Topology A — Cloud Only (current)

```
Designer (browser)
    │
    │ HTTPS
    ▼
Cloudflare Worker (Hono + WASM)
    │
    ├── D1 (SQLite) — metadata
    ├── R2 — Automerge doc bytes, CSV jobs
    └── opcua-howick (edge, on factory LAN)
            │
            │ OPC UA / CSV file drop
            ▼
        Howick Machine
```

Best for: distributed teams, multiple factories, SaaS model.

---

### Topology B — LAN Only (new)

```
Designer (browser, same LAN)
    │
    │ localhost / LAN IP
    ▼
Tauri v2 app (desktop)
    │
    ├── Hono (same code, running in Tauri sidecar)
    ├── SQLite (local D1 equivalent)
    ├── Local filesystem (local R2 equivalent)
    └── opcua-howick (sidecar or separate LAN machine)
            │
            │ OPC UA / CSV file drop
            ▼
        Howick Machine
```

Best for: air-gapped factories, no internet dependency, full data sovereignty.
The browser UI is identical — it just points to localhost instead of CF.

---

### Topology C — Hybrid (offline-first + cloud sync)

```
Designer (browser)
    │
    │ localhost (primary)
    ▼
Tauri v2 (local)              ←──── syncs when online ────→  Cloudflare
    │                                  (Automerge CRDT)
    ├── Local Hono + SQLite + FS
    └── opcua-howick sidecar
            │
            ▼
        Howick Machine
```

Best for: factories that need local reliability but want cloud collaboration
and backup. Automerge CRDT makes this trivial — it was designed for exactly
this use case (offline-first, sync on reconnect, no conflicts).

---

## Why This Works

### Same code everywhere

The Hono backend, WASM Rust logic, and Automerge CRDT are identical across
all three topologies. The only difference is **where they run**:

| Component | Cloud | LAN (Tauri) |
|-----------|-------|-------------|
| Hono routes | CF Worker | Tauri sidecar process |
| Rust WASM | CF Worker WASM | Native binary (no WASM needed) |
| Storage | D1 + R2 | SQLite + local filesystem |
| Auth | CF Access | Local (simplified or skip) |
| Sync | CF R2 as authority | Automerge P2P or CF when online |

### Tauri as the LAN runtime

Tauri v2 is already in plat-trunk. In Topology B/C it does two things:

1. **Serves the browser UI** — same HTML/JS/WASM as the CF deployment
2. **Runs the backend** — Hono server as a Tauri sidecar, same routes

The browser never knows the difference. It makes the same HTTP calls —
they just resolve to localhost instead of workers.example.com.

### opcua-howick placement

| Topology | opcua-howick runs on |
|----------|---------------------|
| A (Cloud) | Separate Pi/NUC on factory LAN, connects to CF |
| B (LAN) | Same machine as Tauri, or Pi on same LAN |
| C (Hybrid) | Same as B, syncs job history to CF when online |

---

## Decision

Design opcua-howick and the plat-trunk Tauri integration so that:

1. **opcua-howick is always a separate process** — never embedded in Tauri.
   It talks to the physical machine and must be restartable independently.

2. **opcua-howick communicates with plat-trunk via the same HTTP API**
   regardless of topology. In cloud: CF Worker URL. In LAN: localhost URL.
   No special-casing.

3. **The Tauri app exposes the same Hono routes as the CF Worker**.
   This means the same OpenAPI schema, same endpoints, same Zod validators.
   Storage adapters swap (R2 → local FS, D1 → SQLite) but the interface
   is identical.

4. **Automerge is the sync layer in all topologies**.
   The CRDT doc is the single source of truth. CF R2 is one possible
   authority; a local file is another. Sync happens when both sides
   are online.

5. **Job files (CSV) are treated as CRDT operations**.
   Submitting a job to the machine is a mutation in the Automerge doc.
   This means job history, status, and output are all synced automatically
   across all connected peers (cloud, other designers, management).

---

## Consequences

- opcua-howick needs a configurable `plat_trunk_url` pointing to either
  CF Worker or localhost depending on topology
- Tauri needs a sidecar config to start Hono on a local port
- The CSV job submission flow is the same in all topologies
- Factories that start on LAN can migrate to cloud/hybrid without
  changing anything on the factory floor

---

## Related

- plat-trunk ADR-0039: Tauri v2 as unification layer
- plat-trunk ADR-0038: Automerge versioning model
- plat-trunk ADR-0008: Sync architecture
- https://github.com/joeblew999/howick-rs
- https://github.com/joeblew999/opcua-howick
