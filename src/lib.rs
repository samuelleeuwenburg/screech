//! A collection of helpers to handle audio
//!
//! **NOTE! this library is unfinished, incomplete and most likely contains bugs!**

#![no_std]
#![warn(missing_docs)]

extern crate alloc;

mod stream;

pub use stream::{FromSamples, Stream, StreamErr};
