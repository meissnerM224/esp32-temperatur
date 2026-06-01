// Deine Datenstruktur - wie ein Model in Flutter
#[derive(Debug)]
pub struct SensorData {
    pub temperature: f32,
    pub humidity: f32,
}

// Deine Result Struktur die wir besprochen haben
#[derive(Debug)]
pub enum SensorResult {
    Ok(SensorData),
    Err(&'static str),
}
