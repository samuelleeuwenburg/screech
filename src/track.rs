use crate::signal::Signal;
use crate::stream::Point;
use crate::traits::Source;
use alloc::vec;
use alloc::vec::Vec;
/// Standard track with panning and volume control
#[derive(Debug, PartialEq, Clone)]
pub struct Track {
    id: usize,
    /// Inputs used for the main output of the track
    pub inputs: Vec<usize>,
    /// Gain setting, -1.0 to 1.0 for -114dB and +6dB respectively
    pub gain: Point,
    /// Panning setting, -1.0 to 1.0 for -114dB and +6dB respectively
    /// inverted to each channel
    pub panning: Point,
}

impl Track {
    /// Create new track with a unique `id`
    ///
    /// ```
    /// use screech::track::Track;
    /// use screech::clip::Clip;
    ///
    /// let track = Track::new(0);
    ///
    /// assert_eq!(track.gain, 0.9);
    /// assert_eq!(track.panning, 0.0);
    /// ```
    pub fn new(id: usize) -> Track {
        Track {
            id,
            inputs: vec![],
            gain: 0.9,
            panning: 0.,
        }
    }

    /// add source to the input, supports multiple inputs
    pub fn add_input(&mut self, source: &dyn Source) -> &mut Self {
        self.inputs.push(source.get_id());
	self
    }

    /// remove source from input
    pub fn remove_input(&mut self, source: &dyn Source) -> &mut Self {
	let a = source.get_id();
        self.inputs.retain(|&b| a != b);
	self
    }

    /// Set gain between `1.0` and `-1.0`
    ///
    /// `0.9` is set to unity gain, for `0.1` increment the level increases by 6dB.
    /// For example setting a gain of `1.0` gives +6dB of amplification
    /// and setting a gain of `-1.0` would result in -114dB
    ///
    pub fn gain(mut self, gain: Point) -> Self {
        self.gain = gain;
        self
    }

    /// Set left and right channel panning,
    /// `0.0` is center resulting in no amplification,
    /// `-1.0` is left channel +6dB, right channel -114dB,
    /// `1.0` is left channel -114dB, right channel +6dB
    ///
    pub fn panning(mut self, panning: Point) -> Self {
        self.panning = panning;
        self
    }
}

impl Source for Track {
    fn sample(&mut self, sources: Vec<(usize, &Signal)>, _buffer_size: usize) -> Signal {
        let mut inputs = vec![];

        for (key, signal) in sources {
            if self.inputs.iter().any(|&k| k == key) {
                inputs.push(signal);
            }
        }

        let sources: Vec<Signal> = inputs
            .iter()
            .map(|&signal| {
                signal
		    .clone()
                    .map(|s| s.amplify(cv_to_db(self.gain)))
                    .map_stereo(|left, right| {
                        (
                            left.amplify(panning_to_db(self.panning)),
                            right.amplify(panning_to_db(-self.panning)),
                        )
                    })
            })
            .collect();

        let refs: Vec<&Signal> = sources.iter().collect();

        Signal::mix(&refs)
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn get_sources(&self) -> Vec<&usize> {
        self.inputs.iter().collect()
    }
}

// convert -1.0 .. 1.0 range to -114dB .. +6dB
fn cv_to_db(cv: Point) -> f32 {
    cv * 60. - 54.
}

// non linear conversion of -1.0 .. 1.0 range to -114dB .. +6dB
fn panning_to_db(cv: Point) -> f32 {
    if cv > 0. {
        cv * 6.
    } else {
        cv * 114.
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
