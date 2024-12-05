use crate::{Module, PatchPoint, Patchbay, Signal};

enum Curve {
    AR(f32, f32),
    ADSR(f32, f32, f32, f32),
}

pub struct Envelope {
    output: PatchPoint,
    trigger: Signal,
    previous_trigger: f32,
    value: f32,
    curve: Curve,
    is_active: bool,
    active_stage: usize,
}

impl Envelope {
    pub fn new(trigger: Signal, output: PatchPoint) -> Self {
        Envelope {
            output,
            trigger,
            previous_trigger: 0.0,
            value: 0.0,
            curve: Curve::AR(0.1, 0.1),
            is_active: false,
            active_stage: 0,
        }
    }

    pub fn output(&self) -> Signal {
        self.output.signal()
    }

    pub fn set_ar(&mut self, a: f32, r: f32) -> &mut Self {
        self.curve = Curve::AR(a, r);
        self
    }

    pub fn set_adsr(&mut self, a: f32, d: f32, s: f32, r: f32) -> &mut Self {
        self.curve = Curve::ADSR(a, d, s, r);
        self
    }

    pub fn process_curve<const SAMPLE_RATE: usize>(&mut self) {
        let seconds_per_sample = 1.0 / SAMPLE_RATE as f32;

        match self.curve {
            Curve::AR(a, r) => match self.active_stage {
                0 => {
                    self.value += seconds_per_sample / a;

                    if self.value >= 1.0 {
                        self.active_stage += 1;
                    }
                }
                1 => {
                    self.value -= seconds_per_sample / r;
                    if self.value <= 0.0 {
                        self.active_stage += 1;
                    }
                }
                _ => self.is_active = false,
            },
            Curve::ADSR(a, d, s, r) => match self.active_stage {
                0 => {
                    self.value += self.value * a;
                    if self.value >= 1.0 {
                        self.active_stage += 1;
                    }
                }
                1 => {
                    self.value -= self.value * d;
                    if self.value <= s {
                        self.active_stage += 1;
                    }
                }
                2 => {
                    // @TODO:
                    self.active_stage += 1;
                }
                3 => {
                    self.value -= self.value * r;
                    if self.value <= 0.0 {
                        self.active_stage += 1;
                    }
                }
                _ => self.is_active = false,
            },
        }
    }
}

impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Envelope {
    fn is_ready<const P: usize>(&self, patchbay: &Patchbay<P>) -> bool {
        patchbay.check(self.trigger)
    }

    fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) {
        let trigger = patchbay.get(self.trigger);
        let triggered = trigger >= 0.5 && self.previous_trigger < 0.5;

        let output = match (self.is_active, triggered) {
            // Active, but retriggered -> restart envelope
            (true, true) => {
                self.active_stage = 0;
                self.process_curve::<P>();
                self.value
            }
            // Inactive, triggered -> start envelope
            (false, true) => {
                // Trigger is in the active region -> activate
                self.is_active = true;
                self.active_stage = 0;
                0.0
            }
            // Active, no trigger -> Continue processing the envelope curve
            (true, false) => {
                self.process_curve::<P>();
                self.value
            }
            // Inactive, no trigger -> no output
            (false, false) => 0.0,
        };

        patchbay.set(&mut self.output, output);

        self.previous_trigger = trigger;
    }
}
