use crate::core::{ExternalSignal, Signal};
use crate::traits::Tracker;

/// Basic fixed size tracker for the creation of unique ids
/// and to keep track of signals belonging to a certain id
///
/// ```
/// use screech::traits::Source;
/// use screech::core::BasicTracker;
/// use screech::basic::{Track, Oscillator};
//
/// const SOURCES_SIZE: usize = 2;
/// const SIGNALS_SIZE: usize = 2;
/// let mut tracker = BasicTracker::<SOURCES_SIZE, SIGNALS_SIZE>::new();
/// let osc = Oscillator::new(&mut tracker);
/// let track = Track::new(&mut tracker);
///
/// // the resulting id is irrelevant as long as it is unique
/// assert_eq!(osc.get_source_id() != track.get_source_id(), true);
/// ```
pub struct BasicTracker<const SOURCES_SIZE: usize, const SIGNALS_SIZE: usize> {
    id_position: usize,
    signals: [[Option<Signal>; SIGNALS_SIZE]; SOURCES_SIZE],
}

impl<const SOURCES_SIZE: usize, const SIGNALS_SIZE: usize>
    BasicTracker<SOURCES_SIZE, SIGNALS_SIZE>
{
    /// create a new tracker
    pub fn new() -> Self {
        BasicTracker {
            id_position: 0,
            signals: [[None; SIGNALS_SIZE]; SOURCES_SIZE],
        }
    }
}

impl<const SOURCES_SIZE: usize, const SIGNALS_SIZE: usize> Tracker
    for BasicTracker<SOURCES_SIZE, SIGNALS_SIZE>
{
    fn create_source_id(&mut self) -> usize {
        // @TODO: implement tracking
        let id = self.id_position;
        self.id_position += 1;
        id
    }

    fn clear_source(&mut self, _id: usize) {
        // @TODO: implement tracking
    }

    fn get_signal(&self, e: &ExternalSignal) -> Option<&Signal> {
        self.signals[*e.get_source_id()][*e.get_signal_id()].as_ref()
    }

    fn set_signal(&mut self, e: &ExternalSignal, signal: Signal) {
        self.signals[*e.get_source_id()][*e.get_signal_id()] = Some(signal);
    }
}
