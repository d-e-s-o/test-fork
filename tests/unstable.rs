// Copyright (C) 2025-2026 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

//! Tests for unstable [`test-fork`] functionality.

#![feature(test)]

extern crate test;

use std::thread::sleep;
use std::time::Duration;

use test::Bencher;


#[inline]
fn action() {
    let () = sleep(Duration::from_millis(1));
}


/// Benchmark an "action" using the regular benchmarking infrastructure.
#[bench]
fn benchmark_regular_cmp(b: &mut Bencher) {
    b.iter(action)
}


/// Benchmark an "action" in a different process.
#[test_fork::bench]
fn benchmark(b: &mut Bencher) {
    b.iter(action)
}


/// Benchmark an "action" in a different process using the
/// `#[test_fork::fork]` attribute.
#[test_fork::fork]
#[bench]
fn benchmark_fork_attr(b: &mut Bencher) {
    b.iter(action)
}
