use crate::{haskell, reflexive};
use hs_bindgen_types::HsType;
use proc_macro::TokenStream;
use quote::{format_ident, quote};

/// Generate extra Rust code that wrap our exposed function
pub(crate) fn generate(
    attrs: TokenStream,
    item_fn: syn::ItemFn,
) -> (haskell::Signature, TokenStream) {
    let rust_fn = format_ident!("{}", item_fn.sig.ident.to_string());

    // Parse targeted Haskell function signature either from proc macro
    // attributes or either from types from Rust `fn` item (using feature
    // `reflexive` which is enabled by default) ...
    let mut sig = {
        let s = attrs.to_string();
        if cfg!(feature = "reflexive") && s.is_empty() {
            let sig = <haskell::Signature as reflexive::Eval<&syn::ItemFn>>::from(&item_fn);
            reflexive::warning(&sig);
            sig
        } else {
            s.parse().unwrap_or_else(|e| panic!("{e}"))
        }
    };

    // Ensure that signature not contain too much args ...
    if sig.fn_type.len() > 8 {
        panic!(
            "Too many arguments! GHC C-ABI implementation does not currently behave well \
with function with more than 8 arguments on platforms apart from x86_64 ..."
        )
    }

    let ret = match sig.fn_type.pop().unwrap_or(HsType::Empty) {
        HsType::IO(x) => x,
        x => Box::new(x),
    };

    // Iterate through function argument types ...
    let mut c_fn_args = quote! {};
    let mut rust_fn_values = quote! {};
    for (i, hs_c_ffi_type) in sig.fn_type.iter().enumerate() {
        let arg = format_ident!("__{i}");
        let c_ffi_safe_type = hs_c_ffi_type.quote();
        c_fn_args.extend(quote! { #arg: #c_ffi_safe_type, });
        rust_fn_values.extend(quote! { traits::FromReprRust::from(#arg), });
    }

    // Generate C-FFI wrapper of Rust function ...
    let c_fn = format_ident!("__c_{}", sig.fn_name);
    let c_ret = ret.quote();
    let extern_c_wrapper = quote! {
        #[no_mangle] // Mangling makes symbol names more difficult to predict.
                     // We disable it to ensure that the resulting symbol is really `#c_fn`.
        extern "C" fn #c_fn(#c_fn_args) -> #c_ret {
            // `traits` module is `hs-bindgen::hs-bindgen-traits`
            // n.b. do not forget to import it, e.g., with `use hs-bindgen::*`
            traits::FromReprC::from(#rust_fn(#rust_fn_values))
        }
    };

    sig.fn_type.push(HsType::IO(ret));
    (sig, extern_c_wrapper.into())
}
