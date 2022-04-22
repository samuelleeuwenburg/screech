//! A collection of helpers for handling audio data in real time
//!
//! **NOTE! this library is unfinished, incomplete and most likely contains bugs!**

#![no_std]
#![warn(missing_docs)]

extern crate alloc;

mod graph;
mod signal;
mod signal_id;
mod tracker;
pub mod traits;

pub type Input = signal_id::SignalId;
pub type Output = signal_id::SignalId;

pub use signal::Signal;
pub use tracker::{BasicTracker, DynamicTracker};

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use graph::{topological_sort, Error as GraphError};
use rustc_hash::FxHashMap;
use signal_id::SignalId;
use traits::{Source, Tracker};

/// Main helper struct to render and manage relations between [`crate::traits::Source`] types.
///
/// ```
/// use screech::traits::{FromPoints, Source, Tracker};
/// use screech::{Screech, Input, Output, Signal, DynamicTracker};
///
/// struct Osc {
///     pub id: usize,
///     pub output: Output,
///     voltage: f32,
/// }
///
/// impl Osc {
///     fn new(screech: &mut Screech) -> Self {
///         let id = screech.create_source_id();
///
///         Osc {
///             id,
///             output: screech.init_output(&id, "output"),
///             voltage: 0.0,
///         }
///     }
/// }
///
/// struct Amp {
///     pub id: usize,
///     pub input: Input,
///     pub output: Output,
/// }
///
/// impl Amp {
///     fn new(screech: &mut Screech) -> Self {
///         let id = screech.create_source_id();
///
///         Amp {
///             id,
///             input: screech.init_input(&id, "input"),
///             output: screech.init_output(&id, "output"),
///         }
///     }
/// }
///
/// impl Source for Osc {
///     fn sample(&mut self, sources: &mut dyn Tracker, _sample_rate: usize) {
///         let mut signal = sources.get_mut_output(&self.output).unwrap();
///
///         for s in signal.samples.iter_mut() {
///             *s = self.voltage;
///             self.voltage += 0.1;
///
///             if self.voltage >= 0.5 {
///                 self.voltage = 0.0;
///             }
///         }
///     }
///
///     fn get_source_id(&self) -> &usize {
///         &self.id
///     }
/// }
///
/// impl Source for Amp {
///     fn sample(&mut self, sources: &mut dyn Tracker, _sample_rate: usize) {
///         for i in 0..*sources.get_buffer_size() {
///             let mut signal = 0.0;
///
///             for input in sources.get_input(&self.input).unwrap().into_iter() {
///                 let s = sources.get_output(&input)
///                     .and_then(|o| o.samples.get(i))
///                     .unwrap_or(&0.0);
///
///                 signal += s * 2.0;
///             }
///
///             let output = sources.get_mut_output(&self.output).unwrap();
///             output.samples[i] = signal;
///         }
///     }
///
///     fn get_source_id(&self) -> &usize {
///         &self.id
///     }
/// }
///
/// let buffer_size = 4;
/// let sample_rate = 48_000;
///
/// // setup entities
/// let mut screech = Screech::new(buffer_size, sample_rate);
/// let mut osc = Osc::new(&mut screech);
/// let mut amp = Amp::new(&mut screech);
///
/// // setup connections
/// screech.create_main_out("mono_out");
/// screech.connect_signal(&osc.output, &amp.input);
/// screech.connect_signal_to_main_out(&amp.output, "mono_out");
///
/// let mut sources = vec![&mut osc as &mut dyn Source, &mut amp as &mut dyn Source];
///
/// screech.sample(&mut sources).unwrap();
/// assert_eq!(screech.get_main_out("mono_out").unwrap().samples, [0.0, 0.2, 0.4, 0.6]);
///
/// screech.sample(&mut sources).unwrap();
/// assert_eq!(screech.get_main_out("mono_out").unwrap().samples, [0.8, 0.0, 0.2, 0.4]);
///
/// screech.sample(&mut sources).unwrap();
/// assert_eq!(screech.get_main_out("mono_out").unwrap().samples, [0.6, 0.8, 0.0, 0.2]);
///
/// ```
pub struct Screech {
    /// sample rate field used for sampling
    pub sample_rate: usize,
    outs: Vec<SignalId>,
    id: usize,
    sorted_cache: Option<Vec<usize>>,
    tracker: Box<dyn Tracker>,
}

