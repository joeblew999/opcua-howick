# Coil Inventory Sensor

## The problem

The Howick FRAMA machine feeds from a steel coil. When the coil runs out
mid-job, the machine stops and the partially-formed members are scrap.
The only way to know how much material is left today is to look at the spool.

If nobody checks before starting a long job, the coil runs out partway through.
The job has to be restarted from scratch on a new coil.

## What the sensor does

A small weight sensor sits under the coil spool. It measures the total weight
of the spool and the remaining steel, and converts that to metres of material left.

The system shows this number on the status dashboard — visible on any phone or
computer on the factory network. When the coil drops below a set level (for
example, 50 metres), the system sends an alert so there is time to load a new
coil before the current one runs out.

## What it costs

~680 THB in additional hardware. See the **Hardware Order** document (doc 03)
for the exact parts list.

We install the sensor remotely and configure the alert level to suit your workflow.

## What you need to do

1. Order the four additional items listed in doc 03 (Lazada Thailand)
2. Weigh the empty coil spool and tell us the weight in kg
3. We do the rest remotely

## Timeline

This is Phase 2. The main job delivery system (Phase 1) installs first.
Once Phase 1 is running, we add the sensor in a separate step — no disruption
to production.
