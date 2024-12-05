use crate::{Module, PatchPoint, Patchbay, Signal};

const INPUTS: usize = 16;

/// 16 channel summing mixer
pub struct Mix {
    output: PatchPoint,
    inputs: [Signal; INPUTS],
}

impl Mix {
    pub fn new(output: PatchPoint) -> Self {
        Mix {
            output,
            inputs: [Signal::None; INPUTS],
        }
    }

    pub fn output(&self) -> Signal {
        self.output.signal()
    }

    pub fn add_input(&mut self, input: Signal, index: usize) {
        self.inputs[index] = input;
    }
}

impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Mix {
    fn is_ready<const P: usize>(&self, patchbay: &Patchbay<P>) -> bool {
        self.inputs.iter().all(|p| patchbay.check(*p))
    }

    fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) {
        let mut sum = 0.0;

        for input in self.inputs {
            sum += patchbay.get(input);
        }

        patchbay.set(&mut self.output, sum);
    }
}
