// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::BiQuadFilterLowPass24db;
use core::fmt::Debug;
use ensnare_core::{
    generators::{Envelope, Oscillator},
    instruments::Synthesizer,
    modulators::Dca,
    prelude::*,
    traits::GeneratesEnvelope,
    voices::StealingVoiceStore,
};
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumCount as EnumCountMacro, FromRepr};

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    EnumCountMacro,
    FromRepr,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
pub enum LfoRouting {
    #[default]
    None,
    Amplitude,
    Pitch,
    PulseWidth,
    FilterCutoff,
}

#[derive(Control, Debug, Default, Serialize, Deserialize)]
pub struct WelshVoice {
    #[control]
    pub oscillator_1: Oscillator,
    #[control]
    pub oscillator_2: Oscillator,
    #[control]
    pub oscillator_2_sync: bool,
    #[control]
    pub oscillator_mix: Normal, // 1.0 = entirely osc 0, 0.0 = entirely osc 1.
    #[control]
    pub amp_envelope: Envelope,
    #[control]
    pub dca: Dca,

    #[control]
    pub lfo: Oscillator,
    pub lfo_routing: LfoRouting,
    #[control]
    pub lfo_depth: Normal,

    #[control]
    pub filter: BiQuadFilterLowPass24db,
    #[control]
    pub filter_cutoff_start: Normal,
    #[control]
    pub filter_cutoff_end: Normal,
    #[control]
    pub filter_envelope: Envelope,

    #[serde(skip)]
    note_on_key: u7,
    #[serde(skip)]
    note_on_velocity: u7,
    #[serde(skip)]
    steal_is_underway: bool,

