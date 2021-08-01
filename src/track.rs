use alloc::boxed::Box;
use crate::traits::{Sample, Source};
use crate::stream::Point;
use crate::signal::Signal;

/// Standard track with panning and volume control
pub struct Track {
    /// Main input to used for the output of the track
    pub main_input: Option<Box<dyn Sample>>,
    /// Gain setting, -1.0 to 1.0 for -114dB and +6dB respectively
    pub gain: Point,
    /// Panning setting, -1.0 to 1.0 for -114dB and +6dB respectively
    /// inverted to each channel
    pub panning: Point,
}

impl Track {
    /// Create new track with no source attached to the `Destination::In`
    /// ```
    /// use screech::track::Track;
    /// use screech::clip::Clip;
    ///
    /// let track = Track::new();
    ///
    /// assert_eq!(track.gain, 0.9);
    /// assert_eq!(track.panning, 0.0);
    /// ```
    pub fn new() -> Track {
        Track { main_input: None, gain: 0.9, panning: 0. }
    }

    /// Set gain between `1.0` and `-1.0`
    ///
    /// `0.9` is set to unity gain, for `0.1` increment the level increases by 6dB.
    /// For example setting a gain of `1.0` gives +6dB of amplification
    /// and setting a gain of `-1.0` would result in -114dB
    /// ```
    /// use screech::traits::{Sample, Source, FromPoints};
    /// use screech::stream::Stream;
    /// use screech::signal::Signal;
    /// use screech::clip::Clip;
    /// use screech::track::{Track, Destination};
    ///
    /// let buffer_size = 2;
    /// let clip = Box::new(Clip::from_points(&[0.1, 0.2]));
    ///
    /// let mut track = Track::new()
    ///     .set_source(Destination::In, clip)
    ///     .gain(1.0);
    ///
    /// assert_eq!(
    ///     track.sample(buffer_size),
    ///     Signal::Stereo(
    ///         Stream { points: vec![0.19952624, 0.39905247]  },
    ///         Stream { points: vec![0.19952624, 0.39905247]  },
    ///     )
    /// )
    /// ```
    pub fn gain(mut self, gain: Point) -> Self {
	self.gain = gain;
	self
    }

    /// Set left and right channel panning,
    /// `0.0` is center resulting in no amplification,
    /// `-1.0` is left channel +6dB, right channel -114dB,
    /// `1.0` is left channel -114dB, right channel +6dB
    /// ```
    /// use screech::traits::{Sample, Source, FromPoints};
    /// use screech::stream::Stream;
    /// use screech::signal::Signal;
    /// use screech::clip::Clip;
    /// use screech::track::{Track, Destination};
    ///
    /// let buffer_size = 2;
    /// let clip = Box::new(Clip::from_points(&[0.1, 0.2]));
    ///
    /// let mut track = Track::new()
    ///     .set_source(Destination::In, clip)
    ///     .panning(0.5);
    ///
    /// assert_eq!(
    ///     track.sample(buffer_size),
    ///     Signal::Stereo(
    ///         Stream { points: vec![0.14125375, 0.2825075]  },
    ///         Stream { points: vec![0.0001412538, 0.0002825076] },
    ///     )
    /// )
    /// ```
    pub fn panning(mut self, panning: Point) -> Self {
	self.panning = panning;
	self
    }
}

/// Enum for [`Track`] input destinations
pub enum Destination {
    /// Main input, will be converted to [`Signal::Stereo`] for panning
    In,
}

impl Source for Track {
    type Destination = Destination;

    fn set_source(mut self, destination: Self::Destination, source: Box<dyn Sample>) -> Self {
        match destination {
            Destination::In => self.main_input = Some(source),
        }

	self
    }
}

// convert -1.0 .. 1.0 range to -114dB .. +6dB
fn cv_to_db(cv: Point) -> f32 {
    cv * 60. - 54.
}

// non linear conversion of -1.0 .. 1.0 range to -114dB .. +6dB
fn panning_to_db(cv: Point) -> f32 {
    if cv > 0. { cv * 6. } else { cv * 114. }
}

impl Sample for Track {
    fn sample(&mut self, buffer_size: usize) -> Signal {
        match self.main_input.as_mut() {
            Some(s) => {
		s.sample(buffer_size)
		    .map(|s| s.amplify(cv_to_db(self.gain)))
		    .map_stereo(|left, right| (
			left.amplify(panning_to_db(self.panning)),
			right.amplify(panning_to_db(-self.panning)),
		    ))
	    }
            None => Signal::silence(buffer_size),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cv_to_db() {
	assert_eq!(cv_to_db(1.0), 6.0);
	assert_eq!(cv_to_db(0.9), 0.0);
	assert_eq!(cv_to_db(-1.0), -114.0);
    }

    #[test]
    fn test_panning_to_db() {
	assert_eq!(panning_to_db(1.0), 6.0);
	assert_eq!(panning_to_db(0.5), 3.0);
	assert_eq!(panning_to_db(0.0), 0.0);
	assert_eq!(panning_to_db(-0.5), -57.0);
	assert_eq!(panning_to_db(-1.0), -114.0);
    }
}
