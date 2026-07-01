// Copyright (C) 2025-2026 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

//-
// Copyright 2018 Jason Lingle
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Supporting crate for `test-fork`.

#[macro_use]
mod sugar;
#[macro_use]
mod fork_test;
mod cmdline;
mod error;
mod fork;
mod procmac;

pub use crate::fork::fork;
pub use crate::fork::fork_in_out;
#[doc(hidden)]
pub use crate::fork_test::fix_module_path;
pub use crate::sugar::ForkId;

pub use crate::procmac::try_bench;
pub use crate::procmac::try_fork;
pub use crate::procmac::try_test;
