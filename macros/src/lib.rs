// Copyright (C) 2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream as Tokens;

use quote::quote;

use syn::parse_macro_input;
use syn::Attribute;
use syn::Error;
use syn::ItemFn;
use syn::Result;
use syn::ReturnType;


#[derive(Debug)]
enum Kind {
    Test,
}

impl Kind {
    #[inline]
    fn as_str(&self) -> &str {
        match self {
            Self::Test => "test",
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
