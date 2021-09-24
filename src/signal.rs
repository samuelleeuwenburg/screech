use crate::stream::Point;
use alloc::vec::Vec;

/// Most fundamental type for carrying signals throughout the library.
/// Each component that has a sound source should be sampleable
/// and be able to produce a Signal
#[derive(Copy, Debug, PartialEq, Clone)]
pub enum Signal {
    /// Mono signal containing one stream
    Mono(Point),
    /// Stereo signal containing two streams, left and right respectively
    Stereo(Point, Point),
}

/// Error enum for signal methods
#[derive(Debug)]
pub enum SignalErr {
    /// Unable to get point for [`Stream`] using [`Stream::get_point`]
    UnableToGetPointForStream,
    /// It is impossible to build up a proper audio signal from a fixed point Signal
    UnableToBuildStreamFromFixedPointSignal,
}

impl Signal {
    /// Generate a silent signal
    ///
    /// ```
    /// use screech::signal::Signal;
    ///
    /// assert_eq!(Signal::silence(), Signal::Mono(0.0));
    /// ```
    pub fn silence() -> Self {
        Signal::Mono(0.0)
    }

    /// Generate a mono signal from a point
    ///
    /// ```
    /// use screech::signal::Signal;
    ///
    /// assert_eq!(Signal::point(0.0), Signal::Mono(0.0));
    /// ```
    pub fn point(point: Point) -> Self {
        Signal::Mono(point)
    }

    /// Generate a stereo signal from two points
    ///
    /// ```
    /// use screech::signal::Signal;
    ///
    /// assert_eq!(Signal::points(0.1, 0.2), Signal::Stereo(0.1, 0.2));
    /// ```
    pub fn points(left: Point, right: Point) -> Self {
        Signal::Stereo(left, right)
    }

    /// Transform inner points,
    /// technically this is not a real map, since you can only manipulate the point to another point
    ///
    /// ```
    /// use screech::signal::Signal;
    ///
    /// let mono = Signal::point(0.1).map(|p| p * 2.0);
    /// let stereo = Signal::points(0.1, 0.2).map(|p| p * 2.0);
    ///
    /// assert_eq!(mono, Signal::Mono(0.2));
    /// assert_eq!(stereo, Signal::Stereo(0.2, 0.4));
    /// ```
    pub fn map<F>(self, f: F) -> Self
    where
        F: Fn(Point) -> Point,
    {
        match self {
            Signal::Mono(point) => Signal::Mono(f(point)),
            Signal::Stereo(left, right) => Signal::Stereo(f(left), f(right)),
        }
    }

    /// Transform inner pointss for left and right channels individually.
    /// When given a mono channel it applies the transformation only to the left channel
    ///
    /// ```
    /// use screech::signal::Signal;
    ///
    /// let signal = Signal::points(0.1, 0.2)
    ///     .map_stereo(|left| left * 2.0, |right| right / 2.0);
    ///
    /// assert_eq!(signal, Signal::Stereo(0.2, 0.1));
    /// ```
    pub fn map_stereo<L, R>(self, l: L, r: R) -> Self
    where
        L: Fn(Point) -> Point,
        R: Fn(Point) -> Point,
    {
        match self {
            Signal::Mono(point) => Signal::Mono(l(point)),
            Signal::Stereo(left, right) => Signal::Stereo(l(left), r(right)),
        }
    }

    /// Transform inner pointss for left and right channels individually.
    /// When given a mono channel it converts it to stereo by cloning left onto right
    ///
    /// ```
    /// use screech::signal::Signal;
    ///
    /// let signal = Signal::point(0.2)
    ///     .map_to_stereo(|left, right| (left * 2.0, right / 2.0));
    ///
    /// assert_eq!(signal, Signal::Stereo(0.4, 0.1));
    /// ```
    pub fn map_to_stereo<F>(self, f: F) -> Self
    where
        F: Fn(Point, Point) -> (Point, Point),
    {
        let (left, right) = match self {
            Signal::Mono(point) => f(point, point),
            Signal::Stereo(left, right) => f(left, right),
        };

        Signal::Stereo(left, right)
    }

    /// Mix signals into existing signal
    ///
    /// ***note*** on mono channels will sum the right channel to the left
    ///
    /// ```
    /// use screech::signal::Signal;
    ///
    /// let signals = [
    ///     &Signal::point(0.1),
    ///     &Signal::points(0.2, 0.3),
    /// ];
    ///
    /// let signal = Signal::point(0.4).mix_into(&signals);
    ///
    /// assert_eq!(signal, Signal::Mono(1.0));
    /// ```
    pub fn mix_into(self, signals: &[&Signal]) -> Self {
        match self {
            Signal::Mono(point) => {
                let sum = signals.iter().fold(point, |sum, p| sum + p.sum_points());
                Signal::Mono(sum)
            }
            Signal::Stereo(left, right) => {
                let lefts = signals.iter().fold(left, |sum, s| sum + s.get_point());
                let rights = signals
                    .iter()
                    .fold(right, |sum, s| sum + s.get_right_point().unwrap_or(&0.0));

                Signal::Stereo(lefts, rights)
            }
        }
    }

