mod to_wav;

use screech::modules::{Oscillator, Vca};
use screech::{Module, Patchbay, Processor};
use screech_macro::modularize;
use std::error::Error;
use to_wav::to_wav_file;

// Set the buffer size and sample rate
const DURATION: usize = 10;
const SAMPLE_RATE: usize = 48000;
const BUFFER_SIZE: usize = SAMPLE_RATE * DURATION;

#[modularize]
enum Modules {
    Oscillator(Oscillator),
    Vca(Vca),
}

fn main() -> Result<(), Box<dyn Error>> {
    // Set up the memory
    let mut buffer = [0.0; BUFFER_SIZE];
    let mut patchbay: Patchbay<8> = Patchbay::new();

    // Build connections
    let mut osc = Oscillator::new(patchbay.point().unwrap());
    let mut lfo = Oscillator::new(patchbay.point().unwrap());
    let mut vca = Vca::new(patchbay.point().unwrap());
    let output = vca.output();

    lfo.set_frequency(1.0 / 3.3).output_sine();
    osc.set_frequency(220.0).output_saw();
    vca.set_input(osc.output());
    vca.set_modulator(lfo.output());

    // Process the modules
    let mut processor: Processor<SAMPLE_RATE, 3, Modules> = Processor::new([
        Some(Modules::Oscillator(osc)),
        Some(Modules::Oscillator(lfo)),
        Some(Modules::Vca(vca)),
    ]);

    for i in 0..BUFFER_SIZE {
        processor.process_modules(&mut patchbay);
        buffer[i] = patchbay.get(output);
    }

    to_wav_file(&buffer, SAMPLE_RATE, "vca")?;

    Ok(())
}
