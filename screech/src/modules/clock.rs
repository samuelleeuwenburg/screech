use crate::{Module, PatchPoint, Patchbay, Signal};

/// Pulse generator, BPM based
pub struct Clock {
    output: PatchPoint,
    bpm: f32,
    value: f32,
}

impl Clock {
    pub fn new(output: PatchPoint, bpm: f32) -> Self {
        Clock {
            output,
            bpm,
            value: 0.0,
        }
    }

    pub fn output(&self) -> Signal {
        self.output.signal()
    }
}

impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Clock {
    fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) {
        self.value += (1.0 / SAMPLE_RATE as f32) * (self.bpm / 60.0);

        if self.value >= 2.0 {
            self.value -= 2.0;
        }

        let output = if self.value > 1.0 { 0.0 } else { 1.0 };

        patchbay.set(&mut self.output, output);
    }
}
