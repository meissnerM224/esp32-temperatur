// mod display;
mod sensor;
// mod sensor_aht;
mod sensor_dht;
fn main() {
    match sensor_dht::read() {
        sensor::SensorResult::Ok(data) => {
            println!("Temperatur: {:.1}°C", data.temperature);
            println!("Luftfeuchtigkeit: {:.1}%", data.humidity);
        }
        sensor::SensorResult::Err(e) => {
            println!("Fehler: {}", e);
        }
    }
    // display::run();
    // sensor_aht::run();
}
