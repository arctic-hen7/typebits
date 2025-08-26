use crate::{
    B0, Bit, BitAnd, BitOr, BitXor, Bitstring, Or, Tape,
    bits::IfB0,
    conditionals::bitstring::{Lazy, Thunk},
};

/// Returns the sum of the two given bitstrings.
pub type Sum<A /*: Bytes*/, B /*: Bytes*/> = <A as Add>::Sum<B>;

/// A trait for bitstrings that can be added to other bitstrings. This is implemented for all
/// bitstrings, and provides methods for adding with all other bitstrings, eliminating the need for
/// complex (and often impossible-to-prove) bounds.
pub trait Add: Bitstring {
    /// The sum of this bitstring with the given one.
    type Sum<Rhs: Bitstring>: Bitstring;

    /// An internal associated type that performs the sum of this bitstring with the given one,
    /// with an additional parameter for the input carry bit. Generally, end users won't need to
    /// use this unless building their own arithmetic routines.
    type SumWithCarry<Rhs: Bitstring, CarryIn: Bit>: Bitstring;
}
impl<B: Bitstring> Add for B {
    // Full sum is just the sum with an initial carry bit of zero
    type Sum<Rhs: Bitstring> = <Self::SumWithCarry<Rhs, B0> as Bitstring>::Trimmed;

    type SumWithCarry<Rhs: Bitstring, CarryIn: Bit> = Tape<
        IfB0<
            // If *both* the head bits are zero, we've reached the final bit (in LSB)
            Or<Self::Head, Rhs::Head>,
            // In that case, just return the remaining carry bit
            Thunk<<B::Lsb as HalfAdd>::Carry<Rhs::Lsb, CarryIn>>,
            // Otherwise, recurse
            AddRecurse<Self, Rhs, CarryIn>,
        >,
        // The LSB is just the half-add, taking on our carry-in
        <Self::Lsb as HalfAdd>::Sum<Rhs::Lsb, CarryIn>,
    >;
}

/// An internal recursion type for adding two bitstrings. You shouldn't need to interact with this
/// as an end user, but the source code is a useful example for implementing recursive type-level
/// conditionals.
pub struct AddRecurse<A: Bitstring, B: Bitstring, CarryIn: Bit> {
    _phantom: ::std::marker::PhantomData<(A, B, CarryIn)>,
}
impl<A: Bitstring, B: Bitstring, CarryIn: Bit> Lazy for AddRecurse<A, B, CarryIn> {
    type Output =
        <A::Head as Add>::SumWithCarry<B::Head, <A::Lsb as HalfAdd>::Carry<B::Lsb, CarryIn>>;
}

/// A half-adder type-level circuit for individual bits.
pub trait HalfAdd: Bit {
    /// The sum of this bit with the given one, done under the given carry.
    type Sum<Rhs: Bit, CarryIn: Bit>: Bit;
    /// The carry-out bit produced from adding this bit and the given one, in the context of the
    /// given carry-in bit.
    type Carry<Rhs: Bit, CarryIn: Bit>: Bit;
}
impl<B: Bit> HalfAdd for B {
    type Sum<Rhs: Bit, CarryIn: Bit> = BitXor<BitXor<Self, Rhs>, CarryIn>; // (A XOR B) XOR Cin
    type Carry<Rhs: Bit, CarryIn: Bit> =
        BitOr<BitAnd<Self, Rhs>, BitAnd<BitXor<Self, Rhs>, CarryIn>>; // (A AND B) OR ((A XOR B) AND C_in)
}

#[test]
fn add() {
    use crate::B1;

    type T10 = Tape<B1, B0>;
    type T01 = Tape<B0, B1>;
    type T101 = Tape<Tape<B1, B0>, B1>;
    type T110 = Tape<Tape<B1, B1>, B0>;

    assert_eq!(Sum::<T10, T01>::render(), "11");
    assert_eq!(Sum::<T01, T10>::render(), "11");
    assert_eq!(Sum::<T10, T101>::render(), "111");
    assert_eq!(Sum::<T101, T10>::render(), "111");
    assert_eq!(Sum::<T10, T110>::render(), "1000");
    assert_eq!(Sum::<T110, T10>::render(), "1000");
    assert_eq!(Sum::<T101, B1>::render(), "110");
}
