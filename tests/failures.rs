// Copyright (C) 2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use trybuild::TestCases;


/// Make sure that certain wrong attribute usages are caught at compile
/// time.
#[test]
fn failures() {
    let t = TestCases::new();
    let () = t.compile_fail("tests/fail/test-invalid-args.rs");
    let () = t.compile_fail("tests/fail/fork-env-mut-capture.rs");
    let () = t.compile_fail("tests/fail/fork-no-inner-test.rs");

    if cfg!(all(feature = "unstable", feature = "unsound")) {
        let () = t.compile_fail("tests/fail/bench-invalid-sig.rs");
    }
}
