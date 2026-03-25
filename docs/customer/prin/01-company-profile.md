# Eastern Mobile House (EMH) — Company Profile

**Company:** Eastern Mobile House (EMH)
**Owner:** Prin
**Location:** Si Racha / Laem Chabang, Chonburi, Thailand

---

## About

Eastern Mobile House (EMH) is Prin's company, based at Si Racha near Laem Chabang
in Chonburi province. The factory produces light gauge steel frames, trusses,
and wall panels for residential and commercial construction.

---

## Machines

| Machine | Type | Location | Status |
|---------|------|----------|--------|
| Howick FRAMA | Light gauge steel roll-forming | Si Racha factory | Primary — docs + hardware quote done |

Machine specs: see [docs/machines/howick-frama.md](../../machines/howick-frama.md).

---

## Design workflow

- **SketchUp** — Prin's permanent 3D design tool
- **FrameBuilderMRD** — generates Howick-format CSV from SketchUp models
- **USB stick** — current delivery method (walk stick to machine)
Reference jobs from Prin's Si Racha machine:
- **T1** — roof truss, 22 components, S8908 profile, 3945 mm chords
- **W1** — wall frame, 42 components, S8908 profile, 4740 mm plates

---

## opcua-howick deployment

### Si Racha — Howick FRAMA

| Item | Status |
|------|--------|
| Option A — software on Design PC | Ready |
| Option B — Pi 5 + Pi Zero hardware | Hardware quote done (~3,700 THB) |
| Phase 2 — coil weight sensor | Software done, awaiting hardware (~680 THB) |
| Phase 3 — plat-trunk CAD path | Future |
| USB folder path on FRAMA | ✅ Resolved — operator browses from Howick software, no fixed path |
| First live test | Awaiting setup on site |

Hardware order from **raspberrypithailand.com** (free nationwide shipping, 3-day delivery).

## Project documents (Si Racha)

| Doc | Description |
|-----|-------------|
| [01-proposal.md](./01-proposal.md) | Project proposal |
| [02-system-overview.md](./02-system-overview.md) | Architecture: Option A, B, Phase 3 |
| [03-hardware-quote.md](./03-hardware-quote.md) | Hardware costs (~3,700 THB Phase 1) |
| [04-setup-guide.md](./04-setup-guide.md) | What Prin does vs what Gerard does |
| [05-roadmap.md](./05-roadmap.md) | Feature status and next steps |
| [06-pi-zero-setup.md](./06-pi-zero-setup.md) | Pi Zero 2W setup instructions |
| [07-ops-runbook.md](./07-ops-runbook.md) | Operations and update process |

---

## Commercial terms

- **Software, setup, management:** Free — no charge
- **Hardware only:** Prin pays for Pi + accessories (~3,700 THB Phase 1)
- **Monthly fees:** None
- **Relationship:** Gerard Webb / ubuntu Software provides this as a free product
