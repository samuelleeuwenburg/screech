use crate::stream::{Stream, StreamErr};
use alloc::vec;

/// Wrapper type to handle contextual channel manipulation for [`Stream`]
/// 
/// Most fundamental type for carrying signals throughout the application.
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
    pub fn silence(size: usize) -> Self {
	Signal::Mono(Stream::empty(size))
    }

    /// Return the length of the internal [`Stream`]
    pub fn len(&self) -> usize {
        match self {
            Signal::Mono(stream) => stream.len(),
            Signal::Stereo(stream, _) => stream.len(),
        }
    }

    /// apply a manipulation to the underlying [`Stream`]
    pub fn map<F>(&self, f: F) -> Self
    where F: Fn(&Stream) -> Stream {
        match self {
            Signal::Mono(stream) => Signal::Mono(f(stream)),
            Signal::Stereo(left, right) => Signal::Stereo(f(left), f(right)),
        }
    }

    /// apply a manipulation that can Err to the underlying [`Stream`]
    pub fn and_then<F>(&self, f: F) -> Result<Self, StreamErr>
    where F: Fn(&Stream) -> Result<Stream, StreamErr>  {
        match self {
            Signal::Mono(stream) => {
                let stream = f(stream)?;
                Ok(Signal::Mono(stream))
            }
            Signal::Stereo(left, right) => {
                let left = f(left)?;
                let right = f(right)?;
                Ok(Signal::Stereo(left, right))
            }
        }
    }

    /// Mix other signals into current signal respecting channels using [`Stream::mix`]
    pub fn mix(&mut self, signals: &[&Signal]) -> &mut Self {
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
	    Signal::Mono(stream) => {
		stream.mix(&left);
	    }
	    Signal::Stereo(l, r) => {
		l.mix(&left);
		r.mix(&right);
	    }
	}

	self
    }

    /// match channels for current Signal based on passed Signal
    /// using [`Signal::sum_to_mono`] for stereo to mono conversion
    pub fn match_channels(self, signal: &Signal) -> Self {
	match signal {
	    Signal::Mono(_) => self.sum_to_mono(),
	    Signal::Stereo(_, _) => self.to_stereo(),
	}
    }


    /// Returns true if the enum instance is of [`Signal::Mono(Stream)`] type
    pub fn is_mono(&self) -> bool {
        match self {
            Signal::Mono(_) => true,
            Signal::Stereo(_, _) => false,
        }
    }

    /// Returns true if the enum instance is of [`Signal::Stereo(Stream, Stream)`] type
    pub fn is_stereo(&self) -> bool {
        match self {
            Signal::Mono(_) => false,
            Signal::Stereo(_, _) => true,
        }
    }

    /// Convert a stereo stream to mono by summing the left and right channel
    /// using [`Stream::mix`]
    pub fn sum_to_mono(self) -> Self {
        match self {
            Signal::Mono(_) => self,
            Signal::Stereo(left, right) => {
                let mut stream = Stream::empty(left.len());
                stream.mix(&[&left, &right]);
                Signal::Mono(stream)
            }
        }
    }

    /// Convert a stereo stream to mono by ditching the right channel
    pub fn to_mono(self) -> Self {
        match self {
            Signal::Mono(_) => self,
            Signal::Stereo(left, _) => Signal::Mono(left),
        }
    }

    /// Convert a mono stream to stereo by cloning the signal to both channels
    pub fn to_stereo(self) -> Self {
        match self {
            Signal::Mono(stream) => Signal::Stereo(stream.clone(), stream),
            Signal::Stereo(_, _) => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_mono() {
        let mono_signal = Signal::Mono(Stream::empty(0));
        let stereo_signal = Signal::Stereo(Stream::empty(0), Stream::empty(0));

        assert_eq!(mono_signal.is_mono(), true);
        assert_eq!(stereo_signal.is_mono(), false);
    }

    #[test]
    fn test_is_stereo() {
        let mono_signal = Signal::Mono(Stream::empty(0));
        let stereo_signal = Signal::Stereo(Stream::empty(0), Stream::empty(0));

        assert_eq!(mono_signal.is_stereo(), false);
        assert_eq!(stereo_signal.is_stereo(), true);
    }
}
