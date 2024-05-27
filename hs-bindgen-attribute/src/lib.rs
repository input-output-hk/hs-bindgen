//! # `hs-bindgen-attribute`
//!
//! This library define the `#[hs_bindgen]` procedural macro used by
//! [`hs-bindgen`](https://github.com/yvan-sraka/hs-bindgen) library.
//!
//! ## Acknowledgments
//!
//! ⚠️ This is still a working experiment, not yet production ready.
//!
//! This project was part of a work assignment as an
//! [IOG](https://github.com/input-output-hk) contractor.
//!
//! ## License
//!
//! Licensed under either of [Apache License](LICENSE-APACHE), Version 2.0 or
//! [MIT license](LICENSE-MIT) at your option.
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted
//! for inclusion in this project by you, as defined in the Apache-2.0 license,
//! shall be dual licensed as above, without any additional terms or conditions.

#![forbid(unsafe_code)]
#![cfg_attr(DIAGNOSTICS, feature(proc_macro_diagnostic))]

use proc_macro::TokenStream;
use std::{fs, path::Path, sync::Mutex};

mod haskell;
mod reflexive;
mod rust;
mod toml;

#[proc_macro_attribute]
pub fn hs_bindgen(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut output = input.clone();
    let item_fn: syn::ItemFn = syn::parse(input)
        .expect("failed to parse as Rust code the content of `#[hs_bindgen]` macro");

    // Generate extra Rust code that wrap our exposed function ...
    let (signature, extern_c_wrapper) = rust::generate(attrs, item_fn);

    // Neat hack to keep track of all exposed functions ...
    static SIGNATURES: Mutex<Vec<haskell::Signature>> = Mutex::new(vec![]);
    let signatures = &mut *SIGNATURES.lock().unwrap();
    signatures.push(signature);

    // Generate Haskell bindings into module defined in `hsbindgen.toml` config ...
    let module = toml::config()
        .default
        .expect("your `hsbindgen.toml` file should contain a `default` field");
    let cargo_manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("environment variable `CARGO_MANIFEST_DIR` must be set");
    let path = Path::new(&cargo_manifest_dir).join(format!("src/{}.hs", module));
    fs::write(&path, haskell::template(&module, signatures))
        .unwrap_or_else(|_| panic!("fail to write `{}` file", path.display()));

    output.extend(extern_c_wrapper);
    output
}
