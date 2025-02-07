- - -
## [0.4.0](https://github.com/oknozor/rusty-forkfork/compare/0.3.0..0.4.0) - 2022-02-05
#### Bug Fixes
- handle return of Err from forked test function - ([2e06752](https://github.com/oknozor/rusty-forkfork/commit/2e067521c80fa85d562d03ccbcb2224a51508698)) - [@Zshoham](https://github.com/Zshoham)
#### Continuous Integration
- migrate CI to github action and add cocogitto - ([5be13ad](https://github.com/oknozor/rusty-forkfork/commit/5be13ad232f63525d758f911b5668adbc1b4ae3e)) - [@oknozor](https://github.com/oknozor)
#### Features
- Add optional return type to rusty_test functions - ([a31edbe](https://github.com/oknozor/rusty-forkfork/commit/a31edbea4c5a7426c7f97a15b964850505b1254a)) - [@oknozor](https://github.com/oknozor)
#### Miscellaneous Chores
- rename crate to rusty_forkfork - ([9ffe50d](https://github.com/oknozor/rusty-forkfork/commit/9ffe50dd927636d033ebd50e96d536fe995ed171)) - [@oknozor](https://github.com/oknozor)
- add cocogitto changelog separator - ([b62f777](https://github.com/oknozor/rusty-forkfork/commit/b62f7771e8dcbce49cb3f87a45f314dc955b068e)) - [@oknozor](https://github.com/oknozor)
- fix lints and format - ([f9c3a36](https://github.com/oknozor/rusty-forkfork/commit/f9c3a3622a17e98d5606b7a67ca1eeebc3d5b30e)) - [@oknozor](https://github.com/oknozor)
- - -

## 0.3.0

### Breaking Changes

- The minimum required Rust version is now 1.32.0.

### Improvements

- `rusty_fork_test!` can now be `use`d in Rust 2018 code.

- The following flags to the test process are now understood: `--ensure-time`,
  `--exclude-should-panic`, `--force-run-in-process`, `--include-ignored`,
  `--report-time`, `--show-output`.

## 0.2.2

### Minor changes

- `wait_timeout` has been bumped to `0.2.0`.

## 0.2.1

### Bug Fixes

- Dependency on `wait_timeout` crate now requires `0.1.4` rather than `0.1`
  since the build doesn't work with older versions.

## 0.2.0

### Breaking changes

- APIs which used to provide a `std::process::Child` now instead provide a
  `rusty_fork::ChildWrapper`.

### Bug fixes

- Fix that using the "timeout" feature, or otherwise using `wait_timeout` on
  the child process, could cause an unrelated process to get killed if the
  child exits within the timeout.

## 0.1.1

### Minor changes

- `tempfile` updated to 3.0.
