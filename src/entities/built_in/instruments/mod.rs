// Copyright (c) 2023 Mike Tsao. All rights reserved.

#[cfg(feature = "egui")]
use crate::egui::{FmSynthWidgetAction, WelshWidgetAction};
use crate::{
    cores::{effects, instruments},
    traits::DisplaysAction,
};
use crate::{prelude::*, util::Paths};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerHandlesMidi, InnerInstrument, InnerSerializable,
    IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(
    Debug,
    InnerControllable,
    InnerConfigurable,
    InnerHandlesMidi,
    InnerInstrument,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "kebab-case")]
#[entity(Controls, Serializable, TransformsAudio)]

pub struct Drumkit {
    uid: Uid,
    inner: instruments::Drumkit,
}
impl Drumkit {
    pub fn new_with(uid: Uid, name: &str, paths: &Paths) -> Self {
        Self {
            uid,
            inner: instruments::Drumkit::new_with(name, paths),
        }
    }
}

#[derive(
    Debug,
    InnerConfigurable,
    InnerControllable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(Controls, TransformsAudio)]
pub struct FmSynth {
    uid: Uid,
    inner: instruments::FmSynth,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    widget_action: Option<FmSynthWidgetAction>,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    action: Option<DisplaysAction>,
}
impl FmSynth {
    pub fn new_with(
        uid: Uid,
        carrier_oscillator: Oscillator,
        carrier_envelope: Envelope,
        modulator_oscillator: Oscillator,
        modulator_envelope: Envelope,
        depth: Normal,
        ratio: Ratio,
        beta: ParameterType,
        dca: Dca,
    ) -> Self {
        Self {
            uid,
            inner: instruments::FmSynth::new_with(
                carrier_oscillator,
                carrier_envelope,
                modulator_oscillator,
                modulator_envelope,
                depth,
                ratio,
                beta,
                dca,
            ),
            widget_action: Default::default(),
            action: Default::default(),
        }
    }
}

#[derive(
    Debug,
    Deserialize,
    InnerConfigurable,
    InnerControllable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
)]
#[entity(Controls, TransformsAudio)]
pub struct Sampler {
    uid: Uid,
    inner: instruments::Sampler,
}
impl Sampler {
    pub fn new_with(uid: Uid, path: PathBuf, root: Option<FrequencyHz>) -> Self {
        Self {
            uid,
            inner: instruments::Sampler::new_with(path, root),
        }
    }

    pub fn load(&mut self, paths: &Paths) -> anyhow::Result<()> {
        self.inner.load(paths)
    }
}

#[derive(
    Debug,
    Deserialize,
    InnerConfigurable,
    InnerControllable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
)]
#[entity(Controls, TransformsAudio)]
pub struct WelshSynth {
    uid: Uid,
    inner: instruments::WelshSynth,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    widget_action: Option<WelshWidgetAction>,
}
impl WelshSynth {
    pub fn new_with(
        uid: Uid,
        oscillator_1: Oscillator,
        oscillator_2: Oscillator,
        oscillator_2_sync: bool,
        oscillator_mix: Normal,
        amp_envelope: Envelope,
        dca: Dca,
        lfo: Oscillator,
        lfo_routing: instruments::LfoRouting,
        lfo_depth: Normal,
        filter: effects::BiQuadFilterLowPass24db,
        filter_cutoff_start: Normal,
        filter_cutoff_end: Normal,
        filter_envelope: Envelope,
    ) -> Self {
        Self {
            uid,
            inner: instruments::WelshSynth::new_with(
                oscillator_1,
                oscillator_2,
                oscillator_2_sync,
                oscillator_mix,
                amp_envelope,
                dca,
                lfo,
                lfo_routing,
                lfo_depth,
                filter,
                filter_cutoff_start,
                filter_cutoff_end,
                filter_envelope,
            ),
            widget_action: Default::default(),
        }
    }

    pub fn new_with_factory_patch(uid: Uid) -> Self {
        WelshSynth::new_with(
            uid,
            Oscillator::new_with_waveform(Waveform::Sine),
            Oscillator::new_with_waveform(Waveform::Sawtooth),
            true,
            0.8.into(),
            Envelope::safe_default(),
            Dca::default(),
            Oscillator::new_with_waveform_and_frequency(Waveform::Sine, FrequencyHz::from(0.2)),
            instruments::LfoRouting::FilterCutoff,
            Normal::from(0.5),
            effects::BiQuadFilterLowPass24db::new_with(FrequencyHz(250.0), 1.0),
            Normal::from(0.1),
            Normal::from(0.8),
            Envelope::safe_default(),
        )
    }
}

#[cfg(feature = "egui")]
mod egui {
    use super::*;
    use crate::{
        egui::{FmSynthWidget, SamplerWidget, WelshWidget},
        traits::DisplaysAction,
    };

    impl Displays for Drumkit {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            ui.label("Coming soon!")
        }
    }

    impl Displays for FmSynth {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            let response = ui.add(FmSynthWidget::widget(
                &mut self.inner,
                &mut self.widget_action,
            ));
            if let Some(action) = self.widget_action.take() {
                match action {
                    FmSynthWidgetAction::Link(uid, index) => {
                        self.set_action(DisplaysAction::Link(uid, index));
                    }
                }
            }
            response
        }

        fn set_action(&mut self, action: DisplaysAction) {
            self.action = Some(action);
        }

        fn take_action(&mut self) -> Option<DisplaysAction> {
            self.action.take()
        }
    }

    impl Displays for Sampler {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            ui.add(SamplerWidget::widget(&mut self.inner))
        }
    }

    impl Displays for WelshSynth {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            ui.add(WelshWidget::widget(
                &mut self.inner,
                &mut self.widget_action,
            ))
        }
    }
}