    #[serde(skip)]
    sample: StereoSample,
    #[serde(skip)]
    ticks: usize,
}
impl IsStereoSampleVoice for WelshVoice {}
impl IsVoice<StereoSample> for WelshVoice {}
impl PlaysNotes for WelshVoice {
    fn is_playing(&self) -> bool {
        !self.amp_envelope.is_idle()
    }
    fn note_on(&mut self, key: u7, velocity: u7) {
        if self.is_playing() {
            self.steal_is_underway = true;
            self.note_on_key = key;
            self.note_on_velocity = velocity;
            self.amp_envelope.trigger_shutdown();
        } else {
            self.amp_envelope.trigger_attack();
            self.filter_envelope.trigger_attack();
            self.set_frequency_hz(MidiNote::from_repr(key.as_int() as usize).unwrap().into());
        }
    }
    fn aftertouch(&mut self, _velocity: u7) {
        // TODO: do something
    }
    fn note_off(&mut self, _velocity: u7) {
        self.amp_envelope.trigger_release();
        self.filter_envelope.trigger_release();
    }
}
impl Generates<StereoSample> for WelshVoice {
    fn value(&self) -> StereoSample {
        self.sample
    }
    fn generate_batch_values(&mut self, _samples: &mut [StereoSample]) {
        todo!()
    }
}
impl Serializable for WelshVoice {}
impl Configurable for WelshVoice {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.ticks = 0;
        self.lfo.update_sample_rate(sample_rate);
        self.amp_envelope.update_sample_rate(sample_rate);
        self.filter_envelope.update_sample_rate(sample_rate);
        self.filter.update_sample_rate(sample_rate);
        self.oscillator_1.update_sample_rate(sample_rate);
        self.oscillator_2.update_sample_rate(sample_rate);
    }
}
impl Ticks for WelshVoice {
    fn tick(&mut self, tick_count: usize) {
        for _ in 0..tick_count {
            self.ticks += 1;
            // It's important for the envelope tick() methods to be called after
            // their handle_note_* methods are called, but before we check whether
            // amp_envelope.is_idle(), because the tick() methods are what determine
            // the current idle state.
            //
            // TODO: this seems like an implementation detail that maybe should be
            // hidden from the caller.
            let (amp_env_amplitude, filter_env_amplitude) = self.tick_envelopes();

            // TODO: various parts of this loop can be precalculated.

            self.sample = if self.is_playing() {
                // TODO: ideally, these entities would get a tick() on every
                // voice tick(), but they are surprisingly expensive. So we will
                // skip calling them unless we're going to look at their output.
                // This means that they won't get a time slice as often as the
                // voice will. If this becomes a problem, we can add something
                // like an empty_tick() method to the Ticks trait that lets
                // entities stay in sync, but skipping any real work that would
                // cost time.
                if !matches!(self.lfo_routing, LfoRouting::None) {
                    self.lfo.tick(1);
                }
                self.oscillator_1.tick(1);
                self.oscillator_2.tick(1);

                // LFO
                let lfo = self.lfo.value();
                if matches!(self.lfo_routing, LfoRouting::Pitch) {
                    let lfo_for_pitch = lfo * self.lfo_depth;
                    self.oscillator_1.set_frequency_modulation(lfo_for_pitch);
                    self.oscillator_2.set_frequency_modulation(lfo_for_pitch);
                }

                // Oscillators
                let osc_sum = {
                    if self.oscillator_2_sync && self.oscillator_1.should_sync() {
                        self.oscillator_2.sync();
                    }
                    self.oscillator_1.value() * self.oscillator_mix
                        + self.oscillator_2.value() * (Normal::maximum() - self.oscillator_mix)
                };

                // Filters
                //
                // https://aempass.blogspot.com/2014/09/analog-and-welshs-synthesizer-cookbook.html
                if self.filter_cutoff_end != Normal::zero() {
                    let new_cutoff_percentage = self.filter_cutoff_start
                        + (1.0 - self.filter_cutoff_start)
                            * self.filter_cutoff_end
                            * filter_env_amplitude;
                    self.filter.set_cutoff(new_cutoff_percentage.into());
                } else if matches!(self.lfo_routing, LfoRouting::FilterCutoff) {
                    let lfo_for_cutoff = lfo * self.lfo_depth;
                    self.filter
                        .set_cutoff((self.filter_cutoff_start * (lfo_for_cutoff.0 + 1.0)).into());
                }
                let filtered_mix = self.filter.transform_channel(0, Sample::from(osc_sum)).0;

                // LFO amplitude modulation
                let lfo_for_amplitude =
                    Normal::from(if matches!(self.lfo_routing, LfoRouting::Amplitude) {
                        lfo * self.lfo_depth
                    } else {
                        BipolarNormal::zero()
                    });

                // Final
                self.dca.transform_audio_to_stereo(Sample(
                    filtered_mix * amp_env_amplitude.0 * lfo_for_amplitude.0,
                ))
            } else {
                StereoSample::SILENCE
            };
        }
    }
}
impl WelshVoice {
    fn tick_envelopes(&mut self) -> (Normal, Normal) {
        if self.is_playing() {
            self.amp_envelope.tick(1);
            self.filter_envelope.tick(1);
            if self.is_playing() {
                return (self.amp_envelope.value(), self.filter_envelope.value());
            }

            if self.steal_is_underway {
                self.steal_is_underway = false;
                self.note_on(self.note_on_key, self.note_on_velocity);
            }
        }
        (Normal::zero(), Normal::zero())
    }

    fn set_frequency_hz(&mut self, frequency_hz: FrequencyHz) {
        // It's safe to set the frequency on a fixed-frequency oscillator; the
        // fixed frequency is stored separately and takes precedence.
        self.oscillator_1.set_frequency(frequency_hz);
        self.oscillator_2.set_frequency(frequency_hz);
    }

    pub fn set_lfo_depth(&mut self, lfo_depth: Normal) {
        self.lfo_depth = lfo_depth;
    }

    pub fn set_filter_cutoff_start(&mut self, filter_cutoff_start: Normal) {
        self.filter_cutoff_start = filter_cutoff_start;
    }

