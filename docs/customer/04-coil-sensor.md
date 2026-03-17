# Coil Inventory Sensor

## Why this matters

The Howick FRAMA machine feeds from a steel coil. When the coil runs out
mid-job, production stops and the partially-formed members are scrap.
With a coil sensor, the system knows how much material remains and can
alert the operator before the coil runs out.

The sensor value is exposed on the OPC UA node `Machine/CoilRemaining`
(in metres) and visible on the plat-trunk status dashboard.

---

## Physical layout

```
[Steel coil spool]
       │
  [Mounting plate]  ← load cell sits between spool axle and floor bracket
       │
  [Load cell 50kg]  ← measures total weight: spool + remaining steel
       │
  [HX711 board]     ← amplifies signal, converts to digital
       │
  [5m shielded cable]
       │
  [Pi Zero 2W GPIO] ← already plugged into FRAMA USB port via 3m cable
```

Coil spool and FRAMA machine are ~5m apart. The cable runs along the floor or wall.

---

## Hardware

Order all items from Lazada Thailand. ~600 THB total.

See [03-bom.md](03-bom.md) Phase 2 section for part numbers and wiring diagram.

---

## Installation steps (we do this remotely)

### 1. Prepare the mounting plate

Place a steel plate (~150×150mm) between the coil spool axle bracket and
its support. The load cell sits between the plate and the bracket so the
full weight of the spool + coil passes through it.

### 2. Wire the load cell to the HX711 board

| Load cell wire | HX711 terminal |
|---------------|----------------|
| Red (E+)      | E+             |
| Black (E-)    | E-             |
| White (A+)    | A+             |
| Green (A-)    | A-             |

### 3. Wire the HX711 to Pi Zero 2W GPIO

| HX711 pin | Pi Zero 2W GPIO pin |
|-----------|---------------------|
| VCC       | Pin 1 (3.3V)        |
| GND       | Pin 6 (GND)         |
| DT        | Pin 29 (GPIO 5)     |
| SCK       | Pin 31 (GPIO 6)     |

Run the 5m shielded cable from the HX711 board along the floor to the Pi Zero 2W.

### 4. Calibrate

Before fitting the first coil, weigh the empty spool and record the weight.
Update `config.toml`:

```toml
[sensor]
enabled            = true
hx711_dt_pin       = 5
hx711_sck_pin      = 6
tare_weight_kg     = 12.5    # empty spool weight in kg
kg_per_metre       = 0.42    # confirm from Howick steel spec sheet
poll_interval_secs = 30
```

The `kg_per_metre` value depends on the steel profile being used. Howick
can provide this from the machine spec sheet, or it can be measured by
weighing a known length of material.

### 5. Deploy updated config

```bash
mise run deploy:config:pi-zero
```

---

## How it works

Every 30 seconds (configurable), the Pi Zero 2W:

1. Reads raw weight from the HX711 ADC
2. Subtracts `tare_weight_kg` (empty spool)
3. Divides remaining weight by `kg_per_metre`
4. Pushes the result (metres) to the Pi 5 OPC UA node `Machine/CoilRemaining`

The Pi 5 broadcasts this value to any OPC UA client on the factory LAN —
including plat-trunk, Prin's phone, or a factory MES system.

---

## Alerts

Once the sensor is live, a low-coil alert can be configured in Netdata on
the Pi 5 — for example, alert when `CoilRemaining < 50m` so there is time
to order or load a new coil before the current one runs out.

---

## Status

**Phase 2** — hardware defined, code not yet written.
See TODO in [src/agent/main.rs](../src/agent/main.rs).

Implementation needed:
- `sensor` module in `howick-agent`: GPIO reads via rppal crate, HX711 protocol, weight → metres
- Push `CoilRemaining` to Pi 5 via HTTP API
- Pi 5 updates OPC UA node `Machine/CoilRemaining`
