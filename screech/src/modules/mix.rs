use crate::module::Module;
use crate::patchbay::{PatchError, PatchPoint, PatchPointOutput, Patchbay};

const INPUTS: usize = 128;

pub struct Mix {
    output: PatchPoint,
    inputs: [Option<PatchPointOutput>; INPUTS],
}

impl Mix {
    pub fn new(output: PatchPoint) -> Self {
        Mix {
            output,
            inputs: [None; INPUTS],
        }
    }

    pub fn add_input(&mut self, input: PatchPointOutput, index: usize) {
        self.inputs[index] = Some(input);
    }
}

impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Mix {
    fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) -> Result<(), PatchError> {
        let mut sum = 0.0;

        for input in self.inputs {
            if let Some(i) = input {
                sum += patchbay.get_sample(i)?;
            }
        }

        patchbay.set_sample(&mut self.output, sum);

        Ok(())
    }
}
