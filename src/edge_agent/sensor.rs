/// Coil weight sensor — Phase 2.
///
/// This module runs on the Pi Zero 2W inside howick-frama.
/// It reads the raw coil weight from the load cell + HX711 ADC and pushes it
/// to opcua-howick (Pi 5) via POST /api/sensor/coil every poll_interval_secs.
///
/// # Hardware wiring (Pi Zero 2W GPIO)
///
///   HX711   Pi Zero
///   VCC  →  3.3V   (pin 1)
///   GND  →  GND    (pin 6)
///   DAT  →  GPIO 5 (pin 29)  — data line, configurable
///   CLK  →  GPIO 6 (pin 31)  — clock line, configurable
///
/// Load cell 5m cable runs from under the coil spool to the Pi Zero.
/// The spool rests directly on the load cell plate.
///
/// # Weight → metres conversion
///
/// Done server-side in opcua-howick (config.rs: SensorConfig::coil_metres()).
/// Pi Zero only reads and pushes raw kg — no calibration needed on this side.
///
/// # Dev / test without hardware
///
/// Set COIL_WEIGHT_KG env var or write a float to /tmp/coil_weight_kg:
///
///   echo "23.5" > /tmp/coil_weight_kg          # 23.5kg on spool
///   COIL_WEIGHT_KG=23.5 mise run dev:agent:local
///
/// A reading of 0.0 is sent when neither source is available (sensor not fitted).
use tokio::time::{sleep, Duration};

/// GPIO pin numbers (BCM) for HX711 on Pi Zero 2W.
/// Override by setting HX711_DAT_PIN / HX711_CLK_PIN env vars.
#[allow(dead_code)]
const HX711_DAT_PIN_DEFAULT: u8 = 5;
#[allow(dead_code)]
const HX711_CLK_PIN_DEFAULT: u8 = 6;

/// Read the current coil weight in kg from the best available source:
///
/// 1. `COIL_WEIGHT_KG` environment variable (dev/test override)
/// 2. `/tmp/coil_weight_kg` file (written by HX711 userspace helper)
/// 3. HX711 GPIO bit-bang (Linux ARM only, gated at compile time)
/// 4. 0.0 (sensor not available — no reading pushed)
///
/// Returns `None` when no sensor source is available (prevents spurious zeros
/// overwriting the last real reading on the server).
pub fn read_weight_kg() -> Option<f64> {
    // 1. Env override — useful for dev and manual calibration tests
    if let Ok(val) = std::env::var("COIL_WEIGHT_KG") {
        if let Ok(kg) = val.trim().parse::<f64>() {
            tracing::debug!(weight_kg = kg, "Coil weight from COIL_WEIGHT_KG env var");
            return Some(kg);
        }
    }

    // 2. File written by HX711 userspace helper script
    //    A small Python script on the Pi Zero reads HX711 and writes /tmp/coil_weight_kg.
    //    See docs/customer/06-pi-zero-setup.md — Phase 2 setup.
    if let Ok(contents) = std::fs::read_to_string("/tmp/coil_weight_kg") {
        if let Ok(kg) = contents.trim().parse::<f64>() {
            tracing::debug!(weight_kg = kg, "Coil weight from /tmp/coil_weight_kg");
            return Some(kg);
        }
    }

    // 3. Direct HX711 GPIO bit-bang (Linux only)
    #[cfg(target_os = "linux")]
    {
        if let Some(kg) = read_hx711_linux() {
            return Some(kg);
        }
    }

    // No sensor source available
    None
}

/// Linux-only: read HX711 via GPIO character device.
///
/// Uses the Linux GPIO character device API (/dev/gpiochip0) to bit-bang
/// the HX711 24-bit ADC protocol. Requires the gpiod kernel module.
///
/// This is a blocking call — call from a dedicated thread or spawn_blocking.
#[cfg(target_os = "linux")]
fn read_hx711_linux() -> Option<f64> {
    let dat_pin = std::env::var("HX711_DAT_PIN")
        .ok()
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(HX711_DAT_PIN_DEFAULT);
    let clk_pin = std::env::var("HX711_CLK_PIN")
        .ok()
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(HX711_CLK_PIN_DEFAULT);

    match hx711_read_raw(dat_pin, clk_pin) {
        Ok(raw) => {
            // Convert raw 24-bit ADC count to kg using calibration constants.
            // CALIBRATION: weigh a known mass (e.g. 1kg) and adjust SCALE.
            // OFFSET: tare value — reading with empty scale.
            // These can be made configurable via SensorConfig in a later iteration.
            const SCALE: f64 = 420.0; // ADC counts per gram — calibrate on site
            const OFFSET: i32 = 0; // tare offset — zero with empty spool

            let grams = (raw - OFFSET) as f64 / SCALE;
            let kg = grams / 1000.0;
            tracing::debug!(weight_kg = kg, raw, "Coil weight from HX711 GPIO");
            Some(kg.max(0.0))
        }
        Err(e) => {
            tracing::warn!("HX711 GPIO read failed: {e}");
            None
        }
    }
}

