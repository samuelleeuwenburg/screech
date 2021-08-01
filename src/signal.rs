use crate::stream::{Stream, StreamErr};
use alloc::vec;

/// Most fundamental type for carrying signals throughout the library.
/// Each component that has a sound source should be sampleable
/// and be able to produce a Signal
#[derive(Debug, PartialEq, Clone)]
pub enum Signal {
    /// Mono signal containing one stream
    Mono(Stream),
    /// Stereo signal containing two streams, left and right respectively
    Stereo(Stream, Stream),
    // @TODO: Other(Vec<Stream>)
}

impl Signal {
    /// Generate a mono signal of `size` length
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
    where F: Fn(Stream) -> Stream {
        match self {
            Signal::Mono(stream) => Signal::Mono(f(stream)),
            Signal::Stereo(left, right) => Signal::Stereo(f(left), f(right)),
        }
    }

    /// Transform inner [`Stream`]s for left and right channels individually.
    /// When given a mono channel it converts it to stereo by cloning left onto right
    /// ```
    /// use screech::signal::Signal;
    /// use screech::stream::{Stream, FromPoints};
    /// 
    /// let signal = Signal::Mono(Stream::from_points(&[0.5, 0.5, 0.5]))
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
    where F: Fn(Stream, Stream) -> (Stream, Stream) {
	let (left, right) = match self {
	    Signal::Mono(stream) => f(stream.clone(), stream),
	    Signal::Stereo(left, right) => f(left, right),
	};

	Signal::Stereo(left, right)
    }

    /// Transform inner [`Stream`] that has the possibility to Err
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
    where F: Fn(Stream) -> Result<Stream, StreamErr>  {
        match self {
            Signal::Mono(stream) => Ok(Signal::Mono(f(stream)?)),
            Signal::Stereo(left, right) => Ok(Signal::Stereo(f(left)?, f(right)?)),
        }
    }

    /// Mix other signals into current signal respecting channels using [`Stream::mix`]
    /// ```
    /// use screech::signal::Signal;
    /// use screech::stream::{Stream, FromPoints};
    ///
    /// let mono_signal_a = Signal::Mono(
    ///     Stream::from_points(&[0.1, 0.0, 0.1]),
    /// );
    ///
    /// let mono_signal_b = Signal::Mono(
    ///     Stream::from_points(&[0.0, 0.1, 0.1, 0.1]),
    /// );
    ///
    /// let mut stereo_signal = Signal::Stereo(
    ///     Stream::from_points(&[0.0, 0.1, 0.2, 0.3]),
    ///     Stream::from_points(&[0.1, 0.2, 0.3, 0.4]),
    /// );
    ///
    /// assert_eq!(
    ///     stereo_signal.mix(&[&mono_signal_a, &mono_signal_b]),
    ///     Signal::Stereo(
    ///         Stream { points: vec![0.1, 0.2, 0.4, 0.4] },
    ///         Stream { points: vec![0.2, 0.3, 0.5, 0.5] },
    ///     )
    /// )
    /// ```
    pub fn mix(self, signals: &[&Signal]) -> Self {
	let mut left = vec![];
	let mut right = vec![];

	for signal in signals {
	    match (&self, signal) {
		(Signal::Mono(_), Signal::Mono(stream)) => {
		    left.push(stream);
		}
		(Signal::Mono(_), Signal::Stereo(l, _r)) => {
		    left.push(l);
		}
		(Signal::Stereo(_, _), Signal::Mono(stream)) => {
		    left.push(stream);
		    right.push(stream);
		}
		(Signal::Stereo(_, _), Signal::Stereo(l, r)) => {
		    left.push(l);
		    right.push(r);
		}
	    }
	}

	match self {
	    Signal::Mono(stream) => Signal::Mono(stream.mix(&left)),
	    Signal::Stereo(l, r) => Signal::Stereo(l.mix(&left), r.mix(&right)),
	}
    }

    /// match channels for current Signal based on passed Signal
    /// using [`Signal::sum_to_mono`] for stereo to mono conversion
    /// ```
    /// use screech::signal::Signal;
    /// use screech::stream::Stream;
    ///
    /// let mono_signal = Signal::Mono(Stream::empty(0));
    /// let stereo_signal = Signal::Stereo(Stream::empty(0), Stream::empty(0));
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
    /// ```
    /// use screech::signal::Signal;
    /// use screech::stream::Stream;
    ///
    /// let mono_signal = Signal::Mono(Stream::empty(0));
    /// let stereo_signal = Signal::Stereo(Stream::empty(0), Stream::empty(0));
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
    /// ```
    /// use screech::signal::Signal;
    /// use screech::stream::Stream;
    ///
    /// let mono_signal = Signal::Mono(Stream::empty(0));
    /// let stereo_signal = Signal::Stereo(Stream::empty(0), Stream::empty(0));
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
    /// ```
    /// use screech::signal::Signal;
    /// use screech::stream::{Stream, FromPoints};
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
            Signal::Stereo(left, right) => Signal::Mono(left.mix(&[&right]))
        }
    }

    /// Convert a stereo stream to mono by ditching the right channel
    /// ```
    /// use screech::signal::Signal;
    /// use screech::stream::{Stream, FromPoints};
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
    /// ```
    /// use screech::signal::Signal;
    /// use screech::stream::{Stream, FromPoints};
    ///
    /// let mono_signal = Signal::Mono(Stream::from_points(&[0.1, 0.2, 0.3, 0.4]));
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
}
