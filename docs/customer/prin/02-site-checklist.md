# Eastern Mobile House (EMH) — Site Information Checklist

**For:** Prin — Si Racha Factory, Laem Chabang
**From:** Gerard Webb, ubuntu Software
**Date:** March 2026

---

Most of this takes five minutes. One answer from the operator unlocks full
end-to-end testing.

---

## Must have — we cannot start without these

- [x] **USB folder path** — ✅ Resolved. The Howick FRAMA has a built-in Windows
  PC with Howick software. The operator browses the USB drive and picks the job
  folder — there is no fixed path. The Pi Zero USB gadget just needs to present
  the CSV in any folder.

- [x] **Sample job files** — ✅ Already have T1 (truss) and W1 (wall frame) from
  Prin's machine. Used to build and test the system.

---

## Nice to have — helps us move faster

- [ ] **Is the Design PC on the factory WiFi?** — or on a separate network?
  This decides whether Option A (software on Design PC) can talk to Option B
  (Pi hardware) without extra setup.

- [ ] **How many Howick FRAMA machines are in the factory?** — we know about one,
  are there more?

- [ ] **Photo of the machine's USB port area** — we need to see where the USB
  stick plugs in, and how far it is from the nearest power point. This tells us
  what cable length we need for the Pi Zero.

- [ ] **Factory WiFi coverage near the machine** — the Pi Zero and Pi 5 both
  need WiFi. What network name?

- [ ] **Empty coil spool weight in kg** — needed to calibrate the Phase 2 coil
  weight sensor. Just weigh the empty spool once.

- [ ] **Are there other roll-forming machines to add later?** — any plans for
  additional machines at Si Racha?

---

## Already done

| Item | Status |
|------|--------|
| Sample job files (T1, W1) | ✅ Received and tested |
| Software — Option A (Design PC) | ✅ Ready |
| Software — Option B (Pi 5 + Pi Zero) | ✅ Ready |
| Hardware quote (~3,700 THB) | ✅ Sent — [03-hardware-quote.md](./03-hardware-quote.md) |
| OPC UA server | ✅ Running, tested |
| Dashboard + HTTP API | ✅ Running, tested |
| Coil sensor software (Phase 2) | ✅ Done — awaiting hardware + spool weight |

---

## What happens next

Once we have the remaining items, Gerard can:

1. Complete the final configuration
2. Run a full end-to-end test
3. Walk you through setup (Option A or B — your choice)

Your existing USB stick workflow continues to work unchanged. This runs alongside it.

---

## Full project documents

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

**Gerard Webb**
ubuntu Software
