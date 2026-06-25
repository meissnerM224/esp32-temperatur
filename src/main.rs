use aht20_driver::{AHT20, SENSOR_ADDRESS};
use esp_idf_svc::hal::delay::{Ets, FreeRtos};
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::units::Hertz;

mod display;
mod sensor;
mod sensor_aht;
// mod sensor_dht;
mod wifi;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    // WLAN verbinden und Zeit synchronisieren
    let _wifi = wifi::connect(unsafe { core::mem::transmute(peripherals.modem) });
    let _sntp = wifi::sync_time();
    unsafe {
        esp_idf_svc::sys::setenv(c"TZ".as_ptr(), c"CET-1CEST,M3.5.0,M10.5.0/3".as_ptr(), 1);
        esp_idf_svc::sys::tzset();
    }

    // Display
    let mut display = display::init(
        unsafe { core::mem::transmute(peripherals.i2c0) },
        unsafe { core::mem::transmute(peripherals.pins.gpio16) },
        unsafe { core::mem::transmute(peripherals.pins.gpio17) },
    );
    let i2c_config = I2cConfig::new().baudrate(Hertz(50_000));
    let i2c = I2cDriver::new(
        peripherals.i2c1,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
        &i2c_config,
    )
    .unwrap();
    let mut aht_uninit = AHT20::new(i2c, SENSOR_ADDRESS);
    let mut aht = aht_uninit
        .init(&mut Ets)
        .expect("AHT25 konnte nicht initialisiert werden");
    println!("AHT25 erfoglreich initialisiert!");

    loop {
        let time = get_time();
        match sensor_aht::read(&mut aht) {
            sensor::SensorResult::Ok(data) => {
                println!("Temperatur: {:.1}°C", data.temperature);
                println!("Luftfeuchtigkeit: {:.1}%", data.humidity);
                display::show(&mut display, data.temperature, data.humidity, &time);
            }
            sensor::SensorResult::Err(e) => {
                println!("Fehler: {}", e);
            }
        }
        FreeRtos::delay_ms(2000);
    }
}

fn get_time() -> heapless::String<16> {
    let now = unsafe { esp_idf_svc::sys::time(core::ptr::null_mut()) };

    // In lokale Zeit umrechnen – nutzt die oben gesetzte Zeitzone inkl. DST
    let mut timeinfo: esp_idf_svc::sys::tm = unsafe { core::mem::zeroed() };
    unsafe { esp_idf_svc::sys::localtime_r(&now, &mut timeinfo) };

    let mut s = heapless::String::<16>::new();
    core::fmt::write(
        &mut s,
        format_args!(
            "{:02}:{:02}:{:02}",
            timeinfo.tm_hour, timeinfo.tm_min, timeinfo.tm_sec
        ),
    )
    .unwrap();
    s
}
