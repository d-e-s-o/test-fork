// Copyright (C) 2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::process;


#[test_fork::test]
fn trivial() {}

#[test_fork::test]
fn trivial_with_ok() -> Result<(), &'static str> {
    Ok(())
}

#[test_fork::test]
#[should_panic]
fn trivial_with_err() -> Result<(), &'static str> {
    Err("should fail.")
}

#[test_fork::test]
#[should_panic]
fn panicking_child() {
    panic!("just testing a panic, nothing to see here")
}

#[test_fork::test]
#[should_panic]
fn aborting_child() {
    process::abort()
}

#[test_fork::fork]
#[test]
fn fork_attr() {}

#[tokio::test]
#[test_fork::test]
async fn async_test() {}

#[tokio::test]
#[test_fork::fork]
async fn async_test_fork_attr() {}

#[tokio::test]
#[test_fork::test]
#[should_panic]
async fn async_test_panicking() {
    panic!("panic makes the world go 'round")
}
