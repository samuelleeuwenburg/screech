use crate::traits::FromPoints;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp;

/// Struct containing audio samples
#[derive(Clone)]
pub struct Signal {
    /// buffer containing samples
    pub samples: Vec<f32>,
}

impl Signal {
    fn new(samples: Vec<f32>) -> Self {
        Signal { samples }
    }

    /// Create zero initialized (silent) signal
    ///
    /// ```
    /// use screech::Signal;
    ///
    /// let signal = Signal::empty(4);
    ///
    /// assert_eq!(&signal.samples, &[0.0, 0.0, 0.0, 0.0]);
    /// ```
    pub fn empty(size: usize) -> Self {
        Signal {
            samples: vec![0.0; size],
        }
    }

    /// Mix multiple signals into a new signal
    ///
    /// **note** the size of the resulting signal is equal to
    /// the longest signal in the slice
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::Signal;
    ///
    /// let signals = [
    ///     &Signal::from_points(vec![0.1, 0.0, -0.1, -0.2, -0.3]),
    ///     &Signal::from_points(vec![0.2, 0.1, 0.0]),
    ///     &Signal::from_points(vec![0.3]),
    /// ];
    ///
    /// let result = Signal::mix(&signals);
    ///
    /// assert_eq!(&result.samples, &[0.6, 0.1, -0.1, -0.2, -0.3]);
    /// ```
    pub fn mix(signals: &[&Signal]) -> Self {
        let length = signals.iter().fold(0, |a, b| cmp::max(a, b.samples.len()));
        let mut signal = Signal::empty(length);

        for (i, signal) in signal.samples.iter_mut().enumerate() {
            let mut mixed_signal = 0.0;

            for other_signal in signals {
                if let Some(s) = other_signal.samples.get(i) {
                    mixed_signal += s;
                }
            }
            *signal = mixed_signal;
        }

        signal
    }

    /// Same as [`Signal::mix`], but mixes sources into the current signal
    ///
    /// **note** the size of the signal will be unchanged
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::Signal;
    ///
    /// let signals = [
    ///     &Signal::from_points(vec![0.2, 0.1, 0.0]),
    ///     &Signal::from_points(vec![0.3]),
    /// ];
    ///
    /// let mut signal = Signal::from_points(vec![0.1, 0.0, -0.1, -0.2, -0.3]);
    /// signal.mix_into(&signals);
    ///
    /// assert_eq!(&signal.samples, &[0.6, 0.1, -0.1, -0.2, -0.3]);
    /// ```
    pub fn mix_into(&mut self, signals: &[&Signal]) -> &mut Self {
        for (i, signal) in self.samples.iter_mut().enumerate() {
            let mut mixed_signal = 0.0;

            for other_signal in signals {
                if let Some(s) = other_signal.samples.get(i) {
                    mixed_signal += s;
                }
            }

            *signal += mixed_signal;
        }

        self
    }

    /// Resample using [linear interpolation](https://en.wikipedia.org/wiki/Linear_interpolation)
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::Signal;
    ///
    /// let signal = Signal::from_points(vec![0.0, 0.5, 1.0, 0.5, 0.0]);
    ///
    /// // should preserve the samples at 1.0
    /// assert_eq!(
    ///     &signal.clone().resample_linear(1.0).samples,
    ///     &signal.clone().samples,
    /// );
    ///
    /// assert_eq!(
    ///     &signal.clone().resample_linear(2.0).samples,
    ///     &[0.0, 0.25, 0.5, 0.75, 1.0, 0.75, 0.5, 0.25, 0.0]
    /// );
    ///
    /// assert_eq!(
    ///     signal.clone().resample_linear(0.5).samples,
    ///     &[0.0, 1.0, 0.0]
    /// );
    ///
    /// assert_eq!(
    ///     signal.clone().resample_linear(0.8).samples,
    ///     &[0.0, 0.625, 0.75, 0.125]
    /// );
    ///
    /// let signal2 = Signal::from_points(
    ///     vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    /// );
    ///
    /// assert_eq!(
    ///     signal2.clone().resample_linear(0.5).samples,
    ///     &[0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
    /// );
    ///
    /// assert_eq!(
    ///     signal2.clone().resample_linear(0.2).samples,
    ///     &[0.0, 0.5, 1.0]
    /// );
    ///
    /// ```
    pub fn resample_linear(&mut self, factor: f32) -> &mut Self {
        let max_loop_size = (self.samples.len() as f32 * factor).ceil() as usize;
        let mut samples = vec![];

        for index in 0..max_loop_size {
            let position_source =
                self.samples.len() as f32 * (index as f32 / (self.samples.len() as f32 * factor));
            let sample_index = position_source.floor() as usize;
            let position_samples = position_source - sample_index as f32;

            match (
                self.samples.get(sample_index),
                self.samples.get(sample_index + 1),
            ) {
                (Some(a), Some(b)) => {
                    let point = a + (b - a) * position_samples;
                    samples.push(point);
                }
                (Some(a), None) => {
                    if position_samples == 0.0 {
                        // no final sample to compare with
                        // however the position aligns _exactly_ with the first sample
                        // so we preserve it either way
                        samples.push(*a);
                    }
                }
                _ => (),
            }
        }

        self.samples = samples;

        self
    }
}

