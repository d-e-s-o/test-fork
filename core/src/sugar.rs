//-
// Copyright 2018 Jason Lingle
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::any::TypeId;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::hash::DefaultHasher;
use std::hash::Hash as _;
use std::hash::Hasher;


/// Produce a hashable identifier unique to the particular macro invocation
/// which is stable across processes of the same executable.
///
/// This is usually the best thing to pass for the `fork_id` argument of
/// [`fork`][crate::fork].
///
/// The type of the expression this macro expands to is [`RustyForkId`].
#[macro_export]
macro_rules! fork_id {
    () => {{
        struct _RustyForkId;
        &std::string::ToString::to_string(&$crate::RustyForkId::of(::std::any::TypeId::of::<
            _RustyForkId,
        >()))
    }};
}


/// The type of the value produced by [`fork_id!`].
#[derive(Clone, Hash, PartialEq, Debug)]
pub struct RustyForkId(TypeId);

impl RustyForkId {
    #[allow(missing_docs)]
    #[doc(hidden)]
    pub fn of(id: TypeId) -> Self {
        RustyForkId(id)
    }
}

impl Display for RustyForkId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut hasher = DefaultHasher::default();
        self.0.hash(&mut hasher);
        write!(f, ":{:016X}", hasher.finish())
    }
}


#[cfg(test)]
mod test {
    use super::*;


    /// Check that IDs created for the same type are considered equal.
    #[test]
    fn ids_for_same_type_are_equal() {
        struct UniqueType;
        let id1 = RustyForkId::of(TypeId::of::<UniqueType>());
        let id2 = RustyForkId::of(TypeId::of::<UniqueType>());
        assert_eq!(id1, id2);
        assert_eq!(id1.to_string(), id2.to_string());
    }

    #[test]
    fn ids_are_actually_distinct() {
        let id1 = fork_id!();
        let id2 = fork_id!();
        assert_ne!(id1, id2);
        assert_ne!(id1.to_string(), id2.to_string());
    }
}
