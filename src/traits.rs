use crate::core::Signal;
use alloc::vec::Vec;

/// To implement [`Source`] means something can be a source of sound,
/// just like when sampling continous sources of sound into discrete slices.
/// Each component that is a sound source should be able to be "sampled".
///
/// So no matter what when sampling a source it should return a
/// [`Signal`], this way each component has
/// to choose what to do in the case of failure, a sound processor
/// can choose to send the audio through unprocessed, or fail and send
/// silence instead
///
/// [`crate::basic::Clip`] as an example implements [`Source`]
/// [`crate::basic::Track`] as an example implements [`Source`]
/// [`crate::basic::Oscillator`] as an example implements [`Source`]
pub trait Source {
    /// move one buffersize forward in discrete time
    fn sample(&mut self, sources: &mut dyn Tracker, sample_rate: usize);

    /// get id for instance, this is to identify this source when building the output
    fn get_id(&self) -> usize;

    /// Get a list of sources
    fn get_sources(&self) -> Vec<usize>;
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
    /// Return a unique ID
    fn create_id(&mut self) -> usize;

    /// clear id for reuse
    fn clear_id(&mut self, id: usize);

    /// get signal for id
    fn get_signal(&self, id: usize) -> Option<&Signal>;

    /// set signal for id
    fn set_signal(&mut self, id: usize, signal: Signal);
}

/// Trait to implement conversion from a slice of sized types to a generic
pub trait FromPoints<T: Sized, U> {
    /// Create new instance based on sequence of points
    fn from_points(points: Vec<T>) -> U;
}
