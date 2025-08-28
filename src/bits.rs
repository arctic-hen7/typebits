use crate::{
    conditional::{Boolean, False, True},
    conditional_system,
};
use std::marker::PhantomData;

/// The single bit `1`.
#[derive(Default)]
pub struct B1;
/// The single bit `0`.
#[derive(Default)]
pub struct B0;

mod sealed {
    use super::{B0, B1};

    /// A sealed trait to prevent external implementations of `Bit`.
    pub trait SealedBit {}
    impl SealedBit for B1 {}
    impl SealedBit for B0 {}

    pub trait SealedBitstring {}
    impl<B: super::Bit> SealedBitstring for B {}
    impl<H: super::Bitstring, B: super::Bit> SealedBitstring for super::Tape<H, B> {}
}

/// A trait for single bits, implemented by [`B0`] and [`B1`] only. This trait is sealed to prevent
/// external implementations.
pub trait Bit: sealed::SealedBit {
    /// Returns the `AND` of this bit with the given one.
    type And<Other: Bit>: Bit;
    /// Returns the `OR` of this bit with the given one.
    type Or<Other: Bit>: Bit;
    /// Returns the `NOT` of this bit.
    type Not: Bit;

    const UNSIGNED: usize;

    /// Returns an internal representation of the boolean value arising from this bit. This is used
    /// internally for type-level conditionals, and generally shouldn't be interacted with by
    /// library users.
    type Bool: Boolean;

    /// The rendered version of this bit, for debugging.
    const RENDER: &'static str;
}
impl Bit for B1 {
    type And<Other: Bit> = Other;
    type Or<Other: Bit> = B1;
    type Not = B0;

    const UNSIGNED: usize = 1;

    type Bool = True;

    const RENDER: &'static str = "1";
}
impl Bit for B0 {
    type And<Other: Bit> = B0;
    type Or<Other: Bit> = Other;
    type Not = B1;

    const UNSIGNED: usize = 0;

    type Bool = False;

    const RENDER: &'static str = "0";
}

/// A trait for bitstrings of arbitrary length. This is implemented for any [`Bit`] and
/// [`Tape<H, B>`].
pub trait Bitstring: IsB0 + sealed::SealedBitstring {
    /// The head of the bitstring, which is itself another bitstring.
    type Head: Bitstring;
    /// The least-significant bit of the bitstring.
    type Lsb: Bit;

    const UNSIGNED: usize;

    /// A "trimmed" version of this bitstring, which will have no leading zeroes.
    type Trimmed: Bitstring;

    /// The bitwise `AND` of this bitstring with the given one.
    type And<Other: Bitstring>: Bitstring;
    /// The bitwise `OR` of this bitstring with the given one.
    type Or<Other: Bitstring>: Bitstring;
    /// The bitwise `NOT` of this bitstring.
    type Not: Bitstring;

    /// Returns a string representation of this bitstring, for debugging.
    fn render() -> String;
}

/// A "tape" of bits, represented as a recursive container. The generic parameter `H` is the "head"
/// of the bitstring, which is itself another bitstring, and `B` is the least-significant bit.
///
/// At runtime, this is a zero-sized type implementing [`Default`].
pub struct Tape<H: Bitstring, B: Bit> {
    _phantom: PhantomData<(H, B)>,
}
impl<H: Bitstring, B: Bit> Default for Tape<H, B> {
    fn default() -> Self {
        Tape {
            _phantom: PhantomData,
        }
    }
}
impl<H: Bitstring, B: Bit> Bitstring for Tape<H, B> {
    type Head = H;
    type Lsb = B;

    const UNSIGNED: usize = H::UNSIGNED * 2 + B::UNSIGNED;

    // If the trimmed head is zero, then this is the final bit, so we should return just that.
    // Otherwise, return a tape with the trimmed head and this bit. This evaluates recursively.
    type Trimmed = bitstring_conditionals::SimpleIf<
        <H::Trimmed as IsB0>::BitstringIsB0,
        B,
        Tape<H::Trimmed, B>,
    >;

