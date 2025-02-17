// Copyright (C) 2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::process;

use test_fork::test;


#[test]
fn trivial() {}

#[test]
fn trivial_with_ok() -> Result<(), &'static str> {
    Ok(())
}

#[test]
#[should_panic]
fn trivial_with_err() -> Result<(), &'static str> {
    Err("should fail.")
}

#[test]
#[should_panic]
fn panicking_child() {
    panic!("just testing a panic, nothing to see here");
}

#[test]
#[should_panic]
fn aborting_child() {
    process::abort();
}