impl FromPoints<u8, Signal> for Signal {
    /// Create new signal based on u8 points,
    /// converts u8 to point value (f32 between -1.0 and 1.0)
    fn from_points(points: Vec<u8>) -> Signal {
        Signal::new(points.iter().copied().map(u8_to_point).collect())
    }
}

impl FromPoints<i16, Signal> for Signal {
    /// Create new signal based on i16 points,
    /// converts i16 to point value (f32 between -1.0 and 1.0)
    fn from_points(points: Vec<i16>) -> Signal {
        Signal::new(points.iter().copied().map(i16_to_point).collect())
    }
}

impl FromPoints<i32, Signal> for Signal {
    /// Create new signal based on i32 points,
    /// converts i32 to point value (f32 between -1.0 and 1.0)
    fn from_points(points: Vec<i32>) -> Signal {
        Signal::new(points.iter().copied().map(i32_to_point).collect())
    }
}

impl FromPoints<f32, Signal> for Signal {
    /// Create new signal based on f32 points
    fn from_points(points: Vec<f32>) -> Signal {
        // @TODO: clamp values?
        Signal::new(points)
    }
}

/// Convert u8 to point value (f32 between -1.0 and 1.0)
fn u8_to_point(n: u8) -> f32 {
    (n as f32 / u8::MAX as f32) * 2.0 - 1.0
}

/// Convert i16 to point value (f32 between -1.0 and 1.0)
fn i16_to_point(n: i16) -> f32 {
    n as f32 / i16::MAX as f32
}

/// Convert i32 to point value (f32 between -1.0 and 1.0)
fn i32_to_point(n: i32) -> f32 {
    n as f32 / i32::MAX as f32
}

#[cfg(test)]
mod tests {
    #![allow(overflowing_literals)]
    use super::*;

    #[test]
    fn test_mix() {
        let signal = Signal::mix(&[
            &Signal::from_points(vec![1.0, 0.2, 1.0, 1.0, 0.2]),
            &Signal::from_points(vec![0.0, 0.0, 0.0, 0.0, 0.0]),
        ]);

        assert_eq!(signal.samples, &[1.0, 0.2, 1.0, 1.0, 0.2]);

        let signal = Signal::mix(&[
            &Signal::from_points(vec![0.1, 0.0, -0.1, -0.2, -0.3]),
            &Signal::from_points(vec![0.2, 0.1, 0.0, -0.1, -0.2]),
            &Signal::from_points(vec![0.3, 0.2, 0.1, 0.0, -0.1]),
        ]);

        assert_eq!(signal.samples, &[0.6, 0.3, 0.0, -0.3, -0.6]);

        let signal = Signal::mix(&[
            &Signal::from_points(vec![0.1, 0.0, -0.1, -0.2, -0.3]),
            &Signal::from_points(vec![0.2, 0.1, 0.0]),
            &Signal::from_points(vec![0.3]),
        ]);

        assert_eq!(signal.samples, &[0.6, 0.1, -0.1, -0.2, -0.3]);
    }

    #[test]
    fn test_u8_to_point() {
        assert_eq!(u8_to_point(u8::MIN), -1.0);
        assert_eq!(u8_to_point(0x80u8), 0.003921628);
        assert_eq!(u8_to_point(u8::MAX), 1.0);
    }

    #[test]
    fn test_i16_to_point() {
        assert_eq!(i16_to_point(i16::MIN + 1), -1.0);
        assert_eq!(i16_to_point(0i16), 0.0);
        assert_eq!(i16_to_point(i16::MAX), 1.0);
    }

    #[test]
    fn test_i32_to_point() {
        assert_eq!(i32_to_point(i32::MIN + 1), -1.0);
        assert_eq!(i32_to_point(0i32), 0.0);
        assert_eq!(i32_to_point(i32::MAX), 1.0);
    }

    #[test]
    fn test_from_u8() {
        let signal = Signal::from_points(vec![0, 80, 128, 220, 256u8]);
        assert_eq!(
            signal.samples,
            &[-1.0, -0.372549, 0.003921628, 0.7254902, -1.0]
        );
    }

    #[test]
    fn test_from_i16() {
        let signal = Signal::from_points(vec![i16::MIN + 1, -1600, 0, 2800, i16::MAX]);
        assert_eq!(signal.samples, &[-1.0, -0.048829615, 0.0, 0.08545183, 1.0]);
    }

    #[test]
    fn test_from_i32() {
        let signal =
            Signal::from_points(vec![i32::MIN, -1_147_483_647, 0, 1_147_483_647, i32::MAX]);
        assert_eq!(signal.samples, &[-1.0, -0.5343387, 0.0, 0.5343387, 1.0]);
    }
}
