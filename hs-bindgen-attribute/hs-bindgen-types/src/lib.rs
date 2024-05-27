use cfg_if::cfg_if;
use core::ffi::*;
use displaydoc::Display;
use proc_macro2::TokenStream;
use quote::quote;
use thiserror::Error;

/// Enumeration of all Haskell C-FFI safe types as the string representation of
/// their token in Haskell.
///
/// FIXME: `Errno(c_int)` should be implemented as a Rust `enum` ...
/// https://hackage.haskell.org/package/base/docs/Foreign-C-Error.html
/// ... using `#[repr(i32)]` https://doc.rust-lang.org/nomicon/other-reprs.html
#[non_exhaustive]
pub enum HsType {
    /// `Int32`
    CInt,
    /// `Int8`
    CChar,
    /// `Int8`
    CSChar,
    /// `Word8`
    CUChar,
    /// `Int16`
    CShort,
    /// `Word16`
    CUShort,
    /// `Word32`
    CUInt,
    /// `Int64`
    CLong,
    /// `Word64`
    CULong,
    /// `Int64`
    CLLong,
    /// `Word64`
    CULLong,
    /// `Word8`
    CBool,
    /// `Ptr CChar`
    CString,
    /// `Double`
    CDouble,
    /// `Float`
    CFloat,
    /// `()`
    Empty,
    /// `Ptr T`
    Ptr(Box<HsType>),
    /// `IO T`
    IO(Box<HsType>),
    /// FunPtr (S -> T)
    FunPtr(Vec<HsType>),
}

impl std::fmt::Display for HsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                HsType::CBool => "CBool".to_string(),
                HsType::CChar => "CChar".to_string(),
                HsType::CDouble => "CDouble".to_string(),
                HsType::CFloat => "CFloat".to_string(),
                HsType::CInt => "CInt".to_string(),
                HsType::CLLong => "CLLong".to_string(),
                HsType::CLong => "CLong".to_string(),
                HsType::CSChar => "CSChar".to_string(),
                HsType::CShort => "CShort".to_string(),
                HsType::CString => "CString".to_string(),
                HsType::CUChar => "CUChar".to_string(),
                HsType::CUInt => "CUInt".to_string(),
                HsType::CULLong => "CULLong".to_string(),
                HsType::CULong => "CULong".to_string(),
                HsType::CUShort => "CUShort".to_string(),
                HsType::Empty => "()".to_string(),
                HsType::Ptr(x) => format!("Ptr ({x})"),
                HsType::IO(x) => format!("IO ({x})"),
                HsType::FunPtr(types) => {
                    let args: Vec<String> = types.iter().map(|arg| format!("{arg}")).collect();
                    format!("FunPtr({})", args.join(" -> "))
                }
            }
        )
    }
}

#[derive(Debug, Display, Error)]
pub enum Error {
    /** type `{0}` isn't in the list of supported Haskell C-FFI types.
     * Consider opening an issue https://github.com/yvan-sraka/hs-bindgen-types
     *
     * The list of available Haskell C-FFI types could be found here:
     * https://hackage.haskell.org/package/base/docs/Foreign-C.html
     */
    UnsupportedHsType(String),
    /// found an open `(` without the matching closing `)`
    UnmatchedParenthesis,
    /// FunPtr is missing type parameter
    FunPtrWithoutTypeArgument,
}

pub struct ArrowIter<'a> {
    remaining: &'a str,
}

impl<'a> Iterator for ArrowIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let ArrowIter { remaining } = self;

        let mut open = 0;
        let mut offset = 0;

        if remaining.trim().is_empty() {
            return None;
        }

        let mut matched: &str = "";

        for c in remaining.chars() {
            if c == '(' {
                open += 1;
            } else if c == ')' {
                open -= 1;
            } else if open == 0 && remaining[offset..].starts_with("->") {
                matched = &remaining[..offset];
                offset += "->".len();
                break;
            }

            offset += c.len_utf8();
            matched = &remaining[..offset];
        }

        *remaining = &remaining[offset..];
        Some(matched)
    }
}

impl<'a> From<&'a str> for ArrowIter<'a> {
    fn from(value: &'a str) -> Self {
        Self { remaining: value }
    }
}

