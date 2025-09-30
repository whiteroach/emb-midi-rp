#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
// ADC driver (Adc) and per-pin channel wrapper (Channel)
use embassy_rp::adc::{Adc, Channel, Config};
// Macro to bind hardware interrupts to Embassy driver handlers
use embassy_rp::bind_interrupts;
// Async timer (lets us do `Timer::after_millis(...).await`)
use embassy_time::Timer;
// Import crates for logging + panic handling (just for side effects)
// - defmt_rtt: send logs over RTT (debug probe)
// - panic_probe: report panics via RTT
use {defmt_rtt as _, panic_probe as _};

// Bind the RP2040 ADC interrupt to the Embassy ADC driver
// This connects the hardware IRQ (ADC_IRQ_FIFO) to the handler that wakes our futures
bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => embassy_rp::adc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the RP2040 peripherals with default settings.
    // Returns a struct `p` that contains tokens for each peripheral and pin.
    let p = embassy_rp::init(Default::default());



    // Create an ADC driver, giving it the ADC peripheral and the bound interrupts
    let mut adc = Adc::new(p.ADC, Irqs, Config::default());

    // Configure GPIO26 as an ADC input channel (ADC0).
    // `Pull::None` means no internal pull-up or pull-down (important for analog).
    let mut ch0 = Channel::new_pin(p.PIN_26, embassy_rp::gpio::Pull::None);

    const SAMPLES_NR: usize = 16;
    let mut pot_buffer = [0u16; SAMPLES_NR];
    let mut buffer_idx = 0;

    //last selected CC number (to apply hysteresis)
    let mut last_cc:u8 = 0;

    // how many CC numbers we want to select among
    const CC_COUNT: u16 = 128;
    // base CC (first number, e.g. 20 means CC20–CC35)
    const BASE_CC: u8 = 0;
    // size of each ADC step
    const STEP: u16 = 4096 / CC_COUNT;
    // hysteresis margin around each boundary
    const DEAD_BAND: u16 = 16;

    // Infinite loop: keep reading and printing the potentiometer value
    loop {
        // Asynchronously read a sample from the ADC (12-bit value: 0–4095).
        // `.await` means the task yields until the ADC conversion finishes.
        let pot_raw_value: u16 = adc.read(&mut ch0).await.unwrap_or_else(|e| {
            warn!("ADC error: {:?}", e);
            0 // fallback value
        });

        pot_buffer[buffer_idx] = pot_raw_value;
        buffer_idx = (buffer_idx + 1) % SAMPLES_NR;
        let avg_pot_value = pot_buffer.iter().sum::<u16>() / SAMPLES_NR as u16;

        // Calculate zone index (which CC slot)
        let mut zone = avg_pot_value / STEP;
        if zone >= CC_COUNT {
            zone = CC_COUNT - 1;
        }
        let candidate_cc: u8 = BASE_CC + zone as u8;

        // Hysteresis: only switch CC if moved far enough into new zone
        let current_boundary = zone * STEP;
        let lower_bound = current_boundary.saturating_sub(DEAD_BAND);
        let upper_bound = (current_boundary + STEP).min(4095).saturating_sub(DEAD_BAND);

        if avg_pot_value < lower_bound || avg_pot_value > upper_bound {
            last_cc = candidate_cc;
        }


        info!(
            "ADC raw={} avg={} => CC {}",
            pot_raw_value, avg_pot_value, last_cc
        );

        Timer::after_millis(200).await;
        // Convert raw ADC value into a voltage (assuming 3.3V reference).
        // let voltage: f32 = (avg_pot_value as f32 / 4095.0) * 3.3;

        // // ADC 0-4095 midi 0-127
        // let midi_value = ((avg_pot_value as u32 * 127) / 4095) as u8;
        // // let midi_value = ((pot_value as u32 * 127) / 4095) as u8;

        // if midi_value.abs_diff(last_midi_value) > 1 {
        //    last_midi_value = midi_value;
        // }
        
        // // info!("ADC raw={} -> {} V {} midi_value", avg_pot_value, voltage, last_midi_value);
        // info!("ADC raw={} -> {} V {} midi_value", pot_raw_value, voltage, last_midi_value);
        // // Wait 500 ms before taking the next reading
        // Timer::after_millis(500).await;
    }
}