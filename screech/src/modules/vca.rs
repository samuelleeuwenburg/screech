use crate::{Module, PatchPoint, Patchbay, Signal};

/// VCA module that takes two inputs (signal and modulator) and has a single output.
pub struct Vca {
    modulator: Signal,
    input: Signal,
    output: PatchPoint,
}

impl Vca {
    pub fn new(output: PatchPoint) -> Self {
        Vca {
            modulator: Signal::None,
            input: Signal::None,
            output,
        }
    }

    pub fn output(&self) -> Signal {
        self.output.signal()
    }

    pub fn set_input(&mut self, signal: Signal) -> &mut Self {
        self.input = signal;
        self
    }

    pub fn set_modulator(&mut self, signal: Signal) -> &mut Self {
        self.modulator = signal;
        self
    }
}

impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Vca {
    fn is_ready<const POINTS: usize>(&self, patchbay: &Patchbay<POINTS>) -> bool {
        patchbay.check(self.input) && patchbay.check(self.modulator)
    }

    fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) {
        // Take the input signal and multiply it by the modulator input.
        patchbay.set(
            &mut self.output,
            patchbay.get(self.input) * patchbay.get(self.modulator),
        );
    }
}
