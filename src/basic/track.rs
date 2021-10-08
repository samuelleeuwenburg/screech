use crate::core::point::{amplify, Point};
use crate::core::{ExternalSignal, Signal};
use crate::traits::{Source, Tracker};
use alloc::vec;
use alloc::vec::Vec;

/// Standard track with panning and volume control
#[derive(Debug, PartialEq)]
pub struct Track {
    inputs: Vec<ExternalSignal>,
    /// main audio output
    pub output: ExternalSignal,
    /// gain cv modulation source
    pub gain_cv: Option<ExternalSignal>,
    /// panning cv modulation source
    pub panning_cv: Option<ExternalSignal>,
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
            output: ExternalSignal::new(tracker.create_source_id(), 0),
            inputs: vec![],
            gain_cv: None,
            panning_cv: None,
            gain: 0.0,
            panning: 0.0,
        }
    }

    /// add source to the input, supports multiple inputs
    pub fn add_input(&mut self, source: ExternalSignal) -> &mut Self {
        self.inputs.push(source);
        self
    }

    /// remove source from input
    pub fn remove_input(&mut self, source: &ExternalSignal) -> &mut Self {
        self.inputs.retain(|b| source != b);
        self
    }
}

impl Source for Track {
    fn sample(&mut self, sources: &mut dyn Tracker, _sample_rate: usize) {
        let inputs: Vec<&Signal> = self
            .inputs
            .iter()
            .filter_map(|e| sources.get_signal(e))
            .collect();

        let gain = self
            .gain_cv
            .and_then(|e| sources.get_signal(&e))
            .map(|s| s.sum_points())
            .unwrap_or(0.9);

        let panning = self
            .panning_cv
            .and_then(|e| sources.get_signal(&e))
            .map(|s| s.sum_points())
            .unwrap_or(0.0);

        let mixed = Signal::silence().mix_into(&inputs);

        let signal = if gain == 0.9 && panning == 0.0 {
            mixed
        } else {
            mixed.map_to_stereo(|left, right| {
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
        };

        sources.set_signal(&self.output, signal);
    }

    fn get_source_id(&self) -> &usize {
        self.output.get_source_id()
    }

    fn get_sources(&self) -> Vec<usize> {
        let mut sources: Vec<usize> = self.inputs.iter().map(|e| *e.get_source_id()).collect();

        if let Some(e) = &self.gain_cv {
            sources.push(*e.get_source_id());
        }

        if let Some(e) = &self.panning_cv {
            sources.push(*e.get_source_id());
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
        let mut primary = Primary::<8>::new(48_000);
        let mut clip1 = Clip::new(&mut primary, Stream::from_points(vec![0.1, 0.2, 0.3, 0.4]));
        let mut clip2 = Clip::new(&mut primary, Stream::from_points(vec![0.1, 0.0, 0.1, 0.0]));
        let mut track = Track::new(&mut primary);

        track.add_input(clip1.output);
        track.add_input(clip2.output);
        primary.add_monitor(track.output);

        assert_eq!(
            primary
                .sample(vec![&mut clip1, &mut clip2, &mut track])
                .unwrap(),
            &[0.2, 0.2, 0.2, 0.2, 0.4, 0.4, 0.4, 0.4],
        );
    }

    #[test]
    fn test_gain() {
        let mut primary = Primary::<8>::new(48_000);
        let mut clip = Clip::new(&mut primary, Stream::from_points(vec![1.0, 1.0, 1.0, 0.0]));
        let mut lfo = Oscillator::new(&mut primary);
        let mut track = Track::new(&mut primary);

        lfo.frequency = 24_000.0;
        track.add_input(clip.output);
        track.gain_cv = Some(lfo.output);
        primary.add_monitor(track.output);

        assert_eq!(
            primary
                .sample(vec![&mut clip, &mut track, &mut lfo])
                .unwrap(),
            &[
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
        let mut primary = Primary::<8>::new(48_000);
        let mut clip = Clip::new(&mut primary, Stream::from_points(vec![0.1, 0.1, 0.1, 0.0]));
        let mut lfo = Oscillator::new(&mut primary);
        let mut track = Track::new(&mut primary);

        lfo.frequency = 24_000.0;
        lfo.amplitude = 1.0;
        lfo.output_square(0.5);

        track.add_input(clip.output);
        track.panning_cv = Some(lfo.output);

        primary.add_monitor(track.output);

        assert_eq!(
            primary
                .sample(vec![&mut clip, &mut track, &mut lfo])
                .unwrap(),
            &[
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
