// Copyright (C) 2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate proc_macro;

use std::ops::Deref as _;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream as Tokens;

use quote::quote;
use quote::ToTokens as _;

use syn::parse_macro_input;
use syn::Attribute;
use syn::Error;
use syn::FnArg;
use syn::ItemFn;
use syn::Pat;
use syn::Result;
use syn::ReturnType;
use syn::Signature;
use syn::Type;


#[derive(Debug)]
enum Kind {
    Test,
    Bench,
}

impl Kind {
    #[inline]
    fn as_str(&self) -> &str {
        match self {
            Self::Test => "test",
            Self::Bench => "bench",
        }
    }
}


/// A procedural macro for running a test in a separate process.
///
/// # Example
///
/// Use the attribute for all tests in scope:
/// ```rust,ignore
/// use test_fork::test;
///
/// #[test]
/// fn test1() {
///   assert_eq!(2 + 2, 4);
/// }
/// ```
///
/// Use it only on a single test:
/// ```rust,ignore
/// #[test_fork::test]
/// fn test2() {
///   assert_eq!(2 + 3, 5);
/// }
/// ```
#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let has_test = input_fn
        .attrs
        .iter()
        .any(|attr| is_attribute_kind(Kind::Test, attr));
    let inner_test = if has_test {
        quote! {}
    } else {
        quote! { #[::core::prelude::v1::test]}
    };

    try_test(attr, input_fn, inner_test)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}


