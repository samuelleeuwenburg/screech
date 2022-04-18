use crate::traits::Tracker;
use crate::{Signal, SignalId};

/// Basic fixed size tracker for the creation of unique ids
/// and to keep track of signals belonging to a certain id
///
/// ```
/// use screech::traits::Tracker;
/// use screech::BasicTracker;
//
/// const SOURCES_SIZE: usize = 2;
/// const SIGNALS_SIZE: usize = 2;
/// let mut tracker = BasicTracker::<SOURCES_SIZE, SIGNALS_SIZE>::new(128);
///
/// // the resulting id is irrelevant as long as it is unique
/// assert_eq!(tracker.create_source_id() != tracker.create_source_id(), true);
/// ```
pub struct BasicTracker<const SOURCES_SIZE: usize, const SIGNALS_SIZE: usize> {
    id_position: usize,
    buffer_size: usize,
    signals: [[Option<Signal>; SIGNALS_SIZE]; SOURCES_SIZE],
}

impl<const SOURCES_SIZE: usize, const SIGNALS_SIZE: usize>
    BasicTracker<SOURCES_SIZE, SIGNALS_SIZE>
{
    const NONE: Option<Signal> = None;
    const SIGNALS_ARR: [Option<Signal>; SIGNALS_SIZE] = [Self::NONE; SIGNALS_SIZE];

    /// create a new tracker
    pub fn new(buffer_size: usize) -> Self {
        BasicTracker {
            buffer_size,
            id_position: 0,
            signals: [Self::SIGNALS_ARR; SOURCES_SIZE],
        }
    }
}

impl<const SOURCES_SIZE: usize, const SIGNALS_SIZE: usize> Tracker
    for BasicTracker<SOURCES_SIZE, SIGNALS_SIZE>
{
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

    fn get_signal(&self, e: &SignalId) -> Option<&Signal> {
        self.signals[*e.get_source_id()][*e.get_signal_id()].as_ref()
    }

    fn get_mut_signal(&mut self, e: &SignalId) -> Option<&mut Signal> {
        self.signals[*e.get_source_id()][*e.get_signal_id()].as_mut()
    }

    fn init_buffer(&mut self, e: &SignalId) {
        self.signals[*e.get_source_id()][*e.get_signal_id()] =
            Some(Signal::empty(self.buffer_size));
    }

    fn resize_buffers(&mut self, buffer_size: usize) {
        self.buffer_size = buffer_size;
        for source in self.signals.iter_mut() {
            for signal in source {
                if let Some(signal) = signal {
                    signal.samples.resize(self.buffer_size, 0.0);
                }
            }
        }
    }
}
