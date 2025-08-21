use crate::{
    B0, Bit, BitAnd, BitNot, BitOr, BitXor, Bitstring, Or, Tape,
    bits::{IfB0, Lazy, Thunk},
};

pub type Diff<A /*: Bytes*/, B /*: Bytes*/> = <A as Subtract>::Difference<B>;

pub trait Subtract: Bitstring {
    type Difference<Rhs: Bitstring>: Bitstring;

    type DifferenceWithBorrow<Rhs: Bitstring, CarryIn: Bit>: Bitstring;
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

pub struct SubtractRecurse<A: Bitstring, B: Bitstring, BorrowIn: Bit> {
    _phantom: ::std::marker::PhantomData<(A, B, BorrowIn)>,
}
impl<A: Bitstring, B: Bitstring, BorrowIn: Bit> Lazy for SubtractRecurse<A, B, BorrowIn> {
    type Output = <A::Head as Subtract>::DifferenceWithBorrow<
        B::Head,
        <A::Lsb as HalfSubtract>::Borrow<B::Lsb, BorrowIn>,
    >;
}

pub trait HalfSubtract: Bit {
    type Difference<Rhs: Bit, BorrowIn: Bit>: Bit;
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
