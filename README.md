[![pipeline](https://github.com/d-e-s-o/test-fork/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/d-e-s-o/test-fork/actions/workflows/test.yml)
[![crates.io](https://img.shields.io/crates/v/test-fork.svg)](https://crates.io/crates/test-fork)

test-fork
=========

- [Documentation][docs-rs]
- [Changelog](CHANGELOG.md)

A custom `#[test]` attribute that makes sure to run the test in a
separate process. Process separation can be useful in many contexts,
including when testing interactions with necessarily process-global
state (e.g., when working environment variables, when testing code that
requires temporary user ID switching, or when adjusting the working
directory).

Usage
-----
This crate provides the `#[test_fork::test]` attribute that can be used
for annotating tests to run in separate processes, as opposed to sharing
the address space with other concurrently running tests:
```rust
use test_fork::test;

#[test]
fn test1() {
  assert_eq!(2 + 2, 4);
}
```

The crate also supports an alternative syntax that nests more easily
with other custom `#[test]` attributes and which allows for easier
annotation of individual tests (e.g., if only a sub-set is meant to
be run in separate processes):
```rust
use test_fork::test as fork;

#[fork]
#[test]
fn test2() {
  assert_eq!(2 + 3, 5);
}
```


[docs-rs]: https://docs.rs/test-fork
