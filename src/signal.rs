use crate::stream::{Stream, StreamErr};
use crate::traits::FromPoints;
use alloc::vec;
use alloc::vec::Vec;

/// Most fundamental type for carrying signals throughout the library.
/// Each component that has a sound source should be sampleable
/// and be able to produce a Signal
#[derive(Debug, PartialEq, Clone)]
pub enum Signal {
    // Fixed unmoving signal, mainly used for 
   //  Fixed(
    /// Mono signal containing one stream
    Mono(Stream),
    /// Stereo signal containing two streams, left and right respectively
    Stereo(Stream, Stream),
    // @TODO: Other(Vec<Stream>)
}

/// Error enum for signal methods
#[derive(Debug)]
pub enum SignalErr {
    /// Unable to get point for [`Stream`] using [`Stream::get_point`]
    UnableToGetPointForStream,
}

impl Signal {
    /// Generate a mono signal of `size` length
    ///
    /// ```
    /// use screech::stream::Stream;
    /// use screech::signal::Signal;
    ///
    /// assert_eq!(Signal::silence(2), Signal::Mono(Stream { points: vec![0.0, 0.0] }));
    /// ```
    pub fn silence(size: usize) -> Self {
        Signal::Mono(Stream::empty(size))
    }

    /// Return the length of the internal [`Stream`]
    ///
    /// ```
    /// use screech::signal::Signal;
    ///
    /// assert_eq!(Signal::silence(2).len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        match self {
            Signal::Mono(stream) => stream.len(),
            Signal::Stereo(stream, _) => stream.len(),
        }
    }

    /// Transform inner [`Stream`]
    ///
    /// ```
    /// use screech::signal::Signal;
    /// use screech::stream::Stream;
    ///
    /// let signal = Signal::silence(10).map(|stream| stream.looped_slice(0, 2));
    ///
    /// assert_eq!(
    ///     signal,
    ///     Signal::Mono(Stream { points: vec![0.0, 0.0] })
    /// );
    ///
    /// let signal = Signal::silence(10).to_stereo().map(|stream| stream.looped_slice(0, 2));
    ///
    /// assert_eq!(
    ///     signal,
    ///     Signal::Stereo(Stream { points: vec![0.0, 0.0] }, Stream { points: vec![0.0, 0.0] })
    /// );
    /// ```
    pub fn map<F>(self, f: F) -> Self
    where
        F: Fn(Stream) -> Stream,
    {
        match self {
            Signal::Mono(stream) => Signal::Mono(f(stream)),
            Signal::Stereo(left, right) => Signal::Stereo(f(left), f(right)),
        }
    }

    /// Transform inner [`Stream`]s for left and right channels individually.
    /// When given a mono channel it converts it to stereo by cloning left onto right
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::signal::Signal;
    /// use screech::stream::Stream;
    ///
    /// let signal = Signal::from_points(&[0.5, 0.5, 0.5])
    ///     .map_stereo(|left, right| (left.amplify(6.0), right.amplify(-6.0)));
    ///
    /// assert_eq!(
    ///     signal,
    ///     Signal::Stereo(
    ///         Stream { points: vec![0.9976312, 0.9976312, 0.9976312] },
    ///         Stream { points: vec![0.2505936, 0.2505936, 0.2505936] },
    ///     )
    /// );
    /// ```
    pub fn map_stereo<F>(self, f: F) -> Self
    where
        F: Fn(Stream, Stream) -> (Stream, Stream),
    {
        let (left, right) = match self {
            Signal::Mono(stream) => f(stream.clone(), stream),
            Signal::Stereo(left, right) => f(left, right),
        };

        Signal::Stereo(left, right)
    }

    /// Transform inner [`Stream`] that has the possibility to Err
    ///
    /// ```
    /// use screech::signal::Signal;
    /// use screech::stream::Stream;
    ///
    /// let signal = Signal::silence(10).and_then(|stream| stream.slice(0, 2));
    ///
    /// assert_eq!(
    ///     signal.unwrap(),
    ///     Signal::Mono(Stream { points: vec![0.0, 0.0] })
    /// );
    ///
    /// let signal = Signal::silence(10).to_stereo().and_then(|stream| stream.slice(0, 2));
    ///
    /// assert_eq!(
    ///     signal.unwrap(),
    ///     Signal::Stereo(Stream { points: vec![0.0, 0.0] }, Stream { points: vec![0.0, 0.0] })
    /// );
    /// ```
    pub fn and_then<F>(self, f: F) -> Result<Self, StreamErr>
    where
        F: Fn(Stream) -> Result<Stream, StreamErr>,
    {
        match self {
            Signal::Mono(stream) => Ok(Signal::Mono(f(stream)?)),
            Signal::Stereo(left, right) => Ok(Signal::Stereo(f(left)?, f(right)?)),
        }
    }

    /// Mix signals together into a new stereo signal
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::signal::Signal;
    /// use screech::stream::Stream;
    ///
    /// let signals = vec![
    ///     Signal::from_points(&[0.1, 0.0, 0.1]),
    ///     Signal::from_points(&[0.0, 0.1, 0.1, 0.1])
    ///     Signal::Stereo(
    ///         Stream::from_points(&[0.0, 0.1, 0.2, 0.3]),
    ///         Stream::from_points(&[0.1, 0.2, 0.3, 0.4]),
    ///     )
    /// ];
    ///
    /// assert_eq!(
    ///     Signal::mix(&signals),
    ///     Signal::Stereo(
    ///         Stream { points: vec![0.1, 0.2, 0.4, 0.4] },
    ///         Stream { points: vec![0.2, 0.3, 0.5, 0.5] },
    ///     )
    /// )
    /// ```
    pub fn mix(signals: &[&Signal]) -> Self {
	signals
	    .iter()
	    .fold(Signal::silence(0), |sum, signal| {
		match (sum, signal) {
		    (Signal::Mono(a), Signal::Mono(b)) => {
			Signal::Mono(Stream::mix(&[&a, b]))
		    }
		    (Signal::Mono(a), Signal::Stereo(left, right)) => {
			Signal::Stereo(
			    Stream::mix(&[&a, left]),
			    Stream::mix(&[&a, right]),
			)
		    }
		    (Signal::Stereo(left, right), Signal::Mono(b)) => {
			Signal::Stereo(
			    Stream::mix(&[b, &left]),
			    Stream::mix(&[b, &right]),
			)
		    }
		    (Signal::Stereo(left_a, right_a), Signal::Stereo(left_b, right_b)) => {
			Signal::Stereo(
			    Stream::mix(&[&left_a, left_b]),
			    Stream::mix(&[&right_a, right_b]),
			)
		    }
		}
	    })
    }

    /// match channels for current Signal based on passed Signal
    /// using [`Signal::sum_to_mono`] for stereo to mono conversion
    ///
    /// ```
    /// use screech::signal::Signal;
    /// use screech::stream::Stream;
    ///
    /// let mono_signal = Signal::silence(0);
    /// let stereo_signal = Signal::silence(0).to_stereo();
    ///
    /// assert_eq!(Signal::silence(0).match_channels(&mono_signal).is_mono(), true);
    /// assert_eq!(Signal::silence(0).match_channels(&stereo_signal).is_stereo(), true);
    /// ```
    pub fn match_channels(self, signal: &Signal) -> Self {
        match signal {
            Signal::Mono(_) => self.sum_to_mono(),
            Signal::Stereo(_, _) => self.to_stereo(),
        }
    }

    /// Returns true if the enum instance is of [`Signal::Mono(Stream)`] type
    ///
    /// ```
    /// use screech::signal::Signal;
    /// use screech::stream::Stream;
    ///
    /// let mono_signal = Signal::silence(0);
    /// let stereo_signal = Signal::silence(0).to_stereo();
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

    /// Returns true if the enum instance is of [`Signal::Stereo(Stream, Stream)`] type
    ///
    /// ```
    /// use screech::signal::Signal;
    /// use screech::stream::Stream;
    ///
    /// let mono_signal = Signal::silence(0);
    /// let stereo_signal = Signal::silence(0).to_stereo();
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

    /// Convert a stereo stream to mono by summing the left and right channel
    /// using [`Stream::mix`]
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::signal::Signal;
    /// use screech::stream::Stream;
    ///
    /// let stereo_signal = Signal::Stereo(
    ///     Stream::from_points(&[0.1, 0.2, 0.3, 0.4]),
    ///     Stream::from_points(&[-0.1, -0.2, -0.3, -0.4]),
    /// );
    ///
    /// assert_eq!(
    ///     stereo_signal.sum_to_mono(),
    ///     Signal::Mono(
    ///         Stream { points: vec![0.0, 0.0, 0.0, 0.0] },
    ///     )
    /// )
    /// ```
    pub fn sum_to_mono(self) -> Self {
        match self {
            Signal::Mono(_) => self,
            Signal::Stereo(left, right) => Signal::Mono(Stream::mix(&[&left, &right])),
        }
    }

    /// Convert a stereo stream to mono by ditching the right channel
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::signal::Signal;
    /// use screech::stream::Stream;
    ///
    /// let stereo_signal = Signal::Stereo(
    ///     Stream::from_points(&[0.1, 0.2, 0.3, 0.4]),
    ///     Stream::from_points(&[0.5, 0.6, 0.7, 0.8]),
    /// );
    ///
    /// assert_eq!(
    ///     stereo_signal.to_mono(),
    ///     Signal::Mono(
    ///         Stream { points: vec![0.1, 0.2, 0.3, 0.4] },
    ///     )
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
    /// use screech::traits::FromPoints;
    /// use screech::signal::Signal;
    /// use screech::stream::Stream;
    ///
    /// let mono_signal = Signal::from_points(&[0.1, 0.2, 0.3, 0.4]);
    ///
    /// assert_eq!(
    ///     mono_signal.to_stereo(),
    ///     Signal::Stereo(
    ///         Stream { points: vec![0.1, 0.2, 0.3, 0.4] },
    ///         Stream { points: vec![0.1, 0.2, 0.3, 0.4] },
    ///     )
    /// )
    /// ```
    pub fn to_stereo(self) -> Self {
        match self {
            Signal::Mono(stream) => Signal::Stereo(stream.clone(), stream),
            Signal::Stereo(_, _) => self,
        }
    }

    /// Returns a sequence of interleaved points from the internal [`Stream`].
    /// Audio data is interleaved: `[left, right, left right...]`
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::signal::Signal;
    /// use screech::stream::Stream;
    ///
    /// let stereo_signal = Signal::Stereo(
    ///     Stream::from_points(&[0.1, 0.1, 0.1, 0.1]),
    ///     Stream::from_points(&[-0.1, -0.1, -0.1, -0.1]),
    /// );
    ///
    /// assert_eq!(
    ///     stereo_signal.get_interleaved_points().unwrap(),
    ///     vec![0.1, -0.1, 0.1, -0.1, 0.1, -0.1, 0.1, -0.1],
    /// )
    /// ```
    pub fn get_interleaved_points(&self) -> Result<Vec<f32>, SignalErr> {
        match self {
            Signal::Mono(stream) => Ok(stream.get_points()),
            Signal::Stereo(left, right) => {
                let mut result: Vec<f32> = vec![];

                for i in 0..left.len() {
                    result.push(
                        left.get_point(i)
                            .ok_or(SignalErr::UnableToGetPointForStream)?
                            .clone(),
                    );
                    result.push(
                        right
                            .get_point(i)
                            .ok_or(SignalErr::UnableToGetPointForStream)?
			    .clone(),
                    );
                }

                Ok(result)
            }
        }
    }
}

impl FromPoints<f32, Signal> for Signal {
    fn from_points(points: &[f32]) -> Signal {
        Signal::Mono(Stream::from_points(points))
    }
}

impl FromPoints<i32, Signal> for Signal {
    fn from_points(points: &[i32]) -> Signal {
        Signal::Mono(Stream::from_points(points))
    }
}

impl FromPoints<i16, Signal> for Signal {
    fn from_points(points: &[i16]) -> Signal {
        Signal::Mono(Stream::from_points(points))
    }
}

impl FromPoints<u8, Signal> for Signal {
    fn from_points(points: &[u8]) -> Signal {
        Signal::Mono(Stream::from_points(points))
    }
}
