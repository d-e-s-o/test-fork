Unreleased
----------
- Added support for `--no-capture` test argument
- Stopped buffering test output when `--nocapture` argument is present


0.1.6
-----
- Updated `syn` dependency to `3`


0.1.5
-----
- Improved error reporting/output on inner test failure
- Improved `#[should_panic]` attribute handling and added support for
  `expected = "..."` argument


0.1.4
-----
- Fixed deadlock for tests with excessive output
- Specified minimum supported Rust version in manifest


0.1.3
-----
- Build `docs.rs` documentation with all supported features
- Documented `async` test support


0.1.2
-----
- Introduced `#[bench]` attribute for running benchmarks in a separate
  process
- Introduced `#[fork]` attribute that unconditionally requires nesting
  with other `#[test]` or `#[bench]` attributes


0.1.1
-----
- Improved documentation of `#[test]` attribute
- Reworked and simplified various internals
- Removed `tempfile` dependency


0.1.0
-----
- Initial release
