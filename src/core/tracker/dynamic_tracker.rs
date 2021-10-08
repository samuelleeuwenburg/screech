use crate::core::{ExternalSignal, Signal};
use crate::traits::Tracker;
use rustc_hash::FxHashMap;

/// Basic dynamically sized tracker for the creation of unique ids
/// and to keep track of signals belonging to a certain id
///
/// ```
/// use screech::traits::Source;
/// use screech::core::DynamicTracker;
/// use screech::basic::{Track, Oscillator};
///
/// let mut tracker = DynamicTracker::new();
/// let osc = Oscillator::new(&mut tracker);
/// let track = Track::new(&mut tracker);
///
/// // the resulting id is irrelevant as long as it is unique
/// assert_eq!(osc.get_source_id() != track.get_source_id(), true);
/// ```
pub struct DynamicTracker {
    id_position: usize,
    signals: FxHashMap<usize, FxHashMap<usize, Signal>>,
}

impl DynamicTracker {
    /// create a new tracker
    pub fn new() -> Self {
        DynamicTracker {
            id_position: 0,
            signals: FxHashMap::default(),
        }
    }
}

impl Tracker for DynamicTracker {
    fn create_source_id(&mut self) -> usize {
        // @TODO: this is pretty naive, best keep track of ids somewhere
        let id = self.id_position;
        self.id_position += 1;
        id
    }

    fn clear_source(&mut self, id: usize) {
        self.signals.remove(&id);
    }

    fn get_signal(&self, e: &ExternalSignal) -> Option<&Signal> {
        self.signals
            .get(&e.get_source_id())
            .and_then(|signals| signals.get(&e.get_signal_id()))
    }

    fn set_signal(&mut self, e: &ExternalSignal, signal: Signal) {
        match self.signals.get_mut(&e.get_source_id()) {
            Some(signals) => {
                signals.insert(*e.get_signal_id(), signal);
            }
            None => {
                let mut map = FxHashMap::default();
                map.insert(*e.get_signal_id(), signal);
                self.signals.insert(*e.get_source_id(), map);
            }
        }
    }
}
