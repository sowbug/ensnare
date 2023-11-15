// Copyright (c) 2023 Mike Tsao. All rights reserved.

use anyhow::anyhow;
use crossbeam_channel::{Receiver, Sender};
use ensnare_core::{piano_roll::PatternUid, prelude::*, selection_set::SelectionSet};
use ensnare_entity::prelude::*;
use ensnare_orchestration::{traits::Orchestrates, OldOrchestrator, Orchestrator};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard},
};

/// Commands that [Orchestrator] accepts.
#[derive(Clone, Debug)]
pub enum OrchestratorInput {
    /// An external MIDI message arrived.
    Midi(MidiChannel, MidiMessage),
    /// Open the project file at the given path, load it, and replace the
    /// current [Orchestrator] instance with it.
    ProjectOpen(PathBuf),
    /// Create a new blank project.
    ProjectNew,
    /// Start playing the current project.
    ProjectPlay,
    /// Save the current project to the specified file.
    ProjectSave(PathBuf),
    /// Stop playing the current project.
    ProjectStop,
    /// Delete all selected tracks.
    TrackDeleteSelected,
    /// Duplicate all selected tracks, placing the new one(s) below them.
    TrackDuplicateSelected,
    /// Create a new audio track.
    TrackNewAudio,
    /// Create a new aux track.
    TrackNewAux,
    /// Create a new MIDI track.
    TrackNewMidi,
    /// Delete the selected arranged patterns.
    TrackPatternRemoveSelected,
    /// Add the given PianoRoll pattern to the track at the specified time.
    TrackPatternAdd(TrackUid, PatternUid, MusicalTime),
    /// Add a new entity to the specified track.
    TrackAddEntity(TrackUid, EntityKey),
    /// Add a new entity to the selected track.
    //TrackAddEntity(EntityKey),
    /// Sets the tempo.
    Tempo(Tempo),
    /// Link the given source controller to the target controllable's parameter.
    LinkControl(Uid, Uid, ControlIndex),

    /// Quit the thread.
    Quit,
}

/// Events that [Orchestrator] generates.
#[derive(Debug)]
pub enum OrchestratorEvent {
    /// This is the current [Tempo].
    Tempo(Tempo),

    /// A new, empty project was created.
    New,

    /// A project has been successfully opened from the specified path with the
    /// specified title (if any).
    Loaded(PathBuf, Option<String>),
    /// A project failed to load.
    LoadError(PathBuf, anyhow::Error),

    /// The current project was successfully saved to the specified path.
    Saved(PathBuf),
    /// An attempt to save the current project failed.
    SaveError(PathBuf, anyhow::Error),

    /// Acknowledge request to quit.
    Quit,
}

/// A wrapper around an [Orchestrator] that manages its lifetime in a separate
/// thread. Communicate with it by sending [OrchestratorInput] messages and
/// receiving [OrchestratorEvent] messages.
#[derive(Debug, Default)]
pub struct OrchestratorService {
    pub orchestrator: Arc<Mutex<Orchestrator>>,
    track_selection_set: Arc<Mutex<SelectionSet<TrackUid>>>,
    input_channel_pair: ChannelPair<OrchestratorInput>,
    event_channel_pair: ChannelPair<OrchestratorEvent>,

    is_control_only_down: bool,
}
impl OrchestratorService {
    pub fn new_with(orchestrator: &Arc<Mutex<Orchestrator>>) -> Self {
        let mut r = Self {
            orchestrator: Arc::clone(orchestrator),
            track_selection_set: Default::default(),
            input_channel_pair: Default::default(),
            event_channel_pair: Default::default(),
            is_control_only_down: Default::default(),
        };
        r.start_thread();
        r
    }

    pub fn set_control_only_down(&mut self, is_control_only_down: bool) {
        self.is_control_only_down = is_control_only_down;
    }

    /// The sending side of the [OrchestratorInput] channel.
    pub fn sender(&self) -> &Sender<OrchestratorInput> {
        &self.input_channel_pair.sender
    }

    /// The receiving side of the [OrchestratorEvent] channel.
    pub fn receiver(&self) -> &Receiver<OrchestratorEvent> {
        &self.event_channel_pair.receiver
    }

    /// Sends the given [OrchestratorInput] to the [Orchestrator].
    pub fn send_to_service(&self, input: OrchestratorInput) {
        match self.sender().send(input) {
            Ok(_) => {}
            Err(err) => eprintln!("sending OrchestratorInput failed with {:?}", err),
        }
    }

    /// Requests that the [Orchestrator] prepare to exit.
    pub fn exit(&self) {
        eprintln!("OrchestratorInput::Quit");
        self.send_to_service(OrchestratorInput::Quit);
    }