/// Bit-bang the HX711 24-bit protocol via Linux sysfs GPIO.
///
/// Returns the raw 24-bit signed ADC count.
/// Protocol: pulse CLK 24 times, reading DAT on each falling edge.
/// One additional CLK pulse sets gain = 128 (channel A).
#[cfg(target_os = "linux")]
fn hx711_read_raw(dat_pin: u8, clk_pin: u8) -> anyhow::Result<i32> {
    // Export pins via sysfs (idempotent)
    let _ = std::fs::write("/sys/class/gpio/export", dat_pin.to_string());
    let _ = std::fs::write("/sys/class/gpio/export", clk_pin.to_string());

    let clk_dir = format!("/sys/class/gpio/gpio{clk_pin}/direction");
    let dat_dir = format!("/sys/class/gpio/gpio{dat_pin}/direction");
    let clk_val = format!("/sys/class/gpio/gpio{clk_pin}/value");
    let dat_val = format!("/sys/class/gpio/gpio{dat_pin}/value");

    std::fs::write(&clk_dir, "out")?;
    std::fs::write(&dat_dir, "in")?;

    // Wait for HX711 DOUT to go low (data ready), up to 200ms
    let ready_deadline = std::time::Instant::now() + std::time::Duration::from_millis(200);
    loop {
        let bit = std::fs::read_to_string(&dat_val)?
            .trim()
            .parse::<u8>()
            .unwrap_or(1);
        if bit == 0 {
            break;
        }
        if std::time::Instant::now() > ready_deadline {
            anyhow::bail!("HX711 not ready (DOUT never went low)");
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    // Read 24 bits, MSB first
    let mut raw: i32 = 0;
    for _ in 0..24 {
        // CLK high
        std::fs::write(&clk_val, "1")?;
        std::thread::sleep(std::time::Duration::from_micros(1));
        // Sample DAT
        let bit = std::fs::read_to_string(&dat_val)?
            .trim()
            .parse::<i32>()
            .unwrap_or(0);
        // CLK low
        std::fs::write(&clk_val, "0")?;
        std::thread::sleep(std::time::Duration::from_micros(1));
        raw = (raw << 1) | bit;
    }

    // One extra pulse: sets gain = 128 for next reading
    std::fs::write(&clk_val, "1")?;
    std::thread::sleep(std::time::Duration::from_micros(1));
    std::fs::write(&clk_val, "0")?;

    // Sign-extend 24-bit to 32-bit
    if raw & 0x80_0000 != 0 {
        raw |= !0x00FF_FFFF;
    }

    Ok(raw)
}

/// Run the sensor push loop on the Pi Zero.
///
/// Reads coil weight every `poll_interval_secs` and POSTs to
/// `{server_url}/api/sensor/coil`.  Runs forever — call from tokio::select!.
///
/// On failure to read sensor: logs warning and skips the push (does not reset
/// server-side reading to 0).
/// On failure to push: logs warning and continues (next reading will retry).
pub async fn run_sensor_push(server_url: String, poll_interval_secs: u64) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let endpoint = format!("{server_url}/api/sensor/coil");
    let interval = Duration::from_secs(poll_interval_secs.max(5));

    tracing::info!(
        endpoint = %endpoint,
        interval_secs = poll_interval_secs,
        "Coil sensor push loop started"
    );

    loop {
        // Read weight in a blocking thread (GPIO bit-bang is blocking)
        let weight = tokio::task::spawn_blocking(read_weight_kg)
            .await
            .ok()
            .flatten();

        match weight {
            Some(kg) => {
                let body = format!(r#"{{"weight_kg":{kg:.3}}}"#);
                match client
                    .post(&endpoint)
                    .header("Content-Type", "application/json")
                    .body(body)
                    .send()
                    .await
                {
                    Ok(resp) if resp.status().is_success() => {
                        tracing::debug!(weight_kg = kg, "Coil weight pushed to server");
                    }
                    Ok(resp) => {
                        tracing::warn!(
                            status = resp.status().as_u16(),
                            "Server rejected coil weight push"
                        );
                    }
                    Err(e) => {
                        tracing::warn!("Coil weight push failed: {e}");
                    }
                }
            }
            None => {
                tracing::debug!("No sensor reading available — skipping push");
            }
        }

        sleep(interval).await;
    }
}
