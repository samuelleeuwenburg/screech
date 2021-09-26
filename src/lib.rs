//! A collection of helpers for handling audio data in real time
//!
//! **NOTE! this library is unfinished, incomplete and most likely contains bugs!**

#![no_std]
#![warn(missing_docs)]

extern crate alloc;

/// Core data structures
pub mod core;

/// Common traits use throughout the library
pub mod traits;

/// Simple audio building blocks to build with
pub mod basic;
