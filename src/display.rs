#![no_std]

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use ssd1306::{mode::{BufferedGraphicsMode, BufferedGraphicsModeAsync}, prelude::*, Ssd1306, Ssd1306Async};

use display_interface_i2c::I2CInterface;
use embedded_hal_async::i2c::I2c as AsyncI2c;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;


/// Initialize the SSD1306 display asynchronously
pub async fn init_display<I2C, E>(
    i2c: I2C,
) -> Ssd1306Async<I2CInterface<I2C>, DisplaySize128x32, BufferedGraphicsModeAsync<ssd1306::size::DisplaySize128x32>>
where
    I2C: AsyncI2c<Error = E>,
    E: core::fmt::Debug,
{
    // Create I2C interface
    let interface = I2CInterface::new(i2c, 0x3C, 0);

    let mut disp = Ssd1306Async::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    disp.init().await.unwrap();
    disp.clear(BinaryColor::Off);
    disp.flush().await.unwrap();

    disp
}

/// Show CC value on display (async safe)
pub fn show_cc<I2C>(display: &Ssd1306Async<I2CInterface<I2C>, DisplaySize128x32, BufferedGraphicsModeAsync<ssd1306::size::DisplaySize128x32>>, cc: u8)
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let mut disp = display.lock(); // lock for exclusive access

    disp.clear();
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::new(&format!("CC{}", cc), Point::new(0, 10), style)
        .draw(disp.deref_mut())
        .unwrap();
    disp.flush().unwrap();
}
