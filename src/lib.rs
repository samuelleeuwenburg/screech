//! A collection of helpers for handling audio
//!
//! **NOTE! this library is unfinished, incomplete and most likely contains bugs!**

#![no_std]
#![warn(missing_docs)]

extern crate alloc;

/// Common traits used throughout the library
pub mod traits;

/// Struct representing a stream of audio data
pub mod stream;

/// Wrapper type to handle contextual channel manipulation for [`crate::stream::Stream`]
pub mod signal;

/// Most basic building block for non-generated sound
pub mod clip;
