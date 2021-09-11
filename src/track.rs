use crate::signal::Signal;
use crate::stream::Point;
use crate::traits::{Tracker, Source};
use crate::mod_source::ModSource;
use alloc::vec;
use alloc::vec::Vec;
use hashbrown::HashMap;

/// Standard track with panning and volume control
#[derive(Debug, PartialEq, Clone)]
pub struct Track {
    id: usize,
    /// Inputs used for the main output of the track
    inputs: Vec<usize>,
    /// Gain setting, -1.0 to 1.0 for -114dB and +6dB respectively
    gain: ModSource,
    /// Panning setting, -1.0 to 1.0 for -114dB and +6dB respectively
    /// inverted to each channel
    panning: ModSource,
}

impl Track {
    /// Create a new track with a unique `id`
    pub fn new(tracker: &mut dyn Tracker) -> Track {
        Track {
            id: tracker.create_id(),
            inputs: vec![],
            gain: ModSource::Owned(Signal::fixed(0.9)),
            panning: ModSource::Owned(Signal::fixed(0.0)),
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

    /// Set gain between `1.0` and `-1.0` by signal source
    ///
    /// `0.9` is set to unity gain, for `0.1` increment the level increases by 6dB.
    /// For example setting a gain of `1.0` gives +6dB of amplification
    /// and setting a gain of `-1.0` would result in -114dB
    ///
    pub fn set_gain(&mut self, cv: Signal) -> &mut Self {
        self.gain = ModSource::Owned(cv);
        self
    }

    /// Set gain between `1.0` and `-1.0` by external source
    ///
    /// `0.9` is set to unity gain, for `0.1` increment the level increases by 6dB.
    /// For example setting a gain of `1.0` gives +6dB of amplification
    /// and setting a gain of `-1.0` would result in -114dB
    ///
    pub fn set_external_gain(&mut self, source: &dyn Source) -> &mut Self {
        self.gain = ModSource::External(source.get_id());
        self
    }

    /// Set left and right channel panning by signal source
    ///
    /// `0.0` is center resulting in no amplification,
    /// `-1.0` is left channel +6dB, right channel -114dB,
    /// `1.0` is left channel -114dB, right channel +6dB
    ///
    pub fn set_panning(&mut self, cv: Signal) -> &mut Self {
        self.panning = ModSource::Owned(cv);
        self
    }

    /// Set left and right channel panning by external source id
    ///
    /// `0.0` is center resulting in no amplification,
    /// `-1.0` is left channel +6dB, right channel -114dB,
    /// `1.0` is left channel -114dB, right channel +6dB
    ///
    pub fn set_external_panning(&mut self, source: &dyn Source) -> &mut Self {
        self.panning = ModSource::External(source.get_id());
        self
    }
}

impl Source for Track {
    fn sample(&mut self, sources: Vec<(usize, &Signal)>, _buffer_size: usize, _sample_rate: usize) -> Signal {
        let mut map = HashMap::new();

	let gain_stream = self.gain
	    .get(&sources)
	    .unwrap_or(Signal::fixed(0.9))
	    .get_stream();

	let panning_stream = self.panning
	    .get(&sources)
	    .unwrap_or(Signal::fixed(0.0))
	    .get_stream();

        for (key, signal) in sources {
            map.insert(key, signal);
        }

        let sources: Vec<Signal> = self
            .inputs
            .iter()
            .filter_map(|k| map.get(k))
            .map(|&signal| {
                signal
                    .clone()
                    .map(|s| s.amplify_with_cv(&gain_stream, |p| cv_to_db(p)))
                    .map_to_stereo(|left, right| {
                        (
                            left.amplify_with_cv(&panning_stream, |p| panning_to_db(p)),
                            right.amplify_with_cv(&panning_stream, |p| panning_to_db(-p)),
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

    fn get_sources(&self) -> Vec<usize> {
        let mut sources = self.inputs.clone();

	if let ModSource::External(key) = self.gain {
	    sources.push(key);
	}

	if let ModSource::External(key) = self.panning {
	    sources.push(key);
	}

	sources
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
    use crate::clip::Clip;
    use crate::signal::Signal;
    use crate::primary::Primary;
    use crate::oscillator::Oscillator;
    use crate::traits::FromPoints;

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

    #[test]
    fn test_panning() {
        let buffer_size = 4;
        let sample_rate = 48_000;

	let mut primary = Primary::new(buffer_size, sample_rate);
        let mut clip = Clip::new(&mut primary, Signal::from_points(&[1.0, 1.0, 1.0, 0.0]));
	let mut lfo = Oscillator::new(&mut primary);
        let mut track = Track::new(&mut primary);

	lfo.frequency = 24_000.0;
        track.add_input(&clip);
	track.set_external_gain(&lfo);
	primary.add_monitor(&track);

        assert_eq!(
            primary.sample(vec![&mut clip, &mut track, &mut lfo]).unwrap(),
            vec![
		0.001995262, 0.001995262,
		0.063095726, 0.063095726,
		0.001995262, 0.001995262,
		0.0, 0.0
	    ],
        );
    }
}
