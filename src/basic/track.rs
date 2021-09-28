use crate::core::point::{amplify, Point};
use crate::core::Signal;
use crate::traits::{Source, Tracker};
use alloc::vec;
use alloc::vec::Vec;

/// Standard track with panning and volume control
#[derive(Debug, PartialEq, Clone)]
pub struct Track {
    id: usize,
    inputs: Vec<usize>,
    gain_cv: Option<usize>,
    panning_cv: Option<usize>,
    /// Gain setting in dBs
    pub gain: f32,
    /// Panning setting, -1.0 to 1.0 for -114dB and +6dB respectively.
    /// inverted to each channel
    pub panning: f32,
}

impl Track {
    /// Create a new track with a unique `id`
    pub fn new(tracker: &mut dyn Tracker) -> Track {
        Track {
            id: tracker.create_id(),
            inputs: vec![],
            gain: 0.0,
            gain_cv: None,
            panning: 0.0,
            panning_cv: None,
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

    /// Set gain between `1.0` and `-1.0` by external source
    ///
    /// `0.9` is set to unity gain, for `0.1` increment the level increases by 6dB.
    /// For example setting a gain of `1.0` gives +6dB of amplification
    /// and setting a gain of `-1.0` would result in -114dB
    ///
    pub fn set_gain_cv(&mut self, cv: &dyn Source) -> &mut Self {
        self.gain_cv = Some(cv.get_id());
        self
    }

    /// Remove gain cv source
    pub fn unset_gain_cv(&mut self) -> &mut Self {
        self.gain_cv = None;
        self
    }

    /// Set left and right channel panning by external source id
    ///
    /// `0.0` is center resulting in no amplification,
    /// `-1.0` is left channel +6dB, right channel -114dB,
    /// `1.0` is left channel -114dB, right channel +6dB
    ///
    pub fn set_panning_cv(&mut self, cv: &dyn Source) -> &mut Self {
        self.panning_cv = Some(cv.get_id());
        self
    }

    /// Remove panning cv source
    pub fn unset_panning_cv(&mut self) -> &mut Self {
        self.panning_cv = None;
        self
    }

    /// Render the next real time signal
    pub fn step(&mut self, inputs: &[&Signal], gain: f32, panning: f32) -> Signal {
        Signal::silence()
            .mix_into(&inputs)
            .map_to_stereo(|left, right| {
                (
                    amplify(
                        left,
                        self.gain + cv_to_db(gain) + panning_to_db(self.panning + panning),
                    ),
                    amplify(
                        right,
                        self.gain + cv_to_db(gain) + panning_to_db(-self.panning + -panning),
                    ),
                )
            })
    }
}

impl Source for Track {
    fn sample(&mut self, sources: &mut dyn Tracker, _sample_rate: usize) {
        let inputs: Vec<&Signal> = self
            .inputs
            .iter()
            .filter_map(|&k| sources.get_signal(k))
            .collect();

        let gain = self
            .gain_cv
            .and_then(|k| sources.get_signal(k))
            .map(|s| s.sum_points())
            .unwrap_or(0.9);

        let panning = self
            .panning_cv
            .and_then(|k| sources.get_signal(k))
            .map(|s| s.sum_points())
            .unwrap_or(0.0);

        let signal = self.step(&inputs, gain, panning);
        sources.set_signal(self.id, signal);
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn get_sources(&self) -> Vec<usize> {
        let mut sources = self.inputs.clone();

        if let Some(id) = self.gain_cv {
            sources.push(id);
        }

        if let Some(id) = self.panning_cv {
            sources.push(id);
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
    use crate::basic::{Clip, Oscillator};
    use crate::core::{Primary, Stream};
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
    fn test_mix() {
        let mut primary = Primary::<4>::new(48_000);
        let mut clip1 = Clip::new(&mut primary, Stream::from_points(vec![0.1, 0.2, 0.3, 0.4]));
        let mut clip2 = Clip::new(&mut primary, Stream::from_points(vec![0.1, 0.0, 0.1, 0.0]));
        let mut track = Track::new(&mut primary);

        track.add_input(&clip1);
        track.add_input(&clip2);
        primary.add_monitor(&track);

        assert_eq!(
            primary
                .sample(vec![&mut clip1, &mut clip2, &mut track])
                .unwrap(),
            vec![0.2, 0.2, 0.2, 0.2, 0.4, 0.4, 0.4, 0.4],
        );
    }

    #[test]
    fn test_gain() {
        let mut primary = Primary::<4>::new(48_000);
        let mut clip = Clip::new(&mut primary, Stream::from_points(vec![1.0, 1.0, 1.0, 0.0]));
        let mut lfo = Oscillator::new(&mut primary);
        let mut track = Track::new(&mut primary);

        lfo.frequency = 24_000.0;
        track.add_input(&clip);
        track.set_gain_cv(&lfo);
        primary.add_monitor(&track);

        assert_eq!(
            primary
                .sample(vec![&mut clip, &mut track, &mut lfo])
                .unwrap(),
            vec![
                0.001995262,
                0.001995262,
                0.063095726,
                0.063095726,
                0.001995262,
                0.001995262,
                0.0,
                0.0
            ],
        );
    }

    #[test]
    fn test_panning() {
        let mut primary = Primary::<4>::new(48_000);
        let mut clip = Clip::new(&mut primary, Stream::from_points(vec![0.1, 0.1, 0.1, 0.0]));
        let mut lfo = Oscillator::new(&mut primary);
        let mut track = Track::new(&mut primary);

        lfo.frequency = 24_000.0;
        lfo.amplitude = 1.0;
        lfo.output_square(0.5);

        track.add_input(&clip);
        track.set_panning_cv(&lfo);

        primary.add_monitor(&track);

        assert_eq!(
            primary
                .sample(vec![&mut clip, &mut track, &mut lfo])
                .unwrap(),
            vec![
                0.19952624,
                0.00000019952631,
                0.00000019952631,
                0.19952624,
                0.19952624,
                0.00000019952631,
                0.0,
                0.0,
            ],
        );
    }
}
