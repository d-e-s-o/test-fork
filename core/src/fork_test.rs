// Copyright (C) 2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

//-
// Copyright 2018, 2020 Jason Lingle
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


/// Given the unqualified name of a `#[test]` function, produce a
/// `&'static str` corresponding to the name of the test as filtered by the
/// standard test harness.
#[macro_export]
macro_rules! fork_test_name {
    ($function_name:ident) => {
        $crate::fix_module_path(concat!(module_path!(), "::", stringify!($function_name)))
    };
}

/// Transform a string representing a qualified path as generated via
/// `module_path!()` into a qualified path as expected by the standard Rust
/// test harness.
pub fn fix_module_path(path: &str) -> &str {
    path.find("::").map(|ix| &path[ix + 2..]).unwrap_or(path)
}
