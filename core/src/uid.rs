// Copyright (c) 2023 Mike Tsao. All rights reserved.

use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    hash::Hash,
    marker::PhantomData,
    sync::atomic::{AtomicUsize, Ordering},
};

/// An optional Uid trait.
pub trait IsUid: Eq + Hash + Clone + From<usize> {
    fn as_usize(&self) -> usize;
}

/// A [Uid] is an [Entity](crate::traits::Entity) identifier that is unique
/// within the current project.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    derive_more::Display,
)]
pub struct Uid(pub usize);
impl IsUid for Uid {
    fn as_usize(&self) -> usize {
        self.0
    }
}
impl From<usize> for Uid {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/// Generates unique [Uid]s.
#[derive(Debug, Serialize, Deserialize)]
pub struct UidFactory<U: IsUid> {
    pub(crate) next_uid_value: AtomicUsize,
    pub(crate) _phantom: PhantomData<U>,
}
impl<U: IsUid> UidFactory<U> {
    /// Creates a new [UidFactory] starting with the given value.
    pub fn new(first_uid: usize) -> Self {
        Self {
            next_uid_value: AtomicUsize::new(first_uid),
            _phantom: Default::default(),
        }
    }

    /// Generates the next unique [Uid].
    pub fn mint_next(&self) -> U {
        let uid_value = self.next_uid_value.fetch_add(1, Ordering::Relaxed);
        U::from(uid_value)
    }

    /// Notifies the factory that a [Uid] exists that might have been created
    /// elsewhere (for example, during deserialization of a project). This gives
    /// the factory an opportunity to adjust `next_uid_value` to stay consistent
    /// with all known [Uid]s.
    pub fn notify_externally_minted_uid(&self, uid: U) {
        if uid.as_usize() >= self.next_uid_value.load(Ordering::Relaxed) {
            self.next_uid_value
                .store(uid.as_usize() + 1, Ordering::Relaxed);
        }
    }
}
impl<U: IsUid> PartialEq for UidFactory<U> {
    fn eq(&self, other: &Self) -> bool {
        self.next_uid_value.load(Ordering::Relaxed) == other.next_uid_value.load(Ordering::Relaxed)
    }
}

pub type EntityUidFactory = UidFactory<Uid>;
impl UidFactory<Uid> {
    pub const FIRST_UID: AtomicUsize = AtomicUsize::new(1024);
}
impl Default for UidFactory<Uid> {
    fn default() -> Self {
        Self {
            next_uid_value: Self::FIRST_UID,
            _phantom: Default::default(),
        }
    }
}

/// Identifies a [Track].
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct TrackUid(pub usize);
impl Default for TrackUid {
    fn default() -> Self {
        Self(1)
    }
}
impl IsUid for TrackUid {
    fn as_usize(&self) -> usize {
        self.0
    }
}
impl From<usize> for TrackUid {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
impl Display for TrackUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

pub type TrackUidFactory = UidFactory<TrackUid>;
impl UidFactory<TrackUid> {
    pub const FIRST_UID: AtomicUsize = AtomicUsize::new(1);
}
impl Default for UidFactory<TrackUid> {
    fn default() -> Self {
        Self {
            next_uid_value: Self::FIRST_UID,
            _phantom: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uid_factory() {
        let f = UidFactory::<Uid>::default();

        let uid_1 = f.mint_next();
        let uid_2 = f.mint_next();
        assert_ne!(uid_1, uid_2, "Minted Uids should not repeat");

        let uid_3 = Uid(uid_2.0 + 1);
        let uid_3_expected_duplicate = f.mint_next();
        assert_eq!(
            uid_3, uid_3_expected_duplicate,
            "Minted Uids will repeat if factory doesn't know about them all"
        );
    }

    #[test]
    fn uid_factory_with_notify_works() {
        let f = UidFactory::<Uid>::default();

        let uid_1 = f.mint_next();
        let uid_2 = f.mint_next();
        assert_ne!(uid_1, uid_2, "Minted Uids should not repeat");

        let uid_3 = Uid(uid_2.0 + 1);
        f.notify_externally_minted_uid(uid_3);
        let uid_4 = f.mint_next();
        assert_ne!(
            uid_3, uid_4,
            "Notifying factory should cause it to skip past."
        );

        f.notify_externally_minted_uid(uid_3);
        let uid_5 = f.mint_next();
        assert_eq!(
            uid_5.0,
            uid_4.0 + 1,
            "Notifying factory about value below next should be no-op."
        );
    }
}
