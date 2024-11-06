use crate::module::Module;
use crate::patchbay::{PatchError, Patchbay};

#[derive(Copy, Clone, PartialEq)]
pub struct Dummy;

impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Dummy {
    fn process<const P: usize>(&mut self, _patchbay: &mut Patchbay<P>) -> Result<(), PatchError> {
        Ok(())
    }
}
