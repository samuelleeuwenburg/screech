mod to_wav;

use screech::modules::{Mix, Oscillator, Vca};
use screech::{Module, PatchPoint, Patchbay, Processor, Signal};
use screech_macro::modularize;
use std::error::Error;
use to_wav::to_wav_file;

const DURATION: usize = 24;
const SAMPLE_RATE: usize = 48000;
const BUFFER_SIZE: usize = SAMPLE_RATE * DURATION;
const STEP_SIZE: usize = SAMPLE_RATE / 2 * 3;
const MODULES: usize = 32;
const PATCHPOINTS: usize = 256;

pub struct Voice {
    output: PatchPoint,
    patchbay: Patchbay<4>,
    lfo: Oscillator,
    osc: Oscillator,
    vca: Vca,
}

impl Voice {
    fn new(output: PatchPoint, frequency: f32) -> Self {
        let mut patchbay: Patchbay<4> = Patchbay::new();

        let mut lfo = Oscillator::new(patchbay.point().unwrap());
        let mut osc = Oscillator::new(patchbay.point().unwrap());
        let mut vca = Vca::new(patchbay.point().unwrap());

        lfo.set_frequency(1.618 / 2.0);
        osc.set_frequency(frequency).set_amplitude(0.1);
        vca.set_input(osc.output());
        vca.set_modulator(lfo.output());

        Voice {
            output,
            patchbay,
            lfo,
            osc,
            vca,
        }
    }

    fn output(&self) -> Signal {
        self.output.signal()
    }
}

impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Voice {
    fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) {
        <Oscillator as Module<SAMPLE_RATE>>::process(&mut self.lfo, &mut self.patchbay);
        <Oscillator as Module<SAMPLE_RATE>>::process(&mut self.osc, &mut self.patchbay);
        <Vca as Module<SAMPLE_RATE>>::process(&mut self.vca, &mut self.patchbay);

        patchbay.set(&mut self.output, self.patchbay.get(self.vca.output()));
    }
}

#[modularize]
enum Modules {
    Voice(Voice),
    Mix(Mix),
}

fn main() -> Result<(), Box<dyn Error>> {
    const EMPTY: Option<Modules> = None;
    let modules = [EMPTY; MODULES];
    let mut buffer = [0.0; BUFFER_SIZE];
    let mut patchbay: Patchbay<PATCHPOINTS> = Patchbay::new();
    let mut processor: Processor<SAMPLE_RATE, MODULES, Modules> = Processor::new(modules);

    // Create a final output mixer
    let mixer = Mix::new(patchbay.point().unwrap());
    let output = mixer.output();
    let mixer_id = processor.insert_module(Modules::Mix(mixer)).unwrap();

    for s in 0..(BUFFER_SIZE / STEP_SIZE) {
        // Keep adding voices
        let frequency = (s + 1) as f32 * 80.0;
        let voice = Voice::new(patchbay.point().unwrap(), frequency);

        if let Some(Modules::Mix(m)) = processor.get_module_mut(mixer_id) {
            m.add_input(voice.output(), s);
        }

        processor.insert_module(Modules::Voice(voice)).unwrap();

        for i in 0..STEP_SIZE {
            let index = i + (s * STEP_SIZE);
            processor.process_modules(&mut patchbay);
            buffer[index] = patchbay.get(output);
        }
    }

    to_wav_file(&buffer, SAMPLE_RATE, "dynamic")?;

    Ok(())
}
