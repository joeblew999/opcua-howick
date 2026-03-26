# IROH — Evaluation for opcua-howick

> **Status:** Research / evaluation (not yet adopted)
> **Date:** 2026-03-26
> **Goal:** Isomorphic remote operations — one protocol, one binary, all targets

---

## What is iroh?

[iroh](https://github.com/n0-computer/iroh) is a Rust networking library by [Number Zero (n0)](https://iroh.computer) that lets devices connect directly using public keys instead of IP addresses. QUIC-based, encrypted, NAT-traversing, relay-fallback.

- **Not a CLI** — it's a library (the CLI was removed in v0.29, Nov 2024)
- **Not a VPN** — it's a connection layer for building protocols
- **Not IPFS** — they broke from IPFS/libp2p in 2023 to go leaner

**Current version:** v0.97 (release candidate, 1.0 target Q1-Q2 2026)

---

## Why iroh for opcua-howick?

### The problem

Our deploy tasks use **three different transports** for three targets:

| Target | Transport | Works? |
|--------|-----------|--------|
| Pi 5 (Linux) | SSH + SCP | Yes |
| Pi Zero 2W (Linux) | SSH + SCP | Yes |
| Windows Design PC | SMB file share (`//howick-pc/c$/...`) | Fragile |

SSH doesn't exist natively on Windows. SMB is a hack. We want **one protocol for all targets**.

### Why not just enable OpenSSH on Windows?

It works, but:
- Requires manual Windows configuration on customer hardware
- SSH is not Rust-native (depends on OpenSSH/sshd)
- No NAT traversal — needs Tailscale or port forwarding
- Tailscale is too heavy for Pi Zero 2W (512MB RAM)

### What iroh gives us

- **One protocol** — QUIC over iroh-net, same code on all platforms
- **NAT traversal built in** — hole-punching + relay fallback, no Tailscale needed
- **Tiny footprint** — runs on ESP32 (4MB flash, 4MB PSRAM), Pi Zero is luxury
- **No central server needed** — discovery via BitTorrent Mainline DHT (10M+ nodes)
- **Pure Rust** — cross-compiles to all our targets
- **Encrypted end-to-end** — QUIC TLS, relay can't read traffic

---

## The iroh ecosystem

### Core (by n0-computer)

| Crate | What | Status |
|-------|------|--------|
| [**iroh**](https://github.com/n0-computer/iroh) | P2P QUIC endpoints, hole-punching, relay, Router | v0.97 RC |
| [**iroh-blobs**](https://github.com/n0-computer/iroh-blobs) | BLAKE3 content-addressed blob transfer | v0.99 (rewrite, use v0.35 for prod) |
| [**iroh-gossip**](https://github.com/n0-computer/iroh-gossip) | Pub-sub overlay networks | Active |
| [**iroh-docs**](https://github.com/n0-computer/iroh-docs) | Eventually-consistent key-value store | Active |
| [**iroh-relay**](https://github.com/n0-computer/iroh) | Self-hostable relay server | Included in monorepo |
| [**noq**](https://github.com/n0-computer/noq) | Quinn fork with QUIC multipath, QAD, QNT | Active |

### CLI tools (by n0-computer)

| Tool | What | Install |
|------|------|---------|
| [**sendme**](https://github.com/n0-computer/sendme) | File transfer (like magic-wormhole but BLAKE3) | `cargo install sendme` / `brew install sendme` |
| [**dumbpipe**](https://github.com/n0-computer/dumbpipe) | Unix pipes between devices (netcat over iroh) | `cargo install dumbpipe` / `brew install dumbpipe` |
| [**iroh-doctor**](https://github.com/n0-computer/iroh-doctor) | Network diagnostics | `cargo install iroh-doctor` |
| [**irpc**](https://github.com/n0-computer/irpc) | Streaming RPC over iroh | Library |

### Community tools (most relevant to us)

| Tool | What | By |
|------|------|----|
| [**iroh-ssh**](https://github.com/rustonbsd/iroh-ssh) | SSH without IP addresses — proxy to local sshd via iroh | rustonbsd |
| [**rustpatcher**](https://github.com/rustonbsd/rustpatcher) | P2P binary auto-update with Ed25519 signing | rustonbsd |
| [**iroh-lan**](https://github.com/rustonbsd/iroh-lan) | Virtual LAN (Hamachi replacement, no accounts) | rustonbsd |
| [**pigglet**](https://github.com/andrewdavidmackenzie/pigg) | Headless Pi GPIO agent controlled remotely via iroh | andrewdavidmackenzie |
| [**malai**](https://github.com/fastn-stack/kulfi) | Expose local HTTP/TCP services over P2P | fastn-stack |
| [**lantun**](https://github.com/maxomatic458/lantun) | Tunnel local ports over iroh | maxomatic458 |
| [**dumbpipe**](https://github.com/n0-computer/dumbpipe) | Pipe stdin/stdout between devices | n0-computer |

### Full list

See [awesome-iroh](https://github.com/n0-computer/awesome-iroh) — 40+ projects across AI/ML, gaming, file sharing, social, IoT.

---

## Proof: iroh runs on tiny hardware

### ESP32 (the floor)

[n0-computer/iroh-esp32-example](https://github.com/n0-computer/iroh-esp32-example) — March 2026

| Resource | ESP32 WROVER | Pi Zero 2W | Pi 5 |
|----------|:------------:|:----------:|:----:|
| CPU | 240MHz Xtensa (≈386DX40) | 1GHz ARM64 quad | 2.4GHz ARM64 quad |
| RAM | 4MB PSRAM + 500KB internal | 512MB | 4-8GB |
| Flash/Storage | 4MB | 32GB SD | 32GB+ SD |
| iroh works? | **Yes** (proof of concept) | **Yes** (pigglet proves it) | **Yes** |

If iroh runs on an ESP32 with 4MB, our Pi Zero is 128x the RAM.

### Raspberry Pi (pigglet proves it)

[pigg](https://github.com/andrewdavidmackenzie/pigg) ships pre-built binaries for:
- `aarch64-unknown-linux-gnu` (Pi Zero 2W, Pi 3/4/5)
- `arm-unknown-linux-gnu` (Pi Zero W original)
- `armv7-unknown-linux-musleabihf`

Uses iroh v0.96 — recent canary series. Full QUIC P2P with NAT traversal on every Pi model.

---

## How discovery works (no central server)

### PKARR + Mainline DHT

1. Every iroh endpoint has an **ed25519 keypair**. Public key = endpoint ID.
2. Endpoint signs a DNS packet with its relay URL → publishes to **BitTorrent Mainline DHT** (BEP44).
3. Connecting peer resolves endpoint ID → queries DHT → gets relay URL → connects.
4. The DHT has **~10 million active nodes** and has been running for **15+ years**.

### Three resolution modes (composable)

| Mode | How | Central server? |
|------|-----|:---------------:|
| **PKARR HTTP relay** | PUT/GET to `dns.iroh.link` | Yes (n0's, or self-host) |
| **DNS lookup** | Standard DNS query for `_iroh.<pubkey>.dns.iroh.link` | Yes (DNS server) |
| **Direct DHT** | Participate in BitTorrent Mainline DHT directly | **No** |

For factory deployment: use **Direct DHT** mode — zero dependency on n0's infrastructure.

---

## Relay infrastructure

### Default relays (by n0)

| Region | Server |
|--------|--------|
| NA East | `use1-1.relay.n0.iroh-canary.iroh.link` |
| NA West | `usw1-1.relay.n0.iroh-canary.iroh.link` |
| EU | `euc1-1.relay.n0.iroh-canary.iroh.link` |
| Asia-Pacific | `aps1-1.relay.n0.iroh-canary.iroh.link` |

### Self-hosting

`iroh-relay` binary is in the iroh monorepo. Features:
- Let's Encrypt ACME TLS or manual certs
- QUIC relay on port 7842
- Rate limiting per client
- Key cache for 1M concurrent clients (~56MB RAM)
- Metrics on port 9090

For our Thailand factories: self-host a relay on a cheap VPS in Singapore.

---

## iroh vs alternatives

| | iroh | Tailscale | SSH/SCP | libp2p |
|--|------|-----------|---------|--------|
| Transport | QUIC (via noq) | WireGuard | TCP | Multi (TCP, QUIC, WebRTC) |
| Identity | ed25519 public key | Machine key + account | SSH keys | PeerID (multihash) |
| NAT traversal | Hole-punch + relay | DERP + hole-punch | None (needs VPN) | AutoNAT + relay |
| Central server needed? | No (DHT mode) | Yes (coordination server) | No (but needs IP) | Optional |
| Pi Zero viable? | **Yes** (ESP32 proven) | Heavy (~50-80MB RSS) | Yes | Heavy |
| Windows native? | **Yes** (pure Rust) | Yes | Needs OpenSSH | Yes |
| Self-hostable? | **Yes** (relay + DNS) | No (Tailscale SaaS) | Yes | Yes |
| Language | Rust only | Go | C (OpenSSH) | Go, Rust, JS, etc. |

---

## Number Zero (n0) — company health

| Indicator | Assessment |
|-----------|------------|
| **Team** | 10-20 people, 70+ years combined distributed systems experience |
| **Funding** | Not disclosed, but level of infra (4 global relay regions, DNS, 3-week release cadence) implies VC-backed |
| **Activity** | 3-week release cadence, v0.90→v0.97 in 9 months |
| **Scale** | 500K+ unique nodes/month hitting public network |
| **Ecosystem** | 40+ projects building on iroh |
| **License** | MIT / Apache-2.0 dual |
| **Origin** | Former IPFS implementation, broke away 2023 for performance |
| **Revenue** | Professional services via "n0ps" |

---

## Community tool maturity assessment

### iroh-ssh (rustonbsd) — most relevant

| Factor | Status |
|--------|--------|
| Version | 0.2.9 (pre-0.3) |
| Stars | 158 |
| Code size | ~1000 lines |
| iroh version | **0.94** (3 behind current) |
| ARM builds | **None** — must cross-compile |
| Windows | x86_64 binary available |
| Auth | Proxies to local sshd (standard SSH auth) |
| File transfer | Yes — scp/sftp/rsync all work through tunnel |
| Persistent mode | `--persist` saves keys, stable endpoint ID |
| Pitchfork-compatible | Yes — just run `iroh-ssh server --persist` |
| Risk | Solo maintainer, iroh dep may go stale |

### rustpatcher (rustonbsd) — interesting but not ready

| Factor | Status |
|--------|--------|
| Version | 0.2.2 |
| Stars | 25 |
| Code size | ~1,500 lines |
| Windows | **BROKEN** (nix crate blocks it) |
| Rollback | **None** — bad update bricks devices |
| Crypto | Pre-release ed25519-dalek 3.0.0-pre.1 |
| Risk | No rollback is unacceptable for remote Pi Zeros |

### pigglet (andrewdavidmackenzie) — best reference architecture

| Factor | Status |
|--------|--------|
| Version | Active, iroh 0.96 |
| Stars | 30+ |
| Pi support | All models — Zero through Pi 5, pre-built ARM binaries |
| Architecture | Headless daemon + remote GUI, custom ALPN protocol |
| Relevance | **Exact same pattern we need** — agent on Pi, control from Mac |

---

## Proposed architecture

### What we'd build

A new crate `crates/hoist/` — a thin deploy agent using iroh as the transport layer.

```
hoist (single binary, all platforms)
│
├── hoist agent                    # runs on target (Pi 5, Pi Zero, Windows)
│   ├── iroh Endpoint (persistent)   listen for connections by public key
│   ├── receive files                write to specified paths
│   ├── exec commands                run commands, return output
│   └── health check                 HTTP on localhost:9090
│
└── hoist push <target-id>         # runs on dev Mac
    ├── iroh Endpoint (ephemeral)    connect to target by endpoint ID
    ├── send files                   binary + configs
    └── exec "pitchfork restart"     trigger restart
```

### How it fits the stack

```toml
# mise.toml on every device (installed automatically)
[tools]
"cargo:hoist" = "latest"

# pitchfork.toml on every device (runs as daemon)
[daemons.hoist]
run = "hoist agent --persist"
boot_start = true
retry = true
ready_http = "http://127.0.0.1:9090/health"
```

### Deploy flow (replaces current SSH/SCP/SMB)

```
# From dev Mac — same command for ALL targets:
mise run deploy:pi5        →  hoist push <pi5-endpoint-id> ./bundle/ "pitchfork restart"
mise run deploy:pi-zero    →  hoist push <zero-endpoint-id> ./bundle/ "pitchfork restart"
mise run deploy:windows    →  hoist push <win-endpoint-id> ./bundle/ "pitchfork restart"
```

### Dependencies (minimal)

```toml
[dependencies]
iroh = "0.97"           # P2P QUIC (the only networking dep)
tokio = { version = "1", features = ["full"] }
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
postcard = "1"          # compact binary serialization (same as pigglet uses)
```

No axum, no hyper, no SSH libraries. Just iroh + serialization.

### Protocol (simple)

```
Client                              Agent
──────                              ─────
connect(endpoint_id, "hoist/1")  →  accept QUIC stream
SendFile { path, bytes }         →  write file to disk
SendFile { path, bytes }         →  write file to disk
Exec { command }                 →  spawn command, stream stdout/stderr back
Done                             →  close stream
```

### Target registry

Endpoint IDs stored in project config:

```toml
# config/targets.toml
[pi5]
endpoint_id = "abc123..."
name = "Pi 5 — Prin factory, Si Racha"

[pi-zero]
endpoint_id = "def456..."
name = "Pi Zero 2W — FRAMA machine"

[windows]
endpoint_id = "ghi789..."
name = "Windows Design PC — Prin office"
```

---

## What we could adopt TODAY (without building hoist)

All iroh ecosystem tools are available via **mise** (cargo backend). Add to `.mise.toml`:

```toml
[tools]
"cargo:sendme" = "0.32.0"
"cargo:dumbpipe" = "0.35.0"
"cargo:iroh-ssh" = "0.2.9"
"cargo:iroh-doctor" = "0.97.0"
```

Then `mise install` on any device — Mac, Pi 5, Pi Zero, Windows. Same tools everywhere.

### Tier 1: Zero effort

| Tool | Use case | mise | Latest |
|------|----------|------|--------|
| **sendme** | Ad-hoc file transfer to any device | `"cargo:sendme" = "0.32.0"` | 0.32.0 |
| **dumbpipe** | Pipe data between devices (e.g., stream logs) | `"cargo:dumbpipe" = "0.35.0"` | 0.35.0 |
| **iroh-doctor** | Network diagnostics (NAT check, relay latency) | `"cargo:iroh-doctor" = "0.97.0"` | 0.97.0 |

### Tier 2: Light effort

| Tool | Use case | mise | Latest |
|------|----------|------|--------|
| **iroh-ssh** | SSH without IP — replace Tailscale for remote access | `"cargo:iroh-ssh" = "0.2.9"` | 0.2.9 |

### Tier 3: Build it

| Tool | Use case | Effort |
|------|----------|--------|
| **hoist** (custom crate) | Isomorphic deploy agent | ~500 lines, pigglet as reference |

---

## Key risks

| Risk | Mitigation |
|------|------------|
| iroh not yet 1.0 | v0.97 RC, 1.0 imminent. Use stable v0.35 APIs where possible. |
| n0 could shut down | Self-host relay + use DHT mode = zero dependency on n0 |
| Community tools are solo projects | Fork if needed — they're tiny (1000-1500 lines) |
| Binary size on Pi Zero | pigglet proves ARM binaries work on all Pi models |
| Factory networks block UDP | Self-host relay on HTTPS (port 443) — works through any firewall |

---

## Next steps

1. [ ] Add iroh tools to `.mise.toml` — `sendme`, `dumbpipe`, `iroh-doctor` (just `mise install`)
2. [ ] Try `sendme` between Mac and a Pi — verify file transfer works over iroh
3. [ ] Run `iroh-doctor` on Pi Zero 2W — check NAT traversal, relay connectivity, memory usage
4. [ ] Add `iroh-ssh` to `.mise.toml` — test SSH-without-IP to a Pi
5. [ ] Read pigglet source — understand the agent/endpoint pattern for hoist
6. [ ] Prototype `hoist` crate — ~500 lines using iroh + postcard
7. [ ] Self-host iroh-relay on a Singapore VPS for factory use

---

## References

- [iroh GitHub](https://github.com/n0-computer/iroh) — core library
- [iroh.computer](https://iroh.computer) — docs and blog
- [awesome-iroh](https://github.com/n0-computer/awesome-iroh) — ecosystem list
- [iroh ESP32 example](https://github.com/n0-computer/iroh-esp32-example) — proof it runs on microcontrollers
- [iroh ESP32 blog post](https://iroh.computer/blog/iroh-on-esp32) — March 24, 2026
- [pigg/pigglet](https://github.com/andrewdavidmackenzie/pigg) — Pi GPIO agent over iroh (reference architecture)
- [iroh-ssh](https://github.com/rustonbsd/iroh-ssh) — SSH without IP
- [rustpatcher](https://github.com/rustonbsd/rustpatcher) — P2P binary updates
- [sendme](https://github.com/n0-computer/sendme) — file transfer CLI
- [dumbpipe](https://github.com/n0-computer/dumbpipe) — cross-device pipes
- [iroh 1.0 roadmap](https://iroh.computer/blog/road-to-1-0) — October 2024
- [PKARR](https://pkarr.org) — public key addressable resource records
