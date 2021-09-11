use alloc::vec::Vec;
use crate::signal::Signal;

#[derive(Debug, PartialEq, Clone)]
pub enum ModSource {
    Owned(Signal),
    External(usize),
}


impl ModSource {
    pub fn get(&self, sources: &Vec<(usize, &Signal)>) -> Option<Signal> {
	match self {
	    ModSource::Owned(signal) => Some(signal.clone()),
	    ModSource::External(key) => {
		sources
		    .iter()
		    .find(|(i, _)| i == key)
		    .map(|&(_, s)| s.clone())
	    }
	}
    }
}

