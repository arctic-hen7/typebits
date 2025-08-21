// WARN: This file contains experimental code that might turn into a compile-time conditional in
// future, but for now it's just informed the other systems throughout this crate. Use at own
// risk!!

#[macro_export]
macro_rules! conditional_system {
    ($empty_data:ty $(, $($data_bounds:tt)+)?) => {
        pub type If<Cond /*: BooleanLike*/, T, F> =
            <<<<Cond as BooleanLike>::Boolean as Boolean>::And<PrivateTrue<T>> as Boolean>::Or<
                <<<Cond as BooleanLike>::Boolean as Boolean>::Not as Boolean>::And<PrivateTrue<F>>,
            > as Boolean>::Data;

        pub type BetterIf<Cond, T, F> = <() as Dispatcher<<Cond as BooleanLike>::Boolean, T, F>>::Output;

        pub struct True;
        pub struct False;
        pub trait BooleanLike {
            type Boolean: Boolean;
        }
        impl BooleanLike for True {
            type Boolean = PrivateTrue<$empty_data>;
        }
        impl BooleanLike for False {
            type Boolean = PrivateFalse;
        }

        pub trait Dispatcher<Bool: Boolean, T $( : $($data_bounds)+ )?, F $( : $($data_bounds)+ )?> {
            type Output $( : $($data_bounds)+ )?;
        }
        impl<D $( : $($data_bounds)+ )?, T $( : $($data_bounds)+ )?, F $( : $($data_bounds)+ )?> Dispatcher<PrivateTrue<D>, T, F> for () {
            type Output = T;
        }
        impl<T $( : $($data_bounds)+ )?, F $( : $($data_bounds)+ )?> Dispatcher<PrivateFalse, T, F> for () {
            type Output = F;
        }

        pub struct PrivateTrue<T $( : $($data_bounds)+ )?> {
            _phantom: ::std::marker::PhantomData<T>,
        }
        pub struct PrivateFalse;

        pub trait Boolean {
            type Data $( : $($data_bounds)+ )?;

            type And<Other: Boolean>: Boolean;
            type Or<Other: Boolean>: Boolean;
            type Not: Boolean;
        }
        impl<T $( : $($data_bounds)+ )?> Boolean for PrivateTrue<T> {
            type Data = T;

            // Ordering of data dropping is specifically structured to support conditionals
            type And<Other: Boolean> = Other; // Drop our data, use the other carrier's
            type Or<Other: Boolean> = PrivateTrue<T>; // Drop the other carrier's data, use our own
            type Not = PrivateFalse;
        }
        impl Boolean for PrivateFalse {
            type Data = $empty_data;

            type And<Other: Boolean> = PrivateFalse;
            type Or<Other: Boolean> = Other;
            type Not = PrivateTrue<$empty_data>;
        }
    };
}

pub struct True;
pub struct False;

mod sealed {
    pub trait SealedBoolean {}
    impl SealedBoolean for super::True {}
    impl SealedBoolean for super::False {}
}

pub trait Boolean: sealed::SealedBoolean {
    type Select<Then: Lazy, Else: Lazy>: Lazy;
}
impl Boolean for True {
    type Select<Then: Lazy, Else: Lazy> = Then;
}
impl Boolean for False {
    type Select<Then: Lazy, Else: Lazy> = Else;
}

pub trait Lazy {
    type Output: Item;
}

pub struct Thunk<T> {
    _phantom: ::std::marker::PhantomData<T>,
}
impl<T: Item> Lazy for Thunk<T> {
    type Output = T;
}

pub type If<B, Then, Else> = <<B as Boolean>::Select<Then, Else> as Lazy>::Output;

// ---

struct Cons<H: ItemList, T: Item> {
    _phantom: ::std::marker::PhantomData<(H, T)>,
}

#[derive(Default)]
pub struct Term;
#[derive(Default)]
pub struct Other;

pub trait Item: Default {
    type IsTerm: Boolean;
}
impl Item for Term {
    type IsTerm = True;
}
impl Item for Other {
    type IsTerm = False;
}

pub trait ItemList {
    type Head: ItemList;
    type Tail: ItemList;

    type IsTerm: Boolean;
}
impl<H: ItemList, T: Item> ItemList for Cons<H, T> {
    type Head = H;
    type Tail = T;

    type IsTerm = False;
}
impl<I: Item> ItemList for I {
    type Head = Term;
    type Tail = I;

    type IsTerm = <I as Item>::IsTerm;
}

pub struct Recurse<L> {
    _phantom: ::std::marker::PhantomData<L>,
}
impl<L: ItemList> Lazy for Recurse<L> {
    type Output = <L::Head as GetTerm>::Term;
}

pub trait GetTerm {
    type Term: Item;
}
impl<L: ItemList> GetTerm for L {
    type Term = If<<L::Tail as ItemList>::IsTerm, Thunk<Term>, Recurse<L>>;
}

fn test() {
    type List = Cons<Cons<Cons<Term, Other>, Other>, Other>;

    let x = <List as GetTerm>::Term::default();
}
