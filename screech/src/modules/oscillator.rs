use crate::{Module, PatchPoint, Patchbay, Signal};

const PI: f32 = 3.141;

enum Waveform {
    Sine,
    Saw,
    Triangle,
    Pulse(f32),
}

/// Basic oscillator with multiple waveshapes
pub struct Oscillator {
    wave_shape: Waveform,
    frequency: f32,
    amplitude: f32,
    output: PatchPoint,
    value: f32,
}

impl Oscillator {
    pub fn new(output: PatchPoint) -> Self {
        Oscillator {
            wave_shape: Waveform::Sine,
            frequency: 440.0,
            amplitude: 0.8,
            output,
            value: 0.0,
        }
    }

    pub fn output(&self) -> Signal {
        self.output.signal()
    }

    pub fn set_frequency(&mut self, frequency: f32) -> &mut Self {
        self.frequency = frequency;
        self
    }

    pub fn get_frequency(&self) -> f32 {
        self.frequency
    }

    pub fn set_amplitude(&mut self, amplitude: f32) -> &mut Self {
        self.amplitude = amplitude;
        self
    }

    pub fn get_amplitude(&self) -> f32 {
        self.amplitude
    }

    pub fn output_sine(&mut self) -> &mut Self {
        self.wave_shape = Waveform::Sine;
        self
    }

    pub fn output_saw(&mut self) -> &mut Self {
        self.wave_shape = Waveform::Saw;
        self
    }

    pub fn output_triangle(&mut self) -> &mut Self {
        self.wave_shape = Waveform::Triangle;
        self
    }

    pub fn output_pulse(&mut self, duty_cycle: f32) -> &mut Self {
        self.wave_shape = Waveform::Pulse(duty_cycle);
        self
    }
}

impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Oscillator {
    fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) {
        // Ramp up from -1.0 to 1.0 based on the set `frequency`
        // then use this value to convert to the specific waveforms
        self.value += (1.0 / SAMPLE_RATE as f32) * self.frequency;

        // Wrap around
        if self.value >= 1.0 {
            self.value -= 2.0;
        }

        // Create the desired waveform
        let wave = match self.wave_shape {
            Waveform::Saw => self.value,
            Waveform::Sine => sine(self.value),
            Waveform::Triangle => triangle(self.value),
            Waveform::Pulse(duty_cycle) => pulse(self.value, duty_cycle),
        };

        // Set the amplitude
        let output = wave * self.amplitude;

        // Update the output value in the patchbay.
        patchbay.set(&mut self.output, output);
    }
}

// Bashkara approximation of a sine
fn sine(input: f32) -> f32 {
    // Calculate with positive values only
    let x = if input < 0.0 { -input * PI } else { input * PI };

    let numerator = 16.0 * x * (PI - x);
    let denominator = 5.0 * PI * PI - 4.0 * x * (PI - x);
    let sine = numerator / denominator;

    // Normalize back positive to negative if needed
    if input < 0.0 {
        -sine
    } else {
        sine
    }
}
fn triangle(input: f32) -> f32 {
    if input < 0.0 {
        (input + 1.0) * 2.0 - 1.0
    } else {
        (input * 2.0) * -1.0 + 1.0
    }
}

fn pulse(input: f32, duty_cycle: f32) -> f32 {
    // Normalize around the centerpoint
    let threshold = (duty_cycle * 2.0) - 1.0;

    if input >= threshold {
        1.0
    } else {
        -1.0
    }
}
