use crate::signal::{Signal, SignalErr};
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
/// let mut clip = Clip::new(0, Signal::from_points(&[0.1, 0.2, 0.3, 0.4]));
/// let mut track = Track::new(1);
/// 
/// track.add_input(&clip);
/// primary.add_monitor(&track);
/// 
/// assert_eq!(
///     primary.sample(vec![&mut clip, &mut track]).unwrap(),
///     vec![0.1, 0.1, 0.2, 0.2]
/// );
///
/// ```
pub struct Primary {
    buffer_size: usize,
    monitored_sources: Vec<usize>,
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
    pub fn remove_input(&mut self, source: &dyn Source) -> &mut Self {
	let a = source.get_id();
        self.monitored_sources.retain(|&b| a != b);
	self
    }

    /// attempt to sample sources into a single output
    pub fn sample(
        &self,
        mut unmapped_sources: Vec<&mut dyn Source>,
    ) -> Result<Vec<Point>, SignalErr> {
        // create hashmap based on ids
        let mut sources = HashMap::new();
        let mut signals = HashMap::<usize, Signal>::new();

        for source in unmapped_sources.iter_mut() {
            sources.insert(source.get_id(), source);
        }

        // loop while rendering dependencies into new hashmap of signals
        while sources.len() != signals.len() {
            for (&key, source) in sources.iter_mut() {
                // skip if the signal has already been rendered
                if signals.contains_key(&key) {
                    continue;
                }

                // render only if all dependencies are available
		let dependencies_are_ready = source
		    .get_sources()
		    .iter()
		    .fold(true, |a, b| a && signals.contains_key(b));

                if dependencies_are_ready {
		    let dependencies: Vec<(usize, &Signal)> = source
			.get_sources()
			.iter()
			.filter_map(|&key| signals.get(key).map(|s| (*key, s)))
			.collect();

                    let signal = source.sample(dependencies, self.buffer_size);
                    signals.insert(key, signal);
                }
            }
        }

        // mix result based on monitored sources
        let sources: Vec<&Signal> = self
            .monitored_sources
            .iter()
            .filter_map(|key| signals.get(key))
            .collect();

        let buffer = Signal::mix(&sources);

        buffer.get_interleaved_points()
    }
}
