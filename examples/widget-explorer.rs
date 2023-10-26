// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! The `widget-explorer` example is a sandbox for developing egui Ensnare
//! widgets.

use anyhow::anyhow;
use eframe::{
    egui::{
        self, warn_if_debug_build, CollapsingHeader, DragValue, Id, Layout, ScrollArea, Slider,
        Style, Ui,
    },
    emath::Align,
    epaint::{vec2, Galley},
    CreationContext,
};
use ensnare::{
    app_version,
    arrangement::{signal_chain, track_widget, TrackAction},
    entities::{
        controllers::{
            atlas, LivePatternSequencer, NoteSequencer, NoteSequencerBuilder, ToyController,
            ToyControllerParams,
        },
        effects::{ToyEffect, ToyEffectParams},
        instruments::{ToyInstrument, ToyInstrumentParams, ToySynth, ToySynthParams},
    },
    prelude::*,
    ui::{
        widgets::{audio, pattern, placeholder, timeline, track},
        CircularSampleBuffer, DragSource, DropTarget,
    },
};

#[derive(Debug)]
struct LegendSettings {
    hide: bool,
    range: std::ops::Range<MusicalTime>,
}
impl LegendSettings {
    const NAME: &'static str = "Legend";

    fn show(&mut self, ui: &mut Ui) {
        if !self.hide {
            ui.add(timeline::legend(&mut self.range));
        }
    }
}
impl Default for LegendSettings {
    fn default() -> Self {
        Self {
            hide: Default::default(),
            range: MusicalTime::START..MusicalTime::new_with_beats(128),
        }
    }
}
impl Displays for LegendSettings {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.checkbox(&mut self.hide, "Hide");
        ui.label("View range");
        let mut range_start = self.range.start.total_beats();
        let mut range_end = self.range.end.total_beats();
        let start_response = ui.add(Slider::new(&mut range_start, 0..=128));
        if start_response.changed() {
            self.range.start = MusicalTime::new_with_beats(range_start);
        };
        let end_response = ui.add(Slider::new(&mut range_end, 1..=256));
        if end_response.changed() {
            self.range.end = MusicalTime::new_with_beats(range_end);
        };
        start_response | end_response
    }
}

#[derive(Debug)]
struct TrackSettings {
    hide: bool,
    track: Track,
    range: std::ops::Range<MusicalTime>,
    view_range: std::ops::Range<MusicalTime>,
}
impl DisplaysInTimeline for TrackSettings {
    fn set_view_range(&mut self, view_range: &std::ops::Range<MusicalTime>) {
        self.track.set_view_range(view_range);
        self.view_range = view_range.clone();
    }
}
impl TrackSettings {
    const NAME: &'static str = "Track";

    fn show(&mut self, ui: &mut Ui) {
        if !self.hide {
            let mut action = None;
            ui.add(track_widget(
                TrackUid(1),
                &mut self.track,
                false,
                Some(MusicalTime::new_with_beats(1)),
                &mut action,
            ));
        }
    }
}
impl Default for TrackSettings {
    fn default() -> Self {
        let mut r = Self {
            hide: Default::default(),
            track: Track::default(),
            range: MusicalTime::START..MusicalTime::new_with_beats(128),
            view_range: MusicalTime::START..MusicalTime::new_with_beats(128),
        };
        let _ = r.track.append_entity(
            Box::new(NoteSequencerBuilder::default().build().unwrap()),
            Uid(345),
        );
        let _ = r.track.append_entity(
            Box::new(ControlAtlasBuilder::default().build().unwrap()),
            Uid(346),
        );
        r
    }
}
impl Displays for TrackSettings {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.checkbox(&mut self.hide, "Hide");
        if ui.button("Next").clicked() {
            self.track.select_next_foreground_timeline_entity();
        }
        ui.label("Range");
        let mut range_start = self.range.start.total_beats();
        let mut range_end = self.range.end.total_beats();
        let start_response = ui.add(Slider::new(&mut range_start, 0..=1024));
        if start_response.changed() {
            self.range.start = MusicalTime::new_with_beats(range_start);
        };
        let end_response = ui.add(Slider::new(&mut range_end, 0..=1024));
        if end_response.changed() {
            self.range.end = MusicalTime::new_with_beats(range_end);
        };
        start_response | end_response
    }
}

