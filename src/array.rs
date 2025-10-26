use crate::{Bitstring, bits::IsB0, conditional_system};
use std::{
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Index, IndexMut},
};

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
impl<T, N: Bitstring> Array<T, N> {
    /// Creates a new [`Array<T, N>`] of uninitialised elements.
    pub const fn uninit() -> Array<MaybeUninit<T>, N> {
        // SAFETY: An uninitialized `[MaybeUninit<_>; N]` is valid, same as a regular array.
        unsafe { MaybeUninit::<Array<MaybeUninit<T>, N>>::uninit().assume_init() }
    }

    /// Creates a new boxed [`Array<T, N>`] of uninitialised elements. You should use this when the
    /// length `N` is likely to overflow the stack.
    pub fn uninit_boxed() -> Box<Array<MaybeUninit<T>, N>> {
        // SAFETY: An uninitialized `[MaybeUninit<_>; N]` is valid, same as a regular array.
        unsafe { Box::new_uninit().assume_init() }
    }

    /// Gets the contents of this [`Array<T, N>`] as a slice. Because we have the same underlying
    /// memory representation as a slice, this works. The returned slice is guaranteed to have
    /// length [`Self::len()`] (equivalently [`N::UNSIGNED`]).
    pub const fn as_slice(&self) -> &[T] {
        let slice_size = N::UNSIGNED;
        // Because of the transparent representation, we can ignore all the zero-sized filler stuff
        // and just get a direct pointer to a bunch of `T`s
        let ptr = self as *const Self as *const T;

        // SAFETY: We have something in memory that is exactly equivalent to a `[u8; N::UNSIGNED]`.
        unsafe { std::slice::from_raw_parts(ptr, slice_size) }
    }

    /// Gets the contents of this [`Array<T, N>`] as a mutable slice.
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        let slice_size = N::UNSIGNED;
        // Because of the transparent representation, we can ignore all the zero-sized filler stuff
        // and just get a direct pointer to a bunch of `T`s
        let ptr = self as *mut Self as *mut T;

        // SAFETY: We have something in memory that is exactly equivalent to a `[u8; N::UNSIGNED]`.
        unsafe { std::slice::from_raw_parts_mut(ptr, slice_size) }
    }

    /// Tries to construct an [`Array<T, N>`] from the given slice. This returns a reference, as
    /// we're just reinterpreting the given slice as our own type. This will fail if the given
    /// slice has the wrong length.
    pub const fn try_from_slice(slice: &[T]) -> Result<&Self, BadLength> {
        if slice.len() != N::UNSIGNED {
            return Err(BadLength {
                found: slice.len(),
                expected: N::UNSIGNED,
            });
        }

        // SAFETY: Again, `Array<T, N>` has the same in-memory representation as a slice, so we can
        // literally just "rename" the type of this slice.
        Ok(unsafe { &*(slice.as_ptr() as *const Self) })
    }

    /// Constructs an [`Array<T, N>`] from the given slice.
    ///
    /// # Panics
    ///
    /// Panics if the given slice is the wrong length.
    pub const fn from_slice(slice: &[T]) -> &Self {
        if slice.len() != N::UNSIGNED {
            panic!("tried to construct array from slice of incorrect length");
        }

        match Self::try_from_slice(slice) {
            Ok(s) => s,
            Err(_) => unreachable!(),
        }
    }

    /// Tries to construct an [`Array<T, N>`] from the given mutable slice. This returns a mutable
    /// reference, as we're just reinterpreting the given mutable slice as our own type. This will
    /// fail if the given slice has the wrong length.
    pub const fn try_from_mut_slice(slice: &mut [T]) -> Result<&mut Self, BadLength> {
        if slice.len() != N::UNSIGNED {
            return Err(BadLength {
                found: slice.len(),
                expected: N::UNSIGNED,
            });
        }

        // SAFETY: Again, `Array<T, N>` has the same in-memory representation as a slice, so we can
        // literally just "rename" the type of this slice
        Ok(unsafe { &mut *(slice.as_mut_ptr() as *mut Self) })
    }

    /// Constructs an [`Array<T, N>`] from the given mutable slice.
    ///
    /// # Panics
    ///
    /// Panics if the given slice is the wrong length.
    pub const fn from_mut_slice(slice: &mut [T]) -> &mut Self {
        if slice.len() != N::UNSIGNED {
            panic!("tried to construct array from slice of incorrect length");
        }

        match Self::try_from_mut_slice(slice) {
            Ok(s) => s,
            Err(_) => unreachable!(),
        }
    }

    /// Returns the length of this [`Array<T, N>`], which is equal to [`N::UNSIGNED`].
    pub const fn len() -> usize {
        N::UNSIGNED
    }
}
impl<T, N: Bitstring> Array<MaybeUninit<T>, N> {
    /// Assumes this array of [`MaybeUninit<T>`] has all elements initialized.
    ///
    /// # Safety
    ///
    /// All elements must actually be initialized, or this will lead to undefined behaviour due to
    /// the underlying transmutation.
    ///
    /// # Boxes
    ///
    /// If you're trying to interpret a `Box<Array<MaybeUninit<T>, N>>`, you'll need
    pub const unsafe fn assume_init(self) -> Array<T, N> {
        // SAFETY: There's no difference between `MaybeUninit<T>` and `T` in memory (literally a
        // union with `()`), so perfectly safe to reinterpret the array as a whole
        unsafe { const_transmute::<_, _>(self) }
    }
}
impl<T: Default, N: Bitstring> Array<T, N> {
    /// Creates a new [`Array<T, N>`] with all elements set to `T::default()`.
    pub fn new() -> Self {
        let mut uninit = Self::uninit();
        for elem in uninit.as_mut_slice() {
            elem.write(T::default());
        }

        // SAFETY: There's no difference between `MaybeUninit<T>` and `T` in memory (literally a
        // union with `()`), so perfectly safe to reinterpret the array as a whole
        unsafe { const_transmute::<_, Self>(uninit) }
    }

