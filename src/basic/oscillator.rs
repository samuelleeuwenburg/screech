use crate::core::{Point, Signal};
use crate::traits::{Source, Tracker};
use alloc::vec;
use alloc::vec::Vec;

/// Basic saw ramp oscillator.
///
///
/// ```
/// use screech::core::Primary;
/// use screech::basic::Oscillator;
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
/// oscillator.amplitude = 0.5;
///
/// primary.add_monitor(&oscillator);
///
/// assert_eq!(
///     primary.sample(vec![&mut oscillator]).unwrap(),
///     vec![
///         0.0, 0.0, 0.25, 0.25, 0.5, 0.5, -0.25, -0.25,
///         0.0, 0.0, 0.25, 0.25, 0.5, 0.5, -0.25, -0.25,
///         0.0, 0.0, 0.25, 0.25, 0.5, 0.5, -0.25, -0.25,
///         0.0, 0.0, 0.25, 0.25, 0.5, 0.5, -0.25, -0.25,
///     ],
/// );
/// ```
pub struct Oscillator {
    /// oscillator frequency per second
    pub frequency: f32,
    /// amplitude peak to peak centered around 0.0
    ///
    /// for example an amplitude of `0.5` will generate a saw
    /// wave between `-0.5` and `0.5`
    pub amplitude: f32,
    id: usize,
    value: Point,
    waveshape: Waveshape,
}

enum Waveshape {
    Saw,
    Square(f32),
    Triangle,
}

impl Oscillator {
    /// Create a new saw oscillator with a default
    /// frequency of `1.0` and an amplitute of `0.5`.
    pub fn new(tracker: &mut dyn Tracker) -> Self {
        Oscillator {
            id: tracker.create_id(),
            frequency: 1.0,
            amplitude: 0.5,
            value: 0.0,
            waveshape: Waveshape::Saw,
        }
    }

    /// Set the main output to triangle
    ///
    /// ```
    /// use screech::core::Primary;
    /// use screech::basic::Oscillator;
    ///
    /// let sample_rate = 4;
    /// let buffer_size = sample_rate * 4;
    ///
    /// let mut primary = Primary::new(buffer_size, sample_rate);
    /// let mut oscillator = Oscillator::new(&mut primary);
    ///
    /// oscillator.frequency = 0.5;
    /// oscillator.amplitude = 1.0;
    /// oscillator.output_triangle();
    ///
    /// primary.add_monitor(&oscillator);
    ///
    /// assert_eq!(
    ///     primary.sample(vec![&mut oscillator]).unwrap(),
    ///     vec![
    ///         -1.0, -1.0, -0.5, -0.5, 0.0, 0.0, 0.5, 0.5,
    ///         1.0, 1.0, 0.5, 0.5, 0.0, 0.0, -0.5, -0.5,
    ///         -1.0, -1.0, -0.5, -0.5, 0.0, 0.0, 0.5, 0.5,
    ///         1.0, 1.0, 0.5, 0.5, 0.0, 0.0, -0.5, -0.5,
    ///     ],
    /// );
    /// ```
    pub fn output_triangle(&mut self) -> &mut Self {
        self.waveshape = Waveshape::Triangle;
        self
    }

    /// Set the main output to square
    /// with a duty cycle between `0.0` (0%) and `1.0` (100%).
    ///
    /// ```
    /// use screech::core::Primary;
    /// use screech::basic::Oscillator;
    ///
    /// let sample_rate = 4;
    /// let buffer_size = sample_rate;
    ///
    /// let mut primary = Primary::new(buffer_size, sample_rate);
    /// let mut oscillator = Oscillator::new(&mut primary);
    ///
    /// oscillator.frequency = 1.0;
    /// oscillator.amplitude = 1.0;
    /// // 25% duty cycle
    /// oscillator.output_square(0.25);
    ///
    /// primary.add_monitor(&oscillator);
    ///
    /// assert_eq!(
    ///          primary.sample(vec![&mut oscillator]).unwrap(),
    ///         vec![-1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0],
    /// );
    /// ```
    pub fn output_square(&mut self, duty_cycle: f32) -> &mut Self {
        self.waveshape = Waveshape::Square(duty_cycle);
        self
    }

