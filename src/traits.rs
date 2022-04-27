use crate::{Input, Output, Signal};
use alloc::vec::Vec;

/// To implement [`Source`] means something can be a source of sound/voltage,
///
/// Two things are required by the trait:
///
/// 1) keeping track of what the source is using an unique `usize` id
/// 2) using the sample method to fill one or multiple [`Signal`] buffers with f32 data.
///
/// A very basic example might be to implement a DC offset module
///
/// ```
/// use screech::traits::{Tracker, Source};
/// use screech::{Screech, Input, Output};
///
/// struct Offset {
///     id: usize,
///     input: Input,
///     output: Output,
///     offset: f32,
/// }
///
/// impl Offset {
///     fn new(screech: &mut Screech, offset: f32) -> Self {
///         // obtain a unique identifier from the main sreech instance
///         let id = screech.create_source_id();
///
///         Offset {
///             id,
///             // initialize a new input to keep track of input connected to our source
///             input: screech.init_input(&id, "signal_in"),
///             // initialize a new output [`Signal`] buffer for our id and name
///             output: screech.init_output(&id, "signal_out"),
///             offset,
///         }
///     }
/// }
///
/// impl Source for Offset {
///     fn sample(&mut self, tracker: &mut dyn Tracker, sample_rate: usize) {
///         for i in 0..*tracker.get_buffer_size() {
///             // set offset as initial value
///             let mut signal = self.offset;
///
///             // add all inputs to the signal
///             for input in tracker.get_input(&self.input).unwrap().into_iter() {
///                 if let Some(s) = tracker.get_output(&input).and_then(|o| o.samples.get(i)) {
///                     signal += s;
///                 }
///             }
///
///             // add signal to the final output
///             let output = tracker.get_mut_output(&self.output).unwrap();
///             output.samples[i] = signal;
///         }
///     }
///
///     fn get_source_id(&self) -> &usize {
///         &self.id
///     }
/// }
/// ```
pub trait Source: Send {
    /// function that gets called by [`crate::Screech`] during sampling.
    ///
    /// use the reference to the tracker to update relevant [`Signal`]s
    fn sample(&mut self, tracker: &mut dyn Tracker, sample_rate: usize);

    /// get reference to the id for the source,
    /// this is used to uniquely identify this source when sampling [`Signal`]s
    fn get_source_id(&self) -> &usize;
}

/// Tracker trait to keep track of buffers and connections between [`Output`]s and [`Input`]s
/// for implementations see [`crate::BasicTracker`] or [`crate::DynamicTracker`]
pub trait Tracker {
    /// return the buffer size
    fn get_buffer_size(&self) -> &usize;

    /// resize internal buffers
    fn resize_buffers(&mut self, buffer_size: usize);

    /// Return a unique ID for keeping track of [`Source`]es
    fn create_source_id(&mut self) -> usize;

    /// clear source id and associated buffers
    fn clear_source(&mut self, id: usize);

    /// get all source ids required for a given source id
    fn get_sources(&self, id: &usize) -> Vec<usize>;

    /// get a reference to an output's [`Signal`]
    fn get_output(&self, output: &Output) -> Option<&Signal>;

    /// get a mutable reference to an output's [`Signal`]
    fn get_mut_output(&mut self, output: &Output) -> Option<&mut Signal>;

    /// initialize empty [`Signal`] for output
    fn init_output(&mut self, output: &Output);

    /// initialize input for tracking outputs connected to it
    fn init_input(&mut self, input: &Input);

    /// return a reference to a list of outputs for a given input
    fn get_input(&self, e: &Input) -> Option<&Vec<Output>>;

    /// connect an [`Output`] to an [`Input`]
    fn connect_signal(&mut self, output: &Output, input: &Input);

    /// clear [`Output`] connection from an [`Input`]
    fn clear_connection(&mut self, output: &Output, input: &Input);
}

/// Trait to implement conversion from a slice of sized types to a generic
pub trait FromPoints<T: Sized, U> {
    /// Create new instance based on sequence of points
    fn from_points(points: Vec<T>) -> U;
}
