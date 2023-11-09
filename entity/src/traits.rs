// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;

/// [Entity] is a generic type of thing that can have various discoverable
/// capabilities. Almost everything in this system is an Entity of some kind.
/// The implementation of these trait methods is usually generated by the Is
/// proc macros ([IsController], [IsEffect], [IsInstrument], etc.)
#[allow(missing_docs)]
//#[typetag::serde(tag = "type")]
pub trait Entity:
    HasMetadata + Displays + Configurable + Serializable + std::fmt::Debug + Send + Sync
{
    fn as_controller(&self) -> Option<&dyn IsController> {
        None
    }
    fn as_controller_mut(&mut self) -> Option<&mut dyn IsController> {
        None
    }
    fn as_effect(&self) -> Option<&dyn IsEffect> {
        None
    }
    fn as_effect_mut(&mut self) -> Option<&mut dyn IsEffect> {
        None
    }
    fn as_instrument(&self) -> Option<&dyn IsInstrument> {
        None
    }
    fn as_instrument_mut(&mut self) -> Option<&mut dyn IsInstrument> {
        None
    }
    fn as_handles_midi(&self) -> Option<&dyn HandlesMidi> {
        None
    }
    fn as_handles_midi_mut(&mut self) -> Option<&mut dyn HandlesMidi> {
        None
    }
    fn as_controllable(&self) -> Option<&dyn Controllable> {
        None
    }
    fn as_controllable_mut(&mut self) -> Option<&mut dyn Controllable> {
        None
    }
}

/// An [IsController] controls things in the system that implement
/// [Controllable]. Examples are sequencers, arpeggiators, and discrete LFOs (as
/// contrasted with LFOs that are integrated into other instruments).
///
/// [IsController] emits messages, either control messages that the system
/// routes to [Controllable]s, or MIDI messages that go over the MIDI bus.
///
/// An [IsController] is the only kind of entity that can "finish." An
/// [IsEffect] or [IsInstrument] can't finish; they wait forever for audio to
/// process, or MIDI commands to handle. A performance ends once all
/// [IsController] entities indicate that they've finished.
pub trait IsController:
    Controls + HandlesMidi + HasMetadata + Displays + Send + std::fmt::Debug
{
}

/// An [IsEffect] transforms audio. It takes audio inputs and produces audio
/// output. It does not get called unless there is audio input to provide to it
/// (which can include silence, e.g., in the case of a muted instrument).
pub trait IsEffect:
    TransformsAudio + Controllable + Configurable + HasMetadata + Displays + Send + std::fmt::Debug
{
}

/// An [IsInstrument] produces audio, usually upon request from MIDI or
/// [IsController] input.
pub trait IsInstrument:
    Generates<StereoSample>
    + HandlesMidi
    + Controllable
    + HasMetadata
    + Displays
    + Send
    + std::fmt::Debug
{
}

/// Something that can be called during egui rendering to display a view of
/// itself.
//
// Adapted from egui_demo_lib/src/demo/mod.rs
pub trait Displays {
    /// Renders this Entity. Returns a [Response](egui::Response).
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}

/// Reports an action requested during [Displays::ui()].
pub trait Acts: Displays {
    type Action: IsAction;

    /// Sets the current action. Typically called by the UI code that just
    /// detected a click, drop, change, etc.
    fn set_action(&mut self, action: Self::Action);

    /// Returns the pending action, if any, and resets it to None. Typically
    /// called by the UI code that has the context needed to perform the action.
    fn take_action(&mut self) -> Option<Self::Action>;
}
pub trait IsAction: std::fmt::Debug + std::fmt::Display {}
