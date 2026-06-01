// sensor_dht.rs
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{AnyIOPin, InputOutput, PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;

use crate::sensor::{SensorData, SensorResult};

const BIT_THRESHOLD_US: u64 = 35;

fn wait_for_pin_state(
    pin: &PinDriver<InputOutput>,
    high: bool,
    timeout_us: u64,
) -> Result<u64, &'static str> {
    let start = unsafe { esp_idf_svc::sys::esp_timer_get_time() } as u64;
    loop {
        let now = unsafe { esp_idf_svc::sys::esp_timer_get_time() } as u64;
        let elapsed = now - start;
        if elapsed > timeout_us {
            return Err("Timeout");
        }
        if pin.is_high() == high {
            return Ok(elapsed);
        }
    }
}

pub fn read() -> SensorResult {
    let peripherals = match Peripherals::take() {
        Ok(p) => p,
        Err(_) => return SensorResult::Err("Peripherals bereits belegt"),
    };

    let pin: AnyIOPin = peripherals.pins.gpio21.into();
    let mut io = PinDriver::input_output_od(pin, Pull::Up).unwrap();

    FreeRtos::delay_ms(1000);

    io.set_high().unwrap();
    FreeRtos::delay_ms(500);

    io.set_low().unwrap();
    FreeRtos::delay_ms(25);

    io.set_high().unwrap();

    unsafe {
        let start = esp_idf_svc::sys::esp_timer_get_time() as u64;
        while (esp_idf_svc::sys::esp_timer_get_time() as u64 - start) < 40 {}
    }

    // CriticalSection VOR dem Handshake
    let cs = esp_idf_svc::hal::task::CriticalSection::new();
    let _guard = cs.enter();

    // Warte bis Sensor LOW zieht (Handshake Start)
    if wait_for_pin_state(&io, false, 500).is_err() {
        drop(_guard);
        return SensorResult::Err("Kein Handshake LOW");
    }
    // Warte bis Sensor wieder HIGH geht (Handshake Ende)
    if wait_for_pin_state(&io, true, 500).is_err() {
        drop(_guard);
        return SensorResult::Err("Kein Handshake HIGH");
    }
    // Warte bis Sensor LOW zieht — das ist der Start von Bit 0
    if wait_for_pin_state(&io, false, 500).is_err() {
        drop(_guard);
        return SensorResult::Err("Kein Bit-Start");
    }

    // Jetzt sind wir exakt am Anfang von Bit 0
    let mut data = [0u8; 5];
    let mut timings = [0u64; 40];

    for byte_idx in 0..5 {
        for bit_idx in 0..8 {
            // Warte auf HIGH — Bit beginnt
            if wait_for_pin_state(&io, true, 1000).is_err() {
                drop(_guard);
                return SensorResult::Err("Bit HIGH Timeout");
            }

            // Messe wie lange HIGH
            let duration = unsafe { esp_idf_svc::sys::esp_timer_get_time() } as u64;

            // Warte bis LOW — Bit endet
            let is_last_bit = byte_idx == 4 && bit_idx == 7;
            let pin_went_low = wait_for_pin_state(&io, false, 5000).is_ok();

            let high_duration = if pin_went_low || is_last_bit {
                (unsafe { esp_idf_svc::sys::esp_timer_get_time() }) as u64 - duration
            } else {
                drop(_guard);
                return SensorResult::Err("Bit Dauer Timeout");
            };

            timings[byte_idx * 8 + bit_idx] = high_duration;

            if high_duration > BIT_THRESHOLD_US {
                data[byte_idx] |= 1 << (7 - bit_idx);
            }
        }
    }

    drop(_guard);

    for i in 0..40 {
        println!("Bit {}: {}µs", i, timings[i]);
    }

    SensorResult::Ok(SensorData {
        temperature: data[2] as f32,
        humidity: data[0] as f32,
    })
}
