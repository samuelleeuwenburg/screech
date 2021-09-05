use crate::signal::Signal;
use crate::traits::Source;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp;

/// Most basic building block for non-generated sound
#[derive(Debug, PartialEq, Clone)]
pub struct Clip {
    id: usize,
    /// audio data for the stream
    pub audio: Signal,
    /// current position of playback
    pub position: usize,
    /// Play style for the sample
    pub play_style: PlayStyle,
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
    SampleFailure,
}

impl Clip {
    /// Create new clip from a [`Signal`]
    pub fn new(id: usize, audio: Signal) -> Self {
        Clip {
            id,
            audio,
            speed: 1.0,
            position: 0,
            play_style: PlayStyle::OneShot,
        }
    }

    fn try_sample(&mut self, buffer_size: usize) -> Result<Signal, ClipErr> {
        let clip_length = self.audio.len();

        let audio_signal = match self.play_style {
            PlayStyle::OneShot => {
                // go no further than the end of the audio signal
                let signal_size = if self.position + buffer_size > clip_length {
                    clip_length - self.position
                } else {
                    buffer_size
                };

                self.audio
                    .clone()
                    .and_then(|stream| stream.slice(self.position, signal_size))
                    .map_err(|_| ClipErr::SampleFailure)?
            }
            PlayStyle::Loop => self
                .audio
                .clone()
                .map(|stream| stream.looped_slice(self.position, buffer_size)),
        };

        let signal = Signal::mix(&[&Signal::silence(buffer_size), &audio_signal]);

        // determine next position based on play style
        self.position = match self.play_style {
            PlayStyle::OneShot => cmp::min(self.position + buffer_size, clip_length),
            PlayStyle::Loop => self.position + buffer_size % clip_length,
        };

        Ok(signal)
    }
}

impl Source for Clip {
    fn sample(&mut self, _sources: Vec<(usize, &Signal)>, buffer_size: usize) -> Signal {
        match self.try_sample(buffer_size) {
            Ok(signal) => signal,
            Err(_) => Signal::silence(buffer_size),
        }
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn get_sources(&self) -> Vec<usize> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::Stream;
    use crate::traits::FromPoints;
    use alloc::vec;

    #[test]
    fn test_play_loop_buffer_smaller_than_sample() {
        let mut clip = Clip::new(
            0,
            Signal::from_points(&[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]),
        );
        clip.play_style = PlayStyle::Loop;
        let buffer_size = 5;

        assert_eq!(
            clip.sample(vec![], buffer_size),
            Signal::Mono(Stream::Points(vec![0.0, 0.1, 0.2, 0.3, 0.4]))
        );

        assert_eq!(
            clip.sample(vec![], buffer_size),
            Signal::Mono(Stream::Points(vec![0.5, 0.6, 0.7, 0.8, 0.0]))
        );

        assert_eq!(
            clip.sample(vec![], buffer_size),
            Signal::Mono(Stream::Points(vec![0.1, 0.2, 0.3, 0.4, 0.5]))
        );
    }

    #[test]
    fn test_play_loop_buffer_larger_than_sample() {
        let mut clip = Clip::new(0, Signal::from_points(&[0.0, 0.1, 0.2, 0.3, 0.4]));
        clip.play_style = PlayStyle::Loop;
        let buffer_size = 8;

        assert_eq!(
            clip.sample(vec![], buffer_size),
            Signal::Mono(Stream::Points(vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.0, 0.1, 0.2]))
        );
        assert_eq!(
            clip.sample(vec![], buffer_size),
            Signal::Mono(Stream::Points(vec![0.3, 0.4, 0.0, 0.1, 0.2, 0.3, 0.4, 0.0]))
        );
    }

    #[test]
    fn test_play_oneshot() {
        let buffer_size = 8;
        let mut clip = Clip::new(0, Signal::from_points(&[0.0, 0.1, 0.2, 0.3, 0.4]));
        clip.play_style = PlayStyle::OneShot;

        assert_eq!(
            clip.sample(vec![], buffer_size),
            Signal::Mono(Stream::Points(vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.0, 0.0, 0.0]))
        );

        assert_eq!(
            clip.sample(vec![], buffer_size),
            Signal::Mono(Stream::Points(vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]))
        );
    }
}
