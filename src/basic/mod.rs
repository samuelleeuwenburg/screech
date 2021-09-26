mod oscillator;
mod slew;
mod track;

/// Basic building block to contain non generated sound
pub mod clip;

pub use clip::Clip;
pub use oscillator::Oscillator;
pub use slew::Slew;
pub use track::Track;
