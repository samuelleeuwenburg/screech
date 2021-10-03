use crate::core::graph::{topological_sort, Error as GraphError};
use crate::core::{DynamicTracker, Point, Signal};
use crate::traits::{Source, Tracker};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use rustc_hash::FxHashMap;

/// Main helper struct to render and manage relations between [`crate::traits::Source`] types.
/// Also implements [`crate::traits::Tracker`]
///
/// ```
/// use screech::traits::{FromPoints, Source};
/// use screech::core::{Primary, Stream, DynamicTracker};
/// use screech::basic::{Clip, Track};
///
/// const BUFFER_SIZE: usize = 4;
/// let sample_rate = 48_000;
///
/// let mut primary = Primary::<BUFFER_SIZE>::new(sample_rate);
///
/// let mut clip_a = Clip::new(&mut primary, Stream::from_points(vec![0.1, 0.2, 0.2, 0.1]));
/// let mut clip_b = Clip::new(&mut primary, Stream::from_points(vec![0.0, 0.0, 0.1, 0.3]));
/// let mut track = Track::new(&mut primary);
///
/// track.add_input(&clip_a);
/// track.add_input(&clip_b);
/// primary.add_monitor(&track);
///
/// assert_eq!(
///     primary.sample(vec![&mut clip_a, &mut clip_b, &mut track]).unwrap(),
///     &[0.1, 0.1, 0.2, 0.2],
/// );
///
/// assert_eq!(
///     primary.sample(vec![&mut clip_a, &mut clip_b, &mut track]).unwrap(),
///     &[0.3, 0.3, 0.4, 0.4],
/// );
///
/// assert_eq!(
///     primary.sample(vec![&mut clip_a, &mut clip_b, &mut track]).unwrap(),
///     &[0.0, 0.0, 0.0, 0.0],
/// );
/// ```
pub struct Primary<const BUFFER_SIZE: usize> {
    buffer: [f32; BUFFER_SIZE],
    /// sample rate field used for sampling
    pub sample_rate: usize,
    monitored_sources: Vec<usize>,
    output_mode: OutputMode,
    tracker: Box<dyn Tracker>,
}

enum OutputMode {
    Mono,
    Stereo,
}

/// Error type for failure to execute [`Primary::sample`]
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Dependency graph contains a cyclic dependency.
    ///
    /// for example, track A -> track B -> track A
    CyclicDependencies,
    /// A monitor has been set using [`Primary::add_monitor`]
    /// which is missing from the sources list
    MissingMonitor,
    /// A source has a dependency in its [`crate::traits::Source::get_sources`]
    /// which is missing from the sources list
    MissingDependency,
}

impl<const BUFFER_SIZE: usize> Primary<BUFFER_SIZE> {
    /// Create new Primary with a default tracker
    pub fn new(sample_rate: usize) -> Self {
        Primary::with_tracker(Box::new(DynamicTracker::new()), sample_rate)
    }

    /// Create a new Primary with a supplied tracker
    ///
    /// ```
    /// use screech::core::{Primary, BasicTracker};
    ///
    /// let tracker = BasicTracker::<256>::new();
    /// let primary = Primary::<128>::with_tracker(Box::new(tracker), 48_000);
    /// ```
    pub fn with_tracker(tracker: Box<dyn Tracker>, sample_rate: usize) -> Self {
        Primary {
            buffer: [0.0; BUFFER_SIZE],
            sample_rate,
            monitored_sources: vec![],
            output_mode: OutputMode::Stereo,
            tracker,
        }
    }

    /// add source to the final output
    pub fn add_monitor(&mut self, source: &dyn Source) -> &mut Self {
        self.monitored_sources.push(source.get_id());
        self
    }

    /// remove source from final output
    pub fn remove_monitor(&mut self, source: &dyn Source) -> &mut Self {
        let a = source.get_id();
        self.monitored_sources.retain(|&b| a != b);
        self
    }

    /// Output a mono signal
    pub fn output_mono(&mut self) -> &mut Self {
        self.output_mode = OutputMode::Mono;
        self
    }

    /// Output an interleaved stereo signal
    pub fn output_stereo(&mut self) -> &mut Self {
        self.output_mode = OutputMode::Stereo;
        self
    }

