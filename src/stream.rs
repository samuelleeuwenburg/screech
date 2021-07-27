pub type BufferSize = usize;

pub type Point = f32;

/// Error type for different failures
#[derive(Debug)]
pub enum StreamErr {
    /// Tried to access index in stream that doesn not exist
    NonExistentIndex(usize),
}

/// Struct representing a stream of audio data
#[derive(Debug, PartialEq, Clone)]
pub struct Stream {
    /// Vec containing all audio points, multiple channels are interleaved
    pub samples: Vec<Point>,
    /// amount of channels
    pub channels: usize,
}

/// Convert u8 to point value (f32 between -1.0 and 1.0)
pub fn u8_to_point(n: u8) -> Point {
    (n as f32 / u8::MAX as f32) * 2.0 - 1.0
}

/// Convert i16 to point value (f32 between -1.0 and 1.0)
pub fn i16_to_point(n: i16) -> Point {
    n as f32 / i16::MAX as f32
}

/// Convert i32 to point value (f32 between -1.0 and 1.0)
pub fn i32_to_point(n: i32) -> Point {
    n as f32 / i32::MAX as f32
}

impl Stream {
    /// Create zero initialized (silent) stream
    pub fn empty(size: BufferSize, channels: usize) -> Self {
        Stream {
            samples: vec![0.0; size],
            channels,
        }
    }

    /// Create new stream based on provided samples
    pub fn from_samples(samples: Vec<Point>, channels: usize) -> Self {
        Stream { samples, channels }
    }

    /// Returns the length of the stream
    ///
    /// *note* this does not take into account the amount of channels
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Get sample for provided position argument, errors when the index does not exist in the stream
    pub fn get_sample(&self, position: usize) -> Result<f32, StreamErr> {
        match self.samples.get(position) {
            Some(&f) => Ok(f),
            None => Err(StreamErr::NonExistentIndex(position)),
        }
    }

    /// Mix together multiple streams into the given stream
    ///
    /// *note* the size of the stream is unchanged,
    /// if the other streams are shorter it inserts silence (0.0)
    /// if the other streams are longer the remaining points are ignored
    ///
    /// *note* this is a naive mix, it does not take into account the channel size, 
    /// it assumes you are mixing together channels of the same size
    pub fn mix(&mut self, streams: &Vec<&Stream>) -> &mut Self {
        for (i, sample) in self.samples.iter_mut().enumerate() {
            *sample = streams.iter().fold(sample.clone(), |xs, x| {
                xs + x.samples.get(i).unwrap_or(&0.0)
            });
        }

        self
    }

    /// Amplify a stream by decibels
    pub fn amplify(&mut self, db: f32) -> &mut Self {
        let ratio = 10_f32.powf(db / 20.0);

        for sample in self.samples.iter_mut() {
            *sample = (sample.clone() * ratio).clamp(-1.0, 1.0);
        }

        self
    }
}

#[cfg(test)]
mod tests {
    #![allow(overflowing_literals)]
    use super::*;

    #[test]
    fn test_mix() {
        let samples = vec![-1.0, -0.5, 0.0, 0.5, 1.0];
        let streams = vec![];
        let mut stream = Stream::from_samples(samples, 1);
        stream.mix(&streams);
        assert_eq!(stream.samples, vec![-1.0, -0.5, 0.0, 0.5, 1.0]);

        let samples = vec![1.0, 0.2, 1.0, 1.0, 0.2];
        let streams = vec![Stream::from_samples(vec![0.0, 0.0, 0.0, 0.0, 0.0], 1)];
        let mut stream = Stream::from_samples(samples, 1);
        stream.mix(&streams.iter().collect());
        assert_eq!(stream.samples, vec![1.0, 0.2, 1.0, 1.0, 0.2]);

        let samples = vec![0.1, 0.0, -0.1, -0.2, -0.3];
        let streams = vec![
            Stream::from_samples(vec![0.2, 0.1, 0.0, -0.1, -0.2], 1),
            Stream::from_samples(vec![0.3, 0.2, 0.1, 0.0, -0.1], 1),
        ];
        let mut stream = Stream::from_samples(samples, 1);
        stream.mix(&streams.iter().collect());
        assert_eq!(stream.samples, vec![0.6, 0.3, 0.0, -0.3, -0.6]);

        let samples = vec![0.1, 0.0, -0.1, -0.2, -0.3];
        let streams = vec![
            Stream::from_samples(vec![0.2, 0.1, 0.0], 1),
            Stream::from_samples(vec![0.3], 1),
        ];
        let mut stream = Stream::from_samples(samples, 1);
        stream.mix(&streams.iter().collect());
        assert_eq!(stream.samples, vec![0.6, 0.1, -0.1, -0.2, -0.3]);
    }

    #[test]
    fn test_amplify() {
        let mut stream = Stream::empty(1, 1);
        stream.amplify(6.0);
        assert_eq!(stream.samples, vec![0.0]);

        // 6 dBs should roughly double / half
        let mut stream = Stream::from_samples(vec![0.1, 0.25, 0.3, -0.1, -0.4], 1);
        stream.amplify(6.0);
        let rounded_samples: Vec<Point> = stream
            .samples
            .iter()
            .map(|x| (x * 10.0).round() / 10.0)
            .collect::<Vec<Point>>();
        assert_eq!(rounded_samples, vec![0.2, 0.5, 0.6, -0.2, -0.8]);

        let mut stream = Stream::from_samples(vec![0.4, 0.5, 0.8, -0.3, -0.6], 1);
        stream.amplify(-6.0);
        let rounded_samples: Vec<Point> = stream
            .samples
            .iter()
            .map(|x| (x * 100.0).round() / 100.0)
            .collect::<Vec<Point>>();
        assert_eq!(rounded_samples, vec![0.2, 0.25, 0.4, -0.15, -0.3]);

        // clamp the value
        let mut stream = Stream::from_samples(vec![0.1, 0.4, 0.6, -0.2, -0.3, -0.5], 1);
        stream.amplify(12.0);
        let rounded_samples: Vec<Point> = stream
            .samples
            .iter()
            .map(|x| (x * 100.0).round() / 100.0)
            .collect::<Vec<Point>>();
        assert_eq!(rounded_samples, vec![0.4, 1.0, 1.0, -0.8, -1.0, -1.0]);
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
}
