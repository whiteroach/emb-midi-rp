#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config};
use embassy_rp::bind_interrupts;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => embassy_rp::adc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut adc = Adc::new(p.ADC, Irqs, Config::default());
    let mut ch0 = Channel::new_pin(p.PIN_26, embassy_rp::gpio::Pull::None);

    // Last stable MIDI value
    let mut last_midi_value: u8 = 0;

    // Parameters for step quantization
    // const MIDI_STEPS: u16 = 128;        // MIDI 0–127
    const DEAD_BAND: u16 = 4;           // ADC counts threshold to ignore small changes

    loop {
        // --- Median-of-9 filter ---
        let mut samples = [0u16; 9];
        for s in samples.iter_mut() {
            *s = adc.read(&mut ch0).await.unwrap_or(0);
        }

        // Bubble sort (no_std compatible)
        for i in 0..samples.len() {
            for j in 0..samples.len() - 1 - i {
                if samples[j] > samples[j + 1] {
                    let tmp = samples[j];
                    samples[j] = samples[j + 1];
                    samples[j + 1] = tmp;
                }
            }
        }

        let median_value = samples[4];

        // --- Quantize ADC to MIDI step ---
        let mut midi_value = ((median_value as u32 * 127) / 4095) as u8;

        // --- Apply deadband to prevent jumps ---
        let last_adc = (last_midi_value as u32 * 4095 / 127) as u16;
        if median_value.abs_diff(last_adc) <= DEAD_BAND {
            midi_value = last_midi_value; // ignore small fluctuations
        }

        if midi_value != last_midi_value {
            last_midi_value = midi_value;
            info!("ADC median={} → MIDI {}", median_value, last_midi_value);
            // TODO: send MIDI CC here
        }

        Timer::after_millis(100).await;
    }
}
