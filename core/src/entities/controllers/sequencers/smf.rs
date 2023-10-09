// Copyright (c) 2023 Mike Tsao. All rights reserved.

#[cfg(obsolete)]
mod obsolete {
    /// [MidiSmfReader] parses MIDI SMF files and programs [MidiTickSequencer] with
    /// the data it finds.
    pub struct MidiSmfReader {}
    impl MidiSmfReader {
        pub fn program_sequencer(sequencer: &mut MidiTickSequencer, data: &[u8]) {
            let parse_result = midly::Smf::parse(data).unwrap();

            struct MetaInfo {
                // Pulses per quarter-note
                ppq: u32,

                // Microseconds per quarter-note
                tempo: u32,

                time_signature_numerator: u8,
                time_signature_denominator_exp: u8,
            }
            let mut meta_info = MetaInfo {
                ppq: match parse_result.header.timing {
                    midly::Timing::Metrical(ticks_per_beat) => ticks_per_beat.as_int() as u32,
                    _ => 0,
                },
                tempo: 0,

                // https://en.wikipedia.org/wiki/Time_signature
                time_signature_numerator: 0,
                time_signature_denominator_exp: 0,
            };
            for (track_number, track) in parse_result.tracks.iter().enumerate() {
                println!("Processing track {track_number}");
                let mut track_time_ticks: usize = 0; // The relative time references start over at zero with each track.

                for t in track.iter() {
                    match t.kind {
                        TrackEventKind::Midi { channel, message } => {
                            let delta = t.delta.as_int() as usize;
                            track_time_ticks += delta;
                            sequencer.insert(MidiTicks(track_time_ticks), channel.into(), message);
                            // TODO: prior version of this code treated vel=0 as
                            // note-off. Do we need to handle that higher up?
                        }

                        TrackEventKind::Meta(meta_message) => match meta_message {
                            midly::MetaMessage::TimeSignature(
                                numerator,
                                denominator_exp,
                                _cc,
                                _bb,
                            ) => {
                                meta_info.time_signature_numerator = numerator;
                                meta_info.time_signature_denominator_exp = denominator_exp;
                                //meta_info.ppq = cc; WHA???
                            }
                            midly::MetaMessage::Tempo(tempo) => {
                                meta_info.tempo = tempo.as_int();
                            }
                            midly::MetaMessage::TrackNumber(track_opt) => {
                                if track_opt.is_none() {
                                    continue;
                                }
                            }
                            midly::MetaMessage::EndOfTrack => {
                                let _time_signature: (u32, u32) = (
                                    meta_info.time_signature_numerator.into(),
                                    2_u32.pow(meta_info.time_signature_denominator_exp.into()),
                                );
                                let ticks_per_quarter_note: f32 = meta_info.ppq as f32;
                                let seconds_per_quarter_note: f32 =
                                    meta_info.tempo as f32 / 1000000.0;
                                let _ticks_per_second =
                                    ticks_per_quarter_note / seconds_per_quarter_note;

                                let _bpm: f32 = (60.0 * 1000000.0) / (meta_info.tempo as f32);

                                // sequencer.set_midi_ticks_per_second(ticks_per_second
                                // as usize);
                            }
                            _ => {}
                        },
                        TrackEventKind::SysEx(_data) => { // TODO
                        }
                        TrackEventKind::Escape(_data) => { // TODO
                        }
                    }
                }
            }
            println!("Done processing MIDI file");
        }
    }
}