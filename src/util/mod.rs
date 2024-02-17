// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Useful things that don't have anything to do with digital audio.

pub mod prelude {
    pub use super::{ChannelPair, ModSerial};
}

pub use channel_pair::ChannelPair;
pub use rng::Rng;
pub use selection_set::SelectionSet;

mod channel_pair;
pub mod rng;
pub mod selection_set;

#[cfg(feature = "std")]
pub use mod_serial::ModSerial;
#[cfg(feature = "std")]
mod mod_serial;

#[cfg(feature = "std")]
pub use paths::{FileType, Paths};
#[cfg(feature = "std")]
pub mod paths;
