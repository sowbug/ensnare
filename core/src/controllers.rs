// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{control::ControlRouter, prelude::*, rng::Rng, traits::prelude::*};
use derive_builder::Builder;
use eframe::{
    egui::{self, Sense},
    emath::RectTransform,
    epaint::{pos2, Color32, Rect, Stroke},
};
use ensnare_proc_macros::{Control, IsController, IsControllerWithTimelineDisplay, Metadata};
use serde::{Deserialize, Serialize};
use strum_macros::Display;

pub use crate::entities::controllers::sequencers::{
    LivePatternSequencer, MidiSequencer, NoteSequencer, PatternSequencer,
};

/// [Timer] runs for a specified amount of time, then indicates that it's done.
/// It is useful when you need something to happen after a certain amount of
/// wall-clock time, rather than musical time.
#[derive(Debug, Default, Control, IsController, Metadata, Serialize, Deserialize)]
pub struct Timer {
    uid: Uid,

    duration: MusicalTime,

    #[serde(skip)]
    is_performing: bool,

    #[serde(skip)]
    is_finished: bool,

    #[serde(skip)]
    end_time: Option<MusicalTime>,
}
impl Serializable for Timer {}
#[allow(missing_docs)]
impl Timer {
    pub fn new_with(duration: MusicalTime) -> Self {
        Self {
            duration,
            ..Default::default()
        }
    }

    pub fn duration(&self) -> MusicalTime {
        self.duration
    }

    pub fn set_duration(&mut self, duration: MusicalTime) {
        self.duration = duration;
    }
}
impl HandlesMidi for Timer {}
impl Configurable for Timer {}
impl Controls for Timer {
    fn update_time(&mut self, range: &std::ops::Range<MusicalTime>) {
        if self.is_performing {
            if self.duration == MusicalTime::default() {
                // Zero-length timers fire immediately.
                self.is_finished = true;
            } else if let Some(end_time) = self.end_time {
                if range.contains(&end_time) {
                    self.is_finished = true;
                }
            } else {
                // The first time we're called with an update_time() while
                // performing, we take that as the start of the timer.
                self.end_time = Some(range.start + self.duration);
            }
        }
    }

    fn is_finished(&self) -> bool {
        self.is_finished
    }

    fn play(&mut self) {
        self.is_performing = true;
    }

    fn stop(&mut self) {
        self.is_performing = false;
    }

    fn is_performing(&self) -> bool {
        self.is_performing
    }
}
impl Displays for Timer {}

// TODO: needs tests!
/// [Trigger] issues a control signal after a specified amount of time.
#[derive(Debug, Control, IsController, Metadata, Serialize, Deserialize)]
pub struct Trigger {
    uid: Uid,

    timer: Timer,

    pub value: ControlValue,

    has_triggered: bool,
    is_performing: bool,
}
impl Serializable for Trigger {}
impl Controls for Trigger {
    fn update_time(&mut self, range: &std::ops::Range<MusicalTime>) {
        self.timer.update_time(range)
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        if self.timer.is_finished() && self.is_performing && !self.has_triggered {
            self.has_triggered = true;
            control_events_fn(None, EntityEvent::Control(self.value));
        }
    }

    fn is_finished(&self) -> bool {
        self.timer.is_finished()
    }

    fn play(&mut self) {
        self.is_performing = true;
        self.timer.play();
    }

    fn stop(&mut self) {
        self.is_performing = false;
        self.timer.stop();
    }

    fn skip_to_start(&mut self) {
        self.has_triggered = false;
        self.timer.skip_to_start();
    }

    fn is_performing(&self) -> bool {
        self.is_performing
    }
}
impl Configurable for Trigger {
    fn sample_rate(&self) -> SampleRate {
        self.timer.sample_rate()
    }
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.timer.update_sample_rate(sample_rate)
    }
}
impl HandlesMidi for Trigger {}
impl Trigger {
    pub fn new_with(timer: Timer, value: ControlValue) -> Self {
        Self {
            uid: Default::default(),
            timer,
            value,
            has_triggered: Default::default(),
            is_performing: Default::default(),
        }
    }

    pub fn set_value(&mut self, value: ControlValue) {
        self.value = value;
    }
}
impl Displays for Trigger {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label(self.name())
    }
}

