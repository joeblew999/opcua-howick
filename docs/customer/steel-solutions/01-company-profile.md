# Steel Solutions — Company Profile

**Company:** Steel Solutions Thailand
**Owner:** Mr Prisit
**Location:** Bangkok, Thailand
**Website:** [steelsolutionsthailand.com](https://www.steelsolutionsthailand.com)

---

## About

Steel Solutions is a construction steel frame company in Bangkok. They specialise
in crafting steel frames for residential and commercial construction, designed to
withstand wind load and earthquake requirements.

---

## Machine

| Machine | Type | Location |
|---------|------|----------|
| FrameCad | Light gauge steel framing | Bangkok |

Machine specs: see [docs/machines/framecad.md](../../machines/framecad.md).

---

## Design workflow

- **FrameCad software** for steel framing design and machine output
- CSV/RFY files delivered to the FrameCad machine via USB
- Exact workflow details TBC

---

## opcua-howick deployment (planned)

| Item | Status |
|------|--------|
| Machine model | TBC |
| CSV/RFY format | TBC — need sample job files |
| Hardware setup | Not started |
| OPC UA namespace | `urn:framecad` (planned) |

---

## The FrameCad lock-in problem

FrameCad machines require a paid FrameCad software license to accept job files.
Customers who have already invested ~$400k USD in the machine are locked into
ongoing software subscriptions (Steelwise + Nexa) just to load jobs onto hardware
they own.

FrameBuilderMRD (Howick's SketchUp plugin) can already output FrameCad-format
files — but the machine won't accept them without the FrameCad license.

**plat-trunk + opcua-howick bypasses this entirely** — delivering jobs to the
machine via USB gadget, no FrameCad software license required.

_(Assumption — needs confirmation with Mr Prisit on site.)_

---

## Deployment notes

- Same Pi Zero USB gadget + Pi 5 server approach as EMH
- FrameCad namespace URI `urn:framecad` — shared config with other FrameCad sites
- Need to confirm machine model and job file format on site
- Steel Solutions is a separate company from EMH (Prin) and Maxxi Factory
