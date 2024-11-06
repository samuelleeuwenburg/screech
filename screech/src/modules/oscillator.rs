use crate::module::Module;
use crate::patchbay::{PatchError, PatchPoint, Patchbay};

const PI: f32 = 3.141;

/// Basic sine wave oscillator
pub struct Oscillator {
    pub frequency: f32,
    pub amplitude: f32,
    output: PatchPoint,
    value: f32,
}

impl Oscillator {
    pub fn new(output: PatchPoint, frequency: f32) -> Self {
        Oscillator {
            frequency,
            amplitude: 0.1,
            output,
            value: 0.0,
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

        // Calculate with positive values only
        let x = if self.value < 0.0 {
            -self.value * PI
        } else {
            self.value * PI
        };

        // Bashkara approximation of a sine
        let numerator = 16.0 * x * (PI - x);
        let denominator = 5.0 * PI * PI - 4.0 * x * (PI - x);
        let sin = numerator / denominator;

        // Normalize back positive to negative if needed
        let output = if self.value < 0.0 { -sin } else { sin };

        // Update the output value in the patchbay.
        patchbay.set_sample(&mut self.output, output * self.amplitude);

        Ok(())
    }
}
