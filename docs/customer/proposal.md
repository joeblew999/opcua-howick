# Factory Automation Proposal

Prepared for: **Prin — Si Racha Factory**
Prepared by: **Gerard Webb, ubuntu Software**

---

## The opportunity

Your factory runs two roll-forming machines — a Howick FRAMA and a FRAMECAD.
Both require jobs to be delivered manually via USB stick. Every job means someone
stops what they are doing, copies a file, walks to the machine, plugs in the stick,
and walks back.

This proposal covers automating both machines so that jobs flow from the design
computer to the machine over WiFi — automatically, in seconds, with no manual steps.

Your existing workflow does not change. SketchUp, FrameBuilderMRD, and your
operator's process stay exactly as they are today.

---

## Phase 1 — Howick FRAMA: Automated job delivery

A small computer plugs into the Howick FRAMA USB port. The machine sees it exactly
as a USB stick. Jobs are sent from the browser to the machine over WiFi and appear
on the machine within 5 seconds — no USB stick, no walking.

A second computer on the factory WiFi provides a live status dashboard showing
machine state, current job, and job queue. Visible on any phone or screen on
the factory network.

**The operator does nothing differently.** The machine runs exactly as before.

**Phase 1 hardware cost: ~3,700 THB** — see the Hardware Quote document.

---

## Phase 2 — Howick FRAMA: Coil inventory sensor

A weight sensor under the coil spool measures how much steel material remains
and displays metres remaining on the status dashboard. When the coil drops below
a set level, an alert fires — giving time to load a new coil before the current
one runs out mid-job.

A coil running out mid-job scraps the partially-formed members and forces the
job to restart from zero. This sensor prevents that.

**Phase 2 hardware cost: ~680 THB additional** — see the Hardware Quote document.

---

## Phase 3 — FRAMECAD: Automated job delivery

The same automated job delivery applied to the FRAMECAD machine. Jobs go from
the design computer to the FRAMECAD over WiFi — no USB stick, same experience
as Phase 1 for the Howick FRAMA.

The FRAMECAD connects via its native API. Hardware cost and timeline to be
confirmed once we have the machine's interface specification.

---

## Investment summary

| Phase | What you get | Hardware cost |
|-------|-------------|---------------|
| 1 | Howick FRAMA: automated job delivery + status dashboard | ~3,700 THB |
| 2 | Howick FRAMA: coil inventory sensor + low-coil alerts | ~680 THB |
| 3 | FRAMECAD: automated job delivery | TBD |

No monthly fees. No subscriptions. No ongoing licence costs.

---

## Support and updates

Both machines are managed remotely by ubuntu Software. Software updates install
automatically every hour. Issues are diagnosed and fixed remotely — no site visit
required. In most cases, problems are resolved within minutes of being reported.

---

## Why ubuntu Software

We build and maintain the plat-trunk platform that this system integrates with.
We wrote the edge agent software that runs on the factory hardware. We manage
deployment, updates, and support end-to-end — no third party involved.

---

## Next steps

1. Review the Hardware Quote and place the order at raspberrypithailand.com
2. Contact Gerard Webb at ubuntu Software to schedule setup
3. We handle everything from there — setup takes one session via remote access

**Gerard Webb — ubuntu Software**
