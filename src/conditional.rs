// WARN: This file contains experimental code that might turn into a compile-time conditional in
// future, but for now it's just informed the other systems throughout this crate. Use at own
// risk!!

pub struct GlobalTrue;
pub struct GlobalFalse;

mod sealed {
    pub trait SealedBoolean {}
    impl SealedBoolean for super::GlobalTrue {}
    impl SealedBoolean for super::GlobalFalse {}
}

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

//
// // (Cond AND T) OR (NOT Cond AND F); because both `T` and `F` are `DataCarrier`s, they evaluate as
// // booleans to true, meaning the result is guaranteed to be one of them
// pub type RawIf<Cond /*: Boolean*/, T /*: DataCarrier*/, F /*: DataCarrier*/> =
//     <<<Cond as Boolean>::And<T> as Boolean>::Or<
//         <<Cond as Boolean>::Not as Boolean>::And<F>,
//     > as Boolean>::Data<T>;
//
// // ---
//
// trait Item {}
// struct Item1;
// impl Item for Item1 {}
// struct Item2;
// impl Item for Item2 {}
// impl Item for () {}
//
// pub struct Data<T: Item> {
//     _phantom: ::std::marker::PhantomData<T>,
// }
// impl<T: Item> DataCarrier for Data<T> {
//     type Default = Data<()>;
// }
//
// #[test]
// fn test() {
//     use std::any::type_name;
//
//     type Test<B> = RawIf<B, Data<Item1>, Data<Item2>>;
//
//     println!("{}", type_name::<Test<True>>());
// }

// pub type If<Cond /*: BooleanLike*/, T, F> =
//     <<<<Cond as BooleanLike>::Boolean as Boolean>::And<PrivateTrue<T>> as Boolean>::Or<
//         <<<Cond as BooleanLike>::Boolean as Boolean>::Not as Boolean>::And<PrivateTrue<F>>,
//     > as Boolean>::Data;
//
// pub type BetterIf<Cond, T, F> = <() as Dispatcher<<Cond as BooleanLike>::Boolean, T, F>>::Output;
//
// pub struct True;
// pub struct False;
// pub trait BooleanLike {
//     type Boolean: Boolean;
// }
// impl BooleanLike for True {
//     type Boolean = PrivateTrue<$empty_data>;
// }
// impl BooleanLike for False {
//     type Boolean = PrivateFalse;
// }
//
// pub trait Dispatcher<Bool: Boolean, T $( : $($data_bounds)+ )?, F $( : $($data_bounds)+ )?> {
//     type Output $( : $($data_bounds)+ )?;
// }
// impl<D $( : $($data_bounds)+ )?, T $( : $($data_bounds)+ )?, F $( : $($data_bounds)+ )?> Dispatcher<PrivateTrue<D>, T, F> for () {
//     type Output = T;
// }
// impl<T $( : $($data_bounds)+ )?, F $( : $($data_bounds)+ )?> Dispatcher<PrivateFalse, T, F> for () {
//     type Output = F;
// }
//
// pub struct PrivateTrue<T $( : $($data_bounds)+ )?> {
//     _phantom: ::std::marker::PhantomData<T>,
// }
// pub struct PrivateFalse;
//
// pub trait Boolean {
//     type Data $( : $($data_bounds)+ )?;
//
//     type And<Other: Boolean>: Boolean;
//     type Or<Other: Boolean>: Boolean;
//     type Not: Boolean;
// }
// impl<T $( : $($data_bounds)+ )?> Boolean for PrivateTrue<T> {
//     type Data = T;
//
//     // Ordering of data dropping is specifically structured to support conditionals
//     type And<Other: Boolean> = Other; // Drop our data, use the other carrier's
//     type Or<Other: Boolean> = PrivateTrue<T>; // Drop the other carrier's data, use our own
//     type Not = PrivateFalse;
// }
// impl Boolean for PrivateFalse {
//     type Data = $empty_data;
//
//     type And<Other: Boolean> = PrivateFalse;
//     type Or<Other: Boolean> = Other;
//     type Not = PrivateTrue<$empty_data>;
// }

// TODO: From that, implement a connector that does `Select`

// pub trait Selector {
//     type Select<Then: Lazy, Else: Lazy>: Lazy;
// }
//
// pub trait Lazy {
//     type Output: Item;
// }
//
// pub struct Thunk<T> {
//     _phantom: ::std::marker::PhantomData<T>,
// }
// impl<T: Item> Lazy for Thunk<T> {
//     type Output = T;
// }
//
// pub type If<B, Then, Else> = <<B as Selector>::Select<Then, Else> as Lazy>::Output;
//
// // ---
//
// struct Cons<H: ItemList, T: Item> {
//     _phantom: ::std::marker::PhantomData<(H, T)>,
// }
//
// // --- User code ---
//
// #[derive(Default)]
// pub struct Term;
// #[derive(Default)]
// pub struct Other;
//
// pub trait Item: Default {
//     type IsTerm: Boolean; // The generic boolean types
// }
// impl Item for Term {
//     type IsTerm = True;
// }
// impl Item for Other {
//     type IsTerm = False;
// }
//
// pub trait ItemList {
//     type Head: ItemList;
//     type Tail: ItemList;
//
//     type IsTerm: Boolean;
// }
// impl<H: ItemList, T: Item> ItemList for Cons<H, T> {
//     type Head = H;
//     type Tail = T;
//
//     type IsTerm = False;
// }
// impl<I: Item> ItemList for I {
//     type Head = Term;
//     type Tail = I;
//
//     type IsTerm = <I as Item>::IsTerm;
// }
//
// // Should come from macro!
// pub struct Recurse<L> {
//     _phantom: ::std::marker::PhantomData<L>,
// }
// impl<L: ItemList> Lazy for Recurse<L> {
//     type Output = <L::Head as GetTerm>::Term;
// }
//
// pub trait GetTerm {
//     type Term: Item;
// }
// impl<L: ItemList> GetTerm for L {
//     // Because `Term: Item`, the conditional itself needs to be bounded
//     type Term = If<<L::Tail as ItemList>::IsTerm, Thunk<Term>, Recurse<L>>;
// }
//
// fn test() {
//     type List = Cons<Cons<Cons<Term, Other>, Other>, Other>;
//
//     let x = <List as GetTerm>::Term::default();
// }
