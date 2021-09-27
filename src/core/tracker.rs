use crate::core::Signal;
use crate::traits::Tracker;

/// Basic tracker for the creation of unique ids
/// and to keep track of signals belonging to a certain id
pub struct BasicTracker<const SIZE: usize> {
    id_position: usize,
    signals: [Option<Signal>; SIZE],
}

impl<const SIZE: usize> BasicTracker<SIZE> {
    /// create a new tracker
    ///
    /// ```
    /// use screech::traits::Source;
    /// use screech::core::BasicTracker;
    /// use screech::basic::{Track, Oscillator};
    //
    /// const SOURCES_SIZE: usize = 2;
    /// let mut tracker = BasicTracker::<SOURCES_SIZE>::new();
    /// let osc = Oscillator::new(&mut tracker);
    /// let track = Track::new(&mut tracker);
    ///
    /// // the resulting id is irrelevant as long as it is unique
    /// assert_eq!(osc.get_id() != track.get_id(), true);
    /// ```
    pub fn new() -> Self {
        BasicTracker {
            id_position: 0,
            // signals: FxHashMap::default(),
            signals: [None; SIZE],
        }
    }
}

impl<const SIZE: usize> Tracker for BasicTracker<SIZE> {
    fn create_id(&mut self) -> usize {
        // @TODO: look for a `None` value inside the array
        let id = self.id_position;
        self.signals[id] = Some(Signal::silence());
        self.id_position += 1;
        id
    }

    fn clear_id(&mut self, id: usize) {
        self.signals[id] = None;
    }

    fn get_signal(&self, id: usize) -> Option<&Signal> {
        self.signals[id].as_ref()
    }

    fn set_signal(&mut self, id: usize, signal: Signal) {
        self.signals[id] = Some(signal);
    }
}
