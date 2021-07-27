use alloc::string::String;

use crate::stream::{Stream, StreamErr};
use crate::traits::Playable;

/// Play style for a sample
#[derive(Debug)]
pub enum PlayStyle {
    /// One time playback
    OneShot,
    /// Return to start after playback is finished
    Loop,
}

/// Most basic building block for non-generated sound
#[derive(Debug)]
pub struct Sample {
    /// description for the sample
    pub name: String,
    /// audio data for the stream
    pub stream: Stream,
    /// current position of playback
    pub position: usize,
    /// Play style for the sample
    pub play_style: PlayStyle,
    buffer: Stream,
    speed: f64,
}

impl Sample {
    /// Create new sample from a [`Stream`]
    pub fn new(name: String, stream: Stream) -> Self {
        let channels = stream.channels;

        Sample {
            name,
            stream,
            buffer: Stream::empty(0, channels),
            speed: 1.0,
            position: 0,
            play_style: PlayStyle::Loop,
        }
    }

    // Change play_style for the sample
    // pub fn set_play_style(&mut self, play_style: PlayStyle) -> &mut Self {
    // 	self.play_style = play_style;
    // 	self
    // }
}

impl Playable for Sample {
    fn play(&mut self) -> Result<&Stream, StreamErr> {
        let sample_length = self.stream.len();

        for byte in self.buffer.samples.iter_mut() {
            *byte = match self.play_style {
                PlayStyle::Loop => self.stream.get_sample(self.position)?,
                PlayStyle::OneShot => {
                    if self.position >= sample_length {
                        0.0
                    } else {
                        self.stream.get_sample(self.position)?
                    }
                }
            };

            self.position = match self.play_style {
                PlayStyle::Loop => {
                    if self.position >= sample_length - 1 {
                        0
                    } else {
                        self.position + 1
                    }
                }
                PlayStyle::OneShot => {
                    if self.position >= sample_length - 1 {
                        sample_length
                    } else {
                        self.position + 1
                    }
                }
            };
        }

        Ok(&self.buffer)
    }

    fn set_buffer_size(&mut self, buffer_size: usize) -> &mut Self {
        self.buffer
            .samples
            .resize_with(buffer_size, Default::default);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::FromSamples;
    use alloc::vec;

    #[test]
    fn test_play_loop_buffer_smaller_than_sample() {
        let stream = Stream::from_samples(vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8], 1);
        let mut sample = Sample::new(String::from("foo"), stream);
        sample.set_buffer_size(5);

        let buffer = sample.play().unwrap();
        assert_eq!(buffer.samples, vec![0.0, 0.1, 0.2, 0.3, 0.4]);

        let buffer = sample.play().unwrap();
        assert_eq!(buffer.samples, vec![0.5, 0.6, 0.7, 0.8, 0.0]);

        let buffer = sample.play().unwrap();
        assert_eq!(buffer.samples, vec![0.1, 0.2, 0.3, 0.4, 0.5]);
    }

    #[test]
    fn test_play_loop_buffer_larger_than_sample() {
        let stream = Stream::from_samples(vec![0.0, 0.1, 0.2, 0.3, 0.4], 1);
        let mut sample = Sample::new(String::from("foo"), stream);
        sample.set_buffer_size(8);

        let buffer = sample.play().unwrap();
        assert_eq!(buffer.samples, vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.0, 0.1, 0.2]);

        let buffer = sample.play().unwrap();
        assert_eq!(buffer.samples, vec![0.3, 0.4, 0.0, 0.1, 0.2, 0.3, 0.4, 0.0]);
    }

    #[test]
    fn test_play_oneshot() {
        let stream = Stream::from_samples(vec![0.0, 0.1, 0.2, 0.3, 0.4], 1);
        let mut sample = Sample::new(String::from("foo"), stream);
        sample.set_buffer_size(8);
        sample.play_style = PlayStyle::OneShot;

        let buffer = sample.play().unwrap();
        assert_eq!(buffer.samples, vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.0, 0.0, 0.0]);

        let buffer = sample.play().unwrap();
        assert_eq!(buffer.samples, vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    }
}
