//! A collection of helpers for handling audio data in real time
//!
//! **NOTE! this library is unfinished, incomplete and most likely contains bugs!**

// #![no_std]
#![warn(missing_docs)]

extern crate alloc;

mod graph;
mod primary;
mod signal;
mod signal_id;
mod tracker;
pub mod traits;

pub use primary::Primary;
pub use signal::Signal;
pub use signal_id::SignalId;
pub use tracker::{BasicTracker, DynamicTracker};
