// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Common data types used throughout the system.

use crate::prelude::*;
use delegate::delegate;
use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Display, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrangementUid(usize);
impl IsUid for ArrangementUid {
    fn as_usize(&self) -> usize {
        self.0
    }
}
impl From<usize> for ArrangementUid {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ArrangementUidFactory(UidFactory<ArrangementUid>);
impl Default for ArrangementUidFactory {
    fn default() -> Self {
        Self(UidFactory::<ArrangementUid>::new(262144))
    }
}
impl ArrangementUidFactory {
    delegate! {
        to self.0 {
            pub fn mint_next(&self) -> ArrangementUid;
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct ControlLink {
    pub uid: Uid,
    pub param: ControlIndex,
}
