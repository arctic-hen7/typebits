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

    /// Returns an internal representation of the boolean value arising from this bit. This is used
    /// internally for type-level conditionals, and generally shouldn't be interacted with by
    /// library users.
    type Bool: byte_conditionals::Boolean;

    /// The rendered version of this bit, for debugging.
    const RENDER: &'static str;
}
impl Bit for B1 {
    type And<Other: Bit> = Other;
    type Or<Other: Bit> = B1;
    type Not = B0;

    type Bool = byte_conditionals::True;

    const RENDER: &'static str = "1";
}
impl Bit for B0 {
    type And<Other: Bit> = B0;
    type Or<Other: Bit> = Other;
    type Not = B1;

    type Bool = byte_conditionals::False;

    const RENDER: &'static str = "0";
}

/// A trait for bitstrings of arbitrary length. This is implemented for any [`Bit`] and
/// [`Tape<H, B>`]. Though it is not sealed, it generally shouldn't be implemented by external
/// types.
pub trait Bitstring: byte_conditionals::IsB0 {
    /// The head of the bitstring, which is itself another bitstring.
    type Head: Bitstring;
    /// The least-significant bit of the bitstring.
    type Lsb: Bit;

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

    // If the trimmed head is zero, then this is the final bit, so we should return just that.
    // Otherwise, return a tape with the trimmed head and this bit. This evaluates recursively.
    type Trimmed = byte_conditionals::SimpleIf<IsB0<H::Trimmed>, B, Tape<H::Trimmed, B>>;

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

/// A type alias for our internal conditional booleans depending on whether the input bit is
/// [`B0`].
pub type IsB0<B /*: Bytes*/> = <B as byte_conditionals::IsB0>::IsB0;

/// A type alias for our internal conditional, which will evaluate to `T` if the input bit is
/// [`B0`], and `F` otherwise.
///
/// You will need to wrap `T` and `F` in a [`Thunk`] to avoid immediate evaluation by the compiler,
/// and if you're using this for recursion, your base case should be in a [`Thunk`], and your
/// recursive case should be a newtype implementing [`Lazy`]. See the implementation of addition in
/// this crate for an example.
pub type IfB0<B /*: Bytes*/, T, F> = byte_conditionals::If<IsB0<B>, T, F>;

pub(crate) use byte_conditionals::{Lazy, Thunk};

mod byte_conditionals {
    use super::*;

    /// A newtype representing truth.
    pub struct True;
    /// A newtype representing falsehood.
    pub struct False;

    mod sealed {
        pub trait SealedBoolean {}
        impl SealedBoolean for super::True {}
        impl SealedBoolean for super::False {}
    }

    /// A trait for our internal boolean types.
    pub trait Boolean: sealed::SealedBoolean {
        /// An associated type for conditionals, where both branches must implement [`Lazy`] in a
        /// strategy for avoiding immediate evaluation by the compiler.
        type Select<Then: Lazy, Else: Lazy>: Lazy;
        /// The negation of this boolean.
        type Not: Boolean;
    }
    impl Boolean for True {
        type Select<Then: Lazy, Else: Lazy> = Then;
        type Not = False;
    }
    impl Boolean for False {
        type Select<Then: Lazy, Else: Lazy> = Else;
        type Not = True;
    }

    /// A trait for avoiding greedy evaluation of conditional branches at compile-time. This is a
    /// very specific hack!
    pub trait Lazy {
        type Output: Bitstring;
    }

    /// A simple wrapper type that implements [`Lazy`]. Wrap anything non-recursive in this.
    pub struct Thunk<T> {
        _phantom: ::std::marker::PhantomData<T>,
    }
    impl<T: Bitstring> Lazy for Thunk<T> {
        type Output = T;
    }

    /// A trait for things which we can detect are [`B0`] or not. This lets us detect the end of a
    /// bitstring, which enables bounded recursion and trimming.
    pub trait IsB0 {
        type IsB0: Boolean;
    }
    impl<B: Bit> IsB0 for B {
        // To get this working, we need an associated type on bits that converts to our local
        // [`Boolean`] type.
        type IsB0 = <B::Not as Bit>::Bool;
    }
    impl<H: Bitstring, B: Bit> IsB0 for Tape<H, B> {
        type IsB0 = False;
    }

    /// A type-level conditional, where the condition implements [`Boolean`] and the branches
    /// implement [`Lazy`].
    pub(super) type If<Cond, T, F> = <<Cond as Boolean>::Select<T, F> as Lazy>::Output;
    /// A simple conditional that wraps both its branches in [`Thunk`]s. This should be used when
    /// you don't have recursion.
    pub(super) type SimpleIf<Cond, T, F> = If<Cond, Thunk<T>, Thunk<F>>;
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
}
