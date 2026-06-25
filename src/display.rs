use display_interface_i2c::I2CInterface;
use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_idf_svc::hal::gpio::{Gpio16, Gpio17};
use esp_idf_svc::hal::i2c::I2C0;
use esp_idf_svc::hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_svc::hal::units::FromValueType;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::{prelude::*, Ssd1306};

pub type Display = Ssd1306<
    I2CInterface<I2cDriver<'static>>,
    DisplaySize128x32,
    BufferedGraphicsMode<DisplaySize128x32>,
>;

pub fn init(i2c: I2C0<'static>, sda: Gpio16<'static>, scl: Gpio17<'static>) -> Display {
    let config = I2cConfig::new().baudrate(100u32.kHz().into());
    let i2c_driver = I2cDriver::new(i2c, sda, scl, &config).unwrap();
    let interface = I2CInterface::new(i2c_driver, 0x3C, 0x40);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();
    display
}

pub fn show(display: &mut Display, temperature: f32, humidity: f32, time: &str) {
    use embedded_graphics::mono_font::ascii::FONT_9X18;
    display.clear(BinaryColor::Off).unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_9X18)
        .text_color(BinaryColor::On)
        .build();

    let mut line1 = heapless::String::<32>::new();
    core::fmt::write(
        &mut line1,
        format_args!("{:.1}C {:.0}%", temperature, humidity),
    )
    .unwrap();

    // let mut lin2 = heapless::String::<32>::new();
    // core::fmt::write(&mut line1, format_args!("Humi: {:.1}%", humidity)).unwrap();

    Text::with_baseline(&line1, Point::new(0, 4), text_style, Baseline::Top)
        .draw(display)
        .unwrap();

    Text::with_baseline(time, Point::new(0, 18), text_style, Baseline::Top)
        .draw(display)
        .unwrap();

    display.flush().unwrap();
}
