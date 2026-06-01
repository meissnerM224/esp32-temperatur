use core::result::Result::{Err, Ok};

// use crate::sensor::{SensorData, SensorResult};
use aht20_driver::{AHT20, SENSOR_ADDRESS};
use esp_idf_svc::hal::delay::{Ets, FreeRtos};
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::units::Hertz;
pub fn run() {
    // Peripherals = Zugriff auf die Hardware des ESP32
    let peripherals = Peripherals::take().unwrap();
    let config = I2cConfig::new().baudrate(Hertz(50_000));

    // I2C Bus initialisieren mit GPIO 21 (SDA) und GPIO 22 (SCL)
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21, // SDA
        peripherals.pins.gpio22, // SCL
        &config,
    )
    .unwrap();
    let mut aht20_uninit = AHT20::new(i2c, SENSOR_ADDRESS);

    // Schritt 2 - initialisieren (gibt neuen Struct zurück!)
    let mut sensor = match aht20_uninit.init(&mut Ets) {
        Ok(s) => {
            println!("Sensor erfolgreich initialisiert!");
            s
        }
        Err(e) => {
            println!("Sensor Init Fehler: {:?}", e);
            return;
        }
    };

    loop {
        match sensor.measure(&mut Ets) {
            Ok(measurement) => {
                println!("Temperatur: {:.1}°C", measurement.temperature);
                println!("Luftfeuchtigkeit: {:.1}%", measurement.humidity);
            }
            Err(e) => {
                println!("Sensor Fehler: {:?}", e);
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    // 5 Sekunden warten
}
