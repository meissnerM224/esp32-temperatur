use crate::sensor::{SensorData, SensorResult};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{InputOutput, PinDriver};
use esp_idf_svc::hal::interrupt;

const BIT_SAMPLE_US: u64 = 45;

#[inline(always)]

fn now_us() -> u64 {
    unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 }
}

#[inline(always)]

fn busy_wait_us(us: u64) {
    let start = now_us();

    while now_us().wrapping_sub(start) < us {}
}

#[inline(always)]

fn wait_for_pin_state(
    pin: &PinDriver<'_, InputOutput>,

    high: bool,

    timeout_us: u64,
) -> Result<(), &'static str> {
    let start = now_us();

    loop {
        if pin.is_high() == high {
            return Ok(());
        }

        if now_us().wrapping_sub(start) > timeout_us {
            return Err("Timeout");
        }
    }
}

fn read_frame(io: &PinDriver<'_, InputOutput>) -> Result<[u8; 5], &'static str> {
    // Sensor-Antwort: LOW ca. 80us, HIGH ca. 80us, dann LOW vor Bit 0.

    wait_for_pin_state(io, false, 200).map_err(|_| "Kein Handshake LOW")?;

    wait_for_pin_state(io, true, 200).map_err(|_| "Kein Handshake HIGH")?;

    wait_for_pin_state(io, false, 200).map_err(|_| "Kein Bit-Start")?;

    let mut data = [0u8; 5];

    for bit in 0..40 {
        // Jedes Bit: erst 50us LOW, dann HIGH-Puls.
        wait_for_pin_state(io, true, 120).map_err(|_| "Bit HIGH Timeout")?;

        // Nach 45us unterscheiden:
        // 0-Puls ist dann schon LOW, 1-Puls ist noch HIGH.
        busy_wait_us(BIT_SAMPLE_US);
        if io.is_high() {
            data[bit / 8] |= 1 << (7 - (bit % 8));
        }

        // Für das nächste Bit wieder auf LOW synchronisieren.
        // Beim letzten Bit nicht hart fehlschlagen, weil danach die Leitung freigegeben werden kann.
        if bit < 39 {
            wait_for_pin_state(io, false, 120).map_err(|_| "Bit LOW Timeout")?;
        } else {
            let _ = wait_for_pin_state(io, false, 120);
        }
    }

    let checksum = data[0]
        .wrapping_add(data[1])
        .wrapping_add(data[2])
        .wrapping_add(data[3]);

    if checksum != data[4] {
        return Err("Checksum Fehler");
    }

    Ok(data)
}

pub fn read(io: &mut PinDriver<InputOutput>) -> SensorResult {
    // Für DHT11 sind 1000ms oft okay, 2000ms schaden aber nicht.
    FreeRtos::delay_ms(2000);

    // Bus freigeben.
    io.set_high().unwrap();

    FreeRtos::delay_ms(10);

    // Startsignal. 25ms ist für DHT11 und DHT22 lang genug.
    io.set_low().unwrap();

    FreeRtos::delay_ms(25);

    // Leitung freigeben, dann 20-40us warten.
    io.set_high().unwrap();

    busy_wait_us(40);

    let data = match interrupt::free(|| read_frame(&io)) {
        Ok(data) => data,

        Err(e) => return SensorResult::Err(e),
    };

    // DHT11-Dekodierung:
    SensorResult::Ok(SensorData {
        humidity: data[0] as f32 + data[1] as f32 / 10.0,

        temperature: data[2] as f32 + data[3] as f32 / 10.0,
    })
}
