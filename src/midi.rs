// Copyright (c) 2023 Mike Tsao. All rights reserved.

use derive_more::Display as DeriveDisplay;
use serde::{Deserialize, Serialize};
use strum_macros::FromRepr;

pub use midly::live::LiveEvent;
pub use midly::{
    num::{u4, u7},
    MidiMessage,
};
/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use crate::midi::{
        new_note_off, new_note_on, u4, u7, HandlesMidi, MidiChannel, MidiMessage, MidiMessagesFn,
        MidiNote,
    };
}

/// Newtype for MIDI channel.
#[derive(
    Clone, Copy, Debug, Default, DeriveDisplay, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct MidiChannel(pub u8);
#[allow(missing_docs)]
impl MidiChannel {
    pub const MAX: u8 = 16;

    pub const fn new(value: u8) -> Self {
        Self(value)
    }
}
impl From<u4> for MidiChannel {
    fn from(value: u4) -> Self {
        Self(value.as_int())
    }
}
impl From<u8> for MidiChannel {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<MidiChannel> for u8 {
    fn from(value: MidiChannel) -> Self {
        value.0
    }
}

/// There are two different mappings of piano notes to MIDI numbers. They both
/// agree that Midi note 0 is a C, but they otherwise differ by an octave. I
/// originally picked C4=60, because that was the top Google search result's
/// answer, but it seems like a slight majority thinks C3=60. I'm going to leave
/// it as-is so that I don't have to rename my test data files. I don't think it
/// matters because we're not actually mapping these to anything user-visible.
///
/// A small disadvantage of C3=60 is that numbers 0-11 don't map to an easily
/// described octave. So I'm calling that octave "Sub0" because I needed
/// something in this enum.
///
/// These also correspond to
/// <https://en.wikipedia.org/wiki/Piano_key_frequencies>
//
// Generated with this Python code:
// ```
// #!/usr/bin/python
//
// NAMES = ["C", "Cs", "D", "Ds", "E", "F", "Fs", "G", "Gs", "A", "As", "B"]
//
// index = 12
// for i in range(0, 10):
//     for name in NAMES:
//         print("%s%d = %d," % (name, i, index))
//         index += 1
// ```
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default, FromRepr)]
pub enum MidiNote {
    CSub0 = 0,
    CsSub0 = 1,
    DSub0 = 2,
    DsSub0 = 3,
    ESub0 = 4,
    FSub0 = 5,
    FsSub0 = 6,
    GSub0 = 7,
    GsSub0 = 8,
    ASub0 = 9,
    AsSub0 = 10,
    BSub0 = 11,
    C0 = 12,
    Cs0 = 13,
    D0 = 14,
    Ds0 = 15,
    E0 = 16,
    F0 = 17,
    Fs0 = 18,
    G0 = 19,
    Gs0 = 20,
    A0 = 21,
    As0 = 22,
    B0 = 23,
    C1 = 24,
    Cs1 = 25,
    D1 = 26,
    Ds1 = 27,
    E1 = 28,
    F1 = 29,
    Fs1 = 30,
    G1 = 31,
    Gs1 = 32,
    A1 = 33,
    As1 = 34,
    B1 = 35,
    C2 = 36,
    Cs2 = 37,
    D2 = 38,
    Ds2 = 39,
    E2 = 40,
    F2 = 41,
    Fs2 = 42,
    G2 = 43,
    Gs2 = 44,
    A2 = 45,
    As2 = 46,
    B2 = 47,
    C3 = 48,
    Cs3 = 49,
    D3 = 50,
    Ds3 = 51,
    E3 = 52,
    F3 = 53,
    Fs3 = 54,
    G3 = 55,
    Gs3 = 56,
    A3 = 57,
    As3 = 58,
    B3 = 59,
    #[default]
    C4 = 60,
    Cs4 = 61,
    D4 = 62,
    Ds4 = 63,
    E4 = 64,
    F4 = 65,
    Fs4 = 66,
    G4 = 67,
    Gs4 = 68,
    A4 = 69,
    As4 = 70,
    B4 = 71,
    C5 = 72,
    Cs5 = 73,
    D5 = 74,
    Ds5 = 75,
    E5 = 76,
    F5 = 77,
    Fs5 = 78,
    G5 = 79,
    Gs5 = 80,
    A5 = 81,
    As5 = 82,
    B5 = 83,
    C6 = 84,
    Cs6 = 85,
    D6 = 86,
    Ds6 = 87,
    E6 = 88,
    F6 = 89,
    Fs6 = 90,
    G6 = 91,
    Gs6 = 92,
    A6 = 93,
    As6 = 94,
    B6 = 95,
    C7 = 96,
    Cs7 = 97,
    D7 = 98,
    Ds7 = 99,
    E7 = 100,
    F7 = 101,
    Fs7 = 102,
    G7 = 103,
    Gs7 = 104,
    A7 = 105,
    As7 = 106,
    B7 = 107,
    C8 = 108,
    Cs8 = 109,
    D8 = 110,
    Ds8 = 111,
    E8 = 112,
    F8 = 113,
    Fs8 = 114,
    G8 = 115,
    Gs8 = 116,
    A8 = 117,
    As8 = 118,
    B8 = 119,
    C9 = 120,
    Cs9 = 121,
    D9 = 122,
    Ds9 = 123,
    E9 = 124,
    F9 = 125,
    Fs9 = 126,
    G9 = 127,
}
#[allow(missing_docs)]
impl MidiNote {
    pub const MIN: MidiNote = Self::C0;
    pub const MAX: MidiNote = Self::G9;
}