    /// Set the main output to saw
    ///
    /// ```
    /// use screech::core::Primary;
    /// use screech::basic::Oscillator;
    ///
    /// let sample_rate = 4;
    /// let buffer_size = sample_rate * 2;
    ///
    /// let mut primary = Primary::new(buffer_size, sample_rate);
    /// let mut oscillator = Oscillator::new(&mut primary);
    ///
    /// oscillator.frequency = 0.5;
    /// oscillator.amplitude = 1.0;
    /// oscillator.output_saw();
    ///
    /// primary.add_monitor(&oscillator);
    ///
    /// assert_eq!(
    ///         primary.sample(vec![&mut oscillator]).unwrap(),
    ///         vec![
    ///              0.0, 0.0,  0.25,  0.25,  0.5,  0.5,  0.75,  0.75,
    ///              1.0, 1.0, -0.75, -0.75, -0.5, -0.5, -0.25, -0.25,
    ///         ],
    /// );
    /// ```
    pub fn output_saw(&mut self) -> &mut Self {
        self.waveshape = Waveshape::Saw;
        self
    }

    /// Render the next real time signal
    pub fn step(&mut self, sample_rate: usize) -> Signal {
        // peak to peak conversion of amplitude
        let peak_to_peak = self.amplitude * 2.0;

        let increase_per_sample = peak_to_peak / sample_rate as f32 * self.frequency;

        let point = match self.waveshape {
            Waveshape::Saw => self.value,
            Waveshape::Square(duty_cycle) => {
                if self.value > self.amplitude * duty_cycle - self.amplitude / 2.0 {
                    -self.amplitude
                } else {
                    self.amplitude
                }
            }
            Waveshape::Triangle => {
                let triangle = if self.value > 0.0 {
                    self.value
                } else {
                    // invert bottom half
                    -self.value
                };

                // normalize for amplitude
                (triangle * 2.0) - self.amplitude
            }
        };

        self.value += increase_per_sample;

        if self.value > peak_to_peak / 2.0 {
            self.value -= peak_to_peak;
        }

        Signal::point(point)
    }
}

impl Source for Oscillator {
    fn sample(&mut self, sources: &mut dyn Tracker, sample_rate: usize) {
        sources.set_signal(self.id, self.step(sample_rate));
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
    use crate::core::Primary;

    #[test]
    fn test_basic_repetition() {
        // 4 samples per second
        let sample_rate = 4;
        // sample a total of 4 seconds
        let buffer_size = sample_rate * 4;

        let mut primary = Primary::new(buffer_size, sample_rate);
        let mut oscillator = Oscillator::new(&mut primary);

        oscillator.frequency = 1.0;
        oscillator.amplitude = 0.25;

        primary.add_monitor(&oscillator);

        assert_eq!(
            primary.sample(vec![&mut oscillator]).unwrap(),
            vec![
                0.0, 0.0, 0.125, 0.125, 0.25, 0.25, -0.125, -0.125, 0.0, 0.0, 0.125, 0.125, 0.25,
                0.25, -0.125, -0.125, 0.0, 0.0, 0.125, 0.125, 0.25, 0.25, -0.125, -0.125, 0.0, 0.0,
                0.125, 0.125, 0.25, 0.25, -0.125, -0.125,
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
        oscillator.amplitude = 0.5;

        primary.add_monitor(&oscillator);

        assert_eq!(
            primary.sample(vec![&mut oscillator]).unwrap(),
            vec![
                0.0, 0.0, 0.375, 0.375, -0.25, -0.25, 0.125, 0.125, 0.5, 0.5, -0.125, -0.125, 0.25,
                0.25, -0.375, -0.375, 0.0, 0.0, 0.375, 0.375, -0.25, -0.25, 0.125, 0.125
            ],
        );
    }
}
