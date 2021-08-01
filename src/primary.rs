use alloc::vec;
use alloc::vec::Vec;
use alloc::boxed::Box;
use crate::traits::Sample;
use crate::stream::{StreamErr, Point};
use crate::signal::Signal;

/// Primary is the main driver of sound,
/// a primary instance contains a collection of sources
/// that it samples down to a stereo interleaved output
pub struct Primary {
    sources: Vec<Box<dyn Sample>>,
    /// total buffer size per channel,
    /// meaning the total size will be double this value for stereo
    pub buffer_size: usize,
}

impl Primary {
    /// Create new instance using a buffer_size
    pub fn new(buffer_size: usize) -> Self {
        Primary {
            sources: vec![],
            buffer_size,
        }
    }

    /// Add source to sample for final output
    pub fn add_source(&mut self, source: Box<dyn Sample>) -> &mut Self {
	self.sources.push(source);
	self
    }

    /// Sample primary track for audio data.
    /// The data is interleaved, `[left, right, left right]`
    pub fn sample(&mut self) -> Result<Vec<Point>, StreamErr> {
	let mut sources = vec![];

	for source in self.sources.iter_mut() {
	    sources.push(source.sample(self.buffer_size));
	}

	let source_refs: Vec<&Signal> = sources.iter().collect();

	let buffer = Signal::silence(self.buffer_size)
	    .to_stereo()
	    .mix(&source_refs);

	buffer.get_interleaved_points()
    }
}
