# screech
[![.github/workflows/main.yml](https://github.com/samuelleeuwenburg/screech/actions/workflows/main.yml/badge.svg)](https://github.com/samuelleeuwenburg/screech/actions/workflows/main.yml)
[![Crates.io](https://img.shields.io/crates/v/screech.svg)](https://crates.io/crates/screech)
[![docs.rs](https://docs.rs/screech/badge.svg)](https://docs.rs/screech/)

# Screech

Opinionated real time audio library with a focus performance and no_std environments.

## Getting started

```
use screech::module::Module;
use screech::patchbay::{PatchError, PatchPoint, PatchPointValue, Patchbay};
use screech::processor::Processor;
use screech::sample::Sample;
use screech_macro::modularize;

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

    fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) -> Result<(), PatchError> {
        // Ramp up from -1.0 to 1.0 based on the set `frequency`
        // then use this value to convert to a sinusoidal waveshape.
        self.value += (1.0 / SAMPLE_RATE as f32) * self.frequency;

        if self.value >= 1.0 {
            self.value -= 2.0;
        }

        let output = (self.value * std::f32::consts::PI).sin();

        // Update the output value in the patchbay.
        patchbay.set_sample(&mut self.output, output);

        Ok(())
    }
}

/// VCA module that takes two inputs (signal and modulator) and has a single output.
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

fn main() {
    // Set up the memory
    let mut buffer = [0.0; BUFFER_SIZE];
    let mut patchbay: Patchbay<8> = Patchbay::new();

    // Build connections
    let osc_point = patchbay.get_point();
    let lfo_point = patchbay.get_point();
    let vca_point = patchbay.get_point();
    let output = vca_point.value();

    let vca = Vca::new(lfo_point.value(), osc_point.value(), vca_point);
    let osc = Oscillator::new(osc_point, 220.0);
    let lfo = Oscillator::new(lfo_point, 1.0);

    // Process the modules
    let mut processor = Processor::new([
        Modules::Oscillator(osc),
        Modules::Oscillator(lfo),
        Modules::Vca(vca),
    ]);

    for i in 0..BUFFER_SIZE {
        processor.process_modules(&mut patchbay);
        buffer[i] = patchbay.get_sample(output).unwrap();
    }
}
```
