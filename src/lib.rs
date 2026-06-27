// Copyright (C) 2025-2026 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

//! A crate providing a replacement #[[macro@test]] attribute (and
//! others) for running tests in separate processes.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub use test_fork_core;
#[cfg(all(feature = "unstable", feature = "unsound"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "unstable", feature = "unsound"))))]
pub use test_fork_macros::bench;
pub use test_fork_macros::fork;
pub use test_fork_macros::test;