    pub fn set_filter_cutoff_end(&mut self, filter_cutoff_end: Normal) {
        self.filter_cutoff_end = filter_cutoff_end;
    }

    pub fn set_oscillator_2_sync(&mut self, oscillator_2_sync: bool) {
        self.oscillator_2_sync = oscillator_2_sync;
    }

    pub fn set_oscillator_mix(&mut self, oscillator_mix: Normal) {
        self.oscillator_mix = oscillator_mix;
    }

    pub fn amp_envelope_mut(&mut self) -> &mut Envelope {
        &mut self.amp_envelope
    }

    pub fn filter_mut(&mut self) -> &mut BiQuadFilterLowPass24db {
        &mut self.filter
    }

    pub fn oscillator_2_sync(&self) -> bool {
        self.oscillator_2_sync
    }

    pub fn oscillator_mix(&self) -> Normal {
        self.oscillator_mix
    }

    pub fn lfo_routing(&self) -> LfoRouting {
        self.lfo_routing
    }

    pub fn lfo_depth(&self) -> Normal {
        self.lfo_depth
    }

    pub fn filter_cutoff_start(&self) -> Normal {
        self.filter_cutoff_start
    }

    pub fn filter_cutoff_end(&self) -> Normal {
        self.filter_cutoff_end
    }

    pub fn clone_fresh(&self) -> Self {
        todo!(" fill in the important fields... note that this likely matches the serde fields.");
        // let r = Self::default();
        // r
    }
}

/// A synthesizer inspired by Fred Welsh's [Welsh's Synthesizer
/// Cookbook](https://www.amazon.com/dp/B000ERHA4S/).
#[derive(Debug, Control, Deserialize, Serialize)]
pub struct WelshSynth {
    #[control]
    pub voice: WelshVoice,

    #[control]
    pub dca: Dca,

    #[serde(skip)]
    pub inner_synth: Synthesizer<WelshVoice>,
}
impl Default for WelshSynth {
    fn default() -> Self {
        const VOICE_CAPACITY: usize = 8;
        let voice_store = StealingVoiceStore::<WelshVoice>::new_with_voice(VOICE_CAPACITY, || {
            WelshVoice::default()
        });
        Self {
            inner_synth: Synthesizer::<WelshVoice>::new_with(Box::new(voice_store)),
            voice: WelshVoice::default(),
            dca: Dca::default(),
        }
    }
}

impl Generates<StereoSample> for WelshSynth {
    fn value(&self) -> StereoSample {
        self.inner_synth.value()
    }

    fn generate_batch_values(&mut self, values: &mut [StereoSample]) {
        self.inner_synth.generate_batch_values(values);
    }
}
impl Serializable for WelshSynth {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {
        const VOICE_CAPACITY: usize = 8;
        let voice_store = StealingVoiceStore::<WelshVoice>::new_with_voice(VOICE_CAPACITY, || {
            self.voice.clone_fresh()
        });
        self.inner_synth = Synthesizer::<WelshVoice>::new_with(Box::new(voice_store));
    }
}
impl Configurable for WelshSynth {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.inner_synth.update_sample_rate(sample_rate);
    }

    fn sample_rate(&self) -> SampleRate {
        self.inner_synth.sample_rate()
    }
}
impl Ticks for WelshSynth {
    fn tick(&mut self, tick_count: usize) {
        self.inner_synth.tick(tick_count);
    }
}
impl HandlesMidi for WelshSynth {
    fn handle_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        midi_messages_fn: &mut MidiMessagesFn,
    ) {
        match message {
            #[allow(unused_variables)]
            MidiMessage::ProgramChange { program } => {
                todo!()
                // if let Some(program) = GeneralMidiProgram::from_u8(program.as_int()) {
                //     if let Ok(_preset) = WelshSynth::general_midi_preset(&program) {
                //         //  self.preset = preset;
                //     } else {
                //         println!("unrecognized patch from MIDI program change: {}", &program);
                //     }
                // }
                // None
            }
            _ => self
                .inner_synth
                .handle_midi_message(channel, message, midi_messages_fn),
        }
    }
}
impl WelshSynth {
    pub fn preset_name(&self) -> &str {
        "none"
        //        self.preset.name.as_str()
    }

