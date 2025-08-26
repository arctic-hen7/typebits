use crate::{Bitstring, bits::IsB0, conditional_system};

/// A stack-allocated array storing instances of `T`, whose length is defined by the [`Bitstring`]
/// `B`. This is almost identical to `generic_array`, except it avoids any `where` clauses, and
/// uses our internal bitstring type.
///
/// Like `generic_array`, this internally stores a binary-tree-like structure that gets cast to a
/// slice, allowing it to operate identically to an array. We basically just build up the size of
/// the type at compile-time based on the given bitstring.
///
/// # Internal representation
///
/// Internally, like `generic_array`, this stores the associated array type of the given bitstring,
/// governed by the [`HasArray`] trait. For an array of length 6 (0b110), you'd get an internal
/// structure like this:
///
/// ```text
/// Array<T, Tape<Tape<B1, B1>, B0>> {
///     data: ArrayEven<T, ArrayOdd<>> {
///         left: ArrayOdd<T, ArrayOdd<T, ArrayTerm>> {
///             left: ArrayOdd<T, ArrayTerm> {
///                 left: ArrayTerm,
///                 right: ArrayTerm,
///                 data: T, // arr[0]
///             },
///             right: ArrayOdd<T, ArrayTerm> {
///                 left: ArrayTerm,
///                 right: ArrayTerm,
///                 data: T, // arr[1]
///             },
///             data: T, // arr[2]
///         },
///         right: ArrayOdd<T, ArrayOdd<T, ArrayTerm>> {
///             left: ArrayOdd<T, ArrayTerm> {
///                 left: ArrayTerm,
///                 right: ArrayTerm,
///                 data: T, // arr[3]
///             },
///             right: ArrayOdd<T, ArrayTerm> {
///                 left: ArrayTerm,
///                 right: ArrayTerm,
///                 data: T, // arr[4]
///             },
///             data: T, // arr[5]
///         },
///     }
/// }
/// ```
///
/// Because this is `log_2(B::UNSIGNED)`-depth, we can avoid stack overflows when dropping these
/// arrays.
#[repr(transparent)]
pub struct Array<T, N: Bitstring> {
    data: <N as HasArray>::ArrayType<T>,
}

/// An internal struct that represents the even side of an array. If we have an array of length 6,
/// then this will divide into two [`ArrayEven`]s whose child types `U` are [`ArrayOdd`]s, and
/// *their* child types will also be [`ArrayOdd`]s! See [`Array`] for details of how this structure
/// works practically.
#[doc(hidden)]
#[repr(C)]
pub struct ArrayEven<T, U: sealed::IsArrayImpl> {
    left: U,
    right: U,
    _phantom: ::std::marker::PhantomData<T>,
}

/// An internal struct that represents the odd side of an array.
#[doc(hidden)]
#[repr(C)]
pub struct ArrayOdd<T, U: sealed::IsArrayImpl> {
    left: U,
    right: U,
    data: T,
}

/// An internal terminator for arrays. This will only ever appear on [`ArrayOdd`]s, and it reduces
/// them to be length-1 arrays.
pub struct ArrayTerm;

mod sealed {
    /// An internal trait implemented for the array constructors [`super::ArrayEven`],
    /// [`super::ArrayOdd`], and [`super::ArrayTerm`].
    pub trait IsArrayImpl {}
    impl<T, U: IsArrayImpl> IsArrayImpl for super::ArrayEven<T, U> {}
    impl<T, U: IsArrayImpl> IsArrayImpl for super::ArrayOdd<T, U> {}

    impl IsArrayImpl for super::ArrayTerm {}
}

/// A trait implemented for all types with "array types" formed from bitstrings. This is
/// implemented by default for every [`Bitstring`] (producing a stack-allocated [`Array<T, N>`]).
pub trait HasArray: Bitstring {
    /// The internal array type. This is sealed to be one of [`ArrayEven`], [`ArrayOdd`], and
    /// [`ArrayTerm`].
    type ArrayType<T>: sealed::IsArrayImpl;
}
impl<B: Bitstring> HasArray for B {
    // If the entire bitstring is `B0`, then return the array terminator, otherwise enter a
    // recursion (because there's recursion in the underlying conditional there, this has to be its
    // own recursor)
    type ArrayType<T> =
        If<<B::Trimmed as IsB0>::ArrayIsB0, Thunk<ArrayTerm>, ArrayRecurse<T, B::Trimmed>>;
}

/// An internal recursion helper for producing [`Array<T, N>`].
pub struct ArrayRecurse<T, B: Bitstring> {
    _phantom: ::std::marker::PhantomData<(T, B)>,
}
impl<T, B: Bitstring> Lazy for ArrayRecurse<T, B> {
    // If the least significant bit is `B0` (i.e. the bitstring represents an even number), go into
    // the even recursion, otherwise the odd recursion
    type Output = If<<B::Lsb as IsB0>::ArrayIsB0, ArrayEvenRecurse<T, B>, ArrayOddRecurse<T, B>>;
}

/// An internal recursion helper for producing [`Array<T, N>`] (even variant).
pub struct ArrayEvenRecurse<T, B: Bitstring> {
    _phantom: ::std::marker::PhantomData<(T, B)>,
}
impl<T, B: Bitstring> Lazy for ArrayEvenRecurse<T, B> {
    // We're an even bitstring, divide by two (i.e. take the head) and put whatever the array type
    // of that is inside an `ArrayEven`
    type Output = ArrayEven<T, <B::Head as HasArray>::ArrayType<T>>;
}

/// An internal recursion helper for producing [`Array<T, N>`] (odd variant).
pub struct ArrayOddRecurse<T, B: Bitstring> {
    _phantom: ::std::marker::PhantomData<(T, B)>,
}
impl<T, B: Bitstring> Lazy for ArrayOddRecurse<T, B> {
    // We're an odd bitstring, divide by two (i.e. take the head) and put whatever the array type
    // of that is inside an `ArrayOdd` (which will actually contain an instance of `T`)
    type Output = ArrayOdd<T, <B::Head as HasArray>::ArrayType<T>>;
}

use array_conditionals::{If, Lazy, Thunk};
conditional_system!(pub array_conditionals, super::sealed::IsArrayImpl);

#[test]
fn arrays() {
    use crate::bs;

    type Untrimmed = Array<u8, bs!(0, 1, 0)>;
    type A2 = Array<u8, bs!(1, 0)>;
    type A3 = Array<u8, bs!(1, 1)>;
    type A14 = Array<u8, bs!(1, 1, 1, 0)>;
    type A5Long = Array<u32, bs!(1, 0, 1)>;

    assert_eq!(size_of::<Untrimmed>(), 2);
    assert_eq!(size_of::<A2>(), 2);
    assert_eq!(size_of::<A3>(), 3);
    assert_eq!(size_of::<A14>(), 14);
    assert_eq!(size_of::<A5Long>(), 4 * 5);
}
