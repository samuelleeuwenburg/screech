use crate::signal::Signal;
use alloc::vec::Vec;

/// To implement [`Source`] means something can be a source of sound,
/// just like when sampling discrete sources of sound, each component
/// that is a sound source should be able to be "sampled".
///
///
/// So no matter what when sampling a source it should return a
/// [`Signal`] of a requested `buffer_size`, this way each component has
/// to choose what to do in the case of failure, a sound processor
/// can choose to send the audio through unprocessed, or fail and send
/// silence instead
///
///
/// [`crate::clip::Clip`] as an example implements [`Source`]
/// [`crate::track::Track`] as an example implements [`Source`]
pub trait Source {
    /// type to define how to use the source, for example using an `Enum` type allows multiple
    /// sources to be set and overwritten to "internal destinations"

    /// move one buffersize forward in discrete time
    fn sample(&mut self, sources: Vec<(usize, &Signal)>, buffer_size: usize) -> Signal;

    /// get id for instance, this is to identify this source when building the output
    fn get_id(&self) -> usize;

    /// Get a list of sources
    fn get_sources(&self) -> Vec<&usize>;
}

/// Trait to implement conversion from a slice of sized types to a generic
pub trait FromPoints<T: Sized, U> {
    /// Create new instance based on sequence of points
    fn from_points(points: &[T]) -> U;
}
