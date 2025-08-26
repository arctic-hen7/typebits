/// A global value indicating truth.
pub struct GlobalTrue;
/// A global value indicating falsehood.
pub struct GlobalFalse;

mod sealed {
    /// A private trait used to make sure the user can't add more booleans.
    pub trait SealedBoolean {}
    impl SealedBoolean for super::GlobalTrue {}
    impl SealedBoolean for super::GlobalFalse {}
}

/// A trait for "global" booleans. These have core boolean logic, and connect to the type-specific
/// booleans we use for conditionals. Unfortunately, precisely what those downstream booleans are
/// need to be statically known here, until we have specialisation to convert with fallbacks.
pub trait GlobalBoolean: sealed::SealedBoolean {
    type And<Other: GlobalBoolean>: GlobalBoolean;
    type Or<Other: GlobalBoolean>: GlobalBoolean;
    type Not: GlobalBoolean;

    type BitstringBoolean: crate::bits::bitstring_conditionals::Boolean;
    #[cfg(feature = "array")]
    type ArrayBoolean: crate::array::array_conditionals::Boolean;
}
impl GlobalBoolean for GlobalTrue {
    type And<Other: GlobalBoolean> = Other;
    type Or<Other: GlobalBoolean> = GlobalTrue;
    type Not = GlobalFalse;

    type BitstringBoolean = crate::bits::bitstring_conditionals::True;
    #[cfg(feature = "array")]
    type ArrayBoolean = crate::array::array_conditionals::True;
}
impl GlobalBoolean for GlobalFalse {
    type And<Other: GlobalBoolean> = GlobalFalse;
    type Or<Other: GlobalBoolean> = Other;
    type Not = GlobalTrue;

    type BitstringBoolean = crate::bits::bitstring_conditionals::False;
    #[cfg(feature = "array")]
    type ArrayBoolean = crate::array::array_conditionals::False;
}

/// Creates a conditional system with the given visibility and bounds. This will produce a module
/// of the given name (e.g. `conditional_system!(pub my_conditionals, MyBound)`). The bounds will
/// be applied to the outputs of any conditional.
#[macro_export]
macro_rules! conditional_system {
    ($vis:vis $name:ident $(, $($data_bounds:tt)+)?) => {
        $vis mod $name {
            /// A system-specific type indicating truth.
            pub struct True;
            /// A system-specific type indicating falsehood.
            pub struct False;

            mod sealed {
                pub trait SealedBoolean {}
                impl SealedBoolean for super::True {}
                impl SealedBoolean for super::False {}
            }

            /// A trait for our internal, system-specific boolean types.
            pub trait Boolean: sealed::SealedBoolean {
                /// An associated type for conditionals, where both branches must implement
                /// [`Lazy`] in a strategy for avoiding immediate evaluation by the compiler.
                type Select<Then: Lazy, Else: Lazy>: Lazy;

                /// An associated type that takes us *back* to the global boolean types.
                type GlobalBoolean: $crate::conditional::GlobalBoolean;
            }
            impl Boolean for True {
                type Select<Then: Lazy, Else: Lazy> = Then;

                type GlobalBoolean = $crate::conditional::GlobalTrue;
            }
            impl Boolean for False {
                type Select<Then: Lazy, Else: Lazy> = Else;

                type GlobalBoolean = $crate::conditional::GlobalFalse;
            }

            /// A trait used as a hack to delay evaluation. This must be unique to each conditional
            /// system, as the bounds on the associated output type define where the system is
            /// generically useful.
            pub trait Lazy {
                /// The thing that will be computed lazily.
                type Output $( : $($data_bounds)+ )?;
            }

            /// A simple wrapper that implements [`Lazy`] for whatever you put in it. Anything that
            /// doesn't need to be recursive should go in this.
            pub struct Thunk<T> {
                _phantom: ::std::marker::PhantomData<T>,
            }
            impl<T $( : $($data_bounds)+ )?> Lazy for Thunk<T> {
                type Output = T;
            }

            /// A type-level conditional, where the condition implements [`Boolean`] and the
            /// branches implement [`Lazy`].
            pub type If<Cond, T, F> = <<Cond as Boolean>::Select<T, F> as Lazy>::Output;
            /// A simple conditional that wraps both its branches in [`Thunk`]s. This should be
            /// used when you don't have recursion.
            pub type SimpleIf<Cond, T, F> = If<Cond, Thunk<T>, Thunk<F>>;
        }
    };
}