    /// Creates a new boxed [`Array<T, N>`] with all elements set to `T::default()`. You should use
    /// this when the length `N` is likely to overflow the stack.
    pub fn new_boxed() -> Box<Self> {
        let mut uninit = Self::uninit_boxed();
        for elem in uninit.as_mut_slice() {
            elem.write(T::default());
        }

        // SAFETY: There's no difference between `MaybeUninit<T>` and `T` in memory (literally a
        // union with `()`), so perfectly safe to reinterpret the array as a whole
        unsafe { const_transmute::<_, Box<Self>>(uninit) }
    }
}
impl<T: Default, N: Bitstring> Default for Array<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T, N: Bitstring> Index<usize> for Array<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}
impl<T, N: Bitstring> IndexMut<usize> for Array<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}
impl<T, N: Bitstring> AsRef<[T]> for Array<T, N> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}
impl<T, N: Bitstring> AsMut<[T]> for Array<T, N> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}
impl<T: Clone, N: Bitstring> Clone for Array<T, N> {
    fn clone(&self) -> Self {
        let mut uninit = Self::uninit();
        for (src, dst) in self.as_slice().iter().zip(uninit.as_mut_slice().iter_mut()) {
            dst.write(src.clone());
        }

        // SAFETY: We've initialised all elements
        unsafe { uninit.assume_init() }
    }
}
impl<T: Clone, N: Bitstring> Array<T, N> {
    pub fn try_new_from_slice(slice: &[T]) -> Result<Self, BadLength> {
        if slice.len() != N::UNSIGNED {
            return Err(BadLength {
                found: slice.len(),
                expected: N::UNSIGNED,
            });
        }

        let mut uninit = Self::uninit();
        for i in 0..N::UNSIGNED {
            uninit[i].write(slice[i].clone());
        }

        Ok(unsafe { uninit.assume_init() })
    }

    pub fn new_from_slice(slice: &[T]) -> Self {
        if slice.len() != N::UNSIGNED {
            panic!("tried to construct array from slice of incorrect length");
        }

        match Self::try_new_from_slice(slice) {
            Ok(s) => s,
            Err(_) => unreachable!(),
        }
    }

    pub fn try_new_boxed_from_slice(slice: &[T]) -> Result<Box<Self>, BadLength> {
        if slice.len() != N::UNSIGNED {
            return Err(BadLength {
                found: slice.len(),
                expected: N::UNSIGNED,
            });
        }

        let mut uninit = Self::uninit_boxed();
        for i in 0..N::UNSIGNED {
            uninit[i].write(slice[i].clone());
        }

        // SAFETY: There's no difference between `MaybeUninit<T>` and `T` in memory (literally a
        // union with `()`), so perfectly safe to reinterpret the array as a whole
        Ok(unsafe { const_transmute::<_, _>(uninit) })
    }

    pub fn new_boxed_from_slice(slice: &[T]) -> Box<Self> {
        if slice.len() != N::UNSIGNED {
            panic!("tried to construct array from slice of incorrect length");
        }

        match Self::try_new_boxed_from_slice(slice) {
            Ok(s) => s,
            Err(_) => unreachable!(),
        }
    }
}

/// Transmutes from `A` to `B`, but at const evaluation time. This is equivalent to
/// [`std::mem::transmute`] in all other respects, and the same safety contracts must be upheld.
///
/// # Safety
///
/// See [`std::mem::transmute`], but in general, the bits you provide must be valid in both `A` and
/// `B`.
const unsafe fn const_transmute<A, B>(a: A) -> B {
    if std::mem::size_of::<A>() != std::mem::size_of::<B>() {
        panic!("size mismatch in const transmute");
    }

    // This will hold either `A` or `B`, and because their sizes are equal, will have whatever that
    // size is
    #[repr(C)]
    union Union<A, B> {
        a: ManuallyDrop<A>,
        b: ManuallyDrop<B>,
    }

    // We instantiate a manually-droppable version of `a` to make sure we don't drop the data when
    // we transmute
    let a = ManuallyDrop::new(a);
    // Then we put that `a` into our magical union
    let union = Union::<A, B> { a };
    // Then we extract it from said union, but as if it were `b`
    //
    // SAFETY: `a` and `b` have the same size (guaranteed above), and the caller has guaranteed to
    // us that the bits are valid in `B`
    let b_transmute = unsafe { union.b };
    // And finally we just extract from the `ManuallyDrop` so we have a normal, droppable type
    // again
    ManuallyDrop::into_inner(b_transmute)
}

/// The error that occurs when we try to convert from a slice into an [`Array<T, N>`], but the
/// length is wrong.
#[derive(Error, Debug)]
#[error("bad slice length: expected {expected}, found {found}")]
pub struct BadLength {
    found: usize,
    expected: usize,
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
use thiserror::Error;
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

#[test]
fn arrays_runtime() {
    use crate::bs;

    type A5Long = Array<u32, bs!(1, 0, 1)>;

    assert!(A5Long::try_from_slice(&[0u32; 5]).is_ok());
    assert!(A5Long::try_from_slice(&[0u32; 4]).is_err());

    let zeroed = A5Long::default();
    assert_eq!(zeroed.as_slice(), &[0u32; 5]);
}
