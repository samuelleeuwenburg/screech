use core::cmp;
use crate::signal::Signal;
use crate::traits::Sample;

/// Most basic building block for non-generated sound
#[derive(Debug, PartialEq, Clone)]
pub struct Clip {
    /// audio data for the stream
    pub audio: Signal,
    /// current position of playback
    pub position: usize,
    /// Play style for the sample
    pub play_style: PlayStyle,
    // buffer: Stream,
    speed: f64,
}


/// Play style for a sample
#[derive(Debug, PartialEq, Clone)]
pub enum PlayStyle {
    /// One time playback
    OneShot,
    /// Return to start after playback is finished
    Loop,
}

/// Enum for different error states
pub enum ClipErr {
    /// Failure to generate sample signal
    SampleFailure
}

impl Clip {
    /// Create new clip from a [`Signal`]
    pub fn new(audio: Signal) -> Self {
        Clip {
            audio,
            speed: 1.0,
            position: 0,
            play_style: PlayStyle::Loop,
        }
    }

    fn try_sample(&mut self, buffer_size: usize) -> Result<Signal, ClipErr> {
	let mut signal = Signal::silence(buffer_size).match_channels(&self.audio);
	let clip_length = self.audio.len();

	let audio_signal = match self.play_style {
	    PlayStyle::OneShot => {
		// go no further than the end of the audio signal
		let signal_size = cmp::min(clip_length, self.position + buffer_size);

		self.audio
		    .and_then(|stream| stream.get_slice(self.position, signal_size))
		    .map_err(|_| ClipErr::SampleFailure)?
	    }
	    PlayStyle::Loop => {
		self.audio.map(|stream| stream.get_looped_slice(self.position, buffer_size))
	    }
	};

	// mix audio in signal
	signal.mix(&[&audio_signal]);

	// determine next position based on play style
	self.position = match self.play_style {
	    PlayStyle::OneShot => cmp::min(self.position + buffer_size, clip_length),
	    PlayStyle::Loop => self.position + buffer_size % clip_length,
	};

	Ok(signal)
    }
}

impl Sample for Clip {
    fn sample(&mut self, buffer_size: usize) -> Signal {
	match self.try_sample(buffer_size) {
	    Ok(signal) => signal,
	    Err(_) => Signal::silence(buffer_size),
	}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::{Stream, FromPoints};
    use alloc::vec;

    #[test]
    fn test_play_loop_buffer_smaller_than_sample() {
        let signal = Signal::Mono(Stream::from_points(&[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]));
        let mut clip = Clip::new(signal);
        let buffer_size = 5;

        assert_eq!(
	    clip.sample(buffer_size),
	    Signal::Mono(Stream { points: vec![0.0, 0.1, 0.2, 0.3, 0.4] })
	);

        assert_eq!(
	    clip.sample(buffer_size),
	    Signal::Mono(Stream { points: vec![0.5, 0.6, 0.7, 0.8, 0.0] })
	);

        assert_eq!(
	    clip.sample(buffer_size),
	    Signal::Mono(Stream { points: vec![0.1, 0.2, 0.3, 0.4, 0.5] })
	);
    }

    #[test]
    fn test_play_loop_buffer_larger_than_sample() {
        let signal = Signal::Mono(Stream::from_points(&[0.0, 0.1, 0.2, 0.3, 0.4]));
        let mut clip = Clip::new(signal);
        let buffer_size = 8;

        assert_eq!(
	    clip.sample(buffer_size),
	    Signal::Mono(Stream { points: vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.0, 0.1, 0.2] })
	);
        assert_eq!(
	    clip.sample(buffer_size),
	    Signal::Mono(Stream { points: vec![0.3, 0.4, 0.0, 0.1, 0.2, 0.3, 0.4, 0.0] })
	);
    }

    #[test]
    fn test_play_oneshot() {
        let signal = Signal::Mono(Stream::from_points(&[0.0, 0.1, 0.2, 0.3, 0.4]));
	let buffer_size = 8;
        let mut clip = Clip::new(signal);

        clip.play_style = PlayStyle::OneShot;

        assert_eq!(
	    clip.sample(buffer_size),
	    Signal::Mono(Stream { points: vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.0, 0.0, 0.0] })
	);

	assert_eq!(
	    clip.sample(buffer_size),
	    Signal::Mono(Stream { points: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0] })
	);
    }

}
