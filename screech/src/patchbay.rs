use crate::sample::Sample;

#[derive(Debug, PartialEq)]
pub enum PatchError {
    MissingSample,
}

#[derive(Copy, Clone)]
pub struct PatchPointOutput {
    id: usize,
}

pub struct PatchPoint {
    id: usize,
}

impl PatchPoint {
    pub fn new(id: usize) -> Self {
        PatchPoint { id }
    }

    pub fn output(&self) -> PatchPointOutput {
        PatchPointOutput { id: self.id }
    }
}

pub struct Patchbay<const POINTS: usize> {
    pub buffer: [Sample; POINTS],
    marks: [bool; POINTS],
    index: usize,
}

impl<const POINTS: usize> Patchbay<POINTS> {
    pub fn new() -> Self {
        Patchbay {
            buffer: [0.0; POINTS],
            marks: [false; POINTS],
            index: 0,
        }
    }

    // @TODO: rename to `get_patch_point`?
    pub fn get_point(&mut self) -> PatchPoint {
        let input = PatchPoint::new(self.index);
        self.index += 1;

        input
    }

    pub fn get_sample(&self, point: PatchPointOutput) -> Result<Sample, PatchError> {
        if self.marks[point.id] {
            Ok(self.buffer[point.id])
        } else {
            Err(PatchError::MissingSample)
        }
    }

    pub fn set_sample(&mut self, point: &mut PatchPoint, sample: Sample) {
        self.buffer[point.id] = sample;
        self.marks[point.id] = true;
    }

    pub fn set_marks(&mut self) {
        for m in self.marks.iter_mut() {
            *m = false;
        }
    }

    pub fn clear_marks(&mut self) {
        for m in self.marks.iter_mut() {
            *m = true;
        }
    }
}
