use crate::stream::{Stream, StreamErr};

/// To implement Playable means something can be a source of sound,
/// A [`Sample`] for example is playable.
pub trait Playable {
    /// move one buffersize forward in discrete time
    /// and return a pointer to the resulting stream,
    fn play(&mut self) -> Result<&Stream, StreamErr>;

    /// change buffer_size
    fn set_buffer_size(&mut self, buffer_size: usize) -> &mut Self;
}
