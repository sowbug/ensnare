// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{prelude::*, traits::prelude::*};
use ensnare_proc_macros::{Control, IsControllerEffect, Metadata, Params};
use serde::{Deserialize, Serialize};

pub use keyboard::KeyboardController;
pub use lfo::{LfoController, LfoControllerParams};

pub(crate) mod arpeggiator;
pub(crate) mod control;
mod keyboard;
pub(crate) mod lfo;
pub mod sequencers;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub enum SignalPassthroughType {
    #[default]
    /// Maps -1.0..=1.0 to 0.0..=1.0. Min amplitude becomes 0.0, silence becomes
    /// 0.5, and max amplitude becomes 1.0.
    Compressed,

    /// Based on the absolute value of the sample. Silence is 0.0, and max
    /// amplitude of either polarity is 1.0.
    Amplitude,

    /// Based on the absolute value of the sample. Silence is 1.0, and max
    /// amplitude of either polarity is 0.0.
    AmplitudeInverted,
}

/// Uses an input signal as a control source. Transformation depends on
/// configuration. Uses the standard Sample::from(StereoSample) methodology of
/// averaging the two channels to create a single signal.
#[derive(Control, Debug, Default, IsControllerEffect, Params, Metadata, Serialize, Deserialize)]
pub struct SignalPassthroughController {
    uid: Uid,
    passthrough_type: SignalPassthroughType,

    #[serde(skip)]
    control_value: ControlValue,

    // We don't issue consecutive identical events, so we need to remember
    // whether we've sent the current value.
    #[serde(skip)]
    has_value_been_issued: bool,

    #[serde(skip)]
    is_performing: bool,
}
impl Serializable for SignalPassthroughController {}
impl Configurable for SignalPassthroughController {}
impl Controls for SignalPassthroughController {
    fn update_time(&mut self, _range: &ViewRange) {
        // We can ignore because we already have our own de-duplicating logic.
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        if !self.is_performing {
            return;
        }
        if !self.has_value_been_issued {
            self.has_value_been_issued = true;
            control_events_fn(None, EntityEvent::Control(self.control_value))
        }
    }

    fn is_finished(&self) -> bool {
        true
    }

    fn play(&mut self) {
        self.is_performing = true;
    }

    fn stop(&mut self) {
        self.is_performing = false;
    }

    fn skip_to_start(&mut self) {}

    fn is_performing(&self) -> bool {
        self.is_performing
    }
}
impl HandlesMidi for SignalPassthroughController {}
impl TransformsAudio for SignalPassthroughController {
    fn transform_audio(&mut self, input_sample: StereoSample) -> StereoSample {
        let sample: Sample = input_sample.into();
        let control_value = match self.passthrough_type {
            SignalPassthroughType::Compressed => {
                let as_bipolar_normal: BipolarNormal = sample.into();
                as_bipolar_normal.into()
            }
            SignalPassthroughType::Amplitude => ControlValue(sample.0.abs()),
            SignalPassthroughType::AmplitudeInverted => ControlValue(1.0 - sample.0.abs()),
        };
        if self.control_value != control_value {
            self.has_value_been_issued = false;
            self.control_value = control_value;
        }
        input_sample
    }

    fn transform_channel(&mut self, _channel: usize, _input_sample: Sample) -> Sample {
        // We've overridden transform_audio(), so nobody should be calling this
        // method.
        todo!();
    }
}
impl Displays for SignalPassthroughController {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label(self.name())
    }
}
impl SignalPassthroughController {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_amplitude_passthrough_type() -> Self {
        Self {
            passthrough_type: SignalPassthroughType::Amplitude,
            ..Default::default()
        }
    }

    pub fn new_amplitude_inverted_passthrough_type() -> Self {
        Self {
            passthrough_type: SignalPassthroughType::AmplitudeInverted,
            ..Default::default()
        }
    }
}
