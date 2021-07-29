use alloc::vec;
use crate::alloc::borrow::ToOwned;
use alloc::vec::Vec;
use libm::powf;

pub type Point = f32;

/// Error type for different Stream failures
#[derive(Debug, PartialEq, Clone)]
pub enum StreamErr {
    /// Tried to access index in stream that doesn not exist
    NonExistentIndex(usize),
    /// Tried to create slice with out of bounds range
    SliceOutOfBounds,
}

/// Struct representing a stream of audio data
#[derive(Debug, PartialEq, Clone)]
pub struct Stream {
    /// Vec containing all audio points
    pub points: Vec<Point>,
}

impl Stream {
    /// Create zero initialized (silent) stream
    pub fn empty(size: usize) -> Self {
        Stream {
            points: vec![0.0; size],
        }
    }

    /// Returns the length of the stream
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Get point for provided position argument, errors when the index does not exist in the stream
    pub fn get_point(&self, position: usize) -> Result<f32, StreamErr> {
        match self.points.get(position) {
            Some(&f) => Ok(f),
            None => Err(StreamErr::NonExistentIndex(position)),
        }
    }

    /// Mix multiple streams into the given stream
    ///
    /// **note** the size of the stream is unchanged,
    /// if the other streams are shorter it inserts silence (0.0)
    /// if the other streams are longer the remaining points are ignored
    pub fn mix(&mut self, streams: &[&Stream]) -> &mut Self {
        for (i, point) in self.points.iter_mut().enumerate() {
            *point = streams
                .iter()
                .fold(point.clone(), |xs, x| xs + x.points.get(i).unwrap_or(&0.0));
        }

        self
    }

    /// Amplify a stream by decibels
    ///
    /// **note** clamps values at -1.0 and 1.0
    pub fn amplify(&mut self, db: f32) -> &mut Self {
        let ratio = powf(10.0, db / 20.0);

        for point in self.points.iter_mut() {
            *point = (point.clone() * ratio).clamp(-1.0, 1.0);
        }

        self
    }

    /// Returns a slice of the points into a new Stream
    pub fn get_slice(&self, from: usize, length: usize) -> Result<Stream, StreamErr> {
	let to = from + length;
        let length = self.len();

        if from > length || to > length || from > to {
            Err(StreamErr::SliceOutOfBounds)
        } else {
            Ok(Stream::from_points(&self.points[from..to].to_vec()))
        }
    }

    /// Returns a slice of the stream that loops around when going out of bounds
    pub fn get_looped_slice(&self, from: usize, length: usize) -> Stream {
	let stream_length = self.len();
	let mut points = vec![];

	for index in 0..length {
	    let pos = (index + from) % stream_length;
	    points.push(self.get_point(pos).unwrap());
	}

	Stream::from_points(&points)
    }
}

/// Trait to implement conversion from sized types to a f32 Stream
pub trait FromPoints<T: Sized> {
    /// Create new stream based on provided points
    fn from_points(points: &[T]) -> Stream;
}

impl FromPoints<u8> for Stream {
    /// Create new stream based on u8 points,
    /// converts u8 to point value (f32 between -1.0 and 1.0)
    fn from_points(points: &[u8]) -> Stream {
        Stream {
            points: points.iter().copied().map(u8_to_point).collect(),
        }
    }
}

impl FromPoints<i16> for Stream {
    /// Create new stream based on i16 points,
    /// converts i16 to point value (f32 between -1.0 and 1.0)
    fn from_points(points: &[i16]) -> Stream {
        Stream {
            points: points.iter().copied().map(i16_to_point).collect(),
        }
    }
}

impl FromPoints<i32> for Stream {
    /// Create new stream based on i32 points,
    /// converts i32 to point value (f32 between -1.0 and 1.0)
    fn from_points(points: &[i32]) -> Stream {
        Stream {
            points: points.iter().copied().map(i32_to_point).collect(),
        }
    }
}

impl FromPoints<f32> for Stream {
    /// Create new stream based on f32 points
    fn from_points(points: &[f32]) -> Stream {
        // @TODO: clamp values?
        Stream { points: points.to_owned() }
    }
}

/// Convert u8 to point value (f32 between -1.0 and 1.0)
fn u8_to_point(n: u8) -> Point {
    (n as f32 / u8::MAX as f32) * 2.0 - 1.0
}

/// Convert i16 to point value (f32 between -1.0 and 1.0)
fn i16_to_point(n: i16) -> Point {
    n as f32 / i16::MAX as f32
}

/// Convert i32 to point value (f32 between -1.0 and 1.0)
fn i32_to_point(n: i32) -> Point {
    n as f32 / i32::MAX as f32
}

#[cfg(test)]
mod tests {
    #![allow(overflowing_literals)]
    use super::*;

