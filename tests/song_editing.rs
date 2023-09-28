// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::prelude::*;

#[test]
fn edit_song() {
    let mut orchestrator = OrchestratorBuilder::default()
        .title(Some("Simple Song (Edits)".to_string()))
        .build()
        .unwrap();
    let factory = register_factory_entities(EntityFactory::default());

    // Create two MIDI tracks.
    let rhythm_track_uid = orchestrator.new_midi_track().unwrap();
    let lead_track_uid = orchestrator.new_midi_track().unwrap();

    // Prepare the rhythm track first. Create a rhythm pattern, add it to the
    // PianoRoll, and then manipulate it. If we were really doing this in Rust
    // code, it would be simpler to create, manipulate, and then add, rather
    // than create, add, and manipulate, because PianoRoll takes ownership. But
    // in a DAW, we expect that PianoRoll's GUI will do the pattern
    // manipulation, so we're modeling that flow. This requires a bit of scoping
    // to satisfy the borrow checker.
    let drum_pattern = PatternBuilder::default().build().unwrap();
    let drum_pattern_uid = {
        let mut piano_roll = orchestrator.piano_roll_mut();
        piano_roll.insert(drum_pattern)
    };
    let drum_pattern = {
        let mut piano_roll = orchestrator.piano_roll_mut();
        let drum_pattern = piano_roll.get_pattern_mut(&drum_pattern_uid).unwrap();

        let mut note = Note {
            key: 60,
            range: MusicalTime::START..(MusicalTime::START + MusicalTime::DURATION_HALF),
        };
        // Add to the pattern.
        drum_pattern.add_note(note.clone());
        // Wait, no, didn't want to do that.
        drum_pattern.remove_note(&note);
        // It should be a kick. Change and then re-add.
        note.key = 35;
        drum_pattern.add_note(note.clone());

        // We don't have to keep removing/re-adding to edit notes. If we can
        // describe them, then we can edit them within the pattern.
        let note = drum_pattern.change_note_key(&note.clone(), 39).unwrap();
        let note = drum_pattern
            .move_note(
                &note.clone(),
                note.range.start + MusicalTime::DURATION_BREVE,
            )
            .unwrap();
        let _ = drum_pattern
            .move_and_resize_note(
                &note.clone(),
                MusicalTime::START,
                MusicalTime::DURATION_SIXTEENTH,
            )
            .unwrap();
        drum_pattern.clone()
    };

    // TEMP while we decide if patterns or notes are the basic sequencer unit

    // Pattern is good; add an instrument to the track.
    let rhythm_track = orchestrator.get_track_mut(&rhythm_track_uid).unwrap();
    let _drumkit_uid = rhythm_track
        .append_entity(
            factory
                .new_entity(&EntityKey::from("toy-instrument"))
                .unwrap(),
        )
        .unwrap();

    // Arrange the drum pattern.
    let _ = rhythm_track
        .sequencer_mut()
        .insert_pattern(&drum_pattern, MusicalTime::START);

    // Now set up the lead track. We need a pattern; we'll whip up something
    // quickly because we already showed the editing process while making the
    // drum pattern.
    let lead_pattern = PatternBuilder::default()
        .note_sequence(
            vec![
                60, 255, 62, 255, 64, 255, 65, 255, 67, 255, 69, 255, 71, 255, 72, 255,
            ],
            None,
        )
        .build()
        .unwrap();
    let _ = {
        let mut piano_roll = orchestrator.piano_roll_mut();
        piano_roll.insert(lead_pattern.clone())
    };

    let lead_track = orchestrator.get_track_mut(&lead_track_uid).unwrap();
    let welsh_synth_uid = lead_track
        .append_entity(factory.new_entity(&EntityKey::from("toy-synth")).unwrap())
        .unwrap();

    // Hmmm, we don't like the sound of that synth; let's replace it with another.
    lead_track.remove_entity(&welsh_synth_uid);
    let _toy_synth_uid = lead_track
        .append_entity(factory.new_entity(&EntityKey::from("toy-synth")).unwrap())
        .unwrap();

    // That's better, but it needs an effect.
    let _lead_reverb_uid = lead_track
        .append_entity(factory.new_entity(&EntityKey::from("reverb")).unwrap())
        .unwrap();
    // And another.
    let lead_gain_uid = lead_track
        .append_entity(factory.new_entity(&EntityKey::from("gain")).unwrap())
        .unwrap();
    // Sounds better if gain is first in chain.
    let _ = lead_track.move_effect(lead_gain_uid, 0);

    // Arrange the lead pattern.
    let _ = lead_track
        .sequencer_mut()
        .insert_pattern(&lead_pattern, MusicalTime::START);

    // https://doc.rust-lang.org/std/path/struct.PathBuf.html example
    let output_path: std::path::PathBuf =
        [env!("CARGO_TARGET_TMPDIR"), "simple-song-with-edits.wav"]
            .iter()
            .collect();
    assert!(orchestrator.write_to_file(&output_path).is_ok());
}