/// Wraps a PretendDevicePalette as a [Widget](eframe::egui::Widget).
fn pretend_device_palette(entity_factory: &EntityFactory) -> impl eframe::egui::Widget + '_ {
    move |ui: &mut eframe::egui::Ui| PretendDevicePalette::new(entity_factory).ui(ui)
}

#[derive(Debug)]
struct PretendDevicePalette<'a> {
    entity_factory: &'a EntityFactory,
}
impl<'a> PretendDevicePalette<'a> {
    fn new(entity_factory: &'a EntityFactory) -> Self {
        Self { entity_factory }
    }
}
impl<'a> Displays for PretendDevicePalette<'a> {
    fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let desired_size = vec2(ui.available_width(), 32.0);
        ui.allocate_ui(desired_size, |ui| {
            ScrollArea::horizontal()
                .show(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        for key in self.entity_factory.sorted_keys() {
                            DragDropManager::drag_source(
                                ui,
                                Id::new(key),
                                DragSource::NewDevice(key.clone()),
                                |ui| {
                                    ui.label(key.to_string());
                                },
                            );
                        }
                    })
                    .response
                })
                .inner
        })
        .response
    }
}

#[derive(Debug, Default)]
struct DevicePaletteSettings {
    hide: bool,
}
impl DevicePaletteSettings {
    const NAME: &'static str = "Device Palette";

    fn show(&mut self, ui: &mut Ui) {
        if !self.hide {
            ui.add(pretend_device_palette(EntityFactory::global()));
        }
    }
}
impl Displays for DevicePaletteSettings {
    fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}

#[derive(Debug, Default)]
struct SignalChainSettings {
    hide: bool,
    is_large_size: bool,
    track: Track,
}
impl SignalChainSettings {
    const NAME: &'static str = "Signal Chain";

    fn show(&mut self, ui: &mut Ui) {
        if !self.hide {
            ui.scope(|ui| {
                // TODO: who should own this value?
                ui.set_max_height(32.0);
                let mut action = None;
                ui.add(signal_chain(
                    TrackUid::default(),
                    &mut self.track,
                    &mut action,
                ));
                if action.is_some() {
                    todo!();
                }
            });
        }
    }

    pub fn append_entity(&mut self, entity: Box<dyn Entity>, uid: Uid) -> anyhow::Result<()> {
        self.track.append_entity(entity, uid)
    }
}
impl Displays for SignalChainSettings {
    fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.checkbox(&mut self.hide, "Hide") | ui.checkbox(&mut self.is_large_size, "Large size")
    }
}

#[derive(Debug)]
struct GridSettings {
    hide: bool,
    range: std::ops::Range<MusicalTime>,
    view_range: std::ops::Range<MusicalTime>,
}
impl GridSettings {
    const NAME: &'static str = "Grid";

    fn show(&mut self, ui: &mut Ui) {
        if !self.hide {
            ui.add(timeline::grid(self.range.clone(), self.view_range.clone()));
        }
    }
}
impl Default for GridSettings {
    fn default() -> Self {
        Self {
            hide: Default::default(),
            range: MusicalTime::START..MusicalTime::new_with_beats(128),
            view_range: MusicalTime::START..MusicalTime::new_with_beats(128),
        }
    }
}
impl DisplaysInTimeline for GridSettings {
    fn set_view_range(&mut self, view_range: &std::ops::Range<MusicalTime>) {
        self.view_range = view_range.clone();
    }
}
impl Displays for GridSettings {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}

