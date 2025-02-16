//-
// Copyright 2018 Jason Lingle
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(missing_docs, unsafe_code)]

//! Supporting crate for `test-fork`.

#[macro_use]
mod sugar;
#[macro_use]
pub mod fork_test;
mod cmdline;
mod error;
mod fork;

pub use crate::fork::fork;
#[doc(hidden)]
pub use crate::fork_test::fix_module_path;
#[doc(hidden)]
pub use crate::fork_test::no_configure_child;
#[doc(hidden)]
pub use crate::fork_test::supervise_child;
pub use crate::sugar::ForkId;
