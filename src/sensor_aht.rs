use crate::sensor::{SensorData, SensorResult};
use aht20_driver::AHT20Initialized;
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::i2c::I2cDriver;

// Liest eine einzelne Messung vom bereits initialisierten AHT25.
// Der Sensor wird in main() EINMALIG aufgebaut und hier nur noch ausgelesen.
pub fn read(sensor: &mut AHT20Initialized<'_, I2cDriver<'_>>) -> SensorResult {
    match sensor.measure(&mut Ets) {
        Ok(m) => SensorResult::Ok(SensorData {
            temperature: m.temperature,
            humidity: m.humidity,
        }),
        Err(e) => {
            println!("AHT25 Messfehler: {:?}", e);
            SensorResult::Err("AHT25 Messfehler")
        }
    }
}
