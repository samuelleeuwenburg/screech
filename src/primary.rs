use crate::graph::{topological_sort, Error as GraphError};
use crate::signal::Signal;
use crate::stream::Point;
use crate::traits::{Source, Tracker};
use alloc::vec;
use alloc::vec::Vec;
use rustc_hash::FxHashMap;

/// ```
/// use screech::primary::Primary;
/// use screech::track::Track;
/// use screech::clip::Clip;
/// use screech::stream::Stream;
/// use screech::traits::{FromPoints, Source};
///
/// let buffer_size = 2;
/// let sample_rate = 48_000;
///
/// let mut primary = Primary::new(buffer_size, sample_rate);
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
///     vec![0.1, 0.1, 0.2, 0.2],
/// );
///
/// assert_eq!(
///     primary.sample(vec![&mut clip_a, &mut clip_b, &mut track]).unwrap(),
///     vec![0.3, 0.3, 0.4, 0.4],
/// );
///
/// assert_eq!(
///     primary.sample(vec![&mut clip_a, &mut clip_b, &mut track]).unwrap(),
///     vec![0.0, 0.0, 0.0, 0.0],
/// );
/// ```
pub struct Primary {
    buffer_size: usize,
    sample_rate: usize,
    monitored_sources: Vec<usize>,
    id_position: usize,
    signals: FxHashMap<usize, Signal>,
    output_mode: OutputMode,
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
    /// Unable to build final stereo stream
    /// see [`crate::signal::Signal::get_interleaved_points`] for more information
    UnableToBuildFinalStream,
}

impl Primary {
    /// Create new Primary "channel"
    pub fn new(buffer_size: usize, sample_rate: usize) -> Self {
        Primary {
            buffer_size,
            sample_rate,
            monitored_sources: vec![],
            id_position: 0,
            signals: FxHashMap::default(),
            output_mode: OutputMode::Stereo,
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

    pub fn output_mono(&mut self) -> &mut Self {
        self.output_mode = OutputMode::Mono;
        self
    }

    pub fn output_stereo(&mut self) -> &mut Self {
        self.output_mode = OutputMode::Stereo;
        self
    }

    pub fn sample(&mut self, unmapped_sources: Vec<&mut dyn Source>) -> Result<Vec<Point>, Error> {
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

        let mut final_stream = Vec::with_capacity(self.buffer_size);
        let sample_rate = self.sample_rate;

        for _ in 0..self.buffer_size {
            for source in sorted_sources.iter_mut() {
                source.sample(self, sample_rate);
            }

            let signals: Vec<&Signal> = self
                .monitored_sources
                .iter()
                .filter_map(|&k| self.get_signal(k))
                .collect();

            final_stream.push(Signal::mix(&signals));
        }

        let mut output = Vec::with_capacity(self.buffer_size * 2);

        for signal in final_stream {
            match self.output_mode {
                OutputMode::Mono => {
                    let point = signal.sum_points();
                    output.push(point);
                }
                OutputMode::Stereo => {
                    let left = signal.get_point();
                    let right = signal.get_right_point().unwrap_or(signal.get_point());

                    output.push(*left);
                    output.push(*right);
                }
            }
        }

        Ok(output)
    }
}

impl Tracker for Primary {
    fn create_id(&mut self) -> usize {
        // @TODO: this is pretty naive, best keep track of ids somewhere in a vec
        let id = self.id_position;
        self.id_position += 1;
        id
    }

    fn get_signal(&self, id: usize) -> Option<&Signal> {
        self.signals.get(&id)
    }

    fn set_signal(&mut self, id: usize, signal: Signal) {
        self.signals.insert(id, signal);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clip::Clip;
    use crate::signal::Signal;
    use crate::stream::Stream;
    use crate::track::Track;
    use crate::traits::FromPoints;

    #[test]
    fn test_complex_dependencies() {
        let buffer_size = 5;
        let sample_rate = 48_000;

        let mut primary = Primary::new(buffer_size, sample_rate);

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
            Ok(vec![0.1, 0.1, 0.2, 0.2, 0.3, 0.3, 0.4, 0.4, 0.5, 0.5,]),
        );
    }

    #[test]
    fn test_dependency_failure() {
        let buffer_size = 2;
        let sample_rate = 48_000;

        let mut primary = Primary::new(buffer_size, sample_rate);
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
        let buffer_size = 2;
        let sample_rate = 48_000;

        let mut primary = Primary::new(buffer_size, sample_rate);
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
}
