mod arithmetic;
#[cfg(feature = "array")]
mod array;
mod bits;
mod conditional;
mod gates;

pub use arithmetic::*;
#[cfg(feature = "array")]
pub use array::Array;
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
    pub use crate::bits::IsB0;
    pub mod bitstring {
        pub use crate::bits::bitstring_conditionals::*;
    }
    #[cfg(feature = "array")]
    pub mod array {
        pub use crate::array::array_conditionals::*;
    }
}

/// Convenience macro for constructing tapes of bits. This accepts syntax like `$crate::bitstring!(1, 0, 1)` to
/// produce `Tape<Tape<B1, B0>, B1>`.
#[macro_export]
macro_rules! bitstring {
    // Public entry: at least two bits
    ($first:tt, $second:tt $(, $rest:tt)* $(,)?) => {
        $crate::bitstring!(@acc $crate::Tape<$crate::bitstring!(@bit $first), $crate::bitstring!(@bit $second)> $(, $rest)*)
    };
    // Allow a single bit (returns just B0/B1)
    ($only:tt) => { $crate::bitstring!(@bit $only) };

    // Accumulator (internal)
    (@acc $acc:ty, $next:tt $(, $rest:tt)*) => {
        $crate::bitstring!(@acc $crate::Tape<$acc, $crate::bitstring!(@bit $next)> $(, $rest)*)
    };
    (@acc $acc:ty) => { $acc };

    // Normalize bit tokens
    (@bit 0) => { $crate::B0 };
    (@bit 1) => { $crate::B1 };
    (@bit B0) => { $crate::B0 };
    (@bit B1) => { $crate::B1 };
}

pub use bitstring as bs;