    fn start_thread(&mut self) {
        let receiver = self.input_channel_pair.receiver.clone();
        let sender = self.event_channel_pair.sender.clone();
        self.introduce();
        let orchestrator = Arc::clone(&self.orchestrator);
        let track_selection_set = Arc::clone(&self.track_selection_set);
        std::thread::spawn(move || loop {
            let recv = receiver.recv();
            if let Ok(mut o) = orchestrator.lock() {
                // TODO: when you have time, arrange for this thread to get a
                // copy of egui::Context so that it can request a repaint after
                // receiving a message. I think that's why drag and drop
                // sometimes needs a wiggle for its results to appear.

                match recv {
                    Ok(input) => match input {
                        OrchestratorInput::Midi(channel, message) => {
                            Self::handle_input_midi(&mut o, channel, message);
                        }
                        OrchestratorInput::ProjectPlay => o.play(),
                        OrchestratorInput::ProjectStop => o.stop(),
                        OrchestratorInput::ProjectNew => {
                            let mo = Orchestrator::default();
                            // o.prepare_successor(&mut mo);
                            // let _ = mo.create_starter_tracks(); // TODO: DRY this
                            *o = mo;
                            let _ = sender.send(OrchestratorEvent::New);
                        }
                        OrchestratorInput::ProjectOpen(path) => {
                            match Self::handle_input_load(&path) {
                                Ok(mo) => {
                                    // o.prepare_successor(&mut mo);
                                    *o = mo;
                                    // let _ = sender
                                    //     .send(OrchestratorEvent::Loaded(path, o.title.clone()));
                                }
                                Err(err) => {
                                    let _ = sender.send(OrchestratorEvent::LoadError(path, err));
                                }
                            }
                            {}
                        }
                        OrchestratorInput::ProjectSave(path) => {
                            match Self::handle_input_save(&o, &path) {
                                Ok(_) => {
                                    let _ = sender.send(OrchestratorEvent::Saved(path));
                                }
                                Err(err) => {
                                    let _ = sender.send(OrchestratorEvent::SaveError(path, err));
                                }
                            }
                        }
                        OrchestratorInput::Quit => {
                            let _ = sender.send(OrchestratorEvent::Quit);
                            break;
                        }
                        OrchestratorInput::TrackNewMidi => {
                            // let _ = o.new_midi_track();
                        }
                        OrchestratorInput::TrackNewAudio => {
                            // let _ = o.new_audio_track();
                        }
                        OrchestratorInput::TrackNewAux => {
                            // let _ = o.new_aux_track();
                        }
                        OrchestratorInput::TrackDeleteSelected => {
                            if let Ok(track_selection_set) = track_selection_set.lock() {
                                let uids = Vec::from_iter(track_selection_set.iter().copied());
                                o.delete_tracks(&uids);
                            }
                        }
                        OrchestratorInput::TrackDuplicateSelected => {
                            todo!("duplicate selected tracks");
                        }
                        OrchestratorInput::TrackPatternRemoveSelected => {
                            unimplemented!()
                        }
                        OrchestratorInput::Tempo(tempo) => {
                            o.update_tempo(tempo);
                            let _ = sender.send(OrchestratorEvent::Tempo(tempo));
                        }
                        OrchestratorInput::LinkControl(source_uid, target_uid, control_index) => {
                            let _ = o.link_control(source_uid, target_uid, control_index);
                        }
                        OrchestratorInput::TrackPatternAdd(track_uid, pattern_uid, position) => {
                            let _ = o.add_pattern_to_track(&track_uid, &pattern_uid, position);
                        }
                        OrchestratorInput::TrackAddEntity(track_uid, key) => {
                            let uid = o.mint_entity_uid();
                            if let Some(entity) = EntityFactory::global().new_entity(&key, uid) {
                                let _ = o.add_entity(&track_uid, entity);
                            }
                        }
                    },
                    Err(err) => {
                        eprintln!("unexpected failure of OrchestratorInput channel: {:?}", err);
                        break;
                    }
                }
            }
        });
    }

    // Send any important initial messages after creation.
    fn introduce(&self) {
        if let Ok(o) = self.orchestrator.lock() {
            self.broadcast_tempo(o.tempo());
        }
    }

    fn broadcast_tempo(&self, tempo: Tempo) {
        self.broadcast(OrchestratorEvent::Tempo(tempo));
    }

    fn broadcast(&self, event: OrchestratorEvent) {
        let _ = self.event_channel_pair.sender.send(event);
    }

    fn handle_input_midi(
        orchestrator: &mut MutexGuard<Orchestrator>,
        channel: MidiChannel,
        message: MidiMessage,
    ) {
        orchestrator.handle_midi_message(channel, message, &mut |_, _| {});
    }

    fn handle_input_load(_path: &PathBuf) -> anyhow::Result<Orchestrator> {
        Err(anyhow!("FIX THIS"))
        // match std::fs::read_to_string(path) {
        //     Ok(project_string) => match serde_json::from_str::<Orchestrator>(&project_string) {
        //         Ok(mut mo) => {
        //             mo.after_deser();
        //             anyhow::Ok(mo)
        //         }
        //         Err(err) => Err(anyhow!("Error while parsing: {}", err)),
        //     },
        //     Err(err) => Err(anyhow!("Error while reading: {}", err)),
        // }
    }

    fn handle_input_save(_o: &MutexGuard<Orchestrator>, _path: &PathBuf) -> anyhow::Result<()> {
        Err(anyhow!("FIX THIS"))
        // let o: &Orchestrator = o;
        // match serde_json::to_string_pretty(o)
        //     .map_err(|_| anyhow::format_err!("Unable to serialize prefs JSON"))
        // {
        //     Ok(json) => match std::fs::write(path, json) {
        //         Ok(_) => Ok(()),
        //         Err(err) => Err(anyhow!("While writing project: {}", err)),
        //     },
        //     Err(err) => Err(anyhow!("While serializing project: {}", err)),
        // }
    }

    /// Whether one or more tracks are currently selected.
    pub fn is_any_track_selected(&self) -> bool {
        if let Ok(tss) = self.track_selection_set.lock() {
            !tss.is_empty()
        } else {
            false
        }
    }
}