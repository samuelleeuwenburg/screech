//! A collection of helpers for handling audio
//!
//! **NOTE! this library is unfinished, incomplete and most likely contains bugs!**

#![no_std]
#![warn(missing_docs)]

extern crate alloc;

mod traits;
mod stream;
mod signal;
mod clip;

pub use traits::Sample;
pub use stream::{FromPoints, Stream, StreamErr};
pub use signal::Signal;
pub use clip::Clip;
