// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_cores_egui::piano_roll::piano_roll;
use ensnare_entity::prelude::*;
use ensnare_proc_macros::{IsController, Metadata};

#[derive(Debug, IsController, Metadata)]
pub struct PianoRoll {
    uid: Uid,
    inner: ensnare_core::piano_roll::PianoRoll,
}
impl Displays for PianoRoll {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(piano_roll(&mut self.inner))
    }
}
impl PianoRoll {
    pub fn new(uid: Uid) -> Self {
        Self {
            uid,
            inner: Default::default(),
        }
    }
    pub fn new_with(uid: Uid, inner: ensnare_core::piano_roll::PianoRoll) -> Self {
        Self { uid, inner }
    }
}
impl Configurable for PianoRoll {}
impl Controls for PianoRoll {}
impl HandlesMidi for PianoRoll {}
impl Serializable for PianoRoll {}