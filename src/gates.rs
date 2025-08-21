use crate::{Bit, bits::Bitstring};

/// Returns the bitwise `AND` of the two given bitstrings.
///
/// Note that this is designed to work for [`Bytes`], not individual bits! However, as bits can be
/// interpreted as bytes, you can use this for single bits as well, though it may not give the
/// desired output, and you should consider [`BitAnd`] instead.
pub type And<A, B> = <A as Bitstring>::And<B>;
/// Returns the bitwise `OR` of the two given bitstrings.
///
/// Note that this is designed to work for [`Bytes`], not individual bits! However, as bits can be
/// interpreted as bytes, you can use this for single bits as well, though it may not give the
/// desired output, and you should consider [`BitOr`] instead.
pub type Or<A, B> = <A as Bitstring>::Or<B>;
/// Returns the bitwise `NOT` of the two given bitstrings.
///
/// Note that this is designed to work for [`Bytes`], not individual bits! However, as bits can be
/// interpreted as bytes, you can use this for single bits as well, though it may not give the
/// desired output, and you should consider [`BitNot`] instead.
pub type Not<A> = <A as Bitstring>::Not;
/// Returns the bitwise `XOR` of the two given bitstrings.
///
/// Note that this is designed to work for [`Bytes`], not individual bits! However, as bits can be
/// interpreted as bytes, you can use this for single bits as well, though it may not give the
/// desired output, and you should consider [`BitXor`] instead.
pub type Xor<A, B> = Or<And<A, Not<B>>, And<Not<A>, B>>;
// Returns the bitwise `NAND` of the two given bitstrings.
///
/// Note that this is designed to work for [`Bytes`], not individual bits! However, as bits can be
/// interpreted as bytes, you can use this for single bits as well, though it may not give the
/// desired output, and you should consider [`BitNand`] instead.
pub type Nand<A, B> = Not<And<A, B>>;
/// Returns the bitwise `NOR` of the two given bitstrings.
///
/// Note that this is designed to work for [`Bytes`], not individual bits! However, as bits can be
/// interpreted as bytes, you can use this for single bits as well, though it may not give the
/// desired output, and you should consider [`BitNor`] instead.
pub type Nor<A, B> = Not<Or<A, B>>;
/// Returns the bitwise `XNOR` of the two given bitstrings.
///
/// Note that this is designed to work for [`Bytes`], not individual bits! However, as bits can be
/// interpreted as bytes, you can use this for single bits as well, though it may not give the
/// desired output, and you should consider [`BitXnor`] instead.
pub type Xnor<A, B> = Not<Xor<A, B>>;

/// Returns the single-bit `AND` of the two given bits.
pub type BitAnd<A, B> = <A as Bit>::And<B>;
/// Returns the single-bit `OR` of the two given bits.
pub type BitOr<A, B> = <A as Bit>::Or<B>;
/// Returns the single-bit `NOT` of the given bit.
pub type BitNot<A> = <A as Bit>::Not;
/// Returns the single-bit `XOR` of the two given bits.
pub type BitXor<A, B> = BitOr<BitAnd<A, BitNot<B>>, BitAnd<BitNot<A>, B>>;
/// Returns the single-bit `NAND` of the two given bits.
pub type BitNand<A, B> = BitNot<BitAnd<A, B>>;
/// Returns the single-bit `NOR` of the two given bits.
pub type BitNor<A, B> = BitNot<BitOr<A, B>>;
/// Returns the single-bit `XNOR` of the two given bits.
pub type BitXnor<A, B> = BitNot<BitXor<A, B>>;

/// A two-bit multiplexer. This will return `A` if `S` is false, and `B` if `S` is true. If you
/// need an if statement for types, consider [`crate::conditional_system!`].
///
/// This is designed to work for two single bits.
pub type BitMux<S, A, B> = BitOr<BitAnd<BitNot<S>, A>, BitAnd<S, B>>;
