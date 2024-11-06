//! Screech
//!
//! Opinionated real time audio library with a focus on performance and no_std environments.
//!
//! # Basic example
//!
//! This example uses two oscillators and a single VCA. One oscillator is set to a low frequency (LFO)
//! and fed to the modulator input of the VCA to modulate the input signal.
//!
//! ```
//! use screech::module::Module;
//! use screech::modules::Oscillator;
//! use screech::patchbay::{PatchError, PatchPoint, PatchPointOutput, Patchbay};
//! use screech::processor::Processor;
//! use screech::sample::Sample;
//! use screech_macro::modularize;
//!
//! // Set the buffer size and sample rate
//! const DURATION: usize = 10;
//! const SAMPLE_RATE: usize = 48000;
//! const BUFFER_SIZE: usize = SAMPLE_RATE * DURATION;
//!
//! /// VCA module that takes two inputs (signal and modulator) and has a single output.
//! struct Vca {
//!     modulator: PatchPointOutput,
//!     input: PatchPointOutput,
//!     output: PatchPoint,
//! }
//!
//! impl Vca {
//!     fn new(modulator: PatchPointOutput, input: PatchPointOutput, output: PatchPoint) -> Self {
//!         Vca {
//!             modulator,
//!             input,
//!             output,
//!         }
//!     }
//! }
//!
//! impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Vca {
//!     fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) -> Result<(), PatchError> {
//!         // Take the input signal and multiply it by the modulator input.
//!         patchbay.set_sample(
//!             &mut self.output,
//!             patchbay.get_sample(self.input)? * patchbay.get_sample(self.modulator)?,
//!         );
//!
//!         Ok(())
//!     }
//! }
//!
//! #[modularize]
//! enum Modules {
//!     Oscillator(Oscillator),
//!     Vca(Vca),
//! }
//!
//! fn main() {
//!     // Set up the memory
//!     let mut buffer = [0.0; BUFFER_SIZE];
//!     let mut patchbay: Patchbay<8> = Patchbay::new();
//!
//!     // Build connections
//!     let osc_point = patchbay.get_point();
//!     let lfo_point = patchbay.get_point();
//!     let vca_point = patchbay.get_point();
//!     let output = vca_point.output();
//!
//!     let vca = Vca::new(lfo_point.output(), osc_point.output(), vca_point);
//!     let osc = Oscillator::new(osc_point, 220.0);
//!     let lfo = Oscillator::new(lfo_point, 1.0);
//!
//!     // Process the modules
//!     let mut processor: Processor<SAMPLE_RATE, 3, Modules> = Processor::new([
//!         Modules::Oscillator(osc),
//!         Modules::Oscillator(lfo),
//!         Modules::Vca(vca),
//!     ]);
//!
//!     for i in 0..BUFFER_SIZE {
//!         processor.process_modules(&mut patchbay);
//!         buffer[i] = patchbay.get_sample(output).unwrap();
//!     }
//! }
//! ```
//! More examples can be found in the `examples` directory.
//!

#![no_std]

pub mod module;
pub mod modules;
pub mod patchbay;
pub mod processor;
pub mod sample;
