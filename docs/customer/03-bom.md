# Hardware Bill of Materials

## What to order

Two Pis. Clean separation of concerns:

- **Pi Zero 2W** — USB gadget only. Pretends to be a USB stick to the Howick FRAMA machine.
- **Pi 5 4GB** — Full OPC UA server + HTTP API. Sits on factory WiFi. Tailscale for remote access.

---

## Device 1 — Pi Zero 2W (USB Gadget / Machine Interface)

| # | Component | SKU / Part # | Spec | Est. Cost (USD) |
|---|-----------|-------------|------|-----------------|
| 1 | Raspberry Pi Zero 2W | **SC1146** (Adafruit #5291, ASIN B09LH5SBPS) | 512 MB RAM, ARM64, USB OTG | ~$15 |
| 2 | SanDisk Ultra microSD 32GB | **SDSQUA4-032G-GN6MA** (ASIN B08GY9NYRM) | Class 10 A1, 120 MB/s | ~$8 |
| 3 | Anker PowerLine+ Micro-USB 10ft | **A8142H11** (ASIN B019Q6F9F4) | USB-A to Micro-USB, double-braided, 3m | ~$10 |
| 4 | USB-A charger (5V/2.5A min) | any spare, e.g. ASIN B077GXJX5M | Power. Use extension cable if outlet is far. | ~$8 |
| | **Subtotal** | | | **~$41** |

## Device 2 — Pi 5 4GB (Full OPC UA + HTTP)

| # | Component | SKU / Part # | Spec | Est. Cost (USD) |
|---|-----------|-------------|------|-----------------|
| 1 | Raspberry Pi 5 4GB | **SC1112** (Adafruit #5813, ASIN B0CK9FZDMD) | 4GB RAM, ARM64, Gigabit Ethernet, WiFi | ~$60 |
| 2 | SanDisk Ultra microSD 32GB | **SDSQUA4-032G-GN6MA** (ASIN B08GY9NYRM) | Class 10 A1, 120 MB/s | ~$8 |
| 3 | Official Raspberry Pi 27W USB-C PSU | **SC1690** | 5.1V/5A — do not use a cheaper charger | ~$12 |
| | **Subtotal** | | | **~$80** |

---

## Grand Total: ~$121 USD

---

## Software (free, install on both Pis)

| Component | Notes |
|-----------|-------|
| Raspberry Pi OS Lite 64-bit | Flashed via Raspberry Pi Imager |
| Tailscale | SSH + remote deploy from anywhere. Free tier covers 2 nodes. |
| Doppler | Secrets management (API keys). Free tier. |
| Netdata | Monitoring dashboard. Install on Pi 5 only. Free tier covers 5 nodes. |

---

## Where to order (Thailand — Si Racha factory)

Order everything locally. No import hassle.

| Store | Notes |
|-------|-------|
| **raspberrypithailand.com** | Official reseller — order everything here. Free shipping, 3-day delivery, full warranty. |
| **th.cytron.io** | Alternative official reseller if raspberrypithailand.com is out of stock. |

---

## Option C — Existing Windows PC (No new hardware, baby steps)

Run `opcua-howick.exe` alongside the **existing** SketchUp + FrameBuilderMRD workflow.
Do not touch or replace anything. Both write to the same watched folder.
Operator keeps using USB sticks exactly as before — opcua-howick just adds a second path.

**Cost: $0**

---

## Integration Details

| Item | Value |
|------|-------|
| Machine USB port | USB-A (Howick FRAMA) |
| Pi Zero 2W connection | USB-A → Micro-USB (gadget mode, not host mode) |
| Virtual disk image | `/piusb.bin` — 512 MB FAT32, label `HOWICK` |
| Mount point | `/mnt/usb_share` |
| Kernel modules | `dwc2`, `g_mass_storage` |
| Pi Zero 2W hostname | `howick-pi-zero.local` |
| Pi 5 hostname | `howick-pi5.local` |
| OPC UA port (Pi 5) | 4840 |
| HTTP status API port (Pi 5) | 4841 |

## Phase 2 — Coil Inventory Sensor

Wires to Pi Zero 2W GPIO. 5m cable runs from coil spool to Pi Zero at the FRAMA machine.
Pi Zero reads weight → converts to metres → pushes to Pi 5 OPC UA `Machine/CoilRemaining`.

| # | Component | SKU / Part # | Spec | Est. Cost (USD) |
|---|-----------|-------------|------|-----------------|
| 1 | Load cell 50kg | ASIN B079FTYDKN (generic bar-type) | Rated 50kg — fits under coil spool mounting plate | ~$8 |
| 2 | HX711 load cell amplifier | ASIN B07TWLP3X8 | 24-bit ADC, connects load cell → Pi Zero GPIO pins | ~$4 |
| 3 | 4-core shielded cable 5m | generic (e.g. 4×0.2mm²) | Load cell wiring to Pi Zero — shielded to reduce noise | ~$5 |
| 4 | Mounting plate (steel, ~150×150mm) | local hardware shop | Sits between floor/bracket and coil spool axle | ~$5 |
| | **Subtotal** | | | **~$22** |

All available from Lazada Thailand or local electronics shops. Total Phase 1 + Phase 2 hardware: **~$143 USD**.

### Wiring

```
Load cell (4 wires: E+, E-, A+, A-)
    └── HX711 board (VCC, GND, DT, SCK)
            └── Pi Zero 2W GPIO
                    Pin 1  (3.3V)  → HX711 VCC
                    Pin 6  (GND)   → HX711 GND
                    Pin 29 (GPIO5) → HX711 DT  (data)
                    Pin 31 (GPIO6) → HX711 SCK (clock)
```

Cable runs 5m from coil spool to Pi Zero 2W at the FRAMA machine USB port.

### Weight → metres conversion

Steel coil weight depends on profile type and coil ID. Calibration values stored in `config.toml`:

```toml
[sensor]
enabled            = true
hx711_dt_pin       = 5
hx711_sck_pin      = 6
tare_weight_kg     = 12.5    # empty spool weight — weigh it before first coil
kg_per_metre       = 0.42    # depends on steel profile — confirm with Howick spec sheet
poll_interval_secs = 30
```

### Code needed

`sensor` module in `howick-agent` (GPIO + HX711 driver, weight → metres, push to Pi 5 OPC UA).
See Phase 2 in [architecture.md](architecture.md) and TODO in [src/agent/main.rs](../src/agent/main.rs).

## Out of Scope

- FRAMECAD Nexa hardware bridge (Phase 3)
- Custom PCB — not required