impl ControlTripBuilder {
    /// Generates a random [ControlTrip]. For development/prototyping only.
    pub fn random(&mut self, start: MusicalTime) -> &mut Self {
        let mut rng = Rng::default();

        let mut pos = start;
        for _ in 0..rng.0.rand_range(5..8) {
            self.step(
                ControlStepBuilder::default()
                    .time(pos)
                    .path(ControlTripPath::Flat)
                    .value(ControlValue(rng.0.rand_float()))
                    .build()
                    .unwrap(),
            );
            pos += MusicalTime::new_with_beats(rng.0.rand_range(4..12) as usize);
        }
        self
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
/// Specifies what a [ControlStep]'s path should look like.
pub enum ControlTripPath {
    /// No path. This step's value should be ignored.
    #[default]
    None,
    /// Stairstep. The path should be level at the [ControlStep]'s value.
    Flat,
    /// Linear. Straight line from this [ControlStep]'s value to the next one.
    Linear,
    /// Curved. Starts out changing quickly and ends up changing slowly.
    Logarithmic,
    /// Curved. Starts out changing slowly and ends up changing quickly.
    Exponential,
}
impl ControlTripPath {
    /// Returns the next enum, wrapping to zero if needed.
    pub fn next(&self) -> Self {
        match self {
            ControlTripPath::None => ControlTripPath::None,
            ControlTripPath::Flat => ControlTripPath::Linear,
            ControlTripPath::Linear => ControlTripPath::Flat,
            ControlTripPath::Logarithmic => ControlTripPath::Logarithmic,
            ControlTripPath::Exponential => ControlTripPath::Exponential,
        }
    }
}

/// Parts of [ControlTrip] that shouldn't be serialized.
#[derive(Clone, Debug)]
pub struct ControlTripEphemerals {
    /// The time range for this work slice. This is a copy of the value passed
    /// in Controls::update_time().
    range: std::ops::Range<MusicalTime>,

    /// The GUI view's time range.
    view_range: std::ops::Range<MusicalTime>,

    /// Which step we're currently processing.
    current_step: usize,
    /// The type of path we should be following.
    current_path: ControlTripPath,
    /// The range of values for the current step.
    value_range: std::ops::RangeInclusive<ControlValue>,
    /// The timespan of the current step.
    time_range: std::ops::Range<MusicalTime>,

    /// The value that we last issued as a Control event. We keep track of this
    /// to avoid issuing consecutive identical events.
    last_published_value: f64,

    /// Whether the current_step working variables are an unknown state --
    /// either just-initialized, or the work cursor is jumping to an earlier
    /// position.
    is_current_step_clean: bool,
}
impl Default for ControlTripEphemerals {
    fn default() -> Self {
        Self {
            range: Default::default(),
            view_range: Default::default(),
            current_step: Default::default(),
            current_path: Default::default(),
            value_range: ControlValue::default()..=ControlValue::default(),
            time_range: MusicalTime::empty_range(),
            last_published_value: Default::default(),
            is_current_step_clean: Default::default(),
        }
    }
}
impl ControlTripEphemerals {
    fn reset_current_path_if_needed(&mut self) {
        if !self.is_current_step_clean {
            self.is_current_step_clean = true;
            self.current_step = Default::default();
            self.current_path = Default::default();
            self.value_range = ControlValue::default()..=ControlValue::default();
            self.time_range = MusicalTime::empty_range();
        }
    }
}

/// A [ControlTrip] is a single track of automation. It can run as long as the
/// whole song.
///
/// A trip consists of [ControlStep]s ordered by time. Each step specifies a
/// point in time, a [ControlValue], and a [ControlPath] that indicates how to
/// progress from the current [ControlStep] to the next one.
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    IsControllerWithTimelineDisplay,
    Metadata,
    Builder,
)]
pub struct ControlTrip {
    #[builder(default)]
    uid: Uid,

    /// The [ControlStep]s that make up this trip. They must be in ascending
    /// time order. TODO: enforce that.
    #[builder(default, setter(each(name = "step", into)))]
    steps: Vec<ControlStep>,

