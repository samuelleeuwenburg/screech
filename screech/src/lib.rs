//! Screech
//!
//! Opinionated real time audio library with a focus on performance and no_std environments.

#![no_std]

mod module;
pub mod modules;
mod patchbay;
mod processor;
mod signal;

pub use module::Module;
pub use patchbay::{PatchPoint, Patchbay};
pub use processor::Processor;
pub use signal::Signal;
