// Copyright (c) 2023 Mike Tsao. All rights reserved.

use crate::{control_bar::ControlBarDisplayMode, ControlBar};
use eframe::{
    egui::{ImageButton, Layout, Widget},
    epaint::vec2,
};
use ensnare_cores_egui::widgets::audio::{frequency_domain, time_domain};
use ensnare_egui_widgets::activity_indicator;
use std::path::PathBuf;
use strum_macros::Display;

/// Actions the user might take via the control panel.
#[derive(Debug, Display)]
pub enum ControlBarAction {
    /// Play button pressed.
    Play,

    /// Stop button pressed.
    Stop,

    /// The user asked to create a new project.
    New,

    /// The user asked to load the project having the given filename.
    Open(PathBuf),

    /// The user asked to save the current project to the given filename.
    Save(PathBuf),

    /// The user pressed the settings icon.
    ToggleSettings,
}

/// Wraps an [ControlBar] as a [Widget](eframe::egui::Widget). Mutates the given view_range.
pub fn control_bar_widget<'a>(
    control_bar: &'a mut ControlBar,
    action: &'a mut Option<ControlBarAction>,
) -> impl eframe::egui::Widget + 'a {
    move |ui: &mut eframe::egui::Ui| ControlBarWidget::new_with(control_bar, action).ui(ui)
}

#[derive(Debug)]
struct ControlBarWidget<'a> {
    control_bar: &'a mut ControlBar,
    action: &'a mut Option<ControlBarAction>,
}
impl<'a> ControlBarWidget<'a> {
    pub fn new_with(
        control_bar: &'a mut ControlBar,
        action: &'a mut Option<ControlBarAction>,
    ) -> Self {
        Self {
            control_bar,
            action,
        }
    }
}
impl<'a> eframe::egui::Widget for ControlBarWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.horizontal_centered(|ui| {
            if ui
                .add(ImageButton::new(eframe::egui::include_image!(
                    "../../../res/images/md-symbols/play_arrow.png"
                )))
                .on_hover_text("Start playback")
                .clicked()
            {
                *self.action = Some(ControlBarAction::Play);
            }
            if ui
                .add(ImageButton::new(eframe::egui::include_image!(
                    "../../../res/images/md-symbols/stop.png"
                )))
                .on_hover_text("Stop playback")
                .clicked()
            {
                *self.action = Some(ControlBarAction::Stop);
            }
            ui.separator();
            if ui
                .add(ImageButton::new(eframe::egui::include_image!(
                    "../../../res/images/md-symbols/new_window.png"
                )))
                .on_hover_text("New project")
                .clicked()
            {
                *self.action = Some(ControlBarAction::New);
            }
            if ui
                .add(ImageButton::new(eframe::egui::include_image!(
                    "../../../res/images/md-symbols/file_open.png"
                )))
                .on_hover_text("Open project")
                .clicked()
            {
                *self.action = Some(ControlBarAction::Open(PathBuf::from(
                    "my-ensnare-project.json",
                )));
            }
            if ui
                .add(ImageButton::new(eframe::egui::include_image!(
                    "../../../res/images/md-symbols/file_save.png"
                )))
                .on_hover_text("Save project")
                .clicked()
            {
                *self.action = Some(ControlBarAction::Save(PathBuf::from(
                    "my-ensnare-project.json",
                )));
            }
            ui.separator();
            ui.allocate_ui_with_layout(
                vec2(4.0, 8.0),
                Layout::top_down(eframe::emath::Align::Center),
                |ui| {
                    ui.add(activity_indicator(self.control_bar.saw_midi_in_activity));
                    ui.add(activity_indicator(self.control_bar.saw_midi_out_activity));
                    self.control_bar.saw_midi_in_activity = false;
                    self.control_bar.saw_midi_out_activity = false;
                },
            );

            // TODO: not on the UI thread!
            while let Ok(samples) = self.control_bar.sample_channel.receiver.try_recv() {
                self.control_bar.sample_buffer.push(&samples);
            }

            let (samples, start) = self.control_bar.sample_buffer.get();
            ui.scope(|ui| {
                ui.set_max_size(vec2(64.0, 32.0));
                if match self.control_bar.display_mode {
                    ControlBarDisplayMode::Time => ui.add(time_domain(samples, start)),
                    ControlBarDisplayMode::Frequency => {
                        self.control_bar.fft_buffer =
                            self.control_bar.sample_buffer.analyze_spectrum().unwrap();
                        ui.add(frequency_domain(&self.control_bar.fft_buffer))
                    }
                }
                .clicked()
                {
                    self.control_bar.display_mode = match self.control_bar.display_mode {
                        ControlBarDisplayMode::Time => ControlBarDisplayMode::Frequency,
                        ControlBarDisplayMode::Frequency => ControlBarDisplayMode::Time,
                    }
                }
            });
            ui.separator();
            if ui.button("settings").clicked() {
                *self.action = Some(ControlBarAction::ToggleSettings);
            }
        })
        .response
    }
}