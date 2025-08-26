use crate::{
    B0, Bit, BitAnd, BitNot, BitOr, BitXor, Bitstring, Or, Tape,
    bits::IfB0,
    conditionals::bitstring::{Lazy, Thunk},
};

/// Returns the difference between the two given bitstrings. See [`Subtract`] for how this handles
/// underflows.
pub type Diff<A /*: Bytes*/, B /*: Bytes*/> = <A as Subtract>::Difference<B>;

/// A trait for the subtraction of two bitstrings. This is implemented for all bitstrings for
/// convenience, but will provide sane results only for the subtraction of a small bitstring from a
/// large bitstring, i.e. where the output is positive.
///
/// When used in a way that would produce a *negative* answer, this will return the
/// "locally-modular" wrapping subtraction. In other words, if you subtract `110 - 1011` (6 - 11),
/// this will produce `1011` (11), because `6 - 11 = 11 (mod 16)`. This is the definition of
/// wrapping subtraction, with the modulus being `2^n` for a bitstring of length `n`. When your
/// bitstrings are of two different lengths, `n` *should* be the length of the larger of the two,
/// though due to internal trimming behaviour, this might not always work. In short, use caution
/// when doing subtraction that would give a negative real number!
pub trait Subtract: Bitstring {
    /// The difference of this bitstring with the given one.
    type Difference<Rhs: Bitstring>: Bitstring;

    /// An internal associated type that performs the difference of this bitstring with the given
    /// one, with an additional parameter for the input borrow bit. Generally, end users won't
    /// need to use this unless building their own arithmetic routines.
    type DifferenceWithBorrow<Rhs: Bitstring, Borrow: Bit>: Bitstring;
}
impl<B: Bitstring> Subtract for B {
    // Full difference is just the partial difference with an initial borrow of zero
    type Difference<Rhs: Bitstring> = <Self::DifferenceWithBorrow<Rhs, B0> as Bitstring>::Trimmed;

    type DifferenceWithBorrow<Rhs: Bitstring, CarryIn: Bit> = Tape<
        IfB0<
            // If *both* the head bits are zero, we've reached the final bit (in LSB)
            Or<Self::Head, Rhs::Head>,
            // In that case, return a leading zero. The final borrow bit is simply a flag to
            // indicate that an underflow occurred
            Thunk<B0>,
            // Thunk<<B::Lsb as HalfSubtract>::Borrow<Rhs::Lsb, CarryIn>>,
            // Otherwise, recurse
            SubtractRecurse<Self, Rhs, CarryIn>,
        >,
        // The LSB is just the half-subtract, taking on our carry-in
        <Self::Lsb as HalfSubtract>::Difference<Rhs::Lsb, CarryIn>,
    >;
}

/// An internal recursion type for subtracting two bitstrings. You shouldn't need to interact with
/// this as an end user.
pub struct SubtractRecurse<A: Bitstring, B: Bitstring, BorrowIn: Bit> {
    _phantom: ::std::marker::PhantomData<(A, B, BorrowIn)>,
}
impl<A: Bitstring, B: Bitstring, BorrowIn: Bit> Lazy for SubtractRecurse<A, B, BorrowIn> {
    type Output = <A::Head as Subtract>::DifferenceWithBorrow<
        B::Head,
        <A::Lsb as HalfSubtract>::Borrow<B::Lsb, BorrowIn>,
    >;
}

/// A half-subtractor type-level circuit for individual bits.
pub trait HalfSubtract: Bit {
    /// The difference of this bit with the given one, done under the given borrow.
    type Difference<Rhs: Bit, BorrowIn: Bit>: Bit;
    /// The borrow-out bit produced from subtracting this bit and the given one, in the context of
    /// the given borrow-in bit.
    type Borrow<Rhs: Bit, BorrowIn: Bit>: Bit;
}
impl<B: Bit> HalfSubtract for B {
    type Difference<Rhs: Bit, BorrowIn: Bit> = BitXor<Self, BitXor<Rhs, BorrowIn>>; // (A XOR B)
    // XOR B_in
    type Borrow<Rhs: Bit, BorrowIn: Bit> =
        BitOr<BitAnd<BitNot<Self>, Rhs>, BitAnd<BitNot<BitXor<Self, Rhs>>, BorrowIn>>; // ((NOT A) AND B) OR ((NOT (A XOR B)) AND B_in)
}

#[test]
fn subtract() {
    use crate::B1;

    type T10 = Tape<B1, B0>;
    type T01 = Tape<B0, B1>;
    type T101 = Tape<Tape<B1, B0>, B1>;
    type T110 = Tape<Tape<B1, B1>, B0>;
    type T1011 = Tape<Tape<Tape<B1, B0>, B1>, B1>;

    assert_eq!(Diff::<T10, T01>::render(), "1");
    assert_eq!(Diff::<T101, T01>::render(), "100");
    assert_eq!(Diff::<T110, T10>::render(), "100");

    // Underflow
    assert_eq!(Diff::<T1011, T110>::render(), "101"); // 11 - 6 = 5
    assert_eq!(Diff::<T110, T1011>::render(), "1011"); // 6 - 11 = 11 (mod 16)
}
