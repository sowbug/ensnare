// Copyright (c) 2023 Mike Tsao. All rights reserved.

#![warn(missing_docs)]

//! Ensnare is a library for generating digital audio.

/// Handles automation, or real-time automatic control of one entity's
/// parameters by another entity's output.
pub mod control;
/// Contains common structures and constants used across the library.
pub mod core;
/// Handles digital-audio, wall-clock, and musical time.
pub mod time;
/// Unique identifiers.
pub mod uid;

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use crate::control::{ControlIndex, ControlName, ControlValue};
    pub use crate::core::{
        BipolarNormal, FrequencyHz, Normal, ParameterType, Ratio, Sample, SampleType, SignalType,
        StereoSample,
    };
    pub use crate::time::Tempo;
    pub use crate::uid::Uid;
}
