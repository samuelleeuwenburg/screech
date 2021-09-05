use crate::graph::{topological_sort, Error as GraphError};
use crate::signal::Signal;
use crate::stream::Point;
use crate::traits::Source;
use alloc::vec;
use alloc::vec::Vec;
use hashbrown::HashMap;

/// ```
/// use screech::primary::Primary;
/// use screech::track::Track;
/// use screech::clip::Clip;
/// use screech::signal::Signal;
/// use screech::traits::{FromPoints, Source};
///
/// let buffer_size = 2;
///
/// let mut primary = Primary::new(buffer_size);
/// let mut clip_a = Clip::new(0, Signal::from_points(&[0.1, 0.2, 0.2, 0.1]));
/// let mut clip_b = Clip::new(1, Signal::from_points(&[0.0, 0.0, 0.1, 0.3]));
/// let mut track = Track::new(2);
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
    monitored_sources: Vec<usize>,
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
    pub fn new(buffer_size: usize) -> Self {
        Primary {
            buffer_size,
            monitored_sources: vec![],
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

    /// attempt to sample sources into a single output
    pub fn sample(&self, unmapped_sources: Vec<&mut dyn Source>) -> Result<Vec<Point>, Error> {
        let mut sources = HashMap::<usize, &mut dyn Source>::new();
        let mut signals = HashMap::<usize, Signal>::new();
        let mut graph = HashMap::<usize, Vec<usize>>::new();

        for source in unmapped_sources {
            graph.insert(source.get_id(), source.get_sources());
            sources.insert(source.get_id(), source);
        }

        let mut sorted = topological_sort(graph).map_err(|e| match e {
            GraphError::NoDirectedAcyclicGraph => Error::CyclicDependencies,
        })?;

        // reverse the dependency graph
        sorted.reverse();

        for key in sorted {
            // build signals
            let source = sources.get_mut(&key).ok_or(Error::MissingDependency)?;
            let dependencies: Vec<(usize, &Signal)> = source
                .get_sources()
                .iter()
                .filter_map(|&key| signals.get(&key).map(|s| (key, s)))
                .collect();

            let signal = source.sample(dependencies, self.buffer_size);
            signals.insert(key, signal);
        }

        // mix result based on monitored sources
        let mut monitored_signals = vec![];

        for key in &self.monitored_sources {
            let s = signals.get(&key).ok_or(Error::MissingMonitor)?;
            monitored_signals.push(s);
        }

        Signal::mix(&monitored_signals)
            .get_interleaved_points()
            .map_err(|_| Error::UnableToBuildFinalStream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clip::Clip;
    use crate::signal::Signal;
    use crate::track::Track;
    use crate::traits::FromPoints;

    #[test]
    fn test_complex_dependencies() {
        let buffer_size = 5;

        let mut primary = Primary::new(buffer_size);

        let mut clip_a = Clip::new(0, Signal::from_points(&[0.1]));
        let mut clip_b = Clip::new(1, Signal::from_points(&[0.0, 0.2]));
        let mut clip_c = Clip::new(2, Signal::from_points(&[0.0, 0.0, 0.3]));
        let mut clip_d = Clip::new(3, Signal::from_points(&[0.0, 0.0, 0.0, 0.4]));
        let mut clip_e = Clip::new(4, Signal::from_points(&[0.0, 0.0, 0.0, 0.0, 0.5]));

        let mut track_a = Track::new(5);
        let mut track_b = Track::new(6);
        let mut track_c = Track::new(7);
        let mut track_d = Track::new(8);

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

        let mut primary = Primary::new(buffer_size);
        let mut clip_a = Clip::new(0, Signal::from_points(&[0.1, 0.2, 0.2, 0.1]));
        let clip_b = Clip::new(1, Signal::from_points(&[0.0, 0.0, 0.1, 0.3]));
        let clip_c = Clip::new(2, Signal::from_points(&[0.0, 0.0, 0.1, 0.3]));
        let mut track = Track::new(3);

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
        // let buffer_size = 2;

        // let mut primary = Primary::new(buffer_size);
        // let mut track_a = Track::new(0);
        // let mut track_b = Track::new(1);

        // track_a.add_input(&track_b);
        // track_b.add_input(&track_a);

        // primary
        //     .add_monitor(&track_a)
        //     .add_monitor(&track_b);

        // assert_eq!(
        //     primary.sample(vec![&mut track_a, &mut track_b]),
        //     Ok(vec![0.0, 0.0]),
        // );
    }
}
