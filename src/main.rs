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

    // Infinite loop: keep reading and printing the potentiometer value
    loop {
        // Asynchronously read a sample from the ADC (12-bit value: 0–4095).
        // `.await` means the task yields until the ADC conversion finishes.
        let value: u16 = adc.read(&mut ch0).await.unwrap_or_else(|e| {
            warn!("ADC error: {:?}", e);
            0 // fallback value
        });

        // Convert raw ADC value into a voltage (assuming 3.3V reference).
        let voltage: f32 = (value as f32 / 4095.0) * 3.3;

        // Print the raw value and voltage via defmt logging
        info!("ADC raw={} -> {} V", value, voltage);

        // Wait 500 ms before taking the next reading
        Timer::after_millis(500).await;
    }
}