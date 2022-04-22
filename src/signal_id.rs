/// Used for pointing to external signals usually stored inside of a Tracker
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SignalId {
    source_id: usize,
    signal_id: &'static str,
}

impl SignalId {
    /// Create new external signal
    pub fn new(source_id: usize, signal_id: &'static str) -> Self {
        SignalId {
            source_id,
            signal_id,
        }
    }

    /// Get a reference to the source id of the external signal
    pub fn get_source_id(&self) -> &usize {
        &self.source_id
    }

    /// Get a reference to the signal id of the external signal
    pub fn get_signal_id(&self) -> &'static str {
        &self.signal_id
    }
}
