# Howick FRAMA — Automated Job Delivery

## The problem today

Every job requires someone to:

1. Save the file on the design computer
2. Copy it to a USB stick
3. Walk to the machine and plug it in
4. Walk back

This is time lost on every single job. If the USB stick is missing, or the wrong
file is copied, production stops. If the machine is mid-run and needs the next
job, someone has to stop what they are doing and walk over.

## What this system does

Jobs go from the design computer to the machine over WiFi — automatically,
in seconds. No USB stick. No walking. No waiting.

The operator does not change how they work. SketchUp and FrameBuilderMRD
continue exactly as before. The new system runs alongside them and adds a
second, faster path for jobs.

## What you get

| | Before | After |
|-|--------|-------|
| Send a job | Copy to USB stick, walk to machine | Click send in browser — done |
| Job arrives | When someone walks over | Within 5 seconds |
| Job fails (wrong file) | Production stops | Immediate alert on dashboard |
| Software update | Manual | Automatic, every hour |
| Something breaks | Call someone to visit | Fixed remotely, usually within minutes |

## What it costs

One-time hardware purchase of ~3,700 THB. No monthly fees. No subscriptions.

See the **Hardware Order** document (doc 03) for the exact list and where to buy.

## How we set it up

We set everything up remotely. You only need to:

- Order the hardware (one website, one order)
- Plug in the cables when it arrives
- Give us your WiFi password once

See the **Setup Guide** document (doc 02) for the full process.

## Phase 2 — Know when your coil is running low (optional)

A small sensor under the coil spool measures how much steel material remains.
The system alerts you before the coil runs out — so you can load a new one
before a job stops mid-run and scraps partially-formed members.

Hardware cost: ~600 THB. See the **Coil Sensor** document (doc 04) for details.