    // We trim here for uniformity down to single bits
    type And<Other: Bitstring> =
        <Tape<H::And<Other::Head>, B::And<Other::Lsb>> as Bitstring>::Trimmed;
    type Or<Other: Bitstring> = <Tape<H::Or<Other::Head>, B::Or<Other::Lsb>> as Bitstring>::Trimmed;
    type Not = <Tape<H::Not, B::Not> as Bitstring>::Trimmed;

    // TODO: Compile-time or const way of doing this?
    fn render() -> String {
        format!("{}{}", H::render(), B::render())
    }
}
impl<B: Bit> Bitstring for B {
    type Head = B0; // A single bit can be interpreted as a two-bit tape with leading bit 0
    type Lsb = B;

    const UNSIGNED: usize = B::UNSIGNED;

    type Trimmed = B;

    // We return the bit directly for AND to avoid extra leading zeroes. We trim for uniformity
    // down to single bits.
    type And<Other: Bitstring> = <B::And<Other::Lsb> as Bitstring>::Trimmed;
    // When given two bits, this will add a leading zero, which leads to the general behavior that
    // ORing two equal-length tapes, or two tapes where the longer one comes first, will add a
    // leading zero.
    type Or<Other: Bitstring> = <Tape<Other::Head, B::Or<Other::Lsb>> as Bitstring>::Trimmed;
    type Not = B::Not;

    fn render() -> String {
        B::RENDER.to_string()
    }
}

/// A type alias for our internal conditional, which will evaluate to `T` if the input bit is
/// [`B0`], and `F` otherwise.
///
/// You will need to wrap `T` and `F` in a [`Thunk`] to avoid immediate evaluation by the compiler,
/// and if you're using this for recursion, your base case should be in a [`Thunk`], and your
/// recursive case should be a newtype implementing [`Lazy`]. See the implementation of addition in
/// this crate for an example.
pub type IfB0<B /*: Bytes*/, T, F> = bitstring_conditionals::If<<B as IsB0>::BitstringIsB0, T, F>;

conditional_system!(pub bitstring_conditionals, crate::Bitstring);

/// A trait for things which we can detect are [`B0`] or not. This lets us detect the end of a
/// bitstring, which enables bounded recursion and trimming.
pub trait IsB0 {
    type GlobalIsB0: crate::conditional::Boolean;
    type BitstringIsB0: bitstring_conditionals::Boolean;
    #[cfg(feature = "array")]
    type ArrayIsB0: crate::array::array_conditionals::Boolean;
}
impl<B: Bit> IsB0 for B {
    type GlobalIsB0 = <B::Not as Bit>::Bool;
    // To get this working, we need an associated type on bits that converts to our local
    // [`Boolean`] type.
    type BitstringIsB0 = <<B::Not as Bit>::Bool as Boolean>::BitstringBoolean;
    #[cfg(feature = "array")]
    type ArrayIsB0 = <<B::Not as Bit>::Bool as Boolean>::ArrayBoolean;
}
impl<H: Bitstring, B: Bit> IsB0 for Tape<H, B> {
    type GlobalIsB0 = crate::conditional::False;
    type BitstringIsB0 = <False as Boolean>::BitstringBoolean;
    #[cfg(feature = "array")]
    type ArrayIsB0 = <False as Boolean>::ArrayBoolean;
}

#[test]
fn bitstrings() {
    use crate::gates::*;

    type T10 = Tape<B1, B0>;
    type T101 = Tape<Tape<B1, B0>, B1>;

    assert_eq!(Or::<T10, T101>::render(), "111");
    assert_eq!(And::<T10, T101>::render(), "0");
    assert_eq!(Or::<T101, T10>::render(), "111");
    assert_eq!(And::<T101, T10>::render(), "0");

    type T910 = crate::bs!(1, 1, 1, 0, 0, 0, 1, 1, 1, 0);
    assert_eq!(T910::UNSIGNED, 910);
}