    // pub fn gain(&self) -> Normal {
    //     self.inner_synth.gain()
    // }

    // pub fn set_gain(&mut self, gain: Normal) {
    //     // This seems like a lot of duplication, but I think it's OK. The outer
    //     // synth handles automation. The inner synth needs a single source of
    //     // gain/pan, and the inner synth can't propagate to the voices because
    //     // (1) it doesn't actually know whether the voice handles those things,
    //     // and (2) I'm not sure we want to codify whether gain/pan are per-voice
    //     // or per-synth, meaning that the propagation is better placed in the
    //     // outer synth.
    //     //
    //     // All that said, I'm still getting used to composition over
    //     // inheritance. It feels weird for the concrete case to be at the top.
    //     // Maybe this is all just fine.
    //     self.gain = gain;
    //     self.dca.set_gain(gain);
    //     self.inner_synth.set_gain(gain);
    //     self.inner_synth.voices_mut().for_each(|v| v.set_gain(gain));
    //     self.inner_synth.set_gain(gain);
    // }

    // pub fn pan(&self) -> BipolarNormal {
    //     self.inner_synth.pan()
    // }

    // pub fn set_pan(&mut self, pan: BipolarNormal) {
    //     self.pan = pan;
    //     self.dca.set_pan(pan);
    //     self.inner_synth.set_pan(pan);
    //     self.inner_synth.voices_mut().for_each(|v| v.set_pan(pan));
    //     self.inner_synth.set_pan(pan);
    // }
}

#[cfg(test)]
mod tests {
    // use convert_case::{Case, Casing};
    // use crate::util::tests::TestOnlyPaths;

    // // TODO dedup
    // pub fn canonicalize_output_filename_and_path(filename: &str) -> String {
    //     let mut path = TestOnlyPaths::data_path();
    //     let snake_filename = format!("{}.wav", filename.to_case(Case::Snake)).to_string();
    //     path.push(snake_filename);
    //     if let Some(path) = path.to_str() {
    //         path.to_string()
    //     } else {
    //         panic!("trouble creating output path")
    //     }
    // }

    #[cfg(obsolete)]
    // TODO: refactor out to common test utilities
    #[allow(dead_code)]
    fn write_voice(voice: &mut WelshVoice, duration: f64, basename: &str) {
        let mut clock = Clock::new_with(&ClockParams {
            bpm: DEFAULT_BPM,
            midi_ticks_per_second: DEFAULT_MIDI_TICKS_PER_SECOND,
            time_signature: TimeSignatureParams { top: 4, bottom: 4 },
        });

        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: clock.sample_rate().value() as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        const AMPLITUDE: SampleType = i16::MAX as SampleType;
        let mut writer =
            hound::WavWriter::create(canonicalize_output_filename_and_path(basename), spec)
                .unwrap();

        let mut last_recognized_time_point = -1.;
        let time_note_off = duration / 2.0;
        while clock.seconds() < duration {
            if clock.seconds() >= 0.0 && last_recognized_time_point < 0.0 {
                last_recognized_time_point = clock.seconds();
                voice.note_on(60, 127);
                voice.tick_envelopes();
            } else if clock.seconds() >= time_note_off && last_recognized_time_point < time_note_off
            {
                last_recognized_time_point = clock.seconds();
                voice.note_off(127);
                voice.tick_envelopes();
            }

            voice.tick(1);
            let sample = voice.value();
            let _ = writer.write_sample((sample.0 .0 * AMPLITUDE) as i16);
            let _ = writer.write_sample((sample.1 .0 * AMPLITUDE) as i16);
            clock.tick(1);
        }
    }
}