#[derive(Debug)]
struct PatternIconSettings {
    hide: bool,
    duration: MusicalTime,
    notes: Vec<Note>,
    is_selected: bool,
}
impl Default for PatternIconSettings {
    fn default() -> Self {
        Self {
            hide: Default::default(),
            duration: MusicalTime::new_with_beats(4),
            notes: vec![
                Self::note(
                    MidiNote::C4,
                    MusicalTime::START,
                    MusicalTime::DURATION_WHOLE,
                ),
                Self::note(
                    MidiNote::G4,
                    MusicalTime::START + MusicalTime::DURATION_WHOLE,
                    MusicalTime::DURATION_WHOLE,
                ),
            ],
            is_selected: Default::default(),
        }
    }
}
impl Displays for PatternIconSettings {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.checkbox(&mut self.hide, "Hide Pattern Icon")
            | ui.checkbox(&mut self.is_selected, "Show selected")
    }
}
impl PatternIconSettings {
    const NAME: &'static str = "Pattern Icon";
    fn note(key: MidiNote, start: MusicalTime, duration: MusicalTime) -> Note {
        Note {
            key: key as u8,
            range: start..start + duration,
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        // Pattern Icon
        if !self.hide {
            DragDropManager::drag_source(
                ui,
                Id::new("pattern icon"),
                DragSource::Pattern(PatternUid(99)),
                |ui| {
                    ui.add(pattern::icon(self.duration, &self.notes, self.is_selected));
                },
            );
        }
    }
}

#[derive(Debug)]
struct ControlAtlasSettings {
    hide: bool,
    control_atlas: ControlAtlas,
    control_router: ControlRouter,
    view_range: std::ops::Range<MusicalTime>,
}
impl Default for ControlAtlasSettings {
    fn default() -> Self {
        Self {
            hide: Default::default(),
            control_atlas: ControlAtlasBuilder::default().random().build().unwrap(),
            control_router: Default::default(),
            view_range: Default::default(),
        }
    }
}
impl Displays for ControlAtlasSettings {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}
impl DisplaysInTimeline for ControlAtlasSettings {
    fn set_view_range(&mut self, view_range: &std::ops::Range<MusicalTime>) {
        self.view_range = view_range.clone();
    }
}
impl ControlAtlasSettings {
    const NAME: &'static str = "Control Atlas";

    fn show(&mut self, ui: &mut Ui) {
        if !self.hide {
            let mut action = None;
            ui.add(atlas(
                &mut self.control_atlas,
                &mut self.control_router,
                self.view_range.clone(),
                &mut action,
            ));
        }
    }
}

#[derive(Debug, Default)]
struct LivePatternSequencerSettings {
    hide: bool,
    sequencer: LivePatternSequencer,
}
impl Displays for LivePatternSequencerSettings {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}
impl DisplaysInTimeline for LivePatternSequencerSettings {
    fn set_view_range(&mut self, view_range: &std::ops::Range<MusicalTime>) {
        self.sequencer.set_view_range(view_range);
    }
}
impl LivePatternSequencerSettings {
    const NAME: &'static str = "Live Pattern Sequencer";

    fn show(&mut self, ui: &mut Ui) {
        if !self.hide {
            self.sequencer.ui(ui);
        }
    }
}

#[derive(Debug)]
struct NoteSequencerSettings {
    hide: bool,
    sequencer: NoteSequencer,
    view_range: std::ops::Range<MusicalTime>,
}
impl Default for NoteSequencerSettings {
    fn default() -> Self {
        Self {
            hide: Default::default(),
            sequencer: NoteSequencerBuilder::default()
                .random(MusicalTime::START..MusicalTime::new_with_beats(128))
                .build()
                .unwrap(),
            view_range: Default::default(),
        }
    }
}

impl Displays for NoteSequencerSettings {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}
impl DisplaysInTimeline for NoteSequencerSettings {
    fn set_view_range(&mut self, view_range: &std::ops::Range<MusicalTime>) {
        self.view_range = view_range.clone();
    }
}
impl NoteSequencerSettings {
    const NAME: &'static str = "Note Sequencer";

    fn show(&mut self, ui: &mut Ui) {
        if !self.hide {
            self.sequencer.ui(ui);
        }
    }
}

#[derive(Debug)]
struct ToySynthSettings {
    hide: bool,
    toy_synth: ToySynth,
}
impl Default for ToySynthSettings {
    fn default() -> Self {
        Self {
            hide: Default::default(),
            toy_synth: ToySynth::new_with(Uid::default(), &ToySynthParams::default()),
        }
    }
}
impl Displays for ToySynthSettings {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}
impl ToySynthSettings {
    const NAME: &'static str = "Toy Synth";

