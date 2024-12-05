use crate::Signal;

/// Virtual patchbay holding sample values.
///
/// ```
/// use screech::Patchbay;
///
/// let mut patchbay: Patchbay<128> = Patchbay::new();
///
/// let mut point = patchbay.point().unwrap();
/// assert_eq!(patchbay.get(point.signal()), 0.0);
///
/// patchbay.set(&mut point, 1.0);
/// assert_eq!(patchbay.get(point.signal()), 1.0);
/// ```
#[derive(Debug)]
pub struct Patchbay<const PATCHPOINTS: usize> {
    buffer: [f32; PATCHPOINTS],
    marks: [bool; PATCHPOINTS],
}

impl<const PATCHPOINTS: usize> Patchbay<PATCHPOINTS> {
    pub fn new() -> Self {
        Patchbay {
            buffer: [0.0; PATCHPOINTS],
            marks: [false; PATCHPOINTS],
        }
    }

    /// Get a free [`PatchPoint`], returns `None` if all available points are taken.
    pub fn point(&mut self) -> Option<PatchPoint> {
        for i in 0..PATCHPOINTS {
            if !self.marks[i] {
                self.marks[i] = true;
                return Some(PatchPoint::new(i));
            }
        }

        None
    }

    /// Get the sample value of a signal.
    pub fn get(&self, signal: Signal) -> f32 {
        match signal {
            Signal::PatchPoint(id) => self.buffer[id],
            Signal::Fixed(s) => s,
            Signal::None => 0.0,
        }
    }

    /// Set the sample value of a patchpoint using the exclusive ownership.
    pub fn set(&mut self, point: &mut PatchPoint, sample: f32) {
        self.buffer[point.id] = sample;
        self.marks[point.id] = true;
    }

    /// Check if a patchpoint sample value is up to date.
    pub fn check(&self, signal: Signal) -> bool {
        match signal {
            Signal::PatchPoint(id) => self.marks[id],
            Signal::Fixed(_) => true,
            Signal::None => true,
        }
    }

    pub fn clear_marks(&mut self) {
        for m in self.marks.iter_mut() {
            *m = false;
        }
    }
}

pub struct PatchPoint {
    id: usize,
}

impl PatchPoint {
    pub(crate) fn new(id: usize) -> Self {
        PatchPoint { id }
    }

    pub fn signal(&self) -> Signal {
        Signal::PatchPoint(self.id)
    }
}
