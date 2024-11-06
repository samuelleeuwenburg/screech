use crate::module::Module;
use crate::patchbay::{PatchError, PatchPoint, PatchPointOutput, Patchbay};

/// VCA module that takes two inputs (signal and modulator) and has a single output.
pub struct Vca {
    modulator: PatchPointOutput,
    input: PatchPointOutput,
    output: PatchPoint,
}

impl Vca {
    pub fn new(modulator: PatchPointOutput, input: PatchPointOutput, output: PatchPoint) -> Self {
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
