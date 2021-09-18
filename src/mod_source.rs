use crate::signal::Signal;
use hashbrown::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum ModSource {
    Owned(Signal),
    External(usize),
}

impl ModSource {
    //@TODO: -> Option<&Signal>
    pub fn get(&self, sources: &HashMap<usize, Signal>) -> Option<Signal> {
        match self {
            ModSource::Owned(signal) => Some(signal.clone()),
            ModSource::External(key) => sources.get(key).cloned(),
        }
    }
}
