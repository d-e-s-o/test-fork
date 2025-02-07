// Copyright (C) 2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;

use quote::quote;

use syn::parse_macro_input;
use syn::Attribute;
use syn::Error;
use syn::ItemFn;
use syn::Result;


#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    try_test(attr, input_fn)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Check whether given attribute is a test attribute of forms:
/// - `#[test]`
/// - `#[core::prelude::*::test]` or `#[::core::prelude::*::test]`
/// - `#[std::prelude::*::test]` or `#[::std::prelude::*::test]`
fn is_test_attribute(attr: &Attribute) -> bool {
    let path = match &attr.meta {
        syn::Meta::Path(path) => path,
        _ => return false,
    };
    let candidates = [
        ["core", "prelude", "*", "test"],
        ["std", "prelude", "*", "test"],
    ];
    if path.leading_colon.is_none()
        && path.segments.len() == 1
        && path.segments[0].arguments.is_none()
        && path.segments[0].ident == "test"
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

fn try_test(attr: TokenStream, input_fn: ItemFn) -> Result<Tokens> {
    if !attr.is_empty() {
        return Err(Error::new_spanned(
            Tokens::from(attr),
            "test_fork::test does not currently accept arguments",
        ))
    }

    let has_test = input_fn.attrs.iter().any(is_test_attribute);
    let inner_test = if has_test {
        quote! {}
    } else {
        quote! { #[::core::prelude::v1::test]}
    };

    let augmented_test = quote! {
        ::test_fork::rusty_fork_test! {
            #inner_test
            #input_fn
        }
    };

    Ok(augmented_test)
}
