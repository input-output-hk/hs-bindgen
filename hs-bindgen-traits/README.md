<!-- cargo-sync-readme start -->

# `hs-bingen-traits`

Utility traits behind [`hs-bindgen`](https://github.com/yvan-sraka/hs-bindgen)
ergonomics. It helps user to easily define wrapper function to derive a Rust
type from and into a C-FFI safe target type (that match the memory layout of
an Haskell type).

## What's this library for?

[Does `repr(C)` define a trait I can use to check structs were declared with `#repr(C)`?](https://users.rust-lang.org/t/16323)
The answer is sadly no ... that's what this library trying to provide, like
what [`safer_ffi`](https://docs.rs/safer-ffi/latest/safer_ffi/layout/trait.ReprC.html)
does, but in a simpler and more minimal way, since the goal here is only to
target Haskell FFI.

## Acknowledgments

⚠️ This is still a working experiment, not yet production ready.

This project was part of a work assignment as an
[IOG](https://github.com/input-output-hk) contractor.

## License

Licensed under either of [Apache License](LICENSE-APACHE), Version 2.0 or
[MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.

<!-- cargo-sync-readme end -->
