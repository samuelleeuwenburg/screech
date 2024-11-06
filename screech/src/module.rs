use crate::patchbay::{PatchError, Patchbay};
use crate::processor::Processor;

/// Reads and/or writes signals to a [`Patchbay`] instance
///
/// Conceptually a module can be thought of as any black box within a sound system that
/// (optionally) reads one or more input signals and outputs one or more signals.
///
/// it requires a single method ([`Module::process`]) that accepts a [`Patchbay`]
/// (for reading & writing signals to & from) and can potentially error
/// (for example if a required signal from the patchbay has not yet been written to).
/// This method gets called periodically by a [`Processor`]
/// to advance all modules one sample at a time in the correct order.
///
/// For example an oscillator that outputs a saw waveshape at a certain frequency might look
/// something like this:
///
/// ```
/// use screech::module::Module;
/// use screech::patchbay::{PatchError, Patchbay};
/// use screech::sample::Sample;
///
/// const SAMPLE_RATE: f32 = 48000.0;
///
/// struct Oscillator {
///     value: Sample,
///     frequency: f32,
///     output: PatchPoint,
/// }
///
/// impl Module for Oscillator {
///     fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) -> Result<(), PatchError> {
///         self.value += (2.0 / SAMPLE_RATE) * self.frequency;
///
///         if self.value >= 1.0 {
///             self.value -= 2.0;
///         }
///
///         patchbay.set_sample(&mut self.output, output);
///
///         Ok(())
///     }
/// }
/// ```
pub trait Module {
    fn process<const POINTS: usize>(
        &mut self,
        patchbay: &mut Patchbay<POINTS>,
    ) -> Result<(), PatchError>;
}
