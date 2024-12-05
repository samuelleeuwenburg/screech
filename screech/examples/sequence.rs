mod to_wav;

use screech::modules::{Clock, Envelope, Oscillator, Vca};
use screech::{Module, Patchbay, Processor};
use screech_macro::modularize;
use std::error::Error;
use to_wav::to_wav_file;

const DURATION: usize = 5;
const SAMPLE_RATE: usize = 48000;
const BUFFER_SIZE: usize = SAMPLE_RATE * DURATION;
const MODULES: usize = 256;
const PATCHPOINTS: usize = 256;

#[modularize]
enum Modules {
    Clock(Clock),
    Envelope(Envelope),
    Oscillator(Oscillator),
    Vca(Vca),
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor: Processor<SAMPLE_RATE, MODULES, Modules> = Processor::empty();
    let mut buffer = [0.0; BUFFER_SIZE];
    let mut patchbay: Patchbay<PATCHPOINTS> = Patchbay::new();

    let mut oscillator = Oscillator::new(patchbay.point().unwrap());
    let clock = Clock::new(patchbay.point().unwrap(), 60.0);
    let mut envelope = Envelope::new(clock.output(), patchbay.point().unwrap());
    let mut vca = Vca::new(patchbay.point().unwrap());

    envelope.set_ar(100.0, 100.0);
    oscillator.output_sine().set_frequency(440.0);
    vca.set_input(oscillator.output());
    vca.set_modulator(envelope.output());

    let output = vca.output();

    processor.insert_module(Modules::Oscillator(oscillator));
    processor.insert_module(Modules::Clock(clock));
    processor.insert_module(Modules::Envelope(envelope));
    processor.insert_module(Modules::Vca(vca));

    for i in 0..BUFFER_SIZE {
        processor.process_modules(&mut patchbay);
        buffer[i] = patchbay.get(output);
    }

    to_wav_file(&buffer, SAMPLE_RATE, "sequence")?;

    Ok(())
}
