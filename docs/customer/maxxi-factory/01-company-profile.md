# Maxxi Factory — Company Profile

**Contact:** Erick — 081-863-7059
**Technical contact:** Ema (Erick's daughter)
**Comms:** LINE — connected 25 March 2026, awaiting reply
**Location:** Hua Hin / Cha Am, Petchaburi, Thailand
**Address:** 1386/5 Sampraya Road, Cha Am, Petchaburi 76120
**Phone:** +66 (0) 83 309 2211
**Website:** [maxxifactory.co.th](https://www.maxxifactory.co.th/)

---

## About

Maxxi Factory is a prefabricated construction company established ~20 years ago
in Thailand. They manufacture floors, walls, and roofs in a factory setting and
are recognized across South-East Asia for large, complex projects.

They have a dedicated **prefab factory in Hua Hin** for prefabrication of
floors, walls, and roof panels.

---

## Machine

| Machine | Type | Location |
|---------|------|----------|
| FrameCad | Light gauge steel framing | Hua Hin prefab factory |

---

## Design workflow

- **SketchUp** for 3D modelling
- **FrameCad software** for steel framing design and machine output
- CSV files delivered to the FrameCad machine via USB

---

## Related companies

| Company | Website | Notes |
|---------|---------|-------|
| Maxxi Building Products | [maxxi.co.th](https://www.maxxi.co.th/) | Rain gutters, drains, building products |
| Maxxi Factory | [maxxifactory.co.th](https://www.maxxifactory.co.th/) | Prefabricated construction |
| Facebook | [facebook.com/maxxifactory](https://www.facebook.com/maxxifactory/) | Company page |

---

## Relationship to this project

Maxxi Factory operates a FrameCad machine — a different brand from Prin's
Howick FRAMA but the same concept: light gauge steel framing from CSV job files.

The opcua-howick system can support FrameCad machines via the multi-agent node
manager architecture (see [ADR 0003](../../adr/0003-multi-agent-node-managers.md)).
Each machine type gets its own namespace URI (e.g. `urn:framecad`) and node
manager, while sharing the same core infrastructure.

---

## The FrameCad lock-in problem

FrameCad machines require a paid FrameCad software license to accept job files.
Customers who have already invested ~$400k USD in the machine are locked into
ongoing software subscriptions (Steelwise + Nexa) just to load jobs onto hardware
they own.

**plat-trunk + opcua-howick bypasses this entirely** — delivering jobs to the
machine via USB gadget, no FrameCad software license required.

_(Assumption — needs confirmation on site.)_

---

## Deployment notes

- FrameCad CSV format may differ from Howick FRAMA — needs investigation
- Namespace URI would be `urn:framecad` (config-driven)
- Same hardware setup applies: Pi Zero 2W as USB gadget, Pi 5 as job server
- Maxxi Factory is a separate company from EMH (Prin) — may need its own config/instance
