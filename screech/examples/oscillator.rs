use screech::module::Module;
use screech::patchbay::{PatchError, PatchPoint, Patchbay};
use screech::processor::Processor;
use screech::sample::Sample;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use wavv::{Data, Wav};

const DURATION: usize = 10;
const SAMPLE_RATE: usize = 48000;
const BUFFER_SIZE: usize = SAMPLE_RATE * DURATION;

struct Oscillator {
    frequency: f32,
    output: PatchPoint,
}

impl Oscillator {
    fn new(output: PatchPoint) -> Self {
        Oscillator {
            frequency: 220.0,
            output,
        }
    }
}

impl Module for Oscillator {
    fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) -> Result<(), PatchError> {
        let mut value = patchbay.get_sample(self.output.value())?;

        value += (2.0 / SAMPLE_RATE as f32) * self.frequency;

        if value >= 1.0 {
            value -= 2.0;
        }

        patchbay.set_sample(&mut self.output, value);

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut buffer = [0.0; BUFFER_SIZE];
    let mut patchbay: Patchbay<8> = Patchbay::new();
    let oscillator_point = patchbay.get_point();
    let output = oscillator_point.value();
    let oscillator = Oscillator::new(oscillator_point);
    let mut processor = Processor::new([oscillator]);

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
