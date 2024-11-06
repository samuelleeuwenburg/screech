use screech::module::Module;
use screech::patchbay::{PatchError, PatchPoint, PatchPointValue, Patchbay};
use screech::processor::Processor;
use screech::sample::Sample;
use screech_macro::modularize;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use wavv::{Data, Wav};

const DURATION: usize = 10;
const SAMPLE_RATE: usize = 48000;
const BUFFER_SIZE: usize = SAMPLE_RATE * DURATION;

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

    fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) -> Result<(), PatchError> {
        self.value += (1.0 / SAMPLE_RATE as f32) * self.frequency;

        if self.value >= 1.0 {
            self.value -= 2.0;
        }

        patchbay.set_sample(&mut self.output, (self.value * std::f32::consts::PI).sin());

        Ok(())
    }
}

struct Vca {
    modulator: PatchPointValue,
    input: PatchPointValue,
    output: PatchPoint,
}

impl Vca {
    fn new(modulator: PatchPointValue, input: PatchPointValue, output: PatchPoint) -> Self {
        Vca {
            modulator,
            input,
            output,
        }
    }

    fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) -> Result<(), PatchError> {
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
    let mut buffer = [0.0; BUFFER_SIZE];
    let mut patchbay: Patchbay<8> = Patchbay::new();

    let osc_point = patchbay.get_point();
    let lfo_point = patchbay.get_point();
    let vca_point = patchbay.get_point();
    let output = vca_point.value();

    let vca = Vca::new(lfo_point.value(), osc_point.value(), vca_point);
    let osc = Oscillator::new(osc_point, 220.0);
    let lfo = Oscillator::new(lfo_point, 1.0);

    let mut processor = Processor::new([
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
