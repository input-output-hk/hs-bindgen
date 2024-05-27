use crate::{private, FromReprRust};

macro_rules! repr_rust_fn {
    () => {
        impl<Output> FromReprRust<unsafe extern "C" fn() -> Output> for Box<dyn Fn() -> Output>
          where
            Output: private::CFFISafe + 'static,
        {
            fn from(f: unsafe extern "C" fn() -> Output) -> Self {
                unsafe { Box::new(move || f())}
            }
        }
    };
    ($x:ident, $y:ident $(,$xs:ident, $ys: ident)*) => {
        repr_rust_fn!($( $xs, $ys ),*);
        impl<$x, $($xs,)* Output> FromReprRust<unsafe extern "C" fn($x $(,$xs)*) -> Output> for Box<dyn Fn($x $(,$xs)*) -> Output>
          where
            Output: private::CFFISafe + 'static,
            $x: private::CFFISafe + 'static$(,
            $xs: private::CFFISafe + 'static)*
        {
            fn from(f: unsafe extern "C" fn($x $(, $xs )*) -> Output) -> Self {
                unsafe { Box::new(move |$y $(,$ys)*| f($y $(,$ys)*))}
            }
        }
    };
}

repr_rust_fn!(A, a, B, b, C, c, D, d, E, e, F, f);
