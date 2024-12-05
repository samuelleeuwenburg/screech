use crate::{Module, Patchbay};

/// Placeholder module.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Dummy;

impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Dummy {
    fn process<const P: usize>(&mut self, _patchbay: &mut Patchbay<P>) {}
}
