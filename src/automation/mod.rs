// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! Support for changing the parameters of instruments and effects over time in
//! a programmatic, reproducible way.
//!
//! For example, suppose a producer wants a pan effect going
//! left-right-left-right throughout the whole song. This could be done by
//! manually turning a knob back and forth, but that's tedious, and it
//! especially won't work when rendering the final output to a song file.
//!
//! Using automation, the producer can instead configure an LFO to emit a
//! [ControlValue] each time its value changes, and then link that output to a
//! synthesizer's pan parameter. Then the synth's pan changes with the LFO
//! output, automatically and identically for each performance of the song.
//!
//! Controllable entities have one or more parameters that are addressable by
//! [ControlName] or [ControlIndex], which are discoverable through the
//! [Controllable] trait. The [Control](ensnare_proc_macros::Control) derive
//! macro, with #[control] derive parameters, usually implements this trait.
//!
//! All values that pass through the automation subsystem are normalized to
//! [ControlValue]s, which range from 0..=1.0. Sensible mappings exist for all
//! applicable types in the system.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{
        ControlEventsFn, ControlIndex, ControlLink, ControlName, ControlRange, ControlValue,
        Controllable, PathUid, PathUidFactory, SignalPath, SignalPathBuilder, SignalPointBuilder,
    };
}

pub use automator::Automator;
pub use controllers::{TimerCore, TriggerCore};
pub use ensnare::automation::{
    ControlEventsFn, ControlIndex, ControlLink, ControlName, ControlRange, ControlValue,
    Controllable, Controls, ControlsAsProxy, PathUid, PathUidFactory,
};
pub use signal_path::{SignalPath, SignalPathBuilder, SignalPoint, SignalPointBuilder};
pub use traits::{ControlLinkSource, ControlProxyEventsFn};

mod automator;
mod controllers;
mod signal_path;
mod traits;
mod types;
