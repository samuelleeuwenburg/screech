use crate::{Input, Output, Signal};
use alloc::vec::Vec;

/// To implement [`Source`] means something can be a source of sound,
/// just like when sampling continous sources of sound into discrete slices.
/// Each component that is a sound source should be able to be "sampled".
///
pub trait Source {
    /// move one buffersize forward in discrete time
    fn sample(&mut self, sources: &mut dyn Tracker, sample_rate: usize);

    /// get id for source, this is to identify this source when building the output
    fn get_source_id(&self) -> &usize;
}

/// Tracker trait to provide [`Source`]-es with unique IDs
///
/// Out of the box [`crate::core::Primary`] implements this trait for you,
/// but you can also roll your own.
///
/// structs like [`crate::basic::Track`] and [`crate::basic::Clip`]
/// take a `&mut dyn Tracker` as their first argument
/// during construction to generate a unique ID
pub trait Tracker {
    /// return the buffer size
    fn get_buffer_size(&self) -> &usize;

    /// resize internal buffers
    fn resize_buffers(&mut self, buffer_size: usize);

    /// Return a unique ID
    fn create_source_id(&mut self) -> usize;

    /// clear source id for reuse
    fn clear_source(&mut self, id: usize);

    /// get all source ids used by inputs for a given source id
    fn get_sources(&self, id: &usize) -> Vec<usize>;

    /// get a reference to output [`Signal`]
    fn get_output(&self, output: &Output) -> Option<&Signal>;

    /// get a mutable reference to output [`Signal`]
    fn get_mut_output(&mut self, output: &Output) -> Option<&mut Signal>;

    /// inits empty [`Signal`] buffer for signal
    fn init_output(&mut self, output: &Output);

    /// inits empty input tracking for signal
    fn init_input(&mut self, input: &Input);

    /// return reference to a list of outputs for a given input
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
