use screech::module::Module;
use screech::modules::{Dummy, Mix, Oscillator, Vca};
use screech::patchbay::{PatchError, PatchPointOutput, Patchbay};
use screech::processor::Processor;
use screech::sample::Sample;
use screech_macro::modularize;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use wavv::{Data, Wav};

const DURATION: usize = 16;
const SAMPLE_RATE: usize = 48000;
const BUFFER_SIZE: usize = SAMPLE_RATE / 2 * 3;
const MODULES: usize = 256;
const PATCHPOINTS: usize = 256;
const MIXER: usize = 255;

#[modularize]
enum Modules {
    Oscillator(Oscillator),
    Vca(Vca),
    Mix(Mix),
    Dummy(Dummy),
}

fn add_voice(
    processor: &mut Processor<SAMPLE_RATE, MODULES, Modules>,
    patchbay: &mut Patchbay<PATCHPOINTS>,
    position: usize,
) -> PatchPointOutput {
    let frequency = (position + 1) as f32 * 80.0;
    let index = position * 3;

    let vca_point = patchbay.get_point();
    let osc_point = patchbay.get_point();
    let lfo_point = patchbay.get_point();
    let final_output = vca_point.output();

    let vca = Vca::new(lfo_point.output(), osc_point.output(), vca_point);
    let lfo = Oscillator::new(lfo_point, 1.618 / 2.0);
    let mut osc = Oscillator::new(osc_point, frequency);
    osc.amplitude = 0.5;

    processor.replace_module(Modules::Oscillator(osc), index);
    processor.replace_module(Modules::Oscillator(lfo), index + 1);
    processor.replace_module(Modules::Vca(vca), index + 2);

    final_output
}

fn add_to_mixer(
    processor: &mut Processor<SAMPLE_RATE, MODULES, Modules>,
    point: PatchPointOutput,
    position: usize,
) {
    if let Modules::Mix(m) = processor.get_module_mut(MIXER) {
        m.add_input(point, position);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    const DUMMY: Modules = Modules::Dummy(Dummy);
    let modules = [DUMMY; MODULES];
    let mut buffer = [0.0; DURATION * SAMPLE_RATE];
    let mut patchbay: Patchbay<PATCHPOINTS> = Patchbay::new();
    let mut processor: Processor<SAMPLE_RATE, MODULES, Modules> = Processor::new(modules);

    // Create a final output mixer
    let mix_point = patchbay.get_point();
    let output = mix_point.output();
    processor.replace_module(Modules::Mix(Mix::new(mix_point)), MIXER);

    for s in 0..(DURATION * SAMPLE_RATE / BUFFER_SIZE) {
        // Keep adding voices
        let voice_output = add_voice(&mut processor, &mut patchbay, s);
        add_to_mixer(&mut processor, osc_output, s);

        for i in 0..BUFFER_SIZE {
            let index = i + (s * BUFFER_SIZE);
            processor.process_modules(&mut patchbay);
            buffer[index] = patchbay.get_sample(output).unwrap();
        }
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

    let path = Path::new("./examples/dynamic.wav");
    let mut file = File::create(&path)?;

    file.write_all(&wav.to_bytes())?;

    Ok(())
}
