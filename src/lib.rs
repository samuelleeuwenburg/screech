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

/// common traits used throughout the library
pub mod traits;

/// Identifier used for keeping track of signals
pub type Input = signal_id::SignalId;

/// Identifier used for keeping track of signals
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
pub enum ScreechError {
    /// Dependency graph contains a cyclic dependency.
    ///
    /// for example, track A -> track B -> track A
    CyclicDependencies,
    /// Output buffer is missing but assigned to an input
    MissingOutput,
    /// no Input found for mains set with [`Screech::create_main_out`]
    MissingInput,
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

    /// create new unique identifier
    pub fn create_source_id(&mut self) -> usize {
        self.tracker.create_source_id()
    }

    /// create new main output based on `&'static str` identifier
    pub fn create_main_out(&mut self, signal_id: &'static str) {
        let out = SignalId::new(self.id, signal_id);
        self.tracker.init_output(&out);
        self.tracker.init_input(&out);
        self.outs.push(out);
    }

    /// return output [`Signal`] based on `&'static str` identifier
    pub fn get_main_out(&self, signal_id: &'static str) -> Option<&Signal> {
        self.outs
            .iter()
            .find(|s| s.get_signal_id() == signal_id)
            .and_then(|out| self.tracker.get_output(&out))
    }

    /// create and initialize a new input
    pub fn init_input(&mut self, source_id: &usize, signal_id: &'static str) -> Input {
        let input = Input::new(*source_id, signal_id);
        self.tracker.init_input(&input);
        input
    }

    /// create and initialize a new output
    pub fn init_output(&mut self, source_id: &usize, signal_id: &'static str) -> Output {
        let output = Output::new(*source_id, signal_id);
        self.tracker.init_output(&output);
        output
    }

    /// connect an [`Output`] to an [`Input`]
    pub fn connect_signal(&mut self, output: &Output, input: &Input) {
        self.tracker.connect_signal(output, input);
    }

    /// connect an [`Output`] to a main output buffer
    pub fn connect_signal_to_main_out(&mut self, output: &Output, signal_id: &'static str) {
        if let Some(input) = self.outs.iter().find(|s| s.get_signal_id() == signal_id) {
            self.tracker.connect_signal(output, input);
        }
    }

    /// Sample multiple sources based on their dependencies into [`Signal`]s stored in a
    /// [`traits::Tracker`]
    pub fn sample(&mut self, unmapped_sources: &mut [&mut dyn Source]) -> Result<(), ScreechError> {
        if let None = self.sorted_cache {
            let mut graph = FxHashMap::<usize, Vec<usize>>::default();

            for source in unmapped_sources.into_iter() {
                let id = source.get_source_id();
                let sources = self.tracker.get_sources(id);
                graph.insert(*id, sources);
            }

            let sorted = topological_sort(graph).map_err(|e| match e {
                GraphError::NoDirectedAcyclicGraph => ScreechError::CyclicDependencies,
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
                let inputs = self
                    .tracker
                    .get_input(&out)
                    .ok_or(ScreechError::MissingInput)?
                    .clone();
                let mut point = 0.0;

                for input in inputs {
                    if let Some(input) = self.tracker.get_output(&input) {
                        point += input.samples[i];
                    }
                }

                let output_signal = self
                    .tracker
                    .get_mut_output(&out)
                    .ok_or(ScreechError::MissingOutput)?;
                output_signal.samples[i] = point;
            }
        }

        Ok(())
    }
}
