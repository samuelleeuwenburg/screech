use crate::traits::Tracker;
use crate::{Input, Message, Output, Signal};
use alloc::vec;
use alloc::vec::Vec;
use rustc_hash::FxHashMap;

/// Basic fixed size tracker for the creation of unique ids
/// and to keep track of signals belonging to a certain id
///
/// ```
/// use screech::traits::Tracker;
/// use screech::BasicTracker;
//
/// const SOURCES_SIZE: usize = 2;
/// let mut tracker = BasicTracker::<SOURCES_SIZE>::new(128);
///
/// // the resulting id is irrelevant as long as it is unique
/// assert_eq!(tracker.create_source_id() != tracker.create_source_id(), true);
/// ```
pub struct BasicTracker<const SOURCES_SIZE: usize, T = ()> {
    id_position: usize,
    buffer_size: usize,
    inputs: [FxHashMap<&'static str, Vec<Output>>; SOURCES_SIZE],
    signals: [FxHashMap<&'static str, Signal>; SOURCES_SIZE],
    messages: FxHashMap<usize, Vec<Message<T>>>,
}

impl<const SOURCES_SIZE: usize, T> BasicTracker<SOURCES_SIZE, T> {
    /// create a new tracker
    pub fn new(buffer_size: usize) -> Self {
        BasicTracker {
            buffer_size,
            id_position: 0,
            inputs: [(); SOURCES_SIZE].map(|_| FxHashMap::default()),
            signals: [(); SOURCES_SIZE].map(|_| FxHashMap::default()),
            messages: FxHashMap::default(),
        }
    }
}

impl<const SOURCES_SIZE: usize, T> Tracker<T> for BasicTracker<SOURCES_SIZE, T> {
    fn get_buffer_size(&self) -> &usize {
        &self.buffer_size
    }

    fn create_source_id(&mut self) -> usize {
        // @TODO: implement tracking
        let id = self.id_position;
        self.id_position += 1;
        id
    }

    fn clear_source(&mut self, _id: usize) {
        // @TODO: implement tracking
    }

    fn get_sources(&self, &id: &usize) -> Vec<usize> {
        let mut sources = vec![];

        for input in self.inputs[id].values() {
            for output in input {
                sources.push(*output.get_source_id());
            }
        }

        sources
    }

    fn get_output(&self, o: &Output) -> Option<&Signal> {
        self.signals[*o.get_source_id()].get(o.get_signal_id())
    }

    fn get_mut_output(&mut self, o: &Output) -> Option<&mut Signal> {
        self.signals[*o.get_source_id()].get_mut(o.get_signal_id())
    }

    fn init_output(&mut self, o: &Output) {
        self.signals[*o.get_source_id()].insert(o.get_signal_id(), Signal::empty(self.buffer_size));
    }

    fn init_input(&mut self, s: &Input) {
        self.inputs[*s.get_source_id()].insert(s.get_signal_id(), vec![]);
    }

    fn get_input(&self, s: &Input) -> Option<&[Output]> {
        self.inputs[*s.get_source_id()]
            .get(s.get_signal_id())
            .map(|v| v.as_slice())
    }

    fn resize_buffers(&mut self, buffer_size: usize) {
        self.buffer_size = buffer_size;
        for source in self.signals.iter_mut() {
            for (_, signal) in source.iter_mut() {
                signal.samples.resize(self.buffer_size, 0.0);
            }
        }
    }

    fn connect_signal(&mut self, output: &Output, input: &Input) {
        if let Some(input) = self.inputs[*input.get_source_id()].get_mut(input.get_signal_id()) {
            input.push(*output);
        }
    }

    fn clear_connection(&mut self, output: &Output, input: &Input) {
        if let Some(input) = self.inputs[*input.get_source_id()].get_mut(input.get_signal_id()) {
            input.retain(|o| o != output);
        }
    }

    fn send_message(&mut self, id: &usize, message: Message<T>) {
        if let Some(messages) = self.messages.get_mut(id) {
            messages.push(message);
        }
    }

    fn get_messages(&self, id: &usize) -> Option<&[Message<T>]> {
        self.messages.get(id).map(|v| v.as_slice())
    }

    fn clear_messages(&mut self) {
        self.messages.clear();
    }
}
