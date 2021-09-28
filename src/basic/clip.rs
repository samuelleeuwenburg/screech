use crate::core::{Signal, Stream};
use crate::traits::{Source, Tracker};
use alloc::vec;
use alloc::vec::Vec;

/// Most basic building block for non-generated sound
#[derive(Debug, PartialEq, Clone)]
pub struct Clip {
    id: usize,
    /// audio data for the stream
    pub audio: Stream,
    /// current position of playback
    position: usize,
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

impl Clip {
    /// Create new clip from a [`Signal`]
    pub fn new(tracker: &mut dyn Tracker, audio: Stream) -> Self {
        Clip {
            id: tracker.create_id(),
            audio,
            speed: 1.0,
            position: 0,
            play_style: PlayStyle::OneShot,
        }
    }

    /// Render the next real time signal
    pub fn step(&mut self) -> Signal {
        let audio_length = self.audio.len();

        let signal = if self.position >= audio_length {
            Signal::silence()
        } else {
            let point = self.audio.get_point(self.position).unwrap();
            Signal::point(*point)
        };

        self.position = if self.position >= audio_length - 1 {
            match self.play_style {
                PlayStyle::OneShot => audio_length,
                PlayStyle::Loop => 0,
            }
        } else {
            self.position + 1
        };

        signal
    }
}

impl Source for Clip {
    fn sample(&mut self, sources: &mut dyn Tracker, _sample_rate: usize) {
        sources.set_signal(self.id, self.step());
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
    use crate::core::{Primary, Stream};
    use crate::traits::FromPoints;
    use alloc::vec;

    #[test]
    fn test_play_loop_buffer_smaller_than_sample() {
        let mut primary = Primary::<5>::new(48_000);
        let mut clip = Clip::new(
            &mut primary,
            Stream::from_points(vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]),
        );
        clip.play_style = PlayStyle::Loop;
        primary.add_monitor(&clip).output_mono();

        assert_eq!(
            primary.sample(vec![&mut clip]).unwrap(),
            vec![0.0, 0.1, 0.2, 0.3, 0.4]
        );

        assert_eq!(
            primary.sample(vec![&mut clip]).unwrap(),
            vec![0.5, 0.6, 0.7, 0.8, 0.0]
        );

        assert_eq!(
            primary.sample(vec![&mut clip]).unwrap(),
            vec![0.1, 0.2, 0.3, 0.4, 0.5]
        );
    }

    #[test]
    fn test_play_loop_buffer_larger_than_sample() {
        let mut primary = Primary::<8>::new(48_000);
        let mut clip = Clip::new(
            &mut primary,
            Stream::from_points(vec![0.0, 0.1, 0.2, 0.3, 0.4]),
        );
        clip.play_style = PlayStyle::Loop;
        primary.add_monitor(&clip).output_mono();

        assert_eq!(
            primary.sample(vec![&mut clip]).unwrap(),
            vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.0, 0.1, 0.2]
        );
        assert_eq!(
            primary.sample(vec![&mut clip]).unwrap(),
            vec![0.3, 0.4, 0.0, 0.1, 0.2, 0.3, 0.4, 0.0]
        );
    }

    #[test]
    fn test_play_oneshot() {
        let mut primary = Primary::<8>::new(48_000);
        let mut clip = Clip::new(
            &mut primary,
            Stream::from_points(vec![0.0, 0.1, 0.2, 0.3, 0.4]),
        );
        clip.play_style = PlayStyle::OneShot;
        primary.add_monitor(&clip).output_mono();

        assert_eq!(
            primary.sample(vec![&mut clip]).unwrap(),
            vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.0, 0.0, 0.0]
        );

        assert_eq!(
            primary.sample(vec![&mut clip]).unwrap(),
            vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
        );
    }
}
