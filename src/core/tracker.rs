use crate::core::Signal;
use crate::traits::Tracker;
use rustc_hash::FxHashMap;

/// Basic tracker for the creation of unique ids
/// and to keep track of signals belonging to a certain id
pub struct BasicTracker {
    id_position: usize,
    signals: FxHashMap<usize, Signal>,
}

impl BasicTracker {
    /// create a new tracker
    ///
    /// ```
    /// use screech::traits::Source;
    /// use screech::core::BasicTracker;
    /// use screech::basic::{Track, Oscillator};
    ///
    /// let mut tracker = BasicTracker::new();
    /// let osc = Oscillator::new(&mut tracker);
    /// let track = Track::new(&mut tracker);
    ///
    /// // the resulting id is irrelevant as long as it is unique
    /// assert_eq!(osc.get_id() != track.get_id(), true);
    /// ```
    pub fn new() -> Self {
        BasicTracker {
            id_position: 0,
            signals: FxHashMap::default(),
        }
    }
}

impl Tracker for BasicTracker {
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