    /// Mix signals together into a new signal,
    /// if only supplied mono signals it stays mono,
    /// otherwise it switches the signal to stereo
    /// putting mono signals across both channels
    ///
    /// ```
    /// use screech::signal::Signal;
    ///
    /// let signal = Signal::mix(&[
    ///     &Signal::point(0.1),
    ///     &Signal::point(0.2),
    ///     &Signal::Stereo(0.3, 0.2),
    /// ]);
    ///
    /// assert_eq!(signal, Signal::Stereo(0.6, 0.5))
    /// ```
    pub fn mix(signals: &[&Signal]) -> Self {
        let mut left = 0.0;
        let mut right = 0.0;
        let mut is_mono = true;

        for s in signals {
            left += s.get_point();
            match s.get_right_point() {
                Some(point) => {
                    is_mono = false;
                    right += point;
                }
                None => right += s.get_point(),
            }
        }

        if is_mono {
            Signal::point(left)
        } else {
            Signal::points(left, right)
        }
    }

    /// match channels for current Signal based on passed Signal
    /// using [`Signal::sum_to_mono`] for stereo to mono conversion
    ///
    /// ```
    /// use screech::signal::Signal;
    ///
    /// let mono_signal = Signal::silence();
    /// let stereo_signal = Signal::silence().to_stereo();
    ///
    /// assert_eq!(Signal::silence().match_channels(&mono_signal).is_mono(), true);
    /// assert_eq!(Signal::silence().match_channels(&stereo_signal).is_stereo(), true);
    /// ```
    pub fn match_channels(self, signal: &Signal) -> Self {
        match signal {
            Signal::Mono(_) => self.sum_to_mono(),
            Signal::Stereo(_, _) => self.to_stereo(),
        }
    }

    /// Returns true if the enum instance is of [`Signal::Mono`] type
    ///
    /// ```
    /// use screech::signal::Signal;
    ///
    /// let mono_signal = Signal::silence();
    /// let stereo_signal = Signal::silence().to_stereo();
    ///
    /// assert_eq!(mono_signal.is_mono(), true);
    /// assert_eq!(stereo_signal.is_mono(), false);
    /// ```
    pub fn is_mono(&self) -> bool {
        match self {
            Signal::Mono(_) => true,
            Signal::Stereo(_, _) => false,
        }
    }

    /// Returns true if the enum instance is of [`Signal::Stereo`] type
    ///
    /// ```
    /// use screech::signal::Signal;
    ///
    /// let mono_signal = Signal::silence();
    /// let stereo_signal = Signal::silence().to_stereo();
    ///
    /// assert_eq!(mono_signal.is_stereo(), false);
    /// assert_eq!(stereo_signal.is_stereo(), true);
    /// ```
    pub fn is_stereo(&self) -> bool {
        match self {
            Signal::Mono(_) => false,
            Signal::Stereo(_, _) => true,
        }
    }

    /// Convert a stereo point to mono by summing the left and right channel
    /// using [`Stream::mix`]
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::signal::Signal;
    /// use screech::stream::Stream;
    ///
    /// let stereo_signal = Signal::Stereo(0.1, -0.1);
    ///
    /// assert_eq!(
    ///     stereo_signal.sum_to_mono(),
    ///     Signal::Mono(0.0),
    /// )
    /// ```
    pub fn sum_to_mono(self) -> Self {
        match self {
            Signal::Mono(_) => self,
            Signal::Stereo(left, right) => Signal::Mono(left + right),
        }
    }

    /// Convert a stereo point to mono by ditching the right channel
    ///
    /// ```
    /// use screech::signal::Signal;
    ///
    /// let stereo_signal = Signal::points(0.1, 0.2);
    ///
    /// assert_eq!(
    ///     stereo_signal.to_mono(),
    ///     Signal::Mono(0.1),
    /// )
    /// ```
    pub fn to_mono(self) -> Self {
        match self {
            Signal::Mono(_) => self,
            Signal::Stereo(left, _) => Signal::Mono(left),
        }
    }

    /// Convert a mono stream to stereo by cloning the signal to both channels
    ///
    /// ```
    /// use screech::signal::Signal;
    ///
    /// let mono_signal = Signal::point(0.1);
    ///
    /// assert_eq!(
    ///     mono_signal.to_stereo(),
    ///     Signal::Stereo(0.1, 0.1),
    /// )
    /// ```
    pub fn to_stereo(self) -> Self {
        match self {
            Signal::Mono(point) => Signal::Stereo(point, point),
            Signal::Stereo(_, _) => self,
        }
    }

    /// Get the inner point,
    /// or the left point if it is a stereo signal
    pub fn get_point(&self) -> &Point {
        match self {
            Signal::Mono(point) => point,
            Signal::Stereo(left, _) => left,
        }
    }

    /// Get the inner point if mono, and sum left and right together for stereo signals
    pub fn sum_points(&self) -> Point {
        match self {
            Signal::Mono(point) => *point,
            Signal::Stereo(left, right) => left + right,
        }
    }

    /// Get the inner right point if available
    pub fn get_right_point(&self) -> Option<&Point> {
        match self {
            Signal::Mono(_) => None,
            Signal::Stereo(_, right) => Some(right),
        }
    }

    pub fn from_points(points: Vec<Point>) -> Vec<Signal> {
        points.iter().map(|p| Signal::point(*p)).collect()
    }
}

// impl FromPoints<f32, Signal> for Signal {
//     fn from_points(points: Vec<f32>) -> Signal {
//         Signal::Mono(Stream::from_points(points))
//     }
// }
//
// impl FromPoints<i32, Signal> for Signal {
//     fn from_points(points: Vec<i32>) -> Signal {
//         Signal::Mono(Stream::from_points(points))
//     }
// }
//
// impl FromPoints<i16, Signal> for Signal {
//     fn from_points(points: Vec<i16>) -> Signal {
//         Signal::Mono(Stream::from_points(points))
//     }
// }
//
// impl FromPoints<u8, Signal> for Signal {
//     fn from_points(points: Vec<u8>) -> Signal {
//         Signal::Mono(Stream::from_points(points))
//     }
// }