    fn show(&mut self, ui: &mut Ui) {
        if !self.hide {
            self.toy_synth.ui(ui);
        }
    }
}

#[derive(Debug)]
struct ToyControllerSettings {
    hide: bool,
    toy: ToyController,
}
impl Default for ToyControllerSettings {
    fn default() -> Self {
        Self {
            hide: Default::default(),
            toy: ToyController::new_with(
                Uid::default(),
                &ToyControllerParams::default(),
                MidiChannel::default(),
            ),
        }
    }
}
impl Displays for ToyControllerSettings {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}
impl ToyControllerSettings {
    const NAME: &'static str = "Toy Controller";

    fn show(&mut self, ui: &mut Ui) {
        if !self.hide {
            self.toy.ui(ui);
        }
    }
}

#[derive(Debug)]
struct ToyEffectSettings {
    hide: bool,
    toy: ToyEffect,
}
impl Default for ToyEffectSettings {
    fn default() -> Self {
        Self {
            hide: Default::default(),
            toy: ToyEffect::new_with(Uid::default(), &ToyEffectParams::default()),
        }
    }
}
impl Displays for ToyEffectSettings {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}
impl ToyEffectSettings {
    const NAME: &'static str = "Toy Effect";

    fn show(&mut self, ui: &mut Ui) {
        if !self.hide {
            self.toy.ui(ui);
        }
    }
}

#[derive(Debug)]
struct ToyInstrumentSettings {
    hide: bool,
    toy: ToyInstrument,
}
impl Default for ToyInstrumentSettings {
    fn default() -> Self {
        Self {
            hide: Default::default(),
            toy: ToyInstrument::new_with(Uid::default(), &ToyInstrumentParams::default()),
        }
    }
}
impl Displays for ToyInstrumentSettings {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}
impl ToyInstrumentSettings {
    const NAME: &'static str = "Toy Instrument";

    fn show(&mut self, ui: &mut Ui) {
        if !self.hide {
            self.toy.ui(ui);
        }
    }
}

#[derive(Debug, Default)]
struct TitleBarSettings {
    hide: bool,
    title: TrackTitle,
    font_galley: Option<std::sync::Arc<Galley>>,
}

impl Displays for TitleBarSettings {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        if self.font_galley.is_none() {
            self.font_galley = Some(track::make_title_bar_galley(ui, &self.title));
        }
        ui.checkbox(&mut self.hide, "Hide");
        let response = ui.text_edit_singleline(&mut self.title.0);
        if response.changed() {
            self.font_galley = None;
        }
        response
    }
}
impl TitleBarSettings {
    const NAME: &'static str = "Title Bar";

    fn show(&mut self, ui: &mut Ui) {
        if !self.hide {
            if let Some(font_galley) = &self.font_galley {
                ui.add(track::title_bar(Some(std::sync::Arc::clone(font_galley))));
            }
        }
    }
}

#[derive(Debug, Default)]
struct PianoRollSettings {
    hide: bool,
    piano_roll: PianoRoll,
}
impl Displays for PianoRollSettings {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}
impl PianoRollSettings {
    const NAME: &'static str = "Piano Roll";

    fn show(&mut self, ui: &mut Ui) {
        if !self.hide {
            self.piano_roll.ui(ui);
        }
    }
}

#[derive(Debug, Default)]
struct WigglerSettings {
    hide: bool,
}

impl Displays for WigglerSettings {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
    }
}
impl WigglerSettings {
    const NAME: &'static str = "Wiggler";

    fn show(&mut self, ui: &mut Ui) {
        if !self.hide {
            ui.add(placeholder::wiggler());
        }
    }
}

#[derive(Debug)]
struct TimeDomainSettings {
    hide: bool,
    max_width: f32,
    max_height: f32,
    buffer: CircularSampleBuffer,
}
impl Default for TimeDomainSettings {
    fn default() -> Self {
        Self {
            hide: Default::default(),
            max_width: 128.0,
            max_height: 64.0,
            buffer: CircularSampleBuffer::new(256),
        }
    }
}

