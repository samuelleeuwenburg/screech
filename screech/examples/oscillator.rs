mod to_wav;

use screech::modules::Oscillator;
use screech::{Patchbay, Processor};
use std::error::Error;
use to_wav::to_wav_file;

const DURATION: usize = 5;
const SAMPLE_RATE: usize = 48000;
const BUFFER_SIZE: usize = SAMPLE_RATE * DURATION;

fn main() -> Result<(), Box<dyn Error>> {
    let mut buffer = [0.0; BUFFER_SIZE];
    let mut patchbay: Patchbay<8> = Patchbay::new();
    let mut oscillator = Oscillator::new(patchbay.point().unwrap());
    let output = oscillator.output();

    oscillator.output_sine().set_frequency(440.0);

    let mut processor: Processor<SAMPLE_RATE, 1, _> = Processor::new([Some(oscillator)]);

    for i in 0..BUFFER_SIZE {
        // Change the waveshape every second
        let osc = processor.get_module_mut(0).unwrap();

        if i > SAMPLE_RATE * 4 {
            osc.output_saw();
        } else if i > SAMPLE_RATE * 3 {
            osc.output_pulse(0.5);
        } else if i > SAMPLE_RATE * 2 {
            let duty_cycle = (i % SAMPLE_RATE) as f32 / SAMPLE_RATE as f32;
            osc.output_pulse(duty_cycle);
        } else if i > SAMPLE_RATE * 1 {
            osc.output_triangle();
        }

        processor.process_modules(&mut patchbay);
        buffer[i] = patchbay.get(output);
    }

    to_wav_file(&buffer, SAMPLE_RATE, "oscillator")?;

    Ok(())
}
