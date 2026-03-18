# ADR 0003 — Multi-Agent NodeManager Architecture

**Status:** Accepted
**Date:** 2026-03-18

---

## Context

The initial implementation hardcoded `const NS_URI: &str = "urn:howick-edge-agent"` in both the
OPC UA server and client. This tied the address space to a single machine type (Howick FRAMA)
at the source-code level.

As additional machine types are added (other roll-formers, CNC machines, press brakes), each
needs its own OPC UA namespace so that:

1. Multiple machine types can be served by a single `opcua-server` instance
2. Each edge agent can locate its own namespace by URI without hardcoding array indices
3. A new machine type requires only a new config file — no code changes

The `async-opcua` `node-managers` sample demonstrates exactly this pattern: one
`.with_node_manager()` call per machine type, each owning its own namespace URI.

---

## Decision

### 1. Namespace URI is config-driven

`opcua.namespace_uri` in every config file declares which OPC UA namespace this binary uses:

```toml
[opcua]
namespace_uri = "urn:howick-frama"   # machine-type identifier
```

- `opcua-server` registers this URI when building its `SimpleNodeManager`
- `howick-frama` (edge agent) resolves the namespace index by reading the server's
  `Server_NamespaceArray` at connect time

No hardcoded URIs anywhere in source code.

### 2. Binary names match machine types

| Binary | Namespace URI | Machine |
|--------|--------------|---------|
| `opcua-server` | (generic host) | Pi 5 / NUC / Mac — serves any machine type |
| `howick-frama` | `urn:howick-frama` | Howick FRAMA roll-former |
| `howick-cnc` *(future)* | `urn:howick-cnc` | Future: CNC machine |

`opcua-server` is generic — it reads `opcua.namespace_uri` from config and registers whatever
URI is supplied. Adding a new machine type means:

1. New config file: `new-machine.pi-zero.toml` with `namespace_uri = "urn:new-machine"`
2. New agent binary (or reuse `howick-frama` with different config if address space is the same)
3. No changes to `opcua-server`

### 3. Multiple node managers per server (future)

When a single `opcua-server` instance needs to host multiple machine types simultaneously,
add multiple `.with_node_manager()` calls:

```rust
// Each machine type gets its own namespace — same pattern as async-opcua node-managers sample
ServerBuilder::new_anonymous(app_name)
    .with_node_manager(simple_node_manager(
        NamespaceMetadata { namespace_uri: "urn:howick-frama".to_owned(), ..Default::default() },
        "howick-frama",
    ))
    .with_node_manager(simple_node_manager(
        NamespaceMetadata { namespace_uri: "urn:howick-cnc".to_owned(), ..Default::default() },
        "howick-cnc",
    ))
```

The agent for each machine type resolves its own namespace index by URI — agents for different
machine types coexist on the same network and connect to the same server without collision.

### 4. application_uri ≠ namespace_uri

The server's OPC UA `application_uri` (its own identity in the namespace table) must be
distinct from the machine namespace URI. Convention: `{namespace_uri}-server`.

For `namespace_uri = "urn:howick-frama"`:
- `application_uri` = `"urn:howick-frama-server"` (registered at index 1)
- machine namespace = `"urn:howick-frama"` (registered at index 2)

If they were the same, clients would resolve to index 1 (server identity) where no nodes
are registered, not index 2 (node manager) where the address space lives.

---

## Consequences

**Good:**
- Adding a new machine type is zero-code: write a config, name the binary, deploy
- OPC UA address spaces are cleanly isolated by namespace URI
- Agents find their namespace by URI — robust to future server restarts or namespace reordering
- Naming is consistent: binary name, config file prefix, and namespace URI all say the same thing

**Neutral:**
- Config files must always include `namespace_uri` — the field has a sensible default
  (`urn:howick-frama`) so existing deployments without the field continue to work

**Trade-off considered:**
- Using `InMemoryNodeManager` (current) vs `NodeManager` trait (fully dynamic, e.g. DB-backed).
  The current `InMemoryNodeManager` with `SimpleNodeManagerImpl` is sufficient for the known
  address space. If nodes need to be dynamically discovered from the machine at connect time,
  implementing the `NodeManager` trait would be appropriate — but that is out of scope until
  a second machine type requires it.
