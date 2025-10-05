use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use heapless::String;
use ssd1306::{mode::{ BufferedGraphicsModeAsync}, prelude::*, Ssd1306Async};

use display_interface_i2c::I2CInterface;
use embedded_hal_async::i2c::I2c as AsyncI2c;
use core::fmt::Write;

// use ssd1306::{prelude::*, Ssd1306Async};

pub async fn init_display<I2C, E>(i2c: I2C) -> Ssd1306Async<I2CInterface<I2C>, DisplaySize128x32, BufferedGraphicsModeAsync<ssd1306::size::DisplaySize128x32>>
where
    I2C: AsyncI2c<Error = E>,
    E: core::fmt::Debug,
{
    let interface = I2CInterface::new(i2c, 0x3C, 0);
    let mut disp = Ssd1306Async::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    disp.init().await.unwrap();
    disp.clear(BinaryColor::Off).unwrap();
    disp
}

pub async fn show_cc<I2C>(disp: &mut Ssd1306Async<I2CInterface<I2C>, DisplaySize128x32, BufferedGraphicsModeAsync<ssd1306::size::DisplaySize128x32>>, cc: u8)
where
    I2C: AsyncI2c,
{
    disp.clear(BinaryColor::Off).unwrap();

    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    // Use heapless String with a capacity of 32 chars (enough for "CC127")
    let mut buf: String<32> = String::new();

    write!(buf, "CC{}", cc).unwrap();

    // Pass as &str
    Text::new(buf.as_str(), Point::new(0, 10), style)
        .draw(disp)
        .unwrap();

    disp.flush().await.unwrap();
}