    #[test]
    fn test_mix() {
        let points = [-1.0, -0.5, 0.0, 0.5, 1.0];
        let streams = vec![];
        let mut stream = Stream::from_points(&points);
        stream.mix(&streams);
        assert_eq!(stream.points, vec![-1.0, -0.5, 0.0, 0.5, 1.0]);

        let points = [1.0, 0.2, 1.0, 1.0, 0.2];
        let streams = [&Stream::from_points(&[0.0, 0.0, 0.0, 0.0, 0.0])];
        let mut stream = Stream::from_points(&points);
        stream.mix(&streams);
        assert_eq!(stream.points, vec![1.0, 0.2, 1.0, 1.0, 0.2]);

        let points = [0.1, 0.0, -0.1, -0.2, -0.3];
        let streams = [
            &Stream::from_points(&[0.2, 0.1, 0.0, -0.1, -0.2]),
            &Stream::from_points(&[0.3, 0.2, 0.1, 0.0, -0.1]),
        ];
        let mut stream = Stream::from_points(&points);
        stream.mix(&streams);
        assert_eq!(stream.points, vec![0.6, 0.3, 0.0, -0.3, -0.6]);

        let points = [0.1, 0.0, -0.1, -0.2, -0.3];
        let streams = [
            &Stream::from_points(&[0.2, 0.1, 0.0]),
            &Stream::from_points(&[0.3]),
        ];
        let mut stream = Stream::from_points(&points);
        stream.mix(&streams);
        assert_eq!(stream.points, vec![0.6, 0.1, -0.1, -0.2, -0.3]);
    }

    #[test]
    fn test_amplify() {
        let mut stream = Stream::empty(1);
        stream.amplify(6.0);
        assert_eq!(stream.points, vec![0.0]);

        // 6 dBs should roughly double / half
        let mut stream = Stream::from_points(&[0.1, 0.25, 0.3, -0.1, -0.4]);
        stream.amplify(6.0);
        let rounded_points: Vec<Point> = stream
            .points
            .iter()
            .map(|x| (x * 10.0).round() / 10.0)
            .collect::<Vec<Point>>();
        assert_eq!(rounded_points, vec![0.2, 0.5, 0.6, -0.2, -0.8]);

        let mut stream = Stream::from_points(&[0.4, 0.5, 0.8, -0.3, -0.6]);
        stream.amplify(-6.0);
        let rounded_points: Vec<Point> = stream
            .points
            .iter()
            .map(|x| (x * 100.0).round() / 100.0)
            .collect::<Vec<Point>>();
        assert_eq!(rounded_points, vec![0.2, 0.25, 0.4, -0.15, -0.3]);

        // clamp the value
        let mut stream = Stream::from_points(&[0.1, 0.4, 0.6, -0.2, -0.3, -0.5]);
        stream.amplify(12.0);
        let rounded_points: Vec<Point> = stream
            .points
            .iter()
            .map(|x| (x * 100.0).round() / 100.0)
            .collect::<Vec<Point>>();
        assert_eq!(rounded_points, vec![0.4, 1.0, 1.0, -0.8, -1.0, -1.0]);
    }

    #[test]
    fn test_get_slice() {
        let stream = Stream::from_points(&[0.0, 0.1, 0.2, 0.3, 0.4, 0.5]);
        assert_eq!(stream.get_slice(0, 2).unwrap().points, vec![0.0, 0.1]);

        let stream = Stream::from_points(&[0.0, 0.1, 0.2, 0.3, 0.4, 0.5]);
        assert_eq!(stream.get_slice(2, 3).unwrap().points, vec![0.2, 0.3, 0.4]);

        let stream = Stream::from_points(&[0.0, 0.1, 0.2, 0.3, 0.4, 0.5]);
	let error = stream.get_slice(7, 5);
        assert_eq!(error, Err(StreamErr::SliceOutOfBounds));

        let stream = Stream::from_points(&[0.0, 0.1, 0.2, 0.3, 0.4, 0.5]);
	let error = stream.get_slice(2, 20);
        assert_eq!(error, Err(StreamErr::SliceOutOfBounds));
    }

    #[test]
    fn test_get_looped_slice() {
        let stream = Stream::from_points(&[0.0, 0.1, 0.2, 0.3, 0.4, 0.5]);
        assert_eq!(stream.get_looped_slice(2, 3).points, vec![0.2, 0.3, 0.4]);

        let stream = Stream::from_points(&[0.0, 0.1, 0.2, 0.3, 0.4, 0.5]);
        assert_eq!(
	    stream.get_looped_slice(0, 8).points,
	    vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.0, 0.1]
	);

        let stream = Stream::from_points(&[0.0, 0.1]);
        assert_eq!(
	    stream.get_looped_slice(1, 7).points,
	    vec![0.1, 0.0, 0.1, 0.0, 0.1, 0.0, 0.1]
	);
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
        let stream = Stream::from_points(&[0, 80, 128, 220, 256u8]);
        assert_eq!(
            stream.points,
            vec![-1.0, -0.372549, 0.003921628, 0.7254902, -1.0]
        );
    }

    #[test]
    fn test_from_i16() {
        let stream = Stream::from_points(&[i16::MIN + 1, -1600, 0, 2800, i16::MAX]);
        assert_eq!(
            stream.points,
            vec![-1.0, -0.048829615, 0.0, 0.08545183, 1.0]
        );
    }

    #[test]
    fn test_from_i32() {
        let stream =
            Stream::from_points(&[i32::MIN, -1_147_483_647, 0, 1_147_483_647, i32::MAX]);
        assert_eq!(stream.points, vec![-1.0, -0.5343387, 0.0, 0.5343387, 1.0]);
    }
}
