use alloc::boxed::Box;
use crate::signal::Signal;

/// To implement [`Sample`] means something can be a source of sound,
/// just like when sampling discrete sources of sound, each component
/// that is a sound source should be able to be "sampled".
///
///
/// So no matter what when sampling a source it should return a
/// [`Signal`] of a requested `buffer_size`, this way each component has
/// to choose what to do in the case of failure, a sound processor
/// can choose to send the audio through unprocessed, or fail to send
/// silence instead
///
///
/// [`crate::clip::Clip`] as an example implements [`Sample`]
pub trait Sample {
    /// move one buffersize forward in discrete time
    /// and return a [`Signal`] instance containing the resulting [`crate::stream::Stream`],
    fn sample(&mut self, buffer_size: usize) -> Signal;
}

/// Something that can source one or multiple signals
///
/// [`crate::track::Track`] as an example implements [`Source`]
pub trait Source {
    /// type to define how to use the source, for example using an `Enum` type allows mulitple
    /// sources to be set and overwritten to "internal destinations"
    type Destination;

    /// Set source to sample from, must implement [`Sample`]
    fn set_source(self, destination: Self::Destination, source: Box<dyn Sample>) -> Self;
}

/// Trait to implement conversion from a slice of sized types to a generic
pub trait FromPoints<T: Sized, U> {
    /// Create new instance based on sequence of points
    fn from_points(points: &[T]) -> U;
}
