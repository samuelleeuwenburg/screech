/// Abstraction to refer to sample values.
///
/// Signals can either originate from a [`crate::Patchbay`],
/// contain a fixed value or be empty.
///
/// Signals can be passed to a patchbay instance to retrieve the sample value.
///
/// ```
/// use screech::{Patchbay, Signal};
///
/// let mut patchbay: Patchbay<128> = Patchbay::new();
///
/// let mut point = patchbay.point().unwrap();
/// patchbay.set(&mut point, 0.4);
///
/// let source = point.signal();
/// let fixed = Signal::Fixed(0.6);
/// let silence = Signal::None;
///
/// assert_eq!(patchbay.get(source), 0.4);
/// assert_eq!(patchbay.get(fixed), 0.6);
/// assert_eq!(patchbay.get(silence), 0.0);
/// ```
#[derive(Copy, Clone)]
pub enum Signal {
    /// Refers to a sample set by another source
    PatchPoint(usize),
    /// Fixed sample value, useful for ad-hoc settings or independent values.
    Fixed(f32),
    /// No signal, for example an input with nothing connected usually references ground.
    None,
}
