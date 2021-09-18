use crate::traits::FromPoints;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp;
use libm::powf;

/// Type alias representing audio data.
/// There are no guarantees but ideally this value
/// should never exceed beyond `-1.0` or `1.0`.
pub type Point = f32;

/// Error type for different Stream failures
#[derive(Debug, PartialEq, Clone)]
pub enum StreamErr {
    /// Tried to create slice with out of bounds range
    SliceOutOfBounds,
}

/// Enum representing a stream of audio data
#[derive(Debug, PartialEq, Clone)]
pub enum Stream {
    /// Stream of data points, comparable to an AC signal
    Points(Vec<Point>),
    /// Fixed value, comparable to a DC signal
    Fixed(Point),
    // @TODO: Zero
}

impl Stream {
    /// Create zero initialized (silent) stream
    ///
    /// ```
    /// use screech::stream::Stream;
    ///
    /// assert_eq!(
    ///     Stream::empty(4),
    ///     Stream::Points(vec![0.0, 0.0, 0.0, 0.0]),
    /// )
    /// ```
    pub fn empty(size: usize) -> Self {
        Stream::Points(vec![0.0; size])
    }

    /// Create a fixed stream containing a specific value
    ///
    /// ```
    /// use screech::stream::Stream;
    ///
    /// assert_eq!(
    ///     Stream::fixed(1.0),
    ///     Stream::Fixed(1.0),
    /// )
    /// ```
    pub fn fixed(value: Point) -> Self {
        Stream::Fixed(value)
    }

    /// Returns the length of the stream
    ///
    /// ```
    /// use screech::stream::Stream;
    ///
    /// assert_eq!(Stream::empty(4).len(), 4);
    /// ```
    pub fn len(&self) -> usize {
        match self {
            Stream::Points(points) => points.len(),
            Stream::Fixed(_) => 1,
        }
    }

    //@TODO: pub fn shrink(self, usize) -> Self
    //@TODO: pub fn resize(self, usize) -> Self

    /// Get point for provided position argument, errors when the index does not exist in the stream
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::stream::Stream;
    ///
    /// let stream = Stream::from_points(vec![0.0, 0.1, 0.2]);
    ///
    /// assert_eq!(stream.get_point(1), Some(&0.1));
    /// assert_eq!(stream.get_point(10), None);
    /// ```
    pub fn get_point(&self, position: usize) -> Option<&Point> {
        match self {
            Stream::Points(points) => points.get(position),
            Stream::Fixed(point) => Some(point),
        }
    }

    /// Get points inside the Stream
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::stream::Stream;
    ///
    /// let stream = Stream::from_points(vec![0.0, 0.1, 0.2]);
    ///
    /// assert_eq!(stream.get_points().unwrap(), &[0.0, 0.1, 0.2]);
    /// ```
    pub fn get_points(&self) -> Option<&Vec<Point>> {
        match self {
            Stream::Points(points) => Some(points),
            Stream::Fixed(_) => None,
        }
    }

    /// Mix multiple streams into a new stream
    ///
    /// **note** the size of the resulting stream is equal to
    /// the longest stream in the `Vec`
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::stream::Stream;
    ///
    /// let streams = [
    ///     &Stream::from_points(vec![0.1, 0.0, -0.1, -0.2, -0.3]),
    ///     &Stream::from_points(vec![0.2, 0.1, 0.0]),
    ///     &Stream::from_points(vec![0.3]),
    /// ];
    ///
    /// let result = Stream::mix(&streams);
    ///
    /// assert_eq!(result.get_points().unwrap(), &[0.6, 0.1, -0.1, -0.2, -0.3]);
    /// ```
    pub fn mix(streams: &[&Stream]) -> Self {
        let length = streams.iter().fold(0, |a, b| cmp::max(a, b.len()));
        let mut points = vec![];

        for pos in 0..length {
            let point = streams
                .iter()
                .fold(0.0, |xs, x| xs + x.get_point(pos).unwrap_or(&0.0));

            points.push(point);
        }

        Stream::from_points(points)
    }

