use crate::traits::Tracker;
use crate::{Input, Output, Signal};
use alloc::vec;
use alloc::vec::Vec;
use rustc_hash::FxHashMap;

/// Basic dynamically sized tracker for the creation of unique ids
/// and to keep track of signals belonging to a certain id
///
/// ```
/// use screech::traits::Tracker;
/// use screech::DynamicTracker;
///
/// let mut tracker = DynamicTracker::new(128);
///
/// // the resulting id is irrelevant as long as it is unique
/// assert_eq!(tracker.create_source_id() != tracker.create_source_id(), true);
/// ```
pub struct DynamicTracker {
    id_position: usize,
    buffer_size: usize,
    inputs: FxHashMap<usize, FxHashMap<&'static str, Vec<Output>>>,
    signals: FxHashMap<usize, FxHashMap<&'static str, Signal>>,
}

impl DynamicTracker {
    /// create a new tracker
    pub fn new(buffer_size: usize) -> Self {
        DynamicTracker {
            id_position: 0,
            buffer_size,
            inputs: FxHashMap::default(),
            signals: FxHashMap::default(),
        }
    }
}

impl Tracker for DynamicTracker {
    fn get_buffer_size(&self) -> &usize {
        &self.buffer_size
    }

    fn create_source_id(&mut self) -> usize {
        // @TODO: this is pretty naive, best keep track of ids somewhere
        let id = self.id_position;
        self.id_position += 1;
        id
    }

    fn clear_source(&mut self, id: usize) {
        self.signals.remove(&id);
    }

    fn get_sources(&self, id: &usize) -> Vec<usize> {
        let mut sources = vec![];

        if let Some(source) = self.inputs.get(id) {
            for input in source.values() {
                for output in input {
                    sources.push(*output.get_source_id());
                }
            }
        }

        sources
    }

    fn get_output(&self, o: &Output) -> Option<&Signal> {
        self.signals
            .get(o.get_source_id())
            .and_then(|signals| signals.get(o.get_signal_id()))
    }

    fn get_mut_output(&mut self, o: &Output) -> Option<&mut Signal> {
        self.signals
            .get_mut(o.get_source_id())
            .and_then(|signals| signals.get_mut(o.get_signal_id()))
    }

    fn init_output(&mut self, o: &Output) {
        if let None = self.signals.get(o.get_source_id()) {
            self.signals
                .insert(*o.get_source_id(), FxHashMap::default());
        }

        let source = self.signals.get_mut(o.get_source_id()).unwrap();

        source.insert(o.get_signal_id(), Signal::empty(self.buffer_size));
    }

    fn init_input(&mut self, s: &Input) {
        if let None = self.inputs.get(s.get_source_id()) {
            self.inputs.insert(*s.get_source_id(), FxHashMap::default());
        }

        let input = self.inputs.get_mut(s.get_source_id()).unwrap();

        input.insert(s.get_signal_id(), vec![]);
    }

    fn get_input(&self, s: &Input) -> Option<&Vec<Output>> {
        self.inputs
            .get(s.get_source_id())
            .and_then(|signals| signals.get(s.get_signal_id()))
    }

    fn resize_buffers(&mut self, buffer_size: usize) {
        self.buffer_size = buffer_size;

        for (_, source) in self.signals.iter_mut() {
            for (_, signal) in source.iter_mut() {
                signal.samples.resize(self.buffer_size, 0.0);
            }
        }
    }

    fn connect_signal(&mut self, output: &Output, input: &Input) {
        if let Some(input) = self
            .inputs
            .get_mut(input.get_source_id())
            .and_then(|signals| signals.get_mut(input.get_signal_id()))
        {
            input.push(*output);
        }
    }

    fn clear_connection(&mut self, output: &Output, input: &Input) {
        if let Some(input) = self
            .inputs
            .get_mut(input.get_source_id())
            .and_then(|signals| signals.get_mut(input.get_signal_id()))
        {
            input.retain(|o| o != output);
        }
    }
}
