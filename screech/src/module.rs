use crate::patchbay::{PatchError, Patchbay};

/// Reads and/or writes signals to a [`Patchbay`] instance
///
/// Conceptually a module can be thought of as any black box within a sound system that
/// (optionally) reads one or more input signals and outputs one or more signals.
///
/// it requires a single method ([`Module::process`]) that accepts a [`Patchbay`]
/// (for reading & writing signals to & from) and can potentially error
/// (for example if a required signal from the patchbay has not yet been written to).
/// This method gets called periodically by a [`crate::processor::Processor`]
/// to advance all modules one sample at a time in the correct order.
///
/// For example an oscillator that outputs a saw waveshape at a certain frequency might look
/// something like this:
///
/// ```
/// use screech::module::Module;
/// use screech::patchbay::{PatchError, Patchbay, PatchPoint};
/// use screech::sample::Sample;
///
/// struct Oscillator {
///     value: Sample,
///     frequency: f32,
///     output: PatchPoint,
/// }
///
/// impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Oscillator {
///     fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) -> Result<(), PatchError> {
///         self.value += (2.0 / SAMPLE_RATE as f32) * self.frequency;
///
///         if self.value >= 1.0 {
///             self.value -= 2.0;
///         }
///
///         patchbay.set_sample(&mut self.output, self.value);
///
///         Ok(())
///     }
/// }
/// ```
pub trait Module<const SAMPLE_RATE: usize> {
    fn process<const POINTS: usize>(
        &mut self,
        patchbay: &mut Patchbay<POINTS>,
    ) -> Result<(), PatchError>;
}