    /// Same as mix, but mixes sources into the current stream
    ///
    /// **note** the size of the stream will be unchanged
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::stream::Stream;
    ///
    /// let streams = [
    ///     &Stream::from_points(vec![0.2, 0.1, 0.0]),
    ///     &Stream::from_points(vec![0.3]),
    /// ];
    ///
    /// let mut stream = Stream::from_points(vec![0.1, 0.0, -0.1, -0.2, -0.3])
    ///     .mix_into(&streams);
    ///
    /// assert_eq!(stream.get_points().unwrap(), &[0.6, 0.1, -0.1, -0.2, -0.3]);
    /// ```
    pub fn mix_into(self, streams: &[&Stream]) -> Self {
        match self {
            Stream::Fixed(point) => Stream::Fixed(point),
            Stream::Points(mut points) => {
                for (pos, point) in points.iter_mut().enumerate() {
                    let sum = streams
                        .iter()
                        .fold(0.0, |xs, x| xs + x.get_point(pos).unwrap_or(&0.0));

                    *point += sum;
                }

                Stream::Points(points)
            }
        }
    }

    /// Map values inside stream
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::stream::Stream;
    ///
    /// let stream = Stream::from_points(vec![0.1, 0.2, 0.3])
    ///     .map(|point| point * 2.0);
    ///
    /// assert_eq!(stream.get_points().unwrap(), &[0.2, 0.4, 0.6]);
    /// ```
    pub fn map<F>(self, f: F) -> Self
    where
        F: Fn(Point) -> Point,
    {
        match self {
            Stream::Fixed(point) => Stream::Fixed(f(point)),
            Stream::Points(mut points) => {
                for point in points.iter_mut() {
                    *point = f(*point);
                }
                Stream::Points(points)
            }
        }
    }

    /// Apply generic manipulation of another stream onto the current one
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::stream::Stream;
    ///
    /// let stream = Stream::from_points(vec![0.1, 0.2, 0.3, 0.4]);
    /// let gate_cv = Stream::from_points(vec![1.0, 0.0, 1.0, 0.0]);
    ///
    /// let result = stream.apply(&gate_cv, |signal_point, cv_point| {
    ///     if cv_point == 1.0 { signal_point } else { 0.0 }
    /// });
    /// assert_eq!(result.get_points().unwrap(), &[0.1, 0.0, 0.3, 0.0]);
    /// ```
    pub fn apply<F>(self, stream: &Self, f: F) -> Self
    where
        F: Fn(Point, Point) -> Point,
    {
        match (&self, stream) {
            (Stream::Fixed(a), Stream::Fixed(b)) => Stream::fixed(f(*a, *b)),
            _ => {
                let mut points = match self {
                    Stream::Fixed(point) => vec![point],
                    Stream::Points(points) => points,
                };

                let length = cmp::max(points.len(), stream.len());
                points.resize(length, 0.0);

                for (pos, point) in points.iter_mut().enumerate() {
                    let b = stream.get_point(pos).unwrap_or(&0.0);
                    *point = f(*point, *b);
                }

                Stream::Points(points)
            }
        }
    }

    /// Amplify a stream by decibels
    ///
    /// **note** clamps values at -1.0 and 1.0
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::stream::Stream;
    ///
    /// // 6 dBs should roughly double / half
    /// let stream = Stream::from_points(vec![0.1, 0.3, 0.6, -0.1, -0.4, -0.8]).amplify(6.0);
    ///
    /// assert_eq!(stream.get_points().unwrap(), &[0.19952624, 0.59857875, 1.0, -0.19952624, -0.79810494, -1.0]);
    /// ```
    pub fn amplify(self, db: f32) -> Self {
        let ratio = powf(10.0, db / 20.0);
        self.map(|point| (point * ratio).clamp(-1.0, 1.0))
    }

    /// Amplify a signal by a another stream as CV,
    /// with a function on how to convert the CV [`Point`]
    /// to decibels
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::stream::Stream;
    ///
    /// let cv = Stream::from_points(vec![1.0, -1.0, 1.0, -1.0]);
    /// let stream = Stream::from_points(vec![0.1, 0.1, 0.1, 0.1, 0.1, 0.1])
    ///     .amplify_with_cv(&cv, |p| p * 6.0); // amplify between -6dB and +6dB
    ///
    /// assert_eq!(stream.get_points().unwrap(), &[0.19952624, 0.050118722, 0.19952624, 0.050118722, 0.1, 0.1]);
    /// ```
    pub fn amplify_with_cv<F>(self, cv: &Self, point_to_db: F) -> Self
    where
        F: Fn(Point) -> f32,
    {
        self.apply(cv, |point, cv| {
            let db = point_to_db(cv);
            let ratio = powf(10.0, db / 20.0);
            (point * ratio).clamp(-1.0, 1.0)
        })
    }