impl Displays for TimeDomainSettings {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
            | ui.add(DragValue::new(&mut self.max_width).prefix("width: "))
            | ui.add(DragValue::new(&mut self.max_height).prefix("height: "))
    }
}
impl TimeDomainSettings {
    const NAME: &'static str = "Audio Time Domain";

    fn show(&mut self, ui: &mut Ui) {
        self.buffer.add_some_noise();
        if !self.hide {
            ui.scope(|ui| {
                ui.set_max_width(self.max_width);
                ui.set_max_height(self.max_height);
                let (buffer, cursor) = self.buffer.get();
                ui.add(audio::time_domain(buffer, cursor));
            });
        }
    }
}

#[derive(Debug)]
struct FrequencyDomainSettings {
    hide: bool,
    max_width: f32,
    max_height: f32,
    buffer: CircularSampleBuffer,

    fft_calc_counter: u8, // Used to test occasional recomputation of FFT
    fft_buffer: Vec<f32>,
}
impl Default for FrequencyDomainSettings {
    fn default() -> Self {
        Self {
            hide: Default::default(),
            max_width: 128.0,
            max_height: 64.0,
            buffer: CircularSampleBuffer::new(256),
            fft_calc_counter: Default::default(),
            fft_buffer: Default::default(),
        }
    }
}
impl Displays for FrequencyDomainSettings {
    fn ui(&mut self, ui: &mut Ui) -> egui::Response {
        ui.checkbox(&mut self.hide, "Hide")
            | ui.add(DragValue::new(&mut self.max_width).prefix("width: "))
            | ui.add(DragValue::new(&mut self.max_height).prefix("height: "))
    }
}
impl FrequencyDomainSettings {
    const NAME: &'static str = "Audio Frequency Domain";

    fn show(&mut self, ui: &mut Ui) {
        self.buffer.add_some_noise();

        // We act on 0 so that it's always initialized by the time we get to the
        // end of this method.
        if self.fft_calc_counter == 0 {
            self.fft_buffer = self.buffer.analyze_spectrum().unwrap();
        }
        self.fft_calc_counter += 1;
        if self.fft_calc_counter > 4 {
            self.fft_calc_counter = 0;
        }
        if !self.hide {
            ui.scope(|ui| {
                ui.set_max_width(self.max_width);
                ui.set_max_height(self.max_height);
                ui.add(audio::frequency_domain(&self.fft_buffer));
            });
        }
    }
}

#[derive(Debug)]
struct SampleClip([Sample; 256]);
impl Default for SampleClip {
    fn default() -> Self {
        Self(audio::init_random_samples())
    }
}

#[derive(Debug, Default)]
struct WidgetExplorer {
    legend: LegendSettings,
    grid: GridSettings,
    track_widget: TrackSettings,
    device_palette: DevicePaletteSettings,
    signal_chain: SignalChainSettings,
    control_atlas: ControlAtlasSettings,
    live_pattern_sequencer: LivePatternSequencerSettings,
    note_sequencer: NoteSequencerSettings,
    pattern_icon: PatternIconSettings,
    title_bar: TitleBarSettings,
    piano_roll: PianoRollSettings,
    wiggler: WigglerSettings,
    time_domain: TimeDomainSettings,
    frequency_domain: FrequencyDomainSettings,
    toy_synth: ToySynthSettings,
    toy_controller: ToyControllerSettings,
    toy_effect: ToyEffectSettings,
    toy_instrument: ToyInstrumentSettings,
}
impl WidgetExplorer {
    pub const NAME: &'static str = "Widget Explorer";

    pub fn new(_cc: &CreationContext) -> Self {
        Self {
            ..Default::default()
        }
    }

