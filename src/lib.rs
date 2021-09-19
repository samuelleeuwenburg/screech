//! A collection of helpers for handling audio data in real time
//!
//! **NOTE! this library is unfinished, incomplete and most likely contains bugs!**

#![no_std]
#![warn(missing_docs)]

extern crate alloc;

mod graph;

/// Common traits used throughout the library
pub mod traits;

/// Struct representing a stream of audio data
pub mod stream;

/// Wrapper type to handle contextual channel manipulation for [`crate::stream::Stream`]
pub mod signal;

/// General purpose oscillator
pub mod oscillator;

/// Slew rate limiter for [`signal::Signal`]
pub mod slew;

/// Most basic building block for non-generated sound
pub mod clip;

/// Standard track with panning and volume control
pub mod track;

/// Primary sound output data structure
pub mod primary;
