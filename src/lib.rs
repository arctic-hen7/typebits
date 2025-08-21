mod arithmetic;
mod bits;
mod gates;

pub use arithmetic::*;
pub use bits::{B0, B1, Bit, Bitstring, Tape};
pub use gates::*;

/// Types related to our internal bitwise conditional system. This is used to implement bitwise
/// recursion for arithmetic, and may be of use to others, though this is far from a generic
/// type-level conditional system!
///
/// Generally, type-level conditionals in Rust seem to be limited by the bounds you need on their
/// inputs/outputs, meaning you need to have a new one for each scenario. A macro may in future
/// provide this, but for now, this may be useful in bitstring-specific cases.
pub mod conditionals {
    pub use crate::bits::{Boolean, False, If, IfB0, IsB0, Lazy, SimpleIf, Thunk, True};
}