unsafe impl Send for Screech {}

/// Error type for failure to execute [`Screech::sample`]
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Dependency graph contains a cyclic dependency.
    ///
    /// for example, track A -> track B -> track A
    CyclicDependencies,
    /// A monitor has been set using [`Screech::add_monitor`]
    /// which is missing from the sources list
    MissingMonitor,
    /// A source has a dependency in its [`crate::traits::Source::get_sources`]
    /// which is missing from the sources list
    MissingDependency,
}

impl Screech {
    /// Create new Screech instance with a default tracker
    pub fn new(buffer_size: usize, sample_rate: usize) -> Self {
        Self::with_tracker(Box::new(DynamicTracker::new(buffer_size)), sample_rate)
    }

    /// Create a new Screech instance with a supplied tracker
    ///
    /// ```
    /// use screech::{Screech, BasicTracker};
    ///
    /// let tracker = BasicTracker::<256>::new(8);
    /// let screech = Screech::with_tracker(Box::new(tracker), 48_000);
    /// ```
    pub fn with_tracker(mut tracker: Box<dyn Tracker>, sample_rate: usize) -> Self {
        Screech {
            id: tracker.create_source_id(),
            outs: vec![],
            sample_rate,
            sorted_cache: None,
            tracker,
        }
    }

    /// invalidate connections cache
    pub fn invalidate_cache(&mut self) {
        self.sorted_cache = None;
    }

    pub fn create_source_id(&mut self) -> usize {
        self.tracker.create_source_id()
    }

    pub fn create_main_out(&mut self, signal_id: &'static str) {
        let out = SignalId::new(self.id, signal_id);
        self.tracker.init_output(&out);
        self.tracker.init_input(&out);
        self.outs.push(out);
    }

    pub fn get_main_out(&self, signal_id: &'static str) -> Option<&Signal> {
        self.outs
            .iter()
            .find(|s| s.get_signal_id() == signal_id)
            .and_then(|out| self.tracker.get_output(&out))
    }

    pub fn init_input(&mut self, source_id: &usize, signal_id: &'static str) -> Input {
        let input = Input::new(*source_id, signal_id);
        self.tracker.init_input(&input);
        input
    }

    pub fn init_output(&mut self, source_id: &usize, signal_id: &'static str) -> Output {
        let output = Output::new(*source_id, signal_id);
        self.tracker.init_output(&output);
        output
    }

    pub fn connect_signal(&mut self, output: &Output, input: &Input) {
        self.tracker.connect_signal(output, input);
    }

    pub fn connect_signal_to_main_out(&mut self, output: &Output, signal_id: &'static str) {
        if let Some(input) = self.outs.iter().find(|s| s.get_signal_id() == signal_id) {
            self.tracker.connect_signal(output, input);
        }
    }

    /// Sample multiple sources based on their dependencies into a single output vec
    pub fn sample(&mut self, unmapped_sources: &mut [&mut dyn Source]) -> Result<(), Error> {
        if let None = self.sorted_cache {
            let mut graph = FxHashMap::<usize, Vec<usize>>::default();

            for source in unmapped_sources.into_iter() {
                let id = source.get_source_id();
                let sources = self.tracker.get_sources(id);
                graph.insert(*id, sources);
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
                    source.sample(self.tracker.as_mut(), sample_rate);
                }
            }
        }

        for i in 0..*self.tracker.get_buffer_size() {
            for out in &self.outs {
                let inputs = self.tracker.get_input(&out).unwrap().clone();
                let mut point = 0.0;

                for input in inputs {
                    if let Some(input) = self.tracker.get_output(&input) {
                        point += input.samples[i];
                    }
                }

                let output_signal = self.tracker.get_mut_output(&out).unwrap();
                output_signal.samples[i] = point;
            }
        }

        Ok(())
    }
}
