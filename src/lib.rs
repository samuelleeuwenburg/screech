//! A collection of helpers for handling audio
//!
//! **NOTE! this library is unfinished, incomplete and most likely contains bugs!**

#![no_std]
#![warn(missing_docs)]

extern crate alloc;

mod traits;
mod sample;
mod stream;

pub use traits::Playable;
pub use sample::Sample;
pub use stream::{FromSamples, Stream, StreamErr};
