// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crossbeam_channel::{Receiver, Sender};
use ensnare_core::{
    audio::{AudioInterfaceEvent, AudioInterfaceInput, AudioStreamService},
    prelude::*,
    types::AudioQueue,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

// TODO: when we get rid of legacy/, look through here and remove unneeded
// pub(crate).

/// The panel provides updates to the app through [AudioPanelEvent] messages.
#[derive(Clone, Debug)]
pub enum AudioPanelEvent {
    /// The audio interface changed, and sample rate etc. might have changed.
    InterfaceChanged,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "SampleRate")]
struct SampleRateDef(usize);

/// Contains persistent audio settings.
#[derive(Debug, Serialize, Deserialize)]
pub struct AudioSettings {
    #[serde(with = "SampleRateDef")]
    sample_rate: SampleRate,
    channel_count: u16,

    #[serde(skip)]
    has_been_saved: bool,
}
impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            sample_rate: SampleRate::default(),
            channel_count: 2,
            has_been_saved: false,
        }
    }
}
impl HasSettings for AudioSettings {
    fn has_been_saved(&self) -> bool {
        self.has_been_saved
    }

    fn needs_save(&mut self) {
        self.has_been_saved = false;
    }

    fn mark_clean(&mut self) {
        self.has_been_saved = true;
    }
}
impl AudioSettings {
    pub(crate) fn new_with(sample_rate: SampleRate, channel_count: u16) -> Self {
        Self {
            sample_rate,
            channel_count,
            has_been_saved: Default::default(),
        }
    }

    pub(crate) fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    pub(crate) fn channel_count(&self) -> u16 {
        self.channel_count
    }
}

// Thanks https://boydjohnson.dev/blog/impl-debug-for-fn-type/
pub trait NeedsAudioFnT: FnMut(&AudioQueue, usize) + Sync + Send {}
impl<F> NeedsAudioFnT for F where F: FnMut(&AudioQueue, usize) + Sync + Send {}
/// Takes an [AudioQueue] that accepts [StereoSample]s, and the number of
/// [StereoSample]s that the audio interface has requested.
pub type NeedsAudioFn = Box<dyn NeedsAudioFnT>;

/// [AudioService] manages the audio interface.
#[derive(Debug)]
pub struct AudioService {
    #[allow(dead_code)]
    sender: Sender<AudioInterfaceInput>,
    app_receiver: Receiver<AudioPanelEvent>, // to give to the app to receive what we sent
    app_sender: Sender<AudioPanelEvent>,     // for us to send to the app

    config: Arc<Mutex<Option<AudioSettings>>>,
}
impl AudioService {
    /// Construct a new [AudioService].
    pub fn new_with(needs_audio_fn: NeedsAudioFn) -> Self {
        let audio_stream_service = AudioStreamService::default();
        let sender = audio_stream_service.sender().clone();

        let (app_sender, app_receiver) = crossbeam_channel::unbounded();

        let r = Self {
            sender,
            app_sender,
            app_receiver,
            config: Default::default(),
        };
        r.start_audio_stream(needs_audio_fn, audio_stream_service.receiver().clone());

        r
    }

    fn start_audio_stream(
        &self,
        mut needs_audio_fn: NeedsAudioFn,
        receiver: Receiver<AudioInterfaceEvent>,
    ) {
        let config = Arc::clone(&self.config);
        let app_sender = self.app_sender.clone();
        std::thread::spawn(move || {
            let mut queue_opt = None;
            loop {
                if let Ok(event) = receiver.recv() {
                    match event {
                        AudioInterfaceEvent::Reset(sample_rate, channel_count, queue) => {
                            if let Ok(mut config) = config.lock() {
                                *config = Some(AudioSettings::new_with(sample_rate, channel_count));
                            }
                            let _ = app_sender.send(AudioPanelEvent::InterfaceChanged);
                            queue_opt = Some(queue);
                        }
                        AudioInterfaceEvent::NeedsAudio(_when, count) => {
                            if let Some(queue) = queue_opt.as_ref() {
                                (*needs_audio_fn)(queue, count);
                            }
                        }
                        AudioInterfaceEvent::Quit => todo!(),
                    }
                } else {
                    eprintln!("Unexpected failure of AudioInterfaceEvent channel");
                    break;
                }
            }
        });
    }

    /// The audio interface's current sample rate
    pub fn sample_rate(&self) -> SampleRate {
        if let Ok(config) = self.config.lock() {
            if let Some(config) = config.as_ref() {
                return config.sample_rate;
            }
        }
        eprintln!("Warning: returning default sample rate because actual was not available");
        SampleRate::DEFAULT
    }

    /// The audio interface's current number of channels. 1 = mono, 2 = stereo
    pub fn channel_count(&self) -> u16 {
        if let Ok(config) = self.config.lock() {
            if let Some(config) = config.as_ref() {
                return config.channel_count;
            }
        }
        0
    }

    /// The receive side of the [AudioPanelEvent] channel
    pub fn receiver(&self) -> &Receiver<AudioPanelEvent> {
        &self.app_receiver
    }

    /// Cleans up the audio service for quitting.
    pub fn exit(&self) {
        // TODO: Create the AudioPanelInput channel, add it to the receiver loop, etc.
        eprintln!("Audio Panel acks the quit... TODO");
    }
}