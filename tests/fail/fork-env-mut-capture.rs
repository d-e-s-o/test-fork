// Copyright (C) 2025-2026 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::process::ExitCode;

use test_fork_core::fork;
use test_fork_core::fork_id;


/// Check that we cannot mutably capture a variable in the function
/// running in the child.
fn env_mut_capture() {
    let mut x = 0;

    let status = fork(fork_id!(), "env_mut_capture", || {
        x += 1;
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
}

fn main() {}
