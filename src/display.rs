use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::units::FromValueType;

use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

pub fn run() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Display Test startet...");

    // Hardware initialisieren
    let peripherals = Peripherals::take().unwrap();
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;

    // I2C Bus konfigurieren
    let config = I2cConfig::new().baudrate(400u32.kHz().into());
    let i2c = I2cDriver::new(i2c, sda, scl, &config).unwrap();

    // Display initialisieren
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x32,
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    display.init().unwrap();

    // Text Stil definieren
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    // Text auf Display schreiben
    Text::with_baseline(
        "Hello",
        Point::new(28, 4),
        text_style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();
    Text::with_baseline(
        "World!",
        Point::new(16,18),
        text_style,
        Baseline::Top
    )
    .draw(&mut display)
    .unwrap();

    // Display aktualisieren
    display.flush().unwrap();

    log::info!("Text auf Display geschrieben!");
}