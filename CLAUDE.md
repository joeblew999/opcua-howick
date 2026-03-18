# CLAUDE.md — opcua-howick

## MANDATORY: Clone async-opcua before any OPC UA work

Before implementing or modifying any OPC UA features, clone the reference library:

```bash
git clone https://github.com/FreeOpcUa/async-opcua/ /private/tmp/async-opcua
```

Then read:
- `/private/tmp/async-opcua/docs/client.md` — client lifecycle, subscriptions, event loop
- `/private/tmp/async-opcua/docs/server.md` — server builder, address space, node manager
- `/private/tmp/async-opcua/samples/simple-client/src/main.rs` — subscription pattern
- `/private/tmp/async-opcua/samples/demo-server/src/methods.rs` — OPC UA method callbacks

Web references (use if the clone is unavailable):
- https://github.com/FreeOpcUa/async-opcua — source + all samples
- https://docs.rs/async-opcua/latest/opcua — full API docs
- https://github.com/FreeOpcUa/async-opcua/blob/master/docs/client.md — client guide
- https://github.com/FreeOpcUa/async-opcua/blob/master/docs/server.md — server guide
- https://github.com/FreeOpcUa/async-opcua/tree/master/samples — all sample code

Related projects:
- https://github.com/joeblew999/opcua-howick — this repo
- https://github.com/joeblew999/howick-rs — Howick CSV parser (frameset format)

**Never reinvent what is already in async-opcua.** If you are about to write something OPC UA related from scratch, stop and check the samples first.

---

## Architecture

### Two binaries

| Binary | Target | Role |
|--------|--------|------|
| `opcua-server` | Pi 5 / NUC / Mac | OPC UA server + HTTP server + file watcher + job poller |
| `howick-frama` | Pi Zero 2W | Minimal: subscribes to Pi 5 OPC UA server, writes CSV to USB gadget |

### Cargo workspace layout

Four crates under `crates/`:

| Crate | Binary | Target | Role |
|-------|--------|--------|------|
| `crates/core` (`opcua-howick`) | — | both | Shared: config, machine state, updater, http_poller, usb_gadget |
| `crates/opcua-server` | `opcua-server` | Pi 5 / NUC / Mac | OPC UA server + HTTP server + file watcher + job poller |
| `crates/howick-frama` | `howick-frama` | Pi Zero 2W | OPC UA client-only + USB gadget write |
| `crates/mock-plat-trunk` | `mock-plat-trunk` | dev | Mock plat-trunk HTTP server for testing |

### Module layout

| Path | Used by | Purpose |
|------|---------|---------|
| `crates/core/src/config.rs` | both | Configuration types and loader |
| `crates/core/src/machine.rs` | both | Shared machine state, job types |
| `crates/core/src/updater.rs` | both | Self-update logic |
| `crates/core/src/http_poller.rs` | both | HTTP polling of plat-trunk for pending jobs |
| `crates/core/src/usb_gadget.rs` | both | USB mass storage gadget write (shared — http_poller and opcua_client both use it) |
| `crates/opcua-server/src/job_server/opcua_server.rs` | Pi 5 only | OPC UA server — exposes address space |
| `crates/opcua-server/src/job_server/http.rs` | Pi 5 only | HTTP JSON API + dashboard |
| `crates/opcua-server/src/job_server/watcher.rs` | Pi 5 only | File watcher for dropped CSVs |
| `crates/howick-frama/src/edge_agent/opcua_client.rs` | Pi Zero only | OPC UA subscription client |
| `crates/howick-frama/src/edge_agent/sensor.rs` | Pi Zero only | Coil weight sensor push |

**Dependency isolation**: `howick-frama` declares `async-opcua = { features = ["client"] }` only — no server code compiled for the Pi Zero binary.

### OPC UA is the M2M backbone

OPC UA is the **primary transport** between Pi Zero and Pi 5. This is real industrial-grade OPC UA — the same protocol used to connect SCADA systems to Siemens PLCs and Fanuc CNCs.

- Pi 5 runs `opcua-server` — exposes machine state + job queue as OPC UA nodes
- Pi Zero runs `howick-frama` — **subscribes** to `Jobs/PendingJobId`, server pushes instantly on change
- No polling. No custom protocol. Standard OPC UA subscriptions.

HTTP API (`job_server/http.rs`) is for the browser dashboard only — Tauri app or direct browser.

### OPC UA address space

```
/Howick/
  Machine/
    Status           String  — "Running" | "Idle" | "Error" | "Offline"
    CurrentJob       String
    PiecesProduced   UInt32
    CoilRemaining    Double  (metres)
    LastError        String
  Jobs/
    QueueDepth       UInt32
    CompletedCount   UInt32
    PendingJobId     String  — job_id of next pending job ("" = none)
    PendingJobName   String  — frameset name
    PendingJobCsv    String  — full CSV content
    CompleteJob      Method  — call with job_id to mark delivered
```

Namespace URI: `urn:howick-frama` (config-driven via `opcua.namespace_uri` — each machine type gets its own URI)

### OPC UA subscription pattern (howick-frama, Pi Zero)

```rust
// DataChangeCallback fires synchronously on session event loop thread
// Use Arc<Mutex<T>> + Arc<Notify> to bridge to async main loop
let pending_job: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
let notify = Arc::new(tokio::sync::Notify::new());
// ... clone arcs, pass to callback ...
session.create_subscription(Duration::from_millis(500), 10, 30, 0, 0, true,
    DataChangeCallback::new(move |dv, _item| {
        if let Some(Variant::String(s)) = dv.value {
            *pending_clone.lock().unwrap() = Some(s.value().clone().unwrap_or_default());
            notify_clone.notify_one();
        }
    })
).await?;
session.create_monitored_items(sub_id, TimestampsToReturn::Both,
    vec![MonitoredItemCreateRequest::from(NodeId::new(ns, "Jobs/PendingJobId"))]
).await?;
loop { notify.notified().await; /* process job */ }
```

### OPC UA method call (CompleteJob)

```rust
// Tuple syntax: (object_node_id, method_node_id, Option<Vec<Variant>>)
session.call_one((
    jobs_folder_node.clone(),
    complete_job_node.clone(),
    Some(vec![Variant::String(job_id.into())]),
)).await?;
```

---

## Key API facts (async-opcua 0.18)

- **UAString**: `s.value() -> &Option<String>` — extract with `s.value().clone().unwrap_or_default()`
- **Server with random port** (tests): `server.run_with(listener)` not `server.run()`
- **Client trust**: `.trust_server_certs(true).create_sample_keypair(true)` for LAN/tests
- **Retry forever**: `.session_retry_limit(-1)` on ClientBuilder
- **Namespace index**: read `VariableId::Server_NamespaceArray`, find URI in array
- **Method callback** is sync `Fn` — use `Arc<std::sync::Mutex>` to bridge to async state

---

## Commercial context

- This is a **free product** for the customer (Prin). Cost is hardware only.
- OPC UA is a key selling point — document it prominently for Prin and all users.
- Prin uses SketchUp permanently; plat-trunk is Ubuntu Software's STEP-based CAD path (long-term goal).
- Always present new capabilities as "runs alongside" existing workflow — never scary.

---

## Running tests

```bash
cargo test                    # all tests (15 total: 5 updater unit + 3 OPC UA integration + 5 HTTP pipeline + 2 update)
cargo test opcua              # OPC UA integration tests only
cargo test --test pipeline    # HTTP pipeline tests only
RUST_LOG=debug cargo test     # verbose logging
```