    /// Invert the phase of the signal
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::stream::Stream;
    ///
    /// let stream = Stream::from_points(vec![0.3, 0.2, 0.1, 0.0, -0.1, -0.2]).invert();
    ///
    /// assert_eq!(stream.get_points().unwrap(), &[-0.3, -0.2, -0.1, 0.0, 0.1, 0.2]);
    /// ```
    pub fn invert(self) -> Self {
        self.map(|point| -point)
    }

    /// Returns a slice of the points into a new Stream
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::stream::Stream;
    ///
    /// let stream = Stream::from_points(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8])
    ///     .slice(2, 3)
    ///     .unwrap();
    ///
    /// assert_eq!(stream.get_points().unwrap(), &[0.3, 0.4, 0.5]);
    /// ```
    pub fn slice(self, from: usize, length: usize) -> Result<Self, StreamErr> {
        let to = from + length;
        let length = self.len();

        match self {
            Stream::Fixed(point) => Ok(Stream::Points(vec![point; length])),
            Stream::Points(points) => {
                if from > length || to > length || from > to {
                    Err(StreamErr::SliceOutOfBounds)
                } else {
                    Ok(Stream::from_points(points[from..to].to_vec()))
                }
            }
        }
    }

    /// Returns a slice of the stream that loops around when going out of bounds
    ///
    /// ```
    /// use screech::traits::FromPoints;
    /// use screech::stream::Stream;
    ///
    /// let stream = Stream::from_points(vec![0.1, 0.2, 0.3])
    ///     .looped_slice(0, 8);
    ///
    /// assert_eq!(stream.get_points().unwrap(), &[0.1, 0.2, 0.3, 0.1, 0.2, 0.3, 0.1, 0.2]);
    /// ```
    pub fn looped_slice(self, from: usize, length: usize) -> Self {
        let stream_length = self.len();
        let mut points = vec![];

        for index in 0..length {
            let pos = (index + from) % stream_length;
            points.push(*self.get_point(pos).unwrap());
        }

        Stream::from_points(points)
    }
}

impl FromPoints<u8, Stream> for Stream {
    /// Create new stream based on u8 points,
    /// converts u8 to point value (f32 between -1.0 and 1.0)
    fn from_points(points: Vec<u8>) -> Stream {
        Stream::Points(points.iter().copied().map(u8_to_point).collect())
    }
}

impl FromPoints<i16, Stream> for Stream {
    /// Create new stream based on i16 points,
    /// converts i16 to point value (f32 between -1.0 and 1.0)
    fn from_points(points: Vec<i16>) -> Stream {
        Stream::Points(points.iter().copied().map(i16_to_point).collect())
    }
}

impl FromPoints<i32, Stream> for Stream {
    /// Create new stream based on i32 points,
    /// converts i32 to point value (f32 between -1.0 and 1.0)
    fn from_points(points: Vec<i32>) -> Stream {
        Stream::Points(points.iter().copied().map(i32_to_point).collect())
    }
}

impl FromPoints<f32, Stream> for Stream {
    /// Create new stream based on f32 points
    fn from_points(points: Vec<f32>) -> Stream {
        // @TODO: clamp values?
        Stream::Points(points)
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
        let stream = Stream::mix(&[
            &Stream::from_points(vec![1.0, 0.2, 1.0, 1.0, 0.2]),
            &Stream::from_points(vec![0.0, 0.0, 0.0, 0.0, 0.0]),
        ]);

        assert_eq!(stream.get_points().unwrap(), &[1.0, 0.2, 1.0, 1.0, 0.2]);

        let stream = Stream::mix(&[
            &Stream::from_points(vec![0.1, 0.0, -0.1, -0.2, -0.3]),
            &Stream::from_points(vec![0.2, 0.1, 0.0, -0.1, -0.2]),
            &Stream::from_points(vec![0.3, 0.2, 0.1, 0.0, -0.1]),
        ]);

        assert_eq!(stream.get_points().unwrap(), &[0.6, 0.3, 0.0, -0.3, -0.6]);

        let stream = Stream::mix(&[
            &Stream::from_points(vec![0.1, 0.0, -0.1, -0.2, -0.3]),
            &Stream::from_points(vec![0.2, 0.1, 0.0]),
            &Stream::from_points(vec![0.3]),
        ]);

        assert_eq!(stream.get_points().unwrap(), &[0.6, 0.1, -0.1, -0.2, -0.3]);
    }

