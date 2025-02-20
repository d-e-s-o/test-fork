[![pipeline](https://github.com/d-e-s-o/test-fork/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/d-e-s-o/test-fork/actions/workflows/test.yml)
[![crates.io](https://img.shields.io/crates/v/test-fork.svg)](https://crates.io/crates/test-fork)
[![Docs](https://docs.rs/test-fork/badge.svg)](https://docs.rs/test-fork)

test-fork
=========

- [Documentation][docs-rs]
- [Changelog](CHANGELOG.md)

Custom attributes making sure a test is run in a separate process.
Process separation can be useful in many contexts, including when
testing interactions with necessarily process-global state (e.g., when
working environment variables, when testing code that requires temporary
user ID switching, or when adjusting the working directory).

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

Also provided is the `#[fork]` attribute, which does not in itself make
a function a test, so it will *always* have to be combined with an
additional `#[test]` attribute. However, it can be more convenient for
annotating only a sub-set of tests for running in separate processes,
especially when non-standard `#[test]` attributes are involved:
```rust
use test_fork::test as fork;

#[fork]
#[test]
fn test2() {
  assert_eq!(2 + 3, 5);
}
```

The crate also supports running `libtest` style benchmarks in a separate
process. This functionality is available only when both the `unstable`
and `unsound` features are enabled. The functionality is unstable
because `libtest` benchmarks are unstable and only available on nightly.
It is potentially unsound because the implementation necessarily needs
to transfer `Bencher` objects across process boundaries, but said
objects don't offer a stable ABI.

```rust
use test_fork::bench;

#[bench]
fn bench1(b: &mut Bencher) {
  b.iter(|| sleep(Duration::from_millis(1)));
}
```

The `#[fork]` attribute is also able to deal with benchmarks.

[docs-rs]: https://docs.rs/test-fork
