# Howick FRAMA — System Setup Guide

## What you need

Two small computers and a few cables. Everything ships from
**raspberrypithailand.com** — order once, free shipping, 3-day delivery.

See the Hardware Bill of Materials document for the full order list (~3,700 THB).

---

## One-time setup (your IT person or us remotely)

### Step 1 — Order and receive hardware

Order everything from raspberrypithailand.com. You will receive:

- Raspberry Pi Zero 2W (small, about the size of a USB stick)
- Raspberry Pi 5 (about the size of a deck of cards)
- Two microSD cards
- Power supplies
- One 3m USB cable

### Step 2 — We do the setup remotely

Once the hardware arrives, we connect both computers to your factory WiFi and
complete the setup remotely via a secure tunnel. You do not need to do anything
technical. We will:

- Install the software on both computers
- Connect them to your factory WiFi
- Plug the Pi Zero 2W into the Howick FRAMA USB port via the 3m cable
- Verify the system is working end-to-end

**You only need to provide:**
- Your factory WiFi name and password
- Physical access to plug in the USB cable and power supplies

### Step 3 — Test

We send a test job from the browser. You confirm the machine receives it and
runs correctly. The existing USB stick method still works — nothing is removed.

---

## After setup — day to day

**Nothing changes for the operator.** The machine runs exactly as before.
New jobs appear automatically on the machine without anyone walking over with a USB stick.

SketchUp and FrameBuilderMRD continue to work as normal.

---

## Software updates

Software updates happen **automatically**. Both computers check for new versions
every hour and update themselves. No action required.

---

## If something goes wrong

We can diagnose and fix issues remotely via the secure Tailscale tunnel —
no need to visit the factory. In most cases problems are resolved within minutes.

---

## Phase 2 — Coil inventory sensor (optional, future)

A small weight sensor placed under the coil spool measures how much steel
material remains. The system then shows coil remaining (in metres) on the
status dashboard, so you know when to order more before running out mid-job.

Hardware cost: ~600 THB. We install it remotely once the sensor arrives.