    fn show_bottom(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            warn_if_debug_build(ui);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(app_version())
            });
        });
    }

    fn show_left(&mut self, ui: &mut Ui) {
        ScrollArea::horizontal().show(ui, |ui| {
            Self::wrap_settings(TimeDomainSettings::NAME, ui, |ui| self.time_domain.ui(ui));
            Self::wrap_settings(FrequencyDomainSettings::NAME, ui, |ui| {
                self.frequency_domain.ui(ui)
            });
            Self::wrap_settings(LegendSettings::NAME, ui, |ui| self.legend.ui(ui));
            Self::wrap_settings(TrackSettings::NAME, ui, |ui| self.track_widget.ui(ui));
            Self::wrap_settings(DevicePaletteSettings::NAME, ui, |ui| {
                self.device_palette.ui(ui)
            });
            Self::wrap_settings(SignalChainSettings::NAME, ui, |ui| self.signal_chain.ui(ui));
            Self::wrap_settings(PianoRollSettings::NAME, ui, |ui| self.piano_roll.ui(ui));
            Self::wrap_settings(GridSettings::NAME, ui, |ui| self.grid.ui(ui));
            Self::wrap_settings(PatternIconSettings::NAME, ui, |ui| self.pattern_icon.ui(ui));
            Self::wrap_settings(ControlAtlasSettings::NAME, ui, |ui| {
                self.control_atlas.ui(ui)
            });
            Self::wrap_settings(LivePatternSequencerSettings::NAME, ui, |ui| {
                self.live_pattern_sequencer.ui(ui)
            });
            Self::wrap_settings(NoteSequencerSettings::NAME, ui, |ui| {
                self.note_sequencer.ui(ui)
            });

            Self::wrap_settings(ToySynthSettings::NAME, ui, |ui| self.toy_synth.ui(ui));
            Self::wrap_settings(ToyControllerSettings::NAME, ui, |ui| {
                self.toy_controller.ui(ui)
            });
            Self::wrap_settings(ToyEffectSettings::NAME, ui, |ui| self.toy_effect.ui(ui));
            Self::wrap_settings(ToyInstrumentSettings::NAME, ui, |ui| {
                self.toy_instrument.ui(ui)
            });

            Self::wrap_settings(TitleBarSettings::NAME, ui, |ui| self.title_bar.ui(ui));
            Self::wrap_settings(WigglerSettings::NAME, ui, |ui| self.wiggler.ui(ui));
            self.debug_ui(ui);
        });
    }

    fn wrap_settings(
        name: &str,
        ui: &mut Ui,
        add_body: impl FnOnce(&mut Ui) -> eframe::egui::Response,
    ) {
        CollapsingHeader::new(name)
            .show_background(true)
            .show_unindented(ui, add_body);
    }

    fn wrap_item(name: &str, ui: &mut Ui, add_body: impl FnOnce(&mut Ui)) {
        ui.heading(name);
        add_body(ui);
        ui.separator();
    }

    fn debug_ui(&mut self, ui: &mut Ui) {
        #[cfg(debug_assertions)]
        {
            let mut debug_on_hover = ui.ctx().debug_on_hover();
            ui.checkbox(&mut debug_on_hover, "🐛 Debug on hover")
                .on_hover_text("Show structure of the ui when you hover with the mouse");
            ui.ctx().set_debug_on_hover(debug_on_hover);
        }
        let style: Style = (*ui.ctx().style()).clone();
        let new_visuals = style.visuals.light_dark_small_toggle_button(ui);
        if let Some(visuals) = new_visuals {
            ui.ctx().set_visuals(visuals);
        }
    }

    fn show_center(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            self.track_widget.set_view_range(&self.legend.range);
            self.control_atlas.set_view_range(&self.legend.range);
            self.grid.set_view_range(&self.legend.range);
            self.live_pattern_sequencer
                .set_view_range(&self.legend.range);
            self.note_sequencer.set_view_range(&self.legend.range);

            ui.horizontal_top(|ui| {
                ui.scope(|ui| {
                    ui.set_max_height(64.0);
                    Self::wrap_item(TimeDomainSettings::NAME, ui, |ui| self.time_domain.show(ui));
                    Self::wrap_item(FrequencyDomainSettings::NAME, ui, |ui| {
                        self.frequency_domain.show(ui)
                    });
                });
            });
            ui.heading("Timeline");
            self.legend.show(ui);
            self.track_widget.show(ui);

            Self::wrap_item(DevicePaletteSettings::NAME, ui, |ui| {
                self.device_palette.show(ui)
            });
            Self::wrap_item(SignalChainSettings::NAME, ui, |ui| {
                self.signal_chain.show(ui)
            });
            Self::wrap_item(PianoRollSettings::NAME, ui, |ui| self.piano_roll.show(ui));

            Self::wrap_item(GridSettings::NAME, ui, |ui| self.grid.show(ui));
            Self::wrap_item(PatternIconSettings::NAME, ui, |ui| {
                self.pattern_icon.show(ui)
            });
            Self::wrap_item(ControlAtlasSettings::NAME, ui, |ui| {
                self.control_atlas.show(ui)
            });
            Self::wrap_item(LivePatternSequencerSettings::NAME, ui, |ui| {
                self.live_pattern_sequencer.show(ui)
            });
            Self::wrap_item(NoteSequencerSettings::NAME, ui, |ui| {
                self.note_sequencer.show(ui)
            });

            Self::wrap_item(ToySynthSettings::NAME, ui, |ui| self.toy_synth.show(ui));
            Self::wrap_item(ToyControllerSettings::NAME, ui, |ui| {
                self.toy_controller.show(ui)
            });
            Self::wrap_item(ToyEffectSettings::NAME, ui, |ui| self.toy_effect.show(ui));
            Self::wrap_item(ToyInstrumentSettings::NAME, ui, |ui| {
                self.toy_instrument.show(ui)
            });

            Self::wrap_item(TitleBarSettings::NAME, ui, |ui| self.title_bar.show(ui));
            Self::wrap_item(WigglerSettings::NAME, ui, |ui| self.wiggler.show(ui));
        });
    }
}
impl eframe::App for WidgetExplorer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let bottom = egui::TopBottomPanel::bottom("bottom-panel")
            .resizable(false)
            .exact_height(24.0);
        let left = egui::SidePanel::left("left-panel")
            .resizable(true)
            .default_width(160.0)
            .width_range(160.0..=480.0);
        let center = egui::CentralPanel::default();

        bottom.show(ctx, |ui| {
            self.show_bottom(ui);
        });
        left.show(ctx, |ui| {
            self.show_left(ui);
        });
        center.show(ctx, |ui| {
            self.show_center(ui);
        });

        // TODO: this is bad design because it does non-GUI processing during
        // the update() method. It's OK here because this is a widget explorer,
        // not a time-critical app.
        if let Some(action) = self.signal_chain.track.take_action() {
            match action {
                TrackAction::NewDevice(key) => {
                    eprintln!("SignalChainAction::NewDevice({key})");
                    if let Some(entity) = EntityFactory::global().new_entity(&key, Uid::default()) {
                        let _ = self.signal_chain.append_entity(entity, Uid(345698));
                    }
                }
                TrackAction::LinkControl(_source_uid, _target_uid, _index) => {
                    eprintln!("{action:?}");
                }
                TrackAction::EntitySelected(uid) => {
                    eprintln!("we should show entity {uid}");
                }
            }
        }
        if let Some((source, target)) = DragDropManager::check_and_clear_drop_event() {
            match source {
                DragSource::NewDevice(_) => todo!(),
                _ => {}
            }
            match target {
                DropTarget::Controllable(_, _) => todo!(),
                _ => {}
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1366.0, 768.0)),
        ..Default::default()
    };

    if EntityFactory::initialize(register_factory_entities(EntityFactory::default())).is_err() {
        return Err(anyhow!("Couldn't initialize EntityFactory"));
    }
    if DragDropManager::initialize(DragDropManager::default()).is_err() {
        return Err(anyhow!("Couldn't set DragDropManager once_cell"));
    }

    if let Err(e) = eframe::run_native(
        WidgetExplorer::NAME,
        options,
        Box::new(|cc| Box::new(WidgetExplorer::new(cc))),
    ) {
        Err(anyhow!("eframe::run_native(): {:?}", e))
    } else {
        Ok(())
    }
}