    #[builder(setter(skip))]
    #[serde(skip)]
    e: ControlTripEphemerals,
}
impl ControlTrip {
    fn update_interval(&mut self) {
        self.e.reset_current_path_if_needed();

        // Are we in the middle of handling a step?
        if self.e.time_range.contains(&self.e.range.start) {
            // Yes; all the work is configured. Let's return so we can do it.
            return;
        }

        // The current step does not contain the current work slice. Find one that does.
        match self.steps.len() {
            0 => {
                // Empty trip. Mark that we don't have a path. This is a
                // terminal state.
                self.e.current_path = ControlTripPath::None;
            }
            1 => {
                // This trip has only one step, indicating that we should stay
                // level at its value.
                let step = &self.steps[0];
                self.e.current_path = ControlTripPath::Flat;
                self.e.value_range = step.value..=step.value;

                // Mark the time range to include all time so that we'll
                // early-exit this method in future calls.
                self.e.time_range = MusicalTime::START..MusicalTime::TIME_MAX;
            }
            _ => {
                // We have multiple steps. Find the one that corresponds to the
                // current work slice. Start with the current step, build a
                // range from it, and see whether it fits.

                let (mut end_time, mut end_value) = if self.e.current_step == 0 {
                    (MusicalTime::START, self.steps[0].value)
                } else {
                    (
                        self.steps[self.e.current_step - 1].time,
                        self.steps[self.e.current_step - 1].value,
                    )
                };
                loop {
                    let is_last = self.e.current_step == self.steps.len() - 1;
                    let step = &self.steps[self.e.current_step];
                    let next_step = if !is_last {
                        self.steps[self.e.current_step + 1].clone()
                    } else {
                        ControlStep {
                            value: step.value,
                            time: MusicalTime::TIME_MAX,
                            path: ControlTripPath::Flat,
                        }
                    };
                    let start_time = end_time;
                    let start_value = end_value;
                    (end_time, end_value) = (next_step.time, next_step.value);

                    // Build the range. Is it the right one?
                    let step_time_range = start_time..end_time;
                    if step_time_range.contains(&self.e.range.start) {
                        // Yes, this range contains the current work slice. Set
                        // it up, and get out of here.
                        self.e.current_path = step.path;
                        self.e.time_range = step_time_range;
                        self.e.value_range = match step.path {
                            ControlTripPath::None => todo!(),
                            ControlTripPath::Flat => start_value..=start_value,
                            ControlTripPath::Linear => start_value..=end_value,
                            ControlTripPath::Logarithmic => todo!(),
                            ControlTripPath::Exponential => todo!(),
                        };
                        break;
                    } else {
                        // No. Continue searching.
                        debug_assert!(
                            !is_last,
                            "Something is wrong. The last step's time range should be endless."
                        );
                        self.e.current_step += 1;
                    }
                }
            }
        }
    }

    pub(crate) fn steps(&mut self) -> &[ControlStep] {
        &self.steps
    }

    pub(crate) fn steps_mut(&mut self) -> &mut Vec<ControlStep> {
        &mut self.steps
    }

    pub fn uid(&self) -> Uid {
        self.uid
    }
}
impl DisplaysInTimeline for ControlTrip {
    fn set_view_range(&mut self, view_range: &std::ops::Range<MusicalTime>) {
        self.e.view_range = view_range.clone();
    }
}
impl Displays for ControlTrip {
    fn ui(&mut self, _ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        unimplemented!("Use the trip widget rather than calling this directly")
    }
}
impl HandlesMidi for ControlTrip {}
impl Controls for ControlTrip {
    fn update_time(&mut self, range: &std::ops::Range<MusicalTime>) {
        if range.start < self.e.range.start {
            // The cursor is jumping around. Mark things dirty.
            self.e.is_current_step_clean = false;
        }
        self.e.range = range.clone();
        self.update_interval();
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        // If we have no current path, then we're all done.
        if matches!(self.e.current_path, ControlTripPath::None) {
            return;
        }
        if self.e.range.start >= self.e.time_range.end
            || self.e.range.end <= self.e.time_range.start
        {
            self.update_interval();
        }
        let current_point = self.e.range.start.total_units() as f64;
        let start = self.e.time_range.start.total_units() as f64;
        let end = self.e.time_range.end.total_units() as f64;
        let duration = end - start;
        let current_point = current_point - start;
        let percentage = if duration > 0.0 {
            current_point / duration
        } else {
            0.0
        };
        let current_value = self.e.value_range.start().0
            + percentage * (self.e.value_range.end().0 - self.e.value_range.start().0);
        if current_value != self.e.last_published_value {
            self.e.last_published_value = current_value;
            control_events_fn(
                Some(self.uid),
                EntityEvent::Control(ControlValue::from(current_value)),
            );
        }
    }

