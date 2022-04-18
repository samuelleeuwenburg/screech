use crate::traits::Tracker;
use crate::{Signal, SignalId};
use rustc_hash::FxHashMap;

/// Basic dynamically sized tracker for the creation of unique ids
/// and to keep track of signals belonging to a certain id
///
/// ```
/// use screech::traits::Source;
/// use screech::DynamicTracker;
/// // use screech::basic::{Track, Oscillator};
///
/// let mut tracker = DynamicTracker::new(128);
/// // let osc = Oscillator::new(&mut tracker);
/// // let track = Track::new(&mut tracker);
///
/// // the resulting id is irrelevant as long as it is unique
/// // assert_eq!(osc.get_source_id() != track.get_source_id(), true);
/// ```
pub struct DynamicTracker {
    id_position: usize,
    buffer_size: usize,
    signals: FxHashMap<usize, FxHashMap<usize, Signal>>,
}

impl DynamicTracker {
    /// create a new tracker
    pub fn new(buffer_size: usize) -> Self {
        DynamicTracker {
            id_position: 0,
            buffer_size,
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

    fn get_signal(&self, e: &SignalId) -> Option<&Signal> {
        self.signals
            .get(e.get_source_id())
            .and_then(|signals| signals.get(e.get_signal_id()))
    }

    fn get_mut_signal(&mut self, e: &SignalId) -> Option<&mut Signal> {
        self.signals
            .get_mut(e.get_source_id())
            .and_then(|signals| signals.get_mut(e.get_signal_id()))
    }

    fn init_buffer(&mut self, e: &SignalId) {
        if let None = self.signals.get(e.get_source_id()) {
            self.signals
                .insert(*e.get_source_id(), FxHashMap::default());
        }

        let source = self.signals.get_mut(e.get_source_id()).unwrap();

        source.insert(*e.get_signal_id(), Signal::empty(self.buffer_size));
    }

    fn resize_buffers(&mut self, buffer_size: usize) {}
}
