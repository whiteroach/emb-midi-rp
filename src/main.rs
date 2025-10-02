#![no_std]
#![no_main]

mod pot;
mod display;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config};
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{I2c, Config as I2cConfig};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use pot::read_midi_val_from_pot;
use display::{init_display, show_cc};

bind_interrupts!(struct AdcIrqs {
    ADC_IRQ_FIFO => embassy_rp::adc::InterruptHandler;
});

bind_interrupts!(struct I2cIrqs {
    I2C0_IRQ => embassy_rp::i2c::InterruptHandler<embassy_rp::peripherals::I2C0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    // ADC setup
    let mut adc = Adc::new(p.ADC, AdcIrqs, Config::default());
    let mut ch0 = Channel::new_pin(p.PIN_26, embassy_rp::gpio::Pull::None);

    //OLED I2C setup
    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = 400_000;
        
    let i2c = I2c::new_async(
    p.I2C0,
    p.PIN_9, // SCL
    p.PIN_8, // SDA
    I2cIrqs,    // provide interrupts for async driver
    i2c_config,
);
    let mut disp = init_display(i2c).await;
    
    // // Last stable MIDI value
    let mut last_midi_value: u8 = 0;

    loop {
        let midi_value = read_midi_val_from_pot(&mut adc, &mut ch0, last_midi_value).await;

        if midi_value != last_midi_value {
            last_midi_value = midi_value;
        } 

        info!("MIDI {}", last_midi_value);

        Timer::after_millis(100).await;
    }
}