    fn is_finished(&self) -> bool {
        matches!(self.e.current_path, ControlTripPath::None)
            || self.e.current_step + 1 == self.steps.len()
    }

    fn play(&mut self) {
        self.update_interval();
    }

    fn stop(&mut self) {}

    fn skip_to_start(&mut self) {
        self.e.reset_current_path_if_needed();
    }

    fn is_performing(&self) -> bool {
        false
    }
}
impl Configurable for ControlTrip {}
impl Serializable for ControlTrip {}

/// Describes a step of a [ControlTrip]. A [ControlStep] has a starting value as
/// of the specified time, and a [ControlPath] that specifies how to get from
/// the current value to the next [ControlStep]'s value.
///
/// If the first [ControlStep] in a [ControlTrip] does not start at
/// MusicalTime::START, then we synthesize a flat path, at this step's value,
/// from time zero to this step's time. Likewise, the last [ControlStep] in a
/// [ControlTrip] is always flat until MusicalTime::MAX.
#[derive(Serialize, Deserialize, Debug, Default, Builder, Clone)]
pub struct ControlStep {
    /// The initial value of this step.
    pub value: ControlValue,
    /// When this step begins.
    pub time: MusicalTime,
    /// How the step should progress to the next step. If this step is the last
    /// in a trip, then it's ControlPath::Flat.
    pub path: ControlTripPath,
}

// /// Wraps a [ControlAtlasWidget] as a [Widget](eframe::egui::Widget).
// pub fn atlas<'a>(
//     control_router: &'a mut ControlRouter,
//     view_range: std::ops::Range<MusicalTime>,
//     action: &'a mut Option<ControlAtlasWidgetAction>,
// ) -> impl eframe::egui::Widget + 'a {
//     move |ui: &mut eframe::egui::Ui| {
//         ControlAtlasWidget::new(control_atlas, control_router, view_range, action).ui(ui)
//     }
// }

#[derive(Debug, Display)]
pub enum ControlAtlasWidgetAction {
    AddTrip,
}

// #[derive(Debug)]
// struct ControlAtlasWidget<'a> {
//     control_router: &'a mut ControlRouter,
//     view_range: std::ops::Range<MusicalTime>,
//     action: &'a mut Option<ControlAtlasWidgetAction>,
// }
// impl<'a> ControlAtlasWidget<'a> {
//     fn new(
//         control_router: &'a mut ControlRouter,
//         view_range: std::ops::Range<MusicalTime>,
//         action: &'a mut Option<ControlAtlasWidgetAction>,
//     ) -> Self {
//         Self {
//             control_router,
//             view_range,
//             action,
//         }
//     }
// }
// impl<'a> DisplaysInTimeline for ControlAtlasWidget<'a> {
//     fn set_view_range(&mut self, view_range: &std::ops::Range<MusicalTime>) {
//         self.view_range = view_range.clone();
//     }
// }
// impl<'a> Displays for ControlAtlasWidget<'a> {
//     fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
//         // This push_id() was needed to avoid an ID conflict. I think it is
//         // because we're drawing widgets on top of each other, but I'm honestly
//         // not sure.
//         ui.push_id(ui.next_auto_id(), |ui| {
//             let (_id, rect) = ui.allocate_space(vec2(ui.available_width(), 64.0));
//             let response = ui
//                 .allocate_ui_at_rect(rect, |ui| {
//                     let mut remove_uid = None;
//                     self.control_atlas.trips_mut().iter_mut().for_each(|t| {
//                         ui.allocate_ui_at_rect(rect, |ui| {
//                             ui.add(trip(t, self.control_router, self.view_range.clone()));

