use crate::core::{Point, Signal};
use core::cell::Cell;

/// Signal slew rate limiter
///
/// Limit the speed of change by voltage per millisecond
///
/// ```
/// use screech::core::Signal;
/// use screech::basic::Slew;
///
/// // set a sample rate of 4000 samples per second
/// let sample_rate = 4000;
/// let signals = Signal::from_points(vec![1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 0.0, 0.0]);
/// // adjust the speed of the slew rate to be 1.0 "volts" per millisecond
/// let mut slew = Slew::new(1.0);
///
/// assert_eq!(
///     signals
///         .into_iter()
///         .map(|s| *slew.process(sample_rate, s).get_point())
///         .collect::<Vec<f32>>(),
///     &[0.25, 0.0, -0.25, 0.0, 0.25, 0.0, -0.25, 0.0, 0.0],
/// );
/// ```
pub struct Slew {
    left: Cell<Point>,
    right: Cell<Point>,
    /// "voltage" or value change per millisecond
    pub value_per_ms: f32,
}

impl Slew {
    /// Initialize a new slew instance
    ///
    /// Sets the value per ms on init
    pub fn new(value_per_ms: f32) -> Self {
        Slew {
            left: Cell::new(0.0),
            right: Cell::new(0.0),
            value_per_ms,
        }
    }

    fn get_new_value(&self, sample_rate: usize, old_value: Point, current_value: Point) -> Point {
        let max_increase = self.value_per_ms / (sample_rate as f32 / 1000.0);

        let rise = current_value - old_value;

        // @TODO: simplify this nested mess
        let limited_rise = if rise < 0.0 {
            if rise < -max_increase {
                -max_increase
            } else {
                rise
            }
        } else if rise > max_increase {
            max_increase
        } else {
            rise
        };

        // determine sign
        old_value + limited_rise
    }

    /// Process a [`crate::core::Signal`] through the slew rate limiter
    pub fn process(&mut self, sample_rate: usize, signal: Signal) -> Signal {
        signal.map_stereo(
            |p| {
                let point = self.left.get();
                let new_value = self.get_new_value(sample_rate, point, p);
                self.left.set(new_value);
                new_value
            },
            |p| {
                let point = self.right.get();
                let new_value = self.get_new_value(sample_rate, point, p);
                self.left.set(new_value);
                new_value
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Signal;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn test_slew_up() {
        let sample_rate = 5000;
        let signals = Signal::from_points(vec![0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
        let mut slew = Slew::new(1.0);

        assert_eq!(
            signals
                .into_iter()
                .map(|s| *slew.process(sample_rate, s).get_point())
                .collect::<Vec<f32>>(),
            &[0.0, 0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.0, 1.0],
        );
    }

    #[test]
    fn test_slew_down() {
        let sample_rate = 5000;
        let signals = Signal::from_points(vec![0.0, 0.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0]);
        let mut slew = Slew::new(1.0);

        assert_eq!(
            signals
                .into_iter()
                .map(|s| *slew.process(sample_rate, s).get_point())
                .collect::<Vec<f32>>(),
            &[0.0, 0.0, -0.2, -0.4, -0.6, -0.8, -1.0, -1.0, -1.0],
        );
    }

    #[test]
    fn test_slew_up_down() {
        let sample_rate = 4000;
        let signals = Signal::from_points(vec![1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 0.0, 0.0]);
        let mut slew = Slew::new(1.0);

        assert_eq!(
            signals
                .into_iter()
                .map(|s| *slew.process(sample_rate, s).get_point())
                .collect::<Vec<f32>>(),
            &[0.25, 0.0, -0.25, 0.0, 0.25, 0.0, -0.25, 0.0, 0.0],
        );
    }
}
