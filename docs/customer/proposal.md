# Statement of Work

**Client:** Prin — Si Racha Factory, Thailand
**Prepared by:** Gerard Webb, ubuntu Software
**Date:** March 2026

---

## Executive Summary

ubuntu Software will connect your Howick FRAMA roll-forming machine to the
factory network so that jobs are delivered automatically from the design
computer to the machine — eliminating manual USB stick transfers. A live
status dashboard will show machine state and job queue on any phone or
screen in the factory.

The project delivers in two phases. Phase 1 connects the machine and provides
the dashboard. Phase 2 adds a coil inventory sensor so the operator knows
how much material remains before starting a job.

Total hardware investment: approximately 4,380 THB. No monthly fees.
No subscriptions. Ubuntu Software manages all software remotely.

---

## The problem

Today, getting a job to the Howick FRAMA machine requires someone to:

1. Save the design file on the design computer
2. Copy it to a USB stick
3. Walk to the machine and plug it in
4. Walk back to the design area

This is repeated for every job. If the USB stick is misplaced, or the wrong
file is copied, production stops. There is no visibility into machine state,
job queue, or material remaining unless someone physically checks.

---

## Phase 1 — Automated job delivery and status dashboard

### What we deliver

A connected edge system that sends jobs from the design computer to the
Howick FRAMA over the factory WiFi network, automatically, within 5 seconds.

A live dashboard — accessible on any phone, tablet, or computer on the factory
network — shows machine status, the current job, and the job queue.

### How it works

Two small computers are installed near the machine:

- A Raspberry Pi Zero 2W plugs into the machine's USB port via a 3m cable.
  The machine sees it exactly as a USB stick. Job files arrive on it
  automatically over WiFi — no manual transfer required.

- A Raspberry Pi 5 sits on the factory WiFi and runs the status dashboard,
  job queue management, and remote management interface.

### What does not change

The existing SketchUp and FrameBuilderMRD workflow continues unchanged.
The operator does not need to learn anything new. USB sticks continue to
work alongside the new system.

### Hardware cost

~3,700 THB one-time. See the Hardware Quote document for the exact parts list.

### Timeline

Setup is completed remotely in a single session after hardware arrives.
Hardware delivery from raspberrypithailand.com is typically 3 days.

---

## Phase 2 — Coil inventory sensor

### What we deliver

A weight sensor installed under the coil spool that measures how much steel
material remains and displays the value in metres on the status dashboard.
When material drops below a configured threshold, an alert is sent so the
operator has time to load a new coil before the current one runs out.

### Why this matters

A coil running out mid-job means the partially-formed members are scrap
and the job must restart from the beginning on a new coil. The sensor
prevents this by giving advance warning.

### Installation

We install and calibrate the sensor remotely after the operator weighs the
empty coil spool and provides the reading. No production downtime.

### Hardware cost

~680 THB additional. See the Hardware Quote document for the parts list.

---

## Phase 3 — Additional machines

The same system can be extended to other roll-forming machines in the factory.
Scope, hardware, and cost are defined per machine. Ubuntu Software will provide
a separate quote when required.

---

## Support and remote management

Ubuntu Software manages both computers remotely via a secure encrypted tunnel.
This covers:

- **Automatic software updates** — installed every hour, no action required
- **Remote diagnostics and fault resolution** — no site visit needed in most cases
- **Configuration changes** — alert thresholds, job routing, new machines

Support is provided by Gerard Webb directly. Response time is same-day for
operational issues.

---

## What we need from you

To begin:

1. Place the hardware order at raspberrypithailand.com (see Hardware Quote)
2. When hardware arrives: plug both computers into power and connect to factory WiFi
3. Provide the factory WiFi name and password (shared securely, used once)

Ubuntu Software handles everything after that.

---

## Investment

| | Cost |
|-|------|
| Phase 1 hardware (Howick FRAMA + dashboard) | ~3,700 THB |
| Phase 2 hardware (coil inventory sensor) | ~680 THB |
| **Total hardware** | **~4,380 THB** |
| Software and setup | Included |
| Ongoing remote management | Included |
| Monthly fees | None |

---

## Next steps

To proceed, contact Gerard Webb at ubuntu Software to confirm the start date.
We will coordinate the hardware order and schedule the setup session.

**Gerard Webb**
ubuntu Software
