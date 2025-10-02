#![no_std]
#![no_main]

mod pot;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config};
use embassy_rp::bind_interrupts;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use pot::read_midi_val_from_pot;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => embassy_rp::adc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut adc = Adc::new(p.ADC, Irqs, Config::default());
    
    let mut ch0 = Channel::new_pin(p.PIN_26, embassy_rp::gpio::Pull::None);

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
