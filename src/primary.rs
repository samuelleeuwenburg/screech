use crate::graph::{topological_sort, Error as GraphError};
use crate::traits::{Source, Tracker};
use crate::{DynamicTracker, Signal, SignalId};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use rustc_hash::FxHashMap;

/// Main helper struct to render and manage relations between [`crate::traits::Source`] types.
/// Also implements [`crate::traits::Tracker`]
///
/// ```
/// use screech::traits::{FromPoints, Source, Tracker};
/// use screech::{Primary, Signal, DynamicTracker, SignalId};
///
/// struct Osc {
///     pub output: SignalId,
///     voltage: f32,
/// }
///
/// impl Osc {
///     fn new(tracker: &mut dyn Tracker) -> Self {
///         let output = SignalId::new(tracker.create_source_id(), 0);
///         tracker.init_buffer(&output);
///
///         Osc {
///             output,
///             voltage: 0.0,
///         }
///     }
/// }
///
/// impl Source for Osc {
///     fn sample(&mut self, sources: &mut dyn Tracker, _sample_rate: usize) {
///         let mut signal = sources.get_mut_signal(&self.output).unwrap();
///
///         for s in signal.samples.iter_mut() {
///             *s = self.voltage;
///             self.voltage += 0.2;
///
///             if self.voltage >= 1.0 {
///                 self.voltage = 0.0;
///             }
///         }
///     }
///
///     fn get_source_id(&self) -> &usize {
///         self.output.get_source_id()
///     }
///
///     fn get_sources(&self) -> Vec<usize> {
///         vec![]
///     }
/// }
///
///
/// let buffer_size = 4;
/// let sample_rate = 48_000;
///
/// let mut primary = Primary::new(buffer_size, sample_rate);
/// let mut osc = Osc::new(&mut primary);
///
/// primary.monitor_signal(&osc.output);
/// primary.invalidate_cache();
///
/// let mut sources = vec![&mut osc as &mut Source];
///
/// let (left, _right) = primary.sample(&mut sources).unwrap();
/// assert_eq!(left, [0.0, 0.2, 0.4, 0.6]);
///
/// let (left, _right) = primary.sample(&mut sources).unwrap();
/// assert_eq!(left, [0.8, 0.0, 0.2, 0.4]);
///
/// let (left, _right) = primary.sample(&mut sources).unwrap();
/// assert_eq!(left, [0.6, 0.8, 0.0, 0.2]);
///
/// ```
pub struct Primary {
    /// sample rate field used for sampling
    pub sample_rate: usize,
    pub output_buffer_left: Signal,
    pub output_buffer_right: Signal,
    sorted_cache: Option<Vec<usize>>,
    signals_left: Vec<SignalId>,
    signals_right: Vec<SignalId>,
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

impl Primary {
    /// Create new Primary with a default tracker
    pub fn new(buffer_size: usize, sample_rate: usize) -> Self {
        Primary::with_tracker(Box::new(DynamicTracker::new(buffer_size)), sample_rate)
    }

    /// Create a new Primary with a supplied tracker
    ///
    /// ```
    /// use screech::{Primary, BasicTracker};
    ///
    /// let tracker = BasicTracker::<256, 8>::new(8);
    /// let primary = Primary::with_tracker(Box::new(tracker), 48_000);
    /// ```
    pub fn with_tracker(tracker: Box<dyn Tracker>, sample_rate: usize) -> Self {
        Primary {
            output_buffer_left: Signal::empty(*tracker.get_buffer_size()),
            output_buffer_right: Signal::empty(*tracker.get_buffer_size()),
            sample_rate,
            signals_left: vec![],
            signals_right: vec![],
            sorted_cache: None,
            output_mode: OutputMode::Stereo,
            tracker,
        }
    }

    /// invalidate connections cache
    pub fn invalidate_cache(&mut self) {
        self.sorted_cache = None;
    }

    pub fn monitor_signal_stereo(&mut self, &left: &SignalId, &right: &SignalId) -> &mut Self {
        self.signals_left.push(left);
        self.signals_right.push(right);
        self
    }

    pub fn monitor_signal(&mut self, &signal: &SignalId) -> &mut Self {
        self.signals_left.push(signal);
        self.signals_right.push(signal);
        self
    }

    pub fn monitor_remove(&mut self, signal: &SignalId) -> &mut Self {
        self.signals_left.retain(|b| signal != b);
        self.signals_right.retain(|b| signal != b);
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
        unmapped_sources: &mut [&mut dyn Source],
    ) -> Result<(&[f32], &[f32]), Error> {
        if let None = self.sorted_cache {
            let mut graph = FxHashMap::<usize, Vec<usize>>::default();

            for source in unmapped_sources.into_iter() {
                graph.insert(*source.get_source_id(), source.get_sources());
            }

            let sorted = topological_sort(graph).map_err(|e| match e {
                GraphError::NoDirectedAcyclicGraph => Error::CyclicDependencies,
            })?;

            self.sorted_cache = Some(sorted);
        }

        let sorted = &self.sorted_cache.as_ref().unwrap().clone();
        let sample_rate = self.sample_rate;

        for key in sorted.iter() {
            for source in unmapped_sources.iter_mut() {
                if key == source.get_source_id() {
                    source.sample(self, sample_rate);
                }
            }
        }

        for i in 0..*self.get_buffer_size() {
            // left
            let mut left = 0.0;

            for monitored in self.signals_left.iter() {
                let signal = self.get_signal(&monitored).ok_or(Error::MissingMonitor)?;
                left += &signal.samples[i];
            }

            // right
            let mut right = 0.0;

            for monitored in self.signals_right.iter() {
                let signal = self.get_signal(&monitored).ok_or(Error::MissingMonitor)?;
                right += &signal.samples[i];
            }

            match self.output_mode {
                OutputMode::Mono => {
                    self.output_buffer_left.samples[i] = left + right;
                }

                OutputMode::Stereo => {
                    self.output_buffer_right.samples[i] = right;
                    self.output_buffer_left.samples[i] = left;
                }
            }
        }

        Ok((
            &self.output_buffer_left.samples,
            &self.output_buffer_right.samples,
        ))
    }
}

impl Tracker for Primary {
    fn get_buffer_size(&self) -> &usize {
        &self.tracker.get_buffer_size()
    }

    fn create_source_id(&mut self) -> usize {
        self.tracker.create_source_id()
    }

    fn clear_source(&mut self, signal: usize) {
        self.tracker.clear_source(signal)
    }

    fn get_signal(&self, signal: &SignalId) -> Option<&Signal> {
        self.tracker.get_signal(signal)
    }

    fn get_mut_signal(&mut self, signal: &SignalId) -> Option<&mut Signal> {
        self.tracker.get_mut_signal(signal)
    }

    fn init_buffer(&mut self, signal: &SignalId) {
        self.tracker.init_buffer(signal);
    }

    fn resize_buffers(&mut self, buffer_size: usize) {
        self.tracker.resize_buffers(buffer_size);
    }
}
