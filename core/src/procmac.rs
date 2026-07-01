// Copyright (C) 2026 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::ops::Deref as _;

use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream as Tokens;

use quote::quote;
use quote::ToTokens as _;

use syn::parse_quote;
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

    #[expect(clippy::indexing_slicing)]
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


/// Check whether the given attribute is a `#[should_panic]` attribute.
fn is_should_panic(attr: &Attribute) -> bool {
    match &attr.meta {
        syn::Meta::Path(path) => path.is_ident("should_panic"),
        _ => false,
    }
}


/// Testable implementation of the `#[test]` attribute's core logic.
pub fn try_test(attr: Tokens, input_fn: ItemFn) -> Result<Tokens> {
    let has_test = input_fn
        .attrs
        .iter()
        .any(|attr| is_attribute_kind(Kind::Test, attr));
    let inner_test = if has_test {
        quote! {}
    } else {
        quote! { #[::core::prelude::v1::test] }
    };

    try_test_inner(attr, input_fn, inner_test)
}

fn try_test_inner(attr: Tokens, input_fn: ItemFn, inner_test: Tokens) -> Result<Tokens> {
    if !attr.is_empty() {
        return Err(Error::new_spanned(
            attr,
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

    let fork_call = quote! {
        ::test_fork::test_fork_core::fork(
            ::test_fork::test_fork_core::fork_id!(),
            ::test_fork::test_fork_core::fork_test_name!(#test_name),
            body_fn as fn() -> _,
        ).expect("forking test failed")
    };

    let (output, tail) = if attrs.iter().any(is_should_panic) {
        // `#[should_panic]` requires the test function to return `()`,
        // and libtest decides pass/fail based on whether the function
        // panics. The actual test body runs in the forked child, so we
        // translate a failing child (which includes a panicking one)
        // back into a panic here, which libtest's `#[should_panic]`
        // handling then observes.
        // TODO: This isn't super clean, because we could conceivably
        //       report an error for other reasons or panic due to a
        //       test-fork setup failure, but it seems the best we can
        //       do?
        (
            ReturnType::Default,
            quote! {
                ::std::assert_eq!(#fork_call, ::std::process::ExitCode::SUCCESS);
            },
        )
    } else {
        // Our test "shims" otherwise map actual test success/failure to
        // a mere `ExitCode`. This way, we introduce the least
        // perturbance compared to, say, `std::process::exit` or
        // `panic`, both of which would be caught by libtest's harness
        // and result in additional output (confusing error messages or
        // additional backtraces, respectively).
        (parse_quote!(-> ::std::process::ExitCode), fork_call)
    };
    sig.output = output;

    let augmented_test = quote! {
        #inner_test
        #(#attrs)*
        #vis #sig {
            #body_fn_sig
            #block

            #tail
        }
    };

    Ok(augmented_test)
}

fn parse_bench_sig(sig: &Signature) -> Option<(Pat, Type)> {
    if sig.inputs.len() != 1 {
        return None
    }

    if let FnArg::Typed(pat_type) = sig.inputs.first()? {
        let ty = match pat_type.ty.deref() {
            Type::Reference(ty_ref) => ty_ref.elem.clone(),
            _ => return None,
        };
        Some((*pat_type.pat.clone(), *ty))
    } else {
        None
    }
}

/// Testable implementation of the `#[bench]` attribute's core logic.
pub fn try_bench(attr: Tokens, input_fn: ItemFn) -> Result<Tokens> {
    let has_bench = input_fn
        .attrs
        .iter()
        .any(|attr| is_attribute_kind(Kind::Bench, attr));
    let inner_bench = if has_bench {
        quote! {}
    } else {
        quote! { #[::core::prelude::v1::bench] }
    };

    try_bench_inner(attr, input_fn, inner_bench)
}

fn try_bench_inner(attr: Tokens, input_fn: ItemFn, inner_bench: Tokens) -> Result<Tokens> {
    if !attr.is_empty() {
        return Err(Error::new_spanned(
            attr,
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
    sig.output = parse_quote!(-> ::std::process::ExitCode);

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

/// Testable implementation of the `#[fork]` attribute's core logic.
pub fn try_fork(attr: Tokens, input_fn: ItemFn, supports_bench: bool) -> Result<Tokens> {
    let has_test = input_fn
        .attrs
        .iter()
        .any(|attr| is_attribute_kind(Kind::Test, attr));
    let has_bench = supports_bench
        && input_fn
            .attrs
            .iter()
            .any(|attr| is_attribute_kind(Kind::Bench, attr));

    let inner_attr = quote! {};
    if has_test {
        try_test_inner(attr, input_fn, inner_attr)
    } else if has_bench {
        try_bench_inner(attr, input_fn, inner_attr)
    } else {
        let inner_attr = if parse_bench_sig(&input_fn.sig).is_some() {
            "#[bench]"
        } else {
            "#[test]"
        };

        Err(Error::new_spanned(
            attr,
            format!("test_fork::fork requires an inner {inner_attr} attribute"),
        ))
    }
}