//                             // Draw the trip controls.
//                             if ui.is_enabled() {
//                                 // TODO: I don't know why this isn't flush with
//                                 // the right side of the component.
//                                 let controls_rect = Rect::from_points(&[
//                                     rect.right_top(),
//                                     pos2(
//                                         rect.right()
//                                             - ui.ctx().style().spacing.interact_size.x * 2.0,
//                                         rect.top(),
//                                     ),
//                                 ]);
//                                 ui.allocate_ui_at_rect(controls_rect, |ui| {
//                                     ui.allocate_ui_with_layout(
//                                         ui.available_size(),
//                                         Layout::right_to_left(eframe::emath::Align::Center),
//                                         |ui| {
//                                             if ui.button("x").clicked() {
//                                                 remove_uid = Some(t.uid());
//                                             }
//                                             // TODO: this will be what you drag
//                                             // to things you want this trip to
//                                             // control
//                                             DragDropManager::drag_source(
//                                                 ui,
//                                                 ui.next_auto_id(),
//                                                 DragSource::ControlTrip(t.uid()),
//                                                 |ui| {
//                                                     ui.label("S");
//                                                 },
//                                             );
//                                         },
//                                     );
//                                 });
//                             }
//                         });
//                     });
//                     if let Some(uid) = remove_uid {
//                         self.control_atlas.remove_trip(uid);
//                     }
//                 })
//                 .response;
//             if ui.is_enabled() {
//                 response.context_menu(|ui| {
//                     if ui.button("Add trip").clicked() {
//                         ui.close_menu();
//                         *self.action = Some(ControlAtlasWidgetAction::AddTrip);
//                     }
//                 })
//             } else {
//                 response
//             }
//         })
//         .inner
//     }
// }

/// Wraps a [ControlTrip] as a [Widget](eframe::egui::Widget).
pub fn trip<'a>(
    trip: &'a mut ControlTrip,
    control_router: &'a mut ControlRouter,
    view_range: std::ops::Range<MusicalTime>,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| Trip::new(trip, control_router, view_range).ui(ui)
}

#[derive(Debug)]
struct Trip<'a> {
    control_trip: &'a mut ControlTrip,
    control_router: &'a mut ControlRouter,
    view_range: std::ops::Range<MusicalTime>,
}
impl<'a> Trip<'a> {
    fn new(
        control_trip: &'a mut ControlTrip,
        control_router: &'a mut ControlRouter,
        view_range: std::ops::Range<MusicalTime>,
    ) -> Self {
        Self {
            control_trip,
            control_router,
            view_range,
        }
    }
}
impl<'a> Displays for Trip<'a> {
    fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
        let to_screen = RectTransform::from_to(
            Rect::from_x_y_ranges(
                self.view_range.start.total_units() as f32
                    ..=self.view_range.end.total_units() as f32,
                ControlValue::MAX.0 as f32..=ControlValue::MIN.0 as f32,
            ),
            response.rect,
        );

        // The first step always starts at the left of the view range.
        let mut pos = to_screen
            * pos2(
                MusicalTime::START.total_units() as f32,
                if let Some(step) = self.control_trip.steps_mut().first() {
                    step.value.0 as f32
                } else {
                    0.0
                },
            );
        let stroke = if ui.is_enabled() {
            ui.ctx().style().visuals.widgets.active.fg_stroke
        } else {
            ui.ctx().style().visuals.widgets.inactive.fg_stroke
        };
        let steps_len = self.control_trip.steps().len();
        self.control_trip
            .steps_mut()
            .iter_mut()
            .enumerate()
            .for_each(|(index, step)| {
                // Get the next step position, adjusting if it's the last one.
                let second_pos = if index + 1 == steps_len {
                    let value = pos.y;
                    // Last step. Extend to end of view range.
                    let mut tmp_pos =
                        to_screen * pos2(self.view_range.end.total_units() as f32, 0.0);
                    tmp_pos.y = value;
                    tmp_pos
                } else {
                    // Not last step. Get the actual value.
                    to_screen * pos2(step.time.total_units() as f32, step.value.0 as f32)
                };

                // If we're hovering over this step, highlight it.
                let stroke = if response.hovered() {
                    if let Some(hover_pos) = ui.ctx().pointer_interact_pos() {
                        if hover_pos.x >= pos.x && hover_pos.x < second_pos.x {
                            if response.clicked() {
                                let from_screen = to_screen.inverse();
                                let hover_pos_local = from_screen * hover_pos;
                                step.value = ControlValue::from(hover_pos_local.y);
                            } else if response.secondary_clicked() {
                                step.path = step.path.next();
                            }

                            Stroke {
                                width: stroke.width * 2.0,
                                color: Color32::YELLOW,
                            }
                        } else {
                            stroke
                        }
                    } else {
                        stroke
                    }
                } else {
                    stroke
                };

                // Draw according to the step type.
                match step.path {
                    ControlTripPath::None => {}
                    ControlTripPath::Flat => {
                        painter.line_segment([pos, pos2(pos.x, second_pos.y)], stroke);
                        painter.line_segment([pos2(pos.x, second_pos.y), second_pos], stroke);
                    }
                    ControlTripPath::Linear => {
                        painter.line_segment([pos, second_pos], stroke);
                    }
                    ControlTripPath::Logarithmic => todo!(),
                    ControlTripPath::Exponential => todo!(),
                }
                pos = second_pos;
            });

