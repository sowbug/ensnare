// Copyright (c) 2023 Mike Tsao. All rights reserved.

#![warn(missing_docs)]

//! TODO

/// Management of [Entities](Entity).
pub mod entities;

pub mod controllers;
pub mod effects;
pub mod instruments;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use crate::{
        controllers::{
            arpeggiator::{Arpeggiator, ArpeggiatorParams},
            lfo::{LfoController, LfoControllerParams},
            SignalPassthroughController, SignalPassthroughControllerParams, SignalPassthroughType,
        },
        effects::{
            chorus::{Chorus, ChorusParams},
            delay::{Delay, DelayParams, RecirculatingDelayLine},
            filter::{
                BiQuadFilterAllPass, BiQuadFilterBandPass, BiQuadFilterLowPass24db,
                BiQuadFilterLowPass24dbParams,
            },
            gain::{Gain, GainParams},
            reverb::{Reverb, ReverbParams},
        },
        instruments::welsh::{WelshSynth, WelshSynthParams},
    };
}