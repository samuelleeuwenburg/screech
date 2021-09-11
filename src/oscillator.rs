use crate::signal::Signal;
use crate::stream::Point;
use crate::traits::{Source, Tracker, FromPoints};
use alloc::vec;
use alloc::vec::Vec;

/// Basic saw ramp oscillator.
///
///
/// ```
/// use screech::primary::Primary;
/// use screech::oscillator::Oscillator;
/// 
/// // 4 samples per second
/// let sample_rate = 4;
/// // sample a total of 4 seconds
/// let buffer_size = sample_rate * 4;
/// 
/// let mut primary = Primary::new(buffer_size, sample_rate);
/// let mut oscillator = Oscillator::new(&mut primary);
/// 
/// oscillator.frequency = 1.0;
/// oscillator.amplitude = 1.0;
/// 
/// primary.add_monitor(&oscillator);
/// 
/// assert_eq!(
///     primary.sample(vec![&mut oscillator]).unwrap(),
///     vec![
/// 	    0.25, 0.25, 0.5, 0.5, -0.25, -0.25, -0.0, -0.0,
/// 	    0.25, 0.25, 0.5, 0.5, -0.25, -0.25, -0.0, -0.0,
/// 	    0.25, 0.25, 0.5, 0.5, -0.25, -0.25, -0.0, -0.0,
/// 	    0.25, 0.25, 0.5, 0.5, -0.25, -0.25, -0.0, -0.0,
///     ],
/// );
/// ```
pub struct Oscillator {
    /// oscillator frequency per second
    pub frequency: f32,
    /// amplitude peak to peak centered around 0.0
    ///
    /// for example an amplitude of `1.0` will generate a saw
    /// wave between `-0.5` and `0.5`
    pub amplitude: f32,
    id: usize,
    value: Point,
}

impl Oscillator {
    /// Create new oscillator
    pub fn new(tracker: &mut dyn Tracker) -> Self {
        Oscillator {
            id: tracker.create_id(),
            frequency: 1.0,
	    amplitude: 1.0,
	    value: 0.0,
        }
    }
}

impl Source for Oscillator {
    fn sample(&mut self, _sources: Vec<(usize, &Signal)>, buffer_size: usize, sample_rate: usize) -> Signal {
	let mut points = vec![];

	let increase_per_sample = self.amplitude / sample_rate as f32 * self.frequency;

	for _ in 0..buffer_size {
	    self.value += increase_per_sample;

	    if self.value > self.amplitude / 2.0 {
		self.value -= self.amplitude;
	    }

	    points.push(self.value);
	}

        Signal::from_points(&points)
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
    use crate::primary::Primary;

    #[test]
    fn test_basic_repetition() {
	// 4 samples per second
        let sample_rate = 4;
	// sample a total of 4 seconds
        let buffer_size = sample_rate * 4;

        let mut primary = Primary::new(buffer_size, sample_rate);
        let mut oscillator = Oscillator::new(&mut primary);

	oscillator.frequency = 1.0;
	oscillator.amplitude = 0.5;

        primary.add_monitor(&oscillator);

        assert_eq!(
            primary.sample(vec![&mut oscillator]).unwrap(),
            vec![
		0.125, 0.125, 0.25, 0.25, -0.125, -0.125, 0.0, 0.0,
		0.125, 0.125, 0.25, 0.25, -0.125, -0.125, 0.0, 0.0,
		0.125, 0.125, 0.25, 0.25, -0.125, -0.125, 0.0, 0.0,
		0.125, 0.125, 0.25, 0.25, -0.125, -0.125, 0.0, 0.0,
	    ],
        );
    }

    #[test]
    fn test_repeat_every_other_second() {
        let sample_rate = 4;
        let buffer_size = sample_rate * 3;

        let mut primary = Primary::new(buffer_size, sample_rate);
        let mut oscillator = Oscillator::new(&mut primary);

	oscillator.frequency = 1.5;
	oscillator.amplitude = 1.0;

        primary.add_monitor(&oscillator);

        assert_eq!(
            primary.sample(vec![&mut oscillator]).unwrap(),
            vec![
		0.375, 0.375, -0.25, -0.25, 0.125, 0.125, 0.5, 0.5,
		-0.125, -0.125, 0.25, 0.25, -0.375, -0.375, 0.0, 0.0,
		0.375, 0.375, -0.25, -0.25, 0.125, 0.125, 0.5, 0.5
	    ],
        );
    }
}
