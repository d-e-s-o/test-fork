// Copyright (C) 2025-2026 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

//! The procedural macro powering `test-fork`.

#![cfg_attr(docsrs, feature(doc_cfg))]


use proc_macro::TokenStream;

use syn::parse_macro_input;
use syn::ItemFn;

#[cfg(all(feature = "unstable", feature = "unsound"))]
use test_fork_core::try_bench;
use test_fork_core::try_fork;
use test_fork_core::try_test;


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

    try_test(attr.into(), input_fn)
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

    try_bench(attr.into(), input_fn)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}


/// A procedural macro for running a test or benchmark in a separate
/// process.
///
/// This attribute is able to cater to both tests and benchmarks, while
/// #[[macro@test]] is specific to tests and #[[macro@bench]] to
/// benchmarks.
///
/// Contrary to both, this attribute does not in itself make a function
/// a test/benchmark, so it will *always* have to be combined with an
/// additional "inner" attribute. However, it can be more convenient for
/// annotating only a sub-set of tests/benchmarks for running in
/// separate processes, especially when non-standard attributes are
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
///
/// #[fork]
/// #[bench]
/// fn bench3(b: &mut Bencher) {
///   b.iter(|| sleep(Duration::from_millis(1)));
/// }
/// ```
#[proc_macro_attribute]
pub fn fork(attr: TokenStream, item: TokenStream) -> TokenStream {
    let supports_bench = cfg!(all(feature = "unstable", feature = "unsound"));
    let input_fn = parse_macro_input!(item as ItemFn);

    try_fork(attr.into(), input_fn, supports_bench)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
