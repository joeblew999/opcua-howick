# Document 4 of 7 — Setup Guide
## Howick FRAMA — Automated Job Delivery

**Prepared for:** Prin — Si Racha Factory, Thailand
**Prepared by:** Gerard Webb, ubuntu Software
**Date:** March 2026

---

## Option A — Software on your Design PC (no hardware needed)

This option requires no hardware purchase and can be running today.

### What you do

1. Tell Gerard: "I want to try Option A"
2. Gerard sends a setup link — one click installs the software on your Design PC
3. Open a browser on the same PC and go to `http://localhost:4841/dashboard`
4. Drag a job file from FrameBuilderMRD into the dashboard — done

Your USB stick method continues to work. Nothing is removed or changed.
The dashboard runs alongside SketchUp and FrameBuilderMRD. It starts automatically
when Windows starts.

### What Gerard does

- Installs `opcua-howick.exe` on your Design PC remotely
- Configures it to point at the folder where your job files are saved
- Runs a test job to confirm it works
- Done in under an hour

---

## Option B — Dedicated hardware (no USB stick, full automation)

Two small computers on your factory WiFi handle everything automatically.
Any browser on the network can reach the dashboard.

### What you do

**Step 1 — Order the hardware**

Place one order at **raspberrypithailand.com**. Parts list in Document 3 (Hardware
Quote). Approximately 3,700 THB. Delivery 3 days.

**Step 2 — Plug in and share WiFi**

When the hardware arrives:

1. Plug the Pi 5 (small server) into power near the factory WiFi router
2. Plug the Pi Zero (tiny USB device) via the 3m cable into the Howick FRAMA USB port
3. Send Gerard the WiFi network name and password

That is all that is required from you. Gerard takes it from there remotely.

**Step 3 — Watch the test run**

Gerard sends a test job from his laptop. You watch it arrive at the machine
and run. Confirm it works. The whole session takes under an hour.

From that point: open `http://pi5.local:4841/dashboard` in any browser on the
factory network to send jobs and monitor the machine.

### What Gerard does

- Provisions both computers remotely over Tailscale (secure remote access)
- Configures the Pi Zero with the correct USB path for your machine
- Runs a test job with your real job files
- Installs automatic software updates on both computers

---

## Day to day after setup

The operator's workflow is identical. The machine runs as before.

- **Option A:** drag job file into browser on Design PC instead of copying to USB
- **Option B:** drag job file into browser on any device — job reaches machine automatically

Software updates install automatically. No action required from anyone at the factory.

---

## Adding the coil sensor — Phase 2 (Option B only, whenever you are ready)

Requires Option B hardware (sensor wires to Pi Zero GPIO). Order the Phase 2 items
from Document 3 (Hardware Quote) on Lazada Thailand. When they arrive:

1. Weigh the empty coil spool — send Gerard the weight in kilograms
2. Gerard installs and calibrates the sensor remotely — no production downtime

The dashboard then shows remaining material in metres and alerts before a coil
runs out mid-job.

---

## If something goes wrong

Contact Gerard directly. The system can be seen and fixed remotely.
Most issues resolved the same day. No site visit needed.

---

→ **Next: Document 5 — Roadmap** (phases, status, and open questions)

---

**Gerard Webb**
ubuntu Software
gerard@ubuntu-software.com
