// Copyright (c) 2024 Mike Tsao. All rights reserved.

use eframe::{
    egui::{vec2, Widget},
    emath::RectTransform,
    epaint::pos2,
};
use ensnare_core::time::{MusicalTime, ViewRange};

/// Wraps a [Cursor] as a [Widget](eframe::egui::Widget).
pub fn cursor<'a>(position: MusicalTime, view_range: ViewRange) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| {
        Cursor::default()
            .position(position)
            .view_range(view_range)
            .ui(ui)
    }
}
/// An egui widget that draws a representation of the playback cursor.
#[derive(Debug, Default)]
pub struct Cursor {
    /// The cursor position.
    position: MusicalTime,

    /// The GUI view's time range.
    view_range: ViewRange,
}
impl Cursor {
    fn position(mut self, position: MusicalTime) -> Self {
        self.position = position;
        self
    }
    fn view_range(mut self, view_range: ViewRange) -> Self {
        self.view_range = view_range;
        self
    }
}
impl eframe::egui::Widget for Cursor {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let desired_size = vec2(ui.available_width(), 64.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, eframe::egui::Sense::hover());
        let to_screen = RectTransform::from_to(
            eframe::epaint::Rect::from_x_y_ranges(
                self.view_range.0.start.total_units() as f32
                    ..=self.view_range.0.end.total_units() as f32,
                0.0..=1.0,
            ),
            rect,
        );
        let visuals = ui.ctx().style().visuals.widgets.noninteractive;
        let start = to_screen * pos2(self.position.total_units() as f32, 0.0);
        let end = to_screen * pos2(self.position.total_units() as f32, 1.0);
        ui.painter().line_segment([start, end], visuals.fg_stroke);
        response
    }
}