/// A procedural macro for running a benchmark in a separate process.
///
/// # Example
///
/// Use the attribute for all benchmarks in scope:
/// ```rust,ignore
/// use test_fork::bench;
///
/// #[bench]
/// fn bench1(b: &mut Bencher) {
///   b.iter(|| sleep(Duration::from_millis(1)));
/// }
/// ```
///
/// Use it only on a single benchmark:
/// ```rust,ignore
/// #[test_fork::bench]
/// fn bench2(b: &mut Bencher) {
///   b.iter(|| sleep(Duration::from_millis(1)));
/// }
#[cfg(all(feature = "unstable", feature = "unsound"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "unstable", feature = "unsound"))))]
#[proc_macro_attribute]
pub fn bench(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let has_bench = input_fn
        .attrs
        .iter()
        .any(|attr| is_attribute_kind(Kind::Bench, attr));
    let inner_bench = if has_bench {
        quote! {}
    } else {
        quote! { #[::core::prelude::v1::bench]}
    };

    try_bench(attr, input_fn, inner_bench)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}


/// A procedural macro for running a test in a separate process.
///
/// Contrary to #[[macro@test]], this attribute does not in itself make
/// a function a test, so it will *always* have to be combined with an
/// additional `#[test]` attribute. However, it can be more convenient
/// for annotating only a sub-set of tests for running in separate
/// processes, especially when non-standard `#[test]` attributes are
/// involved:
///
/// # Example
///
/// ```rust,ignore
/// use test_fork::fork;
///
/// #[fork]
/// #[test]
/// fn test3() {
///   assert_eq!(2 + 4, 6);
/// }
/// ```
#[proc_macro_attribute]
pub fn fork(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let has_test = input_fn
        .attrs
        .iter()
        .any(|attr| is_attribute_kind(Kind::Test, attr));
    if !has_test {
        return Error::new_spanned(
            Tokens::from(attr),
            "test_fork::fork requires an inner #[test] attribute",
        )
        .into_compile_error()
        .into();
    }

    let inner_test = quote! {};
    try_test(attr, input_fn, inner_test)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}


/// Check whether given attribute is a test or bench attribute of the
/// form:
/// - `#[<kind>]`
/// - `#[core::prelude::*::<kind>]` or `#[::core::prelude::*::<kind>]`
/// - `#[std::prelude::*::<kind>]` or `#[::std::prelude::*::<kind>]`
fn is_attribute_kind(kind: Kind, attr: &Attribute) -> bool {
    let path = match &attr.meta {
        syn::Meta::Path(path) => path,
        _ => return false,
    };
    let candidates = [
        ["core", "prelude", "*", kind.as_str()],
        ["std", "prelude", "*", kind.as_str()],
    ];
    if path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments[0].arguments.is_none()
        && path.segments[0].ident == kind.as_str()
    {
        return true;
    } else if path.segments.len() != candidates[0].len() {
        return false;
    }
    candidates.into_iter().any(|segments| {
        path.segments.iter().zip(segments).all(|(segment, path)| {
            segment.arguments.is_none() && (path == "*" || segment.ident == path)
        })
    })
}

fn try_test(attr: TokenStream, input_fn: ItemFn, inner_test: Tokens) -> Result<Tokens> {
    if !attr.is_empty() {
        return Err(Error::new_spanned(
            Tokens::from(attr),
            "the attribute does not currently accept arguments",
        ))
    }

    let ItemFn {
        attrs,
        vis,
        mut sig,
        block,
    } = input_fn;

    let test_name = sig.ident.clone();
    let mut body_fn_sig = sig.clone();
    body_fn_sig.ident = Ident::new("body_fn", Span::call_site());
    // Our tests currently basically have to return (), because we don't
    // have a good way of conveying the result back from the child
    // process.
    sig.output = ReturnType::Default;

    let augmented_test = quote! {
        #inner_test
        #(#attrs)*
        #vis #sig {
            #body_fn_sig
            #block

            ::test_fork::test_fork_core::fork(
                ::test_fork::test_fork_core::fork_id!(),
                ::test_fork::test_fork_core::fork_test_name!(#test_name),
                body_fn as fn() -> _,
            ).expect("forking test failed")
        }
    };

    Ok(augmented_test)
}

fn parse_bench_sig(sig: &Signature) -> Option<(Pat, Type)> {
    if sig.inputs.len() != 1 {
        return None
    }

    if let FnArg::Typed(pat_type) = sig.inputs.first().unwrap() {
        let ty = match pat_type.ty.deref() {
            Type::Reference(ty_ref) => ty_ref.elem.clone(),
            _ => return None,
        };
        Some((*pat_type.pat.clone(), *ty))
    } else {
        None
    }
}

fn try_bench(attr: TokenStream, input_fn: ItemFn, inner_bench: Tokens) -> Result<Tokens> {
    if !attr.is_empty() {
        return Err(Error::new_spanned(
            Tokens::from(attr),
            "the attribute does not currently accept arguments",
        ))
    }

    let ItemFn {
        attrs,
        vis,
        mut sig,
        block,
    } = input_fn;

    let (bencher_name, bencher_ty) = parse_bench_sig(&sig).ok_or_else(|| {
        Error::new_spanned(
            sig.to_token_stream(),
            "benchmark function has unexpected signature (expected single `&mut Bencher` argument)",
        )
    })?;

    let test_name = sig.ident.clone();
    let mut body_fn_sig = sig.clone();
    body_fn_sig.ident = Ident::new("body_fn", Span::call_site());
    sig.output = ReturnType::Default;

    let augmented_bench = quote! {
        #inner_bench
        #(#attrs)*
        #vis #sig {
            #body_fn_sig
            #block

            use ::std::mem::size_of;
            use ::std::mem::transmute;

            type BencherBuf = [u8; size_of::<#bencher_ty>()];

            // SAFETY: Probably unsound. We can't guarantee that the
            //         `Bencher` type is just a bunch of bytes that we
            //         can copy around. And yet, that's the best we can
            //         do.
            let buf_ref = unsafe {
                transmute::<&mut #bencher_ty, &mut BencherBuf>(#bencher_name)
            };

            fn wrapper_fn(buf_ref: &mut [u8]) {
                let buf_ref = <&mut BencherBuf>::try_from(buf_ref).unwrap();
                // SAFETY: See above.
                let bench_ref = unsafe {
                    transmute::<&mut BencherBuf, &mut #bencher_ty>(buf_ref)
                };
                let () = body_fn(bench_ref);
            }

            ::test_fork::test_fork_core::fork_in_out(
                ::test_fork::test_fork_core::fork_id!(),
                ::test_fork::test_fork_core::fork_test_name!(#test_name),
                wrapper_fn as fn(&mut [u8]) -> _,
                buf_ref,
            ).expect("forking test failed")
        }
    };

    Ok(augmented_bench)
}
