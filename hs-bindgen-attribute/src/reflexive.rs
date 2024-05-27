use crate::haskell;
#[cfg(feature = "reflexive")]
use hs_bindgen_types::HsType;

#[cfg(feature = "reflexive")]
lazy_static::lazy_static! {
    static ref SANDBOX: reflexive::Sandbox =
        reflexive::Sandbox::new("hs-bindgen")
            .unwrap()
            .deps(&["hs-bindgen-types@0.8"])
            .unwrap()
    ;
}

/// Use Rust type inference (inside a `reflexive` sandbox) to deduce targeted
/// Haskell type signature that match a given `TokenStream` of a Rust `fn`
pub(crate) trait Eval<T> {
    fn from(_: T) -> Self;
}

impl Eval<&syn::ItemFn> for haskell::Signature {
    #[cfg(feature = "reflexive")]
    fn from(item_fn: &syn::ItemFn) -> Self {
        let fn_name = item_fn.sig.ident.to_string();
        let fn_safe = true;
        let mut fn_type = vec![];
        for arg in &item_fn.sig.inputs {
            fn_type.push(<HsType as Eval<&syn::Type>>::from(match arg {
                syn::FnArg::Typed(p) => &p.ty,
                _ => panic!("functions using `self` are not supported by `hs-bindgen`"),
            }));
        }
        fn_type.push(HsType::IO(Box::new(match &item_fn.sig.output {
            syn::ReturnType::Type(_, p) => <HsType as Eval<&syn::Type>>::from(p),
            _ => HsType::Empty,
        })));
        haskell::Signature {
            fn_name,
            fn_safe,
            fn_type,
        }
    }
    #[cfg(not(feature = "reflexive"))]
    fn from(_: &syn::ItemFn) -> Self {
        unreachable!()
    }
}

#[cfg(feature = "reflexive")]
impl Eval<&syn::Type> for HsType {
    fn from(ty: &syn::Type) -> HsType {
        use quote::quote;
        SANDBOX
            .eval(quote! {
                <#ty as hs_bindgen_types::ReprHs>::into()
            })
            .unwrap_or_else(|_| {
                panic!(
                    "type `{}` doesn't implement `ReprHs` trait
consider opening an issue https://github.com/yvan-sraka/hs_bindgen_types

n.b. if you trying to use a custom defined type, you need to specify the
Haskell type signature of your binding: #[hs_bindgen(HASKELL TYPE SIGNATURE)]",
                    quote! { #ty }
                )
            })
    }
}

/// Warn user about the build-time cost of relying on `reflexive` ...
///
/// n.b. proc-macro diagnostics require nightly `proc_macro_diagnostic` feature
pub(crate) fn warning(_sig: &haskell::Signature) {
    #[cfg(DIAGNOSTICS)]
    proc_macro::Diagnostic::spanned(
        [proc_macro::Span::call_site()].as_ref(),
        proc_macro::Level::Warning,
        format!(
            "Implicit Haskell signature declaration could slow down compilation,
rather derive it as: #[hs_bindgen({_sig})]"
        ),
    )
    .emit();
}
