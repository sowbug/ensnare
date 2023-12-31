// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::{prelude::*, utils::Paths};
use ensnare_cores::{DrumkitParams, FmSynthParams, SamplerParams, WelshSynthParams};
use ensnare_cores_egui::instruments::{fm::fm_synth, sampler, welsh};
use ensnare_entity::prelude::*;
use ensnare_proc_macros::{
    Control, InnerConfigurable, InnerControllable, InnerHandlesMidi, InnerInstrument,
    InnerSerializable, IsEntity2, Metadata,
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, InnerControllable, InnerConfigurable, IsEntity2, Metadata, Serialize, Deserialize,
)]
#[entity2(
    Controls,
    GeneratesStereoSample,
    HandlesMidi,
    Serializable,
    Ticks,
    TransformsAudio
)]

pub struct Drumkit {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_cores::Drumkit,
}
impl Displays for Drumkit {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
impl Drumkit {
    pub fn new_with(uid: Uid, params: &DrumkitParams, paths: &Paths) -> Self {
        Self {
            uid,
            inner: ensnare_cores::Drumkit::new_with(&params, paths),
        }
    }
}

#[derive(
    Control,
    Debug,
    InnerConfigurable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsEntity2,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity2(Controls, TransformsAudio)]
pub struct FmSynth {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_cores::FmSynth,
}
impl Displays for FmSynth {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(fm_synth(&mut self.inner))
    }
}
impl FmSynth {
    pub fn new_with(uid: Uid, params: &FmSynthParams) -> Self {
        Self {
            uid,
            inner: ensnare_cores::FmSynth::new_with(params),
        }
    }
}

#[derive(
    Debug,
    InnerControllable,
    InnerConfigurable,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity2,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity2(GeneratesStereoSample, Ticks, Controls, TransformsAudio)]
pub struct Sampler {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_cores::Sampler,
}
impl Displays for Sampler {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(sampler(&mut self.inner))
    }
}
impl Sampler {
    pub fn new_with(uid: Uid, params: &SamplerParams) -> Self {
        Self {
            uid,
            inner: ensnare_cores::Sampler::new_with(&params),
        }
    }

    pub fn load(&mut self, paths: &Paths) -> anyhow::Result<()> {
        self.inner.load(paths)
    }
}

#[derive(
    Debug,
    InnerConfigurable,
    InnerControllable,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity2,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity2(GeneratesStereoSample, Ticks, Controls, TransformsAudio)]
pub struct WelshSynth {
    uid: Uid,
    #[serde(skip)]
    inner: ensnare_cores::WelshSynth,
}
impl Displays for WelshSynth {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(welsh(self.uid, &mut self.inner))
    }
}
impl WelshSynth {
    pub fn new_with(uid: Uid, params: &WelshSynthParams) -> Self {
        Self {
            uid,
            inner: ensnare_cores::WelshSynth::new_with(params),
        }
    }
}
