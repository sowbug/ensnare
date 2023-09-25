// Copyright (c) 2023 Mike Tsao. All rights reserved.

use eframe::egui::{Response, Ui};
use ensnare_core::{
    generators::{Oscillator, OscillatorParams, Waveform},
    midi::prelude::*,
    prelude::*,
    traits::prelude::*,
};
use ensnare_proc_macros::{Control, IsController, Params, Uid};
use serde::{Deserialize, Serialize};
use std::ops::{Range, RangeInclusive};

use super::WaveformWidget;

/// Uses an internal LFO as a control source.
#[derive(Debug, Control, IsController, Params, Uid, Serialize, Deserialize)]
pub struct LfoController {
    uid: Uid,

    #[control]
    #[params]
    waveform: Waveform,
    #[control]
    #[params]
    frequency: FrequencyHz,

    oscillator: Oscillator,

    #[serde(skip)]
    is_performing: bool,

    #[serde(skip)]
    waveform_widget: WaveformWidget,

    #[serde(skip)]
    time_range: Range<MusicalTime>,

    #[serde(skip)]
    last_frame: usize,
}
impl Serializable for LfoController {}
impl Configurable for LfoController {
    fn sample_rate(&self) -> SampleRate {
        self.oscillator.sample_rate()
    }
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.oscillator.update_sample_rate(sample_rate);
    }
}
impl Controls for LfoController {
    fn update_time(&mut self, range: &Range<MusicalTime>) {
        self.time_range = range.clone();
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        let frames = self.time_range.start.as_frames(
            Tempo::from(120),
            SampleRate::from(self.oscillator.sample_rate()),
        );

        if frames != self.last_frame {
            let tick_count = if frames >= self.last_frame {
                // normal case; oscillator should advance the calculated number
                // of frames
                //
                // TODO: this is unlikely to be frame-accurate, because
                // Orchestrator is currently going from frames -> beats
                // (inaccurate), and then we're going from beats -> frames. We
                // could include frame count in update_time(), as discussed in
                // #132, which would mean we don't have to be smart at all about
                // it.
                frames - self.last_frame
            } else {
                self.last_frame = frames;
                0
            };
            self.last_frame += tick_count;
            self.oscillator.tick(tick_count);
        }
        control_events_fn(
            self.uid,
            EntityEvent::Control(self.oscillator.value().into()),
        );
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

    fn skip_to_start(&mut self) {
        // TODO: think how important it is for LFO oscillator to start at zero
    }

    fn is_performing(&self) -> bool {
        self.is_performing
    }
}
impl HandlesMidi for LfoController {}
impl LfoController {
    pub fn new_with(params: &LfoControllerParams) -> Self {
        Self {
            uid: Default::default(),
            oscillator: Oscillator::new_with(&OscillatorParams {
                waveform: params.waveform,
                frequency: params.frequency,
                ..Default::default()
            }),
            waveform: params.waveform(),
            frequency: params.frequency(),
            is_performing: false,

            waveform_widget: Default::default(),
            time_range: Default::default(),
            last_frame: Default::default(),
        }
    }

    pub const fn frequency_range() -> RangeInclusive<ParameterType> {
        0.0..=100.0
    }

    pub fn waveform(&self) -> Waveform {
        self.waveform
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
        self.oscillator.set_waveform(waveform);
    }

    pub fn frequency(&self) -> FrequencyHz {
        self.frequency
    }

    pub fn set_frequency(&mut self, frequency: FrequencyHz) {
        self.frequency = frequency;
        self.oscillator.set_frequency(frequency);
    }
}

impl Displays for LfoController {
    fn ui(&mut self, ui: &mut Ui) -> Response {
        // TODO: come up with a better pattern for .changed() to happen at
        // the same level as whoever called show().
        if self.frequency.show(ui, Self::frequency_range()) {
            self.set_frequency(self.frequency);
        }
        if self.waveform.show(ui).inner.is_some() {
            self.set_waveform(self.waveform);
        }
        self.waveform_widget.ui(ui)
    }
}