impl std::str::FromStr for HsType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s == "()" {
            Ok(HsType::Empty)
        } else if !s.is_empty() && &s[..1] == "(" {
            Ok(s[1..]
                .strip_suffix(')')
                .ok_or(Error::UnmatchedParenthesis)?
                .parse()?)
        } else if s.len() >= 2 && &s[..2] == "IO" {
            Ok(HsType::IO(Box::new(s[2..].parse()?)))
        } else if s.len() >= 3 && &s[..3] == "Ptr" {
            Ok(HsType::Ptr(Box::new(s[3..].parse()?)))
        } else if s.len() >= 6 && &s[..6] == "FunPtr" {
            let mut s = s[6..].trim();

            if let Some('(') = s.chars().next() {
                s = s[1..]
                    .strip_suffix(')')
                    .ok_or(Error::UnmatchedParenthesis)?;
            }

            let types: Vec<_> = ArrowIter { remaining: s }
                .map(|s| s.parse::<Self>())
                .collect::<Result<_, _>>()?;

            if types.is_empty() {
                return Err(Error::FunPtrWithoutTypeArgument);
            }

            Ok(HsType::FunPtr(types))
        } else {
            match s {
                "CBool" => Ok(HsType::CBool),
                "CChar" => Ok(HsType::CChar),
                "CDouble" => Ok(HsType::CDouble),
                "CFloat" => Ok(HsType::CFloat),
                "CInt" => Ok(HsType::CInt),
                "CLLong" => Ok(HsType::CLLong),
                "CLong" => Ok(HsType::CLong),
                "CSChar" => Ok(HsType::CSChar),
                "CShort" => Ok(HsType::CShort),
                "CString" => Ok(HsType::CString),
                "CUChar" => Ok(HsType::CUChar),
                "CUInt" => Ok(HsType::CUInt),
                "CULLong" => Ok(HsType::CULLong),
                "CULong" => Ok(HsType::CULong),
                "CUShort" => Ok(HsType::CUShort),
                ty => Err(Error::UnsupportedHsType(ty.to_string())),
            }
        }
    }
}

impl HsType {
    /// Get the C-FFI Rust type that match the memory layout of a given HsType.
    ///
    /// This function return a `OUTPUT: proc_macro2::TokenStream` that should
    /// be valid (considered as FFI-safe by `rustc`) in the context of a block
    /// of form: `quote! { extern C fn _(_: #OUTPUT) {} }`
    ///
    /// c.f. https://doc.rust-lang.org/core/ffi/
    pub fn quote(&self) -> TokenStream {
        match self {
            // FIXME: add https://doc.rust-lang.org/core/ffi/enum.c_void.html
            HsType::CBool => quote! { bool },
            HsType::CChar => quote! { core::ffi::c_char },
            HsType::CDouble => quote! { core::ffi::c_double },
            HsType::CFloat => quote! { core::ffi::c_float },
            HsType::CInt => quote! { core::ffi::c_int },
            HsType::CLLong => quote! { core::ffi::c_longlong },
            HsType::CLong => quote! { core::ffi::c_long },
            HsType::CSChar => quote! { core::ffi::c_schar },
            HsType::CShort => quote! { core::ffi::c_short },
            HsType::CString => HsType::Ptr(Box::new(HsType::CChar)).quote(),
            HsType::CUChar => quote! { core::ffi::c_uchar },
            HsType::CUInt => quote! { core::ffi::c_uint },
            HsType::CULLong => quote! { core::ffi::c_ulonglong },
            HsType::CULong => quote! { core::ffi::c_ulong },
            HsType::CUShort => quote! { core::ffi::c_ushort },
            HsType::Empty => quote! { () },
            HsType::Ptr(x) => {
                let ty = x.quote();
                quote! { *const #ty }
            }
            HsType::IO(x) => x.quote(),
            HsType::FunPtr(types) => {
                let ret = types.last().unwrap().quote();
                let args: Vec<_> = types[..types.len() - 1]
                    .iter()
                    .map(|arg| arg.quote())
                    .collect();
                quote!(unsafe extern "C" fn(#(#args),*) -> #ret)
            }
        }
    }
}

/// Turn a given Rust type into his `HsType` target.
///
/// Deducing what's the right Haskell type target given an arbitrary Rust type
/// is provided by `reflexive` feature of `hs-bingen-derive` and rely mostly on
/// Rust type inference through this trait.
pub trait ReprHs {
    fn into() -> HsType;
}

macro_rules! repr_hs {
    ($($ty:ty => $ident:ident,)*) => {$(
        impl ReprHs for $ty {
            fn into() -> HsType {
                HsType::$ident
            }
        }
    )*};
}
pub(crate) use repr_hs;

repr_hs! {
    c_char   => CChar,
    c_double => CDouble,
    c_float  => CFloat,
    c_int    => CInt,
    c_short  => CShort,
    c_uchar  => CUChar,
    c_uint   => CUInt,
    c_ushort => CUShort,
    ()       => Empty,
}

cfg_if! {
    if #[cfg(all(target_pointer_width = "64", not(windows)))] {
        repr_hs! {
            c_long  => CLong,
            c_ulong => CULong,
        }
    } else {
        repr_hs! {
            c_longlong  => CLLong,
            c_ulonglong => CULLong,
        }
    }
}

impl<T> ReprHs for *const T
where
    T: ReprHs,
{
    fn into() -> HsType {
        HsType::Ptr(Box::new(T::into()))
    }
}

impl<T> ReprHs for *mut T
where
    T: ReprHs,
{
    fn into() -> HsType {
        HsType::Ptr(Box::new(T::into()))
    }
}

/* ********** Vector & Slices ********** */

impl<T> ReprHs for Vec<T>
where
    T: ReprHs,
{
    fn into() -> HsType {
        HsType::Ptr(Box::new(T::into()))
    }
}

impl<T, const N: usize> ReprHs for &[T; N]
where
    T: ReprHs,
{
    fn into() -> HsType {
        HsType::Ptr(Box::new(T::into()))
    }
}

/* ********** Strings ********** */

use std::ffi::CString;

repr_hs! {
    CString => CString,
    &CStr   => CString,
    String  => CString,
    &str    => CString,
}