        if ui.is_enabled() {
            let label =
                if let Some(links) = self.control_router.control_links(self.control_trip.uid()) {
                    let link_texts = links.iter().fold(Vec::default(), |mut v, (uid, index)| {
                        // TODO: this can be a descriptive list of controlled things
                        v.push(format!("{uid}-{index:?} "));
                        v
                    });
                    link_texts.join("/")
                } else {
                    String::from("none")
                };
            if ui
                .allocate_ui_at_rect(response.rect, |ui| ui.button(&label))
                .inner
                .clicked()
            {
                // TODO: this is incomplete. It's a placeholder while I figure
                // out the best way to present this information (it might
                // actually be DnD rather than menu-driven).
                self.control_router.link_control(
                    self.control_trip.uid(),
                    Uid(234),
                    ControlIndex(456),
                );
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl ControlTrip {
        // Causes the next work() to emit a Control event, even if the value
        // matches the last event's value.
        fn debug_reset_last_value(&mut self) {
            self.e.last_published_value = f64::MAX;
        }
    }

    #[test]
    fn instantiate_trigger() {
        let ts = TimeSignature::default();
        let mut trigger = Trigger::new_with(
            Timer::new_with(MusicalTime::new_with_bars(&ts, 1)),
            ControlValue::from(0.5),
        );
        trigger.update_sample_rate(SampleRate::DEFAULT);
        trigger.play();

        trigger.update_time(&std::ops::Range {
            start: MusicalTime::default(),
            end: MusicalTime::new_with_parts(1),
        });
        let mut count = 0;
        trigger.work(&mut |_, _| {
            count += 1;
        });
        assert_eq!(count, 0);
        assert!(!trigger.is_finished());

        trigger.update_time(&std::ops::Range {
            start: MusicalTime::new_with_bars(&ts, 1),
            end: MusicalTime::new(&ts, 1, 0, 0, 1),
        });
        let mut count = 0;
        trigger.work(&mut |_, _| {
            count += 1;
        });
        assert!(count != 0);
        assert!(trigger.is_finished());
    }

    #[test]
    fn control_step_basics() {
        let step = ControlStepBuilder::default()
            .value(ControlValue(0.5))
            .time(MusicalTime::START + MusicalTime::DURATION_WHOLE)
            .path(ControlTripPath::Flat)
            .build();
        assert!(step.is_ok());
    }

    #[test]
    fn control_trip_one_step() {
        let mut ct = ControlTripBuilder::default()
            .uid(Uid(234598))
            .step(ControlStep {
                value: ControlValue(0.5),
                time: MusicalTime::START + MusicalTime::DURATION_WHOLE,
                path: ControlTripPath::Flat,
            })
            .build()
            .unwrap();

        let range = MusicalTime::START..MusicalTime::DURATION_QUARTER;
        ct.update_time(&range);
        const MESSAGE: &'static str = "If there is only one control step, then the trip should remain at that step's level at all times.";
        let mut received_event = None;
        ct.work(&mut |_uid, event| {
            assert!(received_event.is_none());
            received_event = Some(event);
        });
        match received_event.unwrap() {
            EntityEvent::Control(value) => assert_eq!(value.0, 0.5, "{}", MESSAGE),
            _ => panic!(),
        }
        assert!(
            ct.is_finished(),
            "A one-step ControlTrip is always finished"
        );
    }

    #[test]
    fn control_trip_two_flat_steps() {
        let mut ct = ControlTripBuilder::default()
            .uid(Uid(20459))
            .step(ControlStep {
                value: ControlValue(0.5),
                time: MusicalTime::START,
                path: ControlTripPath::Flat,
            })
            .step(ControlStep {
                value: ControlValue(0.75),
                time: MusicalTime::START + MusicalTime::DURATION_WHOLE,
                path: ControlTripPath::Flat,
            })
            .build()
            .unwrap();

        let range = MusicalTime::START..MusicalTime::DURATION_QUARTER;
        ct.update_time(&range);
        let mut received_event = None;
        ct.work(&mut |_uid, event| {
            assert!(received_event.is_none());
            received_event = Some(event);
        });
        match received_event.unwrap() {
            EntityEvent::Control(value) => assert_eq!(value.0, 0.5, "{}", "Flat step should work"),
            _ => panic!(),
        }
        assert!(!ct.is_finished());
        let range = MusicalTime::START + MusicalTime::DURATION_WHOLE
            ..MusicalTime::DURATION_WHOLE + MusicalTime::new_with_units(1);
        ct.update_time(&range);
        let mut received_event = None;
        ct.work(&mut |_uid, event| {
            assert!(received_event.is_none());
            received_event = Some(event);
        });
        match received_event.unwrap() {
            EntityEvent::Control(value) => assert_eq!(value.0, 0.75, "{}", "Flat step should work"),
            _ => panic!(),
        }
        assert!(ct.is_finished());
    }

    #[test]
    fn control_trip_linear_step() {
        let mut ct = ControlTripBuilder::default()
            .uid(Uid(5049))
            .step(ControlStep {
                value: ControlValue(0.0),
                time: MusicalTime::START,
                path: ControlTripPath::Linear,
            })
            .step(ControlStep {
                value: ControlValue(1.0),
                time: MusicalTime::new_with_beats(2),
                path: ControlTripPath::Flat,
            })
            .build()
            .unwrap();

        let range = MusicalTime::new_with_beats(1)
            ..MusicalTime::new_with_beats(1) + MusicalTime::new_with_units(1);
        ct.update_time(&range);
        let mut received_event = None;
        ct.work(&mut |_uid, event| {
            assert!(received_event.is_none());
            received_event = Some(event);
        });
        match received_event.unwrap() {
            EntityEvent::Control(value) => assert_eq!(
                value.0, 0.5,
                "{}",
                "Halfway through linear 0.0..=1.0 should be 0.5"
            ),
            _ => panic!(),
        }
        assert!(!ct.is_finished());
    }

    #[test]
    fn control_trip_many_steps() {
        for i in 0..2 {
            let mut ct = ControlTripBuilder::default()
                .uid(Uid(54893))
                .step(ControlStep {
                    value: ControlValue(0.1),
                    time: MusicalTime::new_with_units(10),
                    path: ControlTripPath::Flat,
                })
                .step(ControlStep {
                    value: ControlValue(0.2),
                    time: MusicalTime::new_with_units(20),
                    path: ControlTripPath::Flat,
                })
                .step(ControlStep {
                    value: ControlValue(0.3),
                    time: MusicalTime::new_with_units(30),
                    path: ControlTripPath::Flat,
                })
                .build()
                .unwrap();

            let mut test_values = vec![
                (0, 0.1, false),
                (5, 0.1, false),
                (10, 0.1, false),
                (11, 0.1, false),
                (20, 0.2, false),
                (21, 0.2, false),
                (30, 0.3, true),
                (31, 0.3, true),
                (9999999999, 0.3, true),
            ];
            if i == 1 {
                test_values.reverse();
            }

            for (unit, ev, finished) in test_values {
                let time = MusicalTime::new_with_units(unit);
                ct.update_time(&(time..(time + MusicalTime::new_with_units(1))));
                let mut received_event = None;
                ct.work(&mut |_uid, event| {
                    assert!(received_event.is_none());
                    received_event = Some(event);
                });
                assert!(received_event.is_some());
                match received_event.unwrap() {
                    EntityEvent::Control(value) => {
                        assert_eq!(
                            value.0, ev,
                            "{i}: Expected {ev} at {time} but got {}",
                            value.0
                        )
                    }
                    _ => panic!(),
                }
                assert_eq!(
                    ct.is_finished(),
                    finished,
                    "At time {time} expected is_finished({finished})"
                );
                ct.debug_reset_last_value();
            }
        }
    }
}
