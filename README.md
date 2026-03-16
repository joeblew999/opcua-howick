# opcua-howick

OPC UA edge agent for **Howick FRAMA** roll-forming machines.

Runs on a small compute module (Raspberry Pi / NUC) on the factory LAN.
Bridges [plat-trunk](https://cad.ubuntusoftware.net) CAD output to the physical
machine via OPC UA + CSV file drop — replacing the current USB stick workflow.

**Status:** Early skeleton. Architecture is defined, implementation in progress.

---

## The Problem

Current workflow at Prin's factory (Si Racha, Thailand):

```
SketchUp → FrameBuilderMRD → CSV file → USB stick → Howick machine
```

Manual, no feedback, no status visibility, error-prone.

## The Goal

```
plat-trunk (browser CAD) → CF Worker → opcua-howick → Howick machine
                                              ↑
                                    OPC UA status back to plat-trunk
```

Job submission from browser. Machine status visible in real time. No USB sticks.

---

## Architecture

See [docs/architecture.md](docs/architecture.md) for full design including
OPC UA address space, file interface, and cross-compilation for Raspberry Pi.

---

## Related

- [howick-rs](https://github.com/joeblew999/howick-rs) — CSV parser/serialiser for Howick machines
- [async-opcua](https://github.com/FreeOpcUa/async-opcua) — Pure Rust OPC UA library (client + server)
- [plat-trunk](https://cad.ubuntusoftware.net) — Browser-native B-Rep CAD platform

---

## License

MIT OR Apache-2.0
