// Copyright (C) 2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

struct Bencher;

#[test_fork::fork]
fn missing_bench(b: &mut Bencher) {}

fn main() {}
