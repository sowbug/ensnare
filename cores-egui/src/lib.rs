// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! egui logic for drawing ensnare entities.

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::{controllers::trip, transport::transport};
}

pub use {
    common::ColorSchemeConverter,
    controllers::{arpeggiator, lfo_controller, note_sequencer_widget, pattern_sequencer_widget},
};

pub(crate) mod common;
mod controllers;
pub mod effects;
pub mod instruments;
pub mod modulators;
pub mod transport;
pub mod widgets;
