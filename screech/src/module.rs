use crate::Patchbay;

/// Reads and/or writes signals to a [`Patchbay`] instance.
///
/// Conceptually a module can be thought of as any black box within a sound system that
/// (optionally) reads one or more input signals and outputs one or more signals.
///
/// The [`Module::process`] method gets called periodically by a [`crate::processor::Processor`]
/// to advance all modules one sample at a time in the correct order.
///
/// An example oscillator module that outputs a saw waveshape at a certain frequency might look
/// something like this:
///
/// ```
/// use screech::{Module, Patchbay, PatchPoint};
///
/// struct Oscillator {
///     value: f32,
///     frequency: f32,
///     output: PatchPoint,
/// }
///
/// impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Oscillator {
///     fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) {
///         // Step each frame based on the sample rate and desired frequency.
///         self.value += (2.0 / SAMPLE_RATE as f32) * self.frequency;
///
///         // Wrap around.
///         if self.value >= 1.0 {
///             self.value -= 2.0;
///         }
///
///         // Set the output in the patchbay instance.
///         patchbay.set(&mut self.output, self.value);
///     }
/// }
/// ```
pub trait Module<const SAMPLE_RATE: usize> {
    /// Tell the [`crate::Processor`] the module is ready to be processed.
    ///
    /// Use this method to check if all [`crate::Signal`] values that are required have been set
    /// using the [`Patchbay::check`] method.
    fn is_ready<const P: usize>(&self, _patchbay: &Patchbay<P>) -> bool {
        true
    }

    /// Process the module changing internal state and setting outputs in the [`Patchbay`]
    /// using the [`Patchbay::set`] method.
    fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>);
}
