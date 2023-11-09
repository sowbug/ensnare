// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::egui::{Slider, Widget};
use ensnare_core::{
    prelude::*,
    stuff::{
        bitcrusher::BitcrusherParams, chorus::ChorusParams, compressor::CompressorParams,
        filter::BiQuadFilterLowPass24dbParams, gain::GainParams, limiter::LimiterParams,
        mixer::MixerParams, reverb::ReverbParams,
    },
    types::FrequencyRange,
};
use ensnare_entity::prelude::*;
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEffect, Metadata,
};

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEffect,
    Metadata,
)]
pub struct Bitcrusher {
    uid: Uid,
    inner: ensnare_core::stuff::bitcrusher::Bitcrusher,
}
impl Displays for Bitcrusher {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut bits = self.inner.bits();
        let response = ui.add(
            Slider::new(
                &mut bits,
                ensnare_core::stuff::bitcrusher::Bitcrusher::bits_range(),
            )
            .suffix(" bits"),
        );
        if response.changed() {
            self.inner.set_bits(bits);
        };
        response
    }
}
impl Bitcrusher {
    pub fn new_with(uid: Uid, params: &BitcrusherParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::bitcrusher::Bitcrusher::new_with(&params),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEffect,
    Metadata,
)]
pub struct Chorus {
    uid: Uid,
    inner: ensnare_core::stuff::chorus::Chorus,
}
impl Displays for Chorus {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
impl Chorus {
    pub fn new_with(uid: Uid, params: &ChorusParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::chorus::Chorus::new_with(&params),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEffect,
    Metadata,
)]
pub struct Compressor {
    uid: Uid,
    inner: ensnare_core::stuff::compressor::Compressor,
}
impl Displays for Compressor {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut threshold = self.inner.threshold().0;
        let mut ratio = self.inner.ratio();
        let mut attack = self.inner.attack();
        let mut release = self.inner.release();
        let threshold_response = ui.add(
            Slider::new(&mut threshold, Normal::range())
                .fixed_decimals(2)
                .text("Threshold"),
        );
        if threshold_response.changed() {
            self.inner.set_threshold(threshold.into());
        };
        let ratio_response = ui.add(
            Slider::new(&mut ratio, Normal::range())
                .fixed_decimals(2)
                .text("Ratio"),
        );
        if ratio_response.changed() {
            self.inner.set_ratio(ratio);
        };
        let attack_response = ui.add(
            Slider::new(&mut attack, Normal::range())
                .fixed_decimals(2)
                .text("Attack"),
        );
        if attack_response.changed() {
            self.inner.set_attack(attack);
        };
        let release_response = ui.add(
            Slider::new(&mut release, Normal::range())
                .fixed_decimals(2)
                .text("Release"),
        );
        if release_response.changed() {
            self.inner.set_release(release);
        };
        threshold_response | ratio_response | attack_response | release_response
    }
}
impl Compressor {
    pub fn new_with(uid: Uid, params: &CompressorParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::compressor::Compressor::new_with(&params),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEffect,
    Metadata,
)]
pub struct Gain {
    uid: Uid,
    inner: ensnare_core::stuff::gain::Gain,
}
impl Displays for Gain {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut ceiling = self.inner.ceiling().to_percentage();
        let response = ui.add(
            Slider::new(&mut ceiling, 0.0..=100.0)
                .fixed_decimals(2)
                .suffix(" %")
                .text("Ceiling"),
        );
        if response.changed() {
            self.inner.set_ceiling(Normal::from_percentage(ceiling));
        };
        response
    }
}
impl Gain {
    pub fn new_with(uid: Uid, params: &GainParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::gain::Gain::new_with(&params),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEffect,
    Metadata,
)]
pub struct Limiter {
    uid: Uid,
    inner: ensnare_core::stuff::limiter::Limiter,
}
impl Displays for Limiter {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut min = self.inner.minimum().to_percentage();
        let mut max = self.inner.maximum().to_percentage();
        let min_response = ui.add(
            Slider::new(&mut min, 0.0..=max)
                .suffix(" %")
                .text("min")
                .fixed_decimals(2),
        );
        if min_response.changed() {
            self.inner.set_minimum(min.into());
        };
        let max_response = ui.add(
            Slider::new(&mut max, min..=1.0)
                .suffix(" %")
                .text("max")
                .fixed_decimals(2),
        );
        if max_response.changed() {
            self.inner.set_maximum(Normal::from_percentage(max));
        };
        min_response | max_response
    }
}
impl Limiter {
    pub fn new_with(uid: Uid, params: &LimiterParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::limiter::Limiter::new_with(&params),
        }
    }
}

#[derive(Debug, Default, InnerControllable, InnerEffect, IsEffect, Metadata)]
pub struct Mixer {
    uid: Uid,
    inner: ensnare_core::stuff::mixer::Mixer,
}
impl Displays for Mixer {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
impl Configurable for Mixer {}
impl Serializable for Mixer {}
impl Mixer {
    pub fn new_with(uid: Uid, params: &MixerParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::mixer::Mixer::new_with(&params),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEffect,
    Metadata,
)]
pub struct Reverb {
    uid: Uid,
    inner: ensnare_core::stuff::reverb::Reverb,
}
impl Displays for Reverb {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
impl Reverb {
    pub fn new_with(uid: Uid, params: &ReverbParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::reverb::Reverb::new_with(&params),
        }
    }
}

/// Wraps a [BiQuadFilterLowPass24dbWidget] as a [Widget](eframe::egui::Widget).
pub fn bi_quad_filter_low_pass_24db<'a>(
    inner: &'a mut ensnare_core::stuff::filter::BiQuadFilterLowPass24db,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| BiQuadFilterLowPass24dbWidget::new(inner).ui(ui)
}
struct BiQuadFilterLowPass24dbWidget<'a> {
    inner: &'a mut ensnare_core::stuff::filter::BiQuadFilterLowPass24db,
}
impl<'a> BiQuadFilterLowPass24dbWidget<'a> {
    fn new(inner: &'a mut ensnare_core::stuff::filter::BiQuadFilterLowPass24db) -> Self {
        Self { inner }
    }
}
impl<'a> Widget for BiQuadFilterLowPass24dbWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut cutoff = self.inner.cutoff().0;
        let mut pbr = self.inner.passband_ripple();
        let cutoff_response = ui.add(
            Slider::new(&mut cutoff, FrequencyRange::Audible.as_range())
                .text("Cutoff")
                .suffix(FrequencyHz::UNITS_SUFFIX),
        );
        if cutoff_response.changed() {
            self.inner.set_cutoff(cutoff.into());
        };
        let passband_response = ui.add(Slider::new(&mut pbr, 0.0..=10.0).text("Passband"));
        if passband_response.changed() {
            self.inner.set_passband_ripple(pbr);
        };
        cutoff_response | passband_response
    }
}

#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEffect,
    Metadata,
)]
pub struct BiQuadFilterLowPass24db {
    uid: Uid,
    inner: ensnare_core::stuff::filter::BiQuadFilterLowPass24db,
}
impl Displays for BiQuadFilterLowPass24db {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(bi_quad_filter_low_pass_24db(&mut self.inner))
    }
}
impl BiQuadFilterLowPass24db {
    pub fn new_with(uid: Uid, params: &BiQuadFilterLowPass24dbParams) -> Self {
        Self {
            uid,
            inner: ensnare_core::stuff::filter::BiQuadFilterLowPass24db::new_with(&params),
        }
    }
}