    #[test]
    fn test_amplify() {
        let stream = Stream::empty(1).amplify(6.0);
        assert_eq!(stream.get_points().unwrap(), &[0.0]);

        // 6 dBs should roughly double / half
        let stream = Stream::from_points(vec![0.1, 0.25, 0.3, -0.1, -0.4]).amplify(6.0);

        let rounded_points: Vec<Point> = stream
            .get_points()
            .unwrap()
            .iter()
            .map(|x| (x * 10.0).round() / 10.0)
            .collect::<Vec<Point>>();

        assert_eq!(rounded_points, vec![0.2, 0.5, 0.6, -0.2, -0.8]);

        let stream = Stream::from_points(vec![0.4, 0.5, 0.8, -0.3, -0.6]).amplify(-6.0);

        let rounded_points: Vec<Point> = stream
            .get_points()
            .unwrap()
            .iter()
            .map(|x| (x * 100.0).round() / 100.0)
            .collect::<Vec<Point>>();
        assert_eq!(rounded_points, vec![0.2, 0.25, 0.4, -0.15, -0.3]);

        // clamp the value
        let stream = Stream::from_points(vec![0.1, 0.4, 0.6, -0.2, -0.3, -0.5]).amplify(12.0);

        let rounded_points: Vec<Point> = stream
            .get_points()
            .unwrap()
            .iter()
            .map(|x| (x * 100.0).round() / 100.0)
            .collect::<Vec<Point>>();
        assert_eq!(rounded_points, vec![0.4, 1.0, 1.0, -0.8, -1.0, -1.0]);
    }

    #[test]
    fn test_slice() {
        let stream = Stream::from_points(vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5]);
        assert_eq!(stream.slice(0, 2).unwrap(), Stream::Points(vec![0.0, 0.1]));

        let stream = Stream::from_points(vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5]);
        assert_eq!(
            stream.slice(2, 3).unwrap(),
            Stream::Points(vec![0.2, 0.3, 0.4])
        );

        let stream = Stream::from_points(vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5]);
        let error = stream.slice(7, 5);
        assert_eq!(error, Err(StreamErr::SliceOutOfBounds));

        let stream = Stream::from_points(vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5]);
        let error = stream.slice(2, 20);
        assert_eq!(error, Err(StreamErr::SliceOutOfBounds));
    }

    #[test]
    fn test_looped_slice() {
        let stream = Stream::from_points(vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5]);
        assert_eq!(
            stream.looped_slice(2, 3),
            Stream::Points(vec![0.2, 0.3, 0.4])
        );

        let stream = Stream::from_points(vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5]);
        assert_eq!(
            stream.looped_slice(0, 8),
            Stream::Points(vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.0, 0.1])
        );

        let stream = Stream::from_points(vec![0.0, 0.1]);
        assert_eq!(
            stream.looped_slice(1, 7),
            Stream::Points(vec![0.1, 0.0, 0.1, 0.0, 0.1, 0.0, 0.1])
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
        let stream = Stream::from_points(vec![0, 80, 128, 220, 256u8]);
        assert_eq!(
            stream,
            Stream::Points(vec![-1.0, -0.372549, 0.003921628, 0.7254902, -1.0])
        );
    }

    #[test]
    fn test_from_i16() {
        let stream = Stream::from_points(vec![i16::MIN + 1, -1600, 0, 2800, i16::MAX]);
        assert_eq!(
            stream,
            Stream::Points(vec![-1.0, -0.048829615, 0.0, 0.08545183, 1.0])
        );
    }

    #[test]
    fn test_from_i32() {
        let stream =
            Stream::from_points(vec![i32::MIN, -1_147_483_647, 0, 1_147_483_647, i32::MAX]);
        assert_eq!(
            stream,
            Stream::Points(vec![-1.0, -0.5343387, 0.0, 0.5343387, 1.0])
        );
    }
}