    /// Sample multiple sources based on their dependencies into a single output vec
    pub fn sample(
        &mut self,
        unmapped_sources: Vec<&mut dyn Source>,
    ) -> Result<&[Point; BUFFER_SIZE], Error> {
        let mut sources = FxHashMap::<usize, &mut dyn Source>::default();
        let mut graph = FxHashMap::<usize, Vec<usize>>::default();

        for source in unmapped_sources {
            graph.insert(source.get_id(), source.get_sources());
            sources.insert(source.get_id(), source);
        }

        let sorted = topological_sort(graph).map_err(|e| match e {
            GraphError::NoDirectedAcyclicGraph => Error::CyclicDependencies,
        })?;

        let mut sorted_sources = vec![];

        for key in sorted {
            let source = sources.remove(&key).ok_or(Error::MissingDependency)?;
            sorted_sources.push(source);
        }

        let sample_rate = self.sample_rate;
        let loop_size = match self.output_mode {
            OutputMode::Mono => BUFFER_SIZE,
            OutputMode::Stereo => BUFFER_SIZE / 2,
        };

        for i in 0..loop_size {
            for source in sorted_sources.iter_mut() {
                source.sample(self, sample_rate);
            }

            let mut signals = Vec::with_capacity(self.monitored_sources.len());
            for &key in self.monitored_sources.iter() {
                signals.push(self.get_signal(key).ok_or(Error::MissingMonitor)?);
            }

            match self.output_mode {
                OutputMode::Mono => {
                    self.buffer[i] = Signal::mix(&signals).sum_points();
                }
                OutputMode::Stereo => {
                    let signal = Signal::mix(&signals);
                    self.buffer[i * 2] = *signal.get_point();
                    self.buffer[i * 2 + 1] =
                        *signal.get_right_point().unwrap_or(signal.get_point());
                }
            };
        }

        Ok(&self.buffer)
    }
}

impl<const A: usize> Tracker for Primary<A> {
    fn create_id(&mut self) -> usize {
        self.tracker.create_id()
    }

    fn clear_id(&mut self, id: usize) {
        self.tracker.clear_id(id)
    }

    fn get_signal(&self, id: usize) -> Option<&Signal> {
        self.tracker.get_signal(id)
    }

    fn set_signal(&mut self, id: usize, signal: Signal) {
        self.tracker.set_signal(id, signal);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic::{Clip, Track};
    use crate::core::Stream;
    use crate::traits::FromPoints;

    #[test]
    fn test_complex_dependencies() {
        let mut primary = Primary::<5>::new(48_000);

        let mut clip_a = Clip::new(&mut primary, Stream::from_points(vec![0.1]));
        let mut clip_b = Clip::new(&mut primary, Stream::from_points(vec![0.0, 0.2]));
        let mut clip_c = Clip::new(&mut primary, Stream::from_points(vec![0.0, 0.0, 0.3]));
        let mut clip_d = Clip::new(&mut primary, Stream::from_points(vec![0.0, 0.0, 0.0, 0.4]));
        let mut clip_e = Clip::new(
            &mut primary,
            Stream::from_points(vec![0.0, 0.0, 0.0, 0.0, 0.5]),
        );

        let mut track_a = Track::new(&mut primary);
        let mut track_b = Track::new(&mut primary);
        let mut track_c = Track::new(&mut primary);
        let mut track_d = Track::new(&mut primary);

        track_a.add_input(&clip_a).add_input(&clip_b);

        track_b.add_input(&track_a).add_input(&clip_c);

        track_c
            .add_input(&track_b)
            .add_input(&clip_d)
            .add_input(&clip_e);

        track_d.add_input(&track_c);

        primary.add_monitor(&track_d);
        primary.output_mono();

        assert_eq!(
            primary.sample(vec![
                &mut clip_a,
                &mut clip_b,
                &mut clip_c,
                &mut clip_d,
                &mut clip_e,
                &mut track_a,
                &mut track_b,
                &mut track_c,
                &mut track_d,
            ]),
            Ok(&[0.1, 0.2, 0.3, 0.4, 0.5]),
        );
    }

    #[test]
    fn test_dependency_failure() {
        let mut primary = Primary::<2>::new(48_000);
        let mut clip_a = Clip::new(&mut primary, Stream::from_points(vec![0.1, 0.2, 0.2, 0.1]));
        let clip_b = Clip::new(&mut primary, Stream::from_points(vec![0.0, 0.0, 0.1, 0.3]));
        let clip_c = Clip::new(&mut primary, Stream::from_points(vec![0.0, 0.0, 0.1, 0.3]));
        let mut track = Track::new(&mut primary);

        track
            .add_input(&clip_a)
            .add_input(&clip_b)
            .add_input(&clip_c);

        primary.add_monitor(&track);

        assert_eq!(
            primary.sample(vec![&mut clip_a, &mut track]),
            Err(Error::MissingDependency),
        )
    }

    #[test]
    fn test_circular_dependency_failure() {
        let mut primary = Primary::<2>::new(48_000);
        let mut track_a = Track::new(&mut primary);
        let mut track_b = Track::new(&mut primary);

        track_a.add_input(&track_b);
        track_b.add_input(&track_a);

        primary.add_monitor(&track_a).add_monitor(&track_b);

        assert_eq!(
            primary.sample(vec![&mut track_a, &mut track_b]),
            Err(Error::CyclicDependencies),
        );
    }

    #[test]
    fn test_circular_missing_monitor_failure() {
        let mut primary = Primary::<2>::new(48_000);
        let track = Track::new(&mut primary);

        primary.add_monitor(&track);

        assert_eq!(primary.sample(vec![]), Err(Error::MissingMonitor),);
    }
}
