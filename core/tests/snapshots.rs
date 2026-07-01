// Copyright (C) 2026 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

//! Snapshot tests for the macro expansion logic.

use proc_macro2::TokenStream;

use syn::parse2;
use syn::parse_quote;
use syn::AttrStyle;
use syn::ItemFn;
use syn::Meta;
use syn::Result;

use insta::assert_snapshot;


/// Accept a fully-defined test function annotated with some `#[...]`
/// macro and invoke the corresponding `try_*` helper to expand it into
/// the final Rust code.
fn expand(mut input: ItemFn) -> String {
    fn try_fork(attr: TokenStream, input_fn: ItemFn) -> Result<TokenStream> {
        let supports_bench = true;
        test_fork_core::try_fork(attr, input_fn, supports_bench)
    }

    // Remove the decorating `#[...]` style attribute, as a true
    // proc-macro would.
    let pos = input
        .attrs
        .iter()
        .position(|attr| matches!(attr.style, AttrStyle::Outer))
        .expect("input must carry an outer #[...] attribute");

    let attr = input.attrs.remove(pos);
    let attr_args = match &attr.meta {
        Meta::List(list) => list.tokens.clone(),
        _ => TokenStream::new(),
    };

    let segments = attr
        .path()
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>();
    let try_fn = match &segments[..] {
        [_, kind] if kind == "test" => test_fork_core::try_test,
        [_, kind] if kind == "bench" => test_fork_core::try_bench,
        [_, kind] if kind == "fork" => try_fork,
        [..] => panic!("encountered unsupported attribute"),
    };

    let tokens = (try_fn)(attr_args, input).unwrap();
    let file = parse2(tokens).unwrap();
    let snapshot = prettyplease::unparse(&file);

    snapshot
}


/// Check expansion of a plain `#[test_fork::test]` test.
#[test]
fn snapshot_test_attr() {
    let output = expand(parse_quote! {
        #[test_fork::test]
        fn it_works() {
            assert_eq!(2 + 2, 4);
        }
    });
    assert_snapshot!(output);
}

/// Check expansion of a `#[test_fork::test]` test that returns a
/// `Result`.
#[test]
fn snapshot_test_result() {
    let output = expand(parse_quote! {
        #[test_fork::test]
        fn it_works() -> Result<(), &'static str> {
            assert_eq!(2 + 2, 4);
            Ok(())
        }
    });
    assert_snapshot!(output);
}

/// Check expansion of a plain `#[test_fork::fork]` test.
#[test]
fn snapshot_fork_attr() {
    let output = expand(parse_quote! {
        #[test_fork::fork]
        #[test]
        fn it_works() {
            assert_eq!(2 + 2, 4);
        }
    });
    assert_snapshot!(output);
}

/// Check expansion of a plain `#[test_fork::bench]` test.
#[test]
fn snapshot_bench_attr() {
    let output = expand(parse_quote! {
        #[test_fork::bench]
        fn bench_it(b: &mut Bencher) {
            let () = b.iter(|| 2 + 2);
        }
    });
    assert_snapshot!(output);
}