/// Convenience function to make a note-on [MidiMessage].
pub fn new_note_on(note: u8, vel: u8) -> MidiMessage {
    MidiMessage::NoteOn {
        key: u7::from(note),
        vel: u7::from(vel),
    }
}

/// Convenience function to make a note-off [MidiMessage].
pub fn new_note_off(note: u8, vel: u8) -> MidiMessage {
    MidiMessage::NoteOff {
        key: u7::from(note),
        vel: u7::from(vel),
    }
}

/// The General MIDI instruments. https://en.wikipedia.org/wiki/General_MIDI
#[allow(missing_docs)]
#[derive(DeriveDisplay, Debug)]
pub enum GeneralMidiProgram {
    AcousticGrand = 0,
    BrightAcoustic = 1,
    ElectricGrand = 2,
    HonkyTonk = 3,
    ElectricPiano1 = 4,
    ElectricPiano2 = 5,
    Harpsichord = 6,
    Clav = 7,
    Celesta = 8,
    Glockenspiel = 9,
    MusicBox = 10,
    Vibraphone = 11,
    Marimba = 12,
    Xylophone = 13,
    TubularBells = 14,
    Dulcimer = 15,
    DrawbarOrgan = 16,
    PercussiveOrgan = 17,
    RockOrgan = 18,
    ChurchOrgan = 19,
    ReedOrgan = 20,
    Accordion = 21,
    Harmonica = 22,
    TangoAccordion = 23,
    AcousticGuitarNylon = 24,
    AcousticGuitarSteel = 25,
    ElectricGuitarJazz = 26,
    ElectricGuitarClean = 27,
    ElectricGuitarMuted = 28,
    OverdrivenGuitar = 29,
    DistortionGuitar = 30,
    GuitarHarmonics = 31,
    AcousticBass = 32,
    ElectricBassFinger = 33,
    ElectricBassPick = 34,
    FretlessBass = 35,
    SlapBass1 = 36,
    SlapBass2 = 37,
    SynthBass1 = 38,
    SynthBass2 = 39,
    Violin = 40,
    Viola = 41,
    Cello = 42,
    Contrabass = 43,
    TremoloStrings = 44,
    PizzicatoStrings = 45,
    OrchestralHarp = 46,
    Timpani = 47,
    StringEnsemble1 = 48,
    StringEnsemble2 = 49,
    Synthstrings1 = 50,
    Synthstrings2 = 51,
    ChoirAahs = 52,
    VoiceOohs = 53,
    SynthVoice = 54,
    OrchestraHit = 55,
    Trumpet = 56,
    Trombone = 57,
    Tuba = 58,
    MutedTrumpet = 59,
    FrenchHorn = 60,
    BrassSection = 61,
    Synthbrass1 = 62,
    Synthbrass2 = 63,
    SopranoSax = 64,
    AltoSax = 65,
    TenorSax = 66,
    BaritoneSax = 67,
    Oboe = 68,
    EnglishHorn = 69,
    Bassoon = 70,
    Clarinet = 71,
    Piccolo = 72,
    Flute = 73,
    Recorder = 74,
    PanFlute = 75,
    BlownBottle = 76,
    Shakuhachi = 77,
    Whistle = 78,
    Ocarina = 79,
    Lead1Square = 80,
    Lead2Sawtooth = 81,
    Lead3Calliope = 82,
    Lead4Chiff = 83,
    Lead5Charang = 84,
    Lead6Voice = 85,
    Lead7Fifths = 86,
    Lead8BassLead = 87,
    Pad1NewAge = 88,
    Pad2Warm = 89,
    Pad3Polysynth = 90,
    Pad4Choir = 91,
    Pad5Bowed = 92,
    Pad6Metallic = 93,
    Pad7Halo = 94,
    Pad8Sweep = 95,
    Fx1Rain = 96,
    Fx2Soundtrack = 97,
    Fx3Crystal = 98,
    Fx4Atmosphere = 99,
    Fx5Brightness = 100,
    Fx6Goblins = 101,
    Fx7Echoes = 102,
    Fx8SciFi = 103,
    Sitar = 104,
    Banjo = 105,
    Shamisen = 106,
    Koto = 107,
    Kalimba = 108,
    Bagpipe = 109,
    Fiddle = 110,
    Shanai = 111,
    TinkleBell = 112,
    Agogo = 113,
    SteelDrums = 114,
    Woodblock = 115,
    TaikoDrum = 116,
    MelodicTom = 117,
    SynthDrum = 118,
    ReverseCymbal = 119,
    GuitarFretNoise = 120,
    BreathNoise = 121,
    Seashore = 122,
    BirdTweet = 123,
    TelephoneRing = 124,
    Helicopter = 125,
    Applause = 126,
    Gunshot = 127,
}

