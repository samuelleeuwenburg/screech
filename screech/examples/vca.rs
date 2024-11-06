use screech::module::Module;
use screech::patchbay::{PatchError, PatchPoint, PatchPointOutput, Patchbay};
use screech::processor::Processor;
use screech::sample::Sample;
use screech_macro::modularize;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use wavv::{Data, Wav};

// Set the buffer size and sample rate
const DURATION: usize = 10;
const SAMPLE_RATE: usize = 48000;
const BUFFER_SIZE: usize = SAMPLE_RATE * DURATION;

/// Basic sine wave oscillator
struct Oscillator {
    value: f32,
    frequency: f32,
    output: PatchPoint,
}

impl Oscillator {
    fn new(output: PatchPoint, frequency: f32) -> Self {
        Oscillator {
            value: 0.0,
            frequency,
            output,
        }
    }
}

impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Oscillator {
    fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) -> Result<(), PatchError> {
        // Ramp up from -1.0 to 1.0 based on the set `frequency`
        // then use this value to convert to a sinusoidal waveshape.
        self.value += (1.0 / SAMPLE_RATE as f32) * self.frequency;

        if self.value >= 1.0 {
            self.value -= 2.0;
        }

        // Update the output value in the patchbay.
        patchbay.set_sample(&mut self.output, (self.value * std::f32::consts::PI).sin());

        Ok(())
    }
}

/// VCA module that takes two inputs (signal and modulator) and has a single output.
struct Vca {
    modulator: PatchPointOutput,
    input: PatchPointOutput,
    output: PatchPoint,
}

impl Vca {
    fn new(modulator: PatchPointOutput, input: PatchPointOutput, output: PatchPoint) -> Self {
        Vca {
            modulator,
            input,
            output,
        }
    }
}

impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Vca {
    fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) -> Result<(), PatchError> {
        // Take the input signal and multiply it by the modulator input.
        patchbay.set_sample(
            &mut self.output,
            patchbay.get_sample(self.input)? * patchbay.get_sample(self.modulator)?,
        );

        Ok(())
    }
}

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
    let osc_point = patchbay.get_point();
    let lfo_point = patchbay.get_point();
    let vca_point = patchbay.get_point();
    let output = vca_point.output();

    let vca = Vca::new(lfo_point.output(), osc_point.output(), vca_point);
    let osc = Oscillator::new(osc_point, 220.0);
    let lfo = Oscillator::new(lfo_point, 1.0);

    // Process the modules
    let mut processor: Processor<SAMPLE_RATE, 3, Modules> = Processor::new([
        Modules::Oscillator(osc),
        Modules::Oscillator(lfo),
        Modules::Vca(vca),
    ]);

    for i in 0..BUFFER_SIZE {
        processor.process_modules(&mut patchbay);
        buffer[i] = patchbay.get_sample(output).unwrap();
    }

    buffer_to_wave_file(&buffer)?;

    Ok(())
}

fn buffer_to_wave_file(buffer: &[Sample]) -> Result<(), Box<dyn Error>> {
    let normalized: Vec<i16> = buffer
        .into_iter()
        .map(|x| (x * (i16::MAX as f32)) as i16)
        .collect();

    let wav = Wav::from_data(Data::BitDepth16(normalized), SAMPLE_RATE, 1);

    let path = Path::new("./examples/oscillator.wav");
    let mut file = File::create(&path)?;

    file.write_all(&wav.to_bytes())?;

    Ok(())
}
