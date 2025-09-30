use embassy_rp::adc::{Adc, Async, Channel};

pub async fn read_midi_val_from_pot(
    adc: &mut Adc<'_, Async>,
    ch: &mut Channel<'_>,
    last_midi_value: u8,
) -> u8 {
    const DEAD_BAND: u16 = 4; // ADC counts threshold to ignore small changes

    // --- Median-of-9 filter ---
    let mut samples = [0u16; 9];
    for s in samples.iter_mut() {
        *s = adc.read(ch).await.unwrap_or(0);
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

    midi_value
}