/// The General MIDI percussion instruments. https://en.wikipedia.org/wiki/General_MIDI#Percussion
#[allow(missing_docs)]
#[derive(Clone, Copy)]
pub enum GeneralMidiPercussionProgram {
    AcousticBassDrum = 35,
    ElectricBassDrum = 36,
    SideStick = 37,
    AcousticSnare = 38,
    HandClap = 39,
    ElectricSnare = 40,
    LowFloorTom = 41,
    ClosedHiHat = 42,
    HighFloorTom = 43,
    PedalHiHat = 44,
    LowTom = 45,
    OpenHiHat = 46,
    LowMidTom = 47,
    HiMidTom = 48,
    CrashCymbal1 = 49,
    HighTom = 50,
    RideCymbal1 = 51,
    ChineseCymbal = 52,
    RideBell = 53,
    Tambourine = 54,
    SplashCymbal = 55,
    Cowbell = 56,
    CrashCymbal2 = 57,
    Vibraslap = 58,
    RideCymbal2 = 59,
    HighBongo = 60,
    LowBongo = 61,
    MuteHighConga = 62,
    OpenHighConga = 63,
    LowConga = 64,
    HighTimbale = 65,
    LowTimbale = 66,
    HighAgogo = 67,
    LowAgogo = 68,
    Cabasa = 69,
    Maracas = 70,
    ShortWhistle = 71,
    LongWhistle = 72,
    ShortGuiro = 73,
    LongGuiro = 74,
    Claves = 75,
    HighWoodblock = 76,
    LowWoodblock = 77,
    MuteCuica = 78,
    OpenCuica = 79,
    MuteTriangle = 80,
    OpenTriangle = 81,
}

/// The callback signature for handle_midi_message().
pub type MidiMessagesFn<'a> = dyn FnMut(MidiChannel, MidiMessage) + 'a;

/// Takes standard MIDI messages. Implementers can ignore MidiChannel if it's
/// not important, as the virtual cabling model tries to route only relevant
/// traffic to individual devices.
pub trait HandlesMidi {
    #[allow(missing_docs)]
    #[allow(unused_variables)]
    fn handle_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        midi_messages_fn: &mut MidiMessagesFn,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use crate::{midi::MidiNote, prelude::FrequencyHz};

    #[test]
    fn midi_note_is_complete() {
        for key in 0..127 {
            assert_eq!(MidiNote::from_repr(key).unwrap() as usize, key);
        }
    }

    #[test]
    fn note_to_frequency() {
        // https://www.colincrawley.com/midi-note-to-audio-frequency-calculator/
        assert_eq!(
            FrequencyHz::from(MidiNote::C0),
            16.351_597_831_287_414.into()
        );
        assert_eq!(
            FrequencyHz::from(MidiNote::C4),
            261.625_565_300_598_6.into()
        );
        assert_eq!(
            FrequencyHz::from(MidiNote::D5),
            587.329_535_834_815_1.into()
        );
        assert_eq!(
            FrequencyHz::from(MidiNote::D6),
            1_174.659_071_669_630_3.into()
        );
        assert_eq!(
            FrequencyHz::from(MidiNote::G9),
            12_543.853_951_415_975.into()
        );
    }
}
