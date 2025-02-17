// Copyright (C) 2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

//-
// Copyright 2018 Jason Lingle
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io;


/// Enum for errors produced by the rusty-fork crate.
#[derive(Debug)]
pub enum Error {
    /// An unknown flag was encountered when examining the current
    /// process's argument list.
    ///
    /// The string is the flag that was encountered.
    UnknownFlag(String),
    /// A flag was encountered when examining the current process's
    /// argument list which is known but cannot be handled in any sensible
    /// way.
    ///
    /// The strings are the flag encountered and a human-readable message
    /// about why the flag could not be handled.
    DisallowedFlag(String, String),
    /// Spawning a subprocess failed.
    SpawnError(io::Error),
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        Self::SpawnError(other)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match *self {
            Error::UnknownFlag(ref flag) => {
                f.write_fmt(format_args!(
                    "The flag '{flag}' was passed to the Rust test process, but rusty-fork does not know how to handle it."
                ))
            },
            Error::DisallowedFlag(ref flag, ref message) => {
                f.write_fmt(format_args!(
                    "The flag '{flag}' was passed to the Rust test process, but rusty-fork cannot handle it; reason: {message}"
                ))
            },
            Error::SpawnError(ref err) => {
                f.write_fmt(format_args!("Spawn failed: {err}"))
            },
        }
    }
}


/// General `Result` type for rusty-fork.
pub type Result<T> = ::std::result::Result<T, Error>;
