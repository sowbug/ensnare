// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare::prelude::*;
use strum_macros::Display;

#[derive(Clone, Debug, Display)]
pub(crate) enum MenuBarAction {
    Quit,
    ProjectNew,
    ProjectOpen,
    ProjectSave,
    TrackNewMidi,
    TrackNewAudio,
    TrackNewAux,
    TrackDuplicate,
    TrackDelete,
    TrackRemoveSelectedPatterns,
    TrackAddThing(EntityKey),
    ComingSoon,
}

#[derive(Debug)]
struct MenuBarItem {
    name: String,
    children: Option<Vec<MenuBarItem>>,
    action: Option<MenuBarAction>,
    enabled: bool,
}
impl MenuBarItem {
    fn node(name: &str, children: Vec<MenuBarItem>) -> Self {
        Self {
            name: name.to_string(),
            children: Some(children),
            action: None,
            enabled: true,
        }
    }
    fn leaf(name: &str, action: MenuBarAction, enabled: bool) -> Self {
        Self {
            name: name.to_string(),
            children: None,
            action: Some(action),
            enabled,
        }
    }
    fn show(&self, ui: &mut eframe::egui::Ui) -> Option<MenuBarAction> {
        let mut action = None;
        if let Some(children) = self.children.as_ref() {
            ui.menu_button(&self.name, |ui| {
                for child in children.iter() {
                    if let Some(a) = child.show(ui) {
                        action = Some(a);
                    }
                }
            });
        } else if let Some(action_to_perform) = &self.action {
            if ui
                .add_enabled(self.enabled, eframe::egui::Button::new(&self.name))
                .clicked()
            {
                ui.close_menu();
                action = Some(action_to_perform.clone());
            }
        }
        action
    }
}

#[derive(Debug, Default)]
pub(crate) struct MenuBar {
    action: Option<MenuBarAction>,
    is_track_selected: bool,
}
impl Displays for MenuBar {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        // Menus should look like menus, not buttons
        ui.style_mut().visuals.button_frame = false;

        ui.horizontal(|ui| {
            let mut device_submenus = Vec::default();
            if self.is_track_selected {
                device_submenus.push(MenuBarItem::node("New", self.new_entity_menu()));
            }
            device_submenus.extend(vec![
                MenuBarItem::leaf("Shift Left", MenuBarAction::ComingSoon, true),
                MenuBarItem::leaf("Shift Right", MenuBarAction::ComingSoon, true),
                MenuBarItem::leaf("Move Up", MenuBarAction::ComingSoon, true),
                MenuBarItem::leaf("Move Down", MenuBarAction::ComingSoon, true),
            ]);
            let menus = vec![
                MenuBarItem::node(
                    "Project",
                    vec![
                        MenuBarItem::leaf("New", MenuBarAction::ProjectNew, true),
                        MenuBarItem::leaf("Open", MenuBarAction::ProjectOpen, true),
                        MenuBarItem::leaf("Save", MenuBarAction::ProjectSave, true),
                        MenuBarItem::leaf("Quit", MenuBarAction::Quit, true),
                    ],
                ),
                MenuBarItem::node(
                    "Track",
                    vec![
                        MenuBarItem::leaf("New MIDI", MenuBarAction::TrackNewMidi, true),
                        MenuBarItem::leaf("New Audio", MenuBarAction::TrackNewAudio, true),
                        MenuBarItem::leaf("New Aux", MenuBarAction::TrackNewAux, true),
                        MenuBarItem::leaf(
                            "Duplicate",
                            MenuBarAction::TrackDuplicate,
                            self.is_track_selected,
                        ),
                        MenuBarItem::leaf(
                            "Delete",
                            MenuBarAction::TrackDelete,
                            self.is_track_selected,
                        ),
                        MenuBarItem::leaf(
                            "Remove Selected Patterns",
                            MenuBarAction::TrackRemoveSelectedPatterns,
                            true,
                        ), // TODO: enable only if some patterns selected
                    ],
                ),
                MenuBarItem::node("Device", device_submenus),
                MenuBarItem::node(
                    "Control",
                    vec![
                        MenuBarItem::leaf("Connect", MenuBarAction::ComingSoon, true),
                        MenuBarItem::leaf("Disconnect", MenuBarAction::ComingSoon, true),
                    ],
                ),
            ];
            for item in menus.iter() {
                if let Some(a) = item.show(ui) {
                    self.action = Some(a);
                }
            }
        })
        .response
    }
}
impl Acts for MenuBar {
    type Action = MenuBarAction;

    fn set_action(&mut self, action: Self::Action) {
        self.action = Some(action);
    }

    fn take_action(&mut self) -> Option<Self::Action> {
        self.action.take()
    }
}
impl MenuBar {
    fn new_entity_menu(&self) -> Vec<MenuBarItem> {
        vec![MenuBarItem::node(
            "Things",
            EntityFactory::global()
                .keys()
                .iter()
                .map(|k| {
                    MenuBarItem::leaf(
                        &k.to_string(),
                        MenuBarAction::TrackAddThing(k.clone()),
                        true,
                    )
                })
                .collect(),
        )]
    }

    pub(crate) fn set_is_any_track_selected(&mut self, is_any_track_selected: bool) {
        self.is_track_selected = is_any_track_selected;
    }
}