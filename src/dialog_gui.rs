use bevy::prelude::*;
use bevy_egui::*;

use crate::inspector_gui::Library;
use crate::interaction::HotkeyEvent;

use std::{fs, path};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(DialogState::None)
            .add_startup_system(load_assets.system())
            .add_system(ui.system());
    }
}

fn load_assets(_world: &mut World, resources: &mut Resources) {
    let mut egui_context = resources.get_mut::<EguiContext>().unwrap();
    let asset_server = resources.get::<AssetServer>().unwrap();

    let texture_handle = asset_server.load("baseline_folder_white_36dp.png");
    egui_context.set_egui_texture(0, texture_handle);
    egui_context.set_egui_texture(1, asset_server.load("baseline_text_snippet_white_36dp.png"));
}

enum OpenIntent {
    SpecSelection,
}

impl OpenIntent {
    fn title(&self) -> &'static str {
        match self {
            OpenIntent::SpecSelection => &"Add panel from spec",
        }
    }
}

enum DialogState {
    None,
    Open {
        intent: OpenIntent,
        current: path::PathBuf,
        contents: Vec<(fs::DirEntry, fs::Metadata)>,
    },
}

impl Default for DialogState {
    fn default() -> Self {
        DialogState::None
    }
}

pub enum DialogActionEvent {
    Open,
}

fn read_dir(current: &path::PathBuf) -> Vec<(fs::DirEntry, fs::Metadata)> {
    let mut out = match current.read_dir() {
        Ok(dirs) => {
            let result: Result<Vec<_>, _> = dirs.collect();
            match result {
                Ok(result) => result
                    .into_iter()
                    .map(|e| match e.metadata() {
                        Ok(meta) => Some((e, meta)),
                        Err(err) => {
                            eprintln!("failed reading metadata for {:?}: {:?}", e, err);
                            None
                        }
                    })
                    .filter(|e| e.is_some())
                    .map(|e| e.unwrap())
                    .collect(),
                Err(e) => {
                    eprintln!("Failed collecting contents: {:?}", e);
                    vec![]
                }
            }
        }
        Err(e) => {
            eprintln!("Failed reading dir: {:?}", e);
            vec![]
        }
    };
    out.sort_unstable_by(|a, b| {
        if a.1.is_dir() != b.1.is_dir() {
            a.1.is_dir().cmp(&b.1.is_dir()).reverse()
        } else {
            a.0.path().cmp(&b.0.path())
        }
    });
    out
}

enum DialogAction {
    TopbarNavigate(usize),
}

fn ui(
    commands: &mut Commands,
    library: ResMut<Library>,

    ev_hotkey: Res<Events<HotkeyEvent>>,
    mut hotkey_reader: Local<EventReader<HotkeyEvent>>,

    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<DialogState>,
) {
    let mut action: Option<DialogAction> = None;

    for ev in hotkey_reader.iter(&ev_hotkey) {
        match ev {
            HotkeyEvent::Open => {
                let current = directories::BaseDirs::new()
                    .unwrap()
                    .home_dir()
                    .to_path_buf();
                let contents = read_dir(&current);

                *state = DialogState::Open {
                    current,
                    contents,
                    intent: OpenIntent::SpecSelection,
                };
            }
            HotkeyEvent::Escape => {
                *state = DialogState::None;
            }
            _ => {}
        }
    }

    let ctx = &mut egui_context.ctx;
    let screen = ctx.available_rect();
    let rect = egui::Rect::from_min_max(
        egui::pos2(310., 85.),
        egui::pos2(screen.right() - 310., screen.bottom() - 85.),
    );

    match *state {
        DialogState::None => (),
        DialogState::Open {
            ref intent,
            ref current,
            ref contents,
        } => {
            egui::Window::new(intent.title())
                .id(egui::Id::new("dialog"))
                .fixed_rect(rect)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    let last = current.iter().last();
                    ui.horizontal_wrapped(|ui| {
                        for (i, c) in current.iter().enumerate() {
                            if ui
                                .selectable_label(last == Some(c), c.to_str().unwrap())
                                .clicked()
                            {
                                action = Some(DialogAction::TopbarNavigate(i));
                            };
                        }
                    });
                    ui.separator();
                    ui.allocate_space(egui::Vec2::new(0., 4.));

                    egui::containers::ScrollArea::auto_sized()
                        .id_source("files")
                        .show(ui, |ui| {
                            for entry in contents {
                                ui.horizontal(|ui| {
                                    ui.allocate_ui(egui::Vec2::new(32., 16.), |ui| {
                                        if entry.1.is_dir() {
                                            ui.add(
                                                egui::widgets::Image::new(
                                                    egui::TextureId::User(0),
                                                    [16.0, 16.0],
                                                )
                                                .tint(egui::Color32::LIGHT_GRAY),
                                            );
                                        } else {
                                            ui.add(
                                                egui::widgets::Image::new(
                                                    egui::TextureId::User(1),
                                                    [16.0, 16.0],
                                                )
                                                .tint(egui::Color32::LIGHT_GRAY),
                                            );
                                        }
                                    });
                                    if let Some(name) = entry.0.path().file_name() {
                                        ui.label(name.to_str().unwrap());
                                    }
                                });
                            }
                        });
                });
        }
    }

    if let Some(action) = action {
        match (&mut *state, action) {
            (
                DialogState::Open {
                    current, contents, ..
                },
                DialogAction::TopbarNavigate(idx),
            ) => {
                for _ in 0..(current
                    .components()
                    .count()
                    .saturating_sub(idx)
                    .saturating_sub(1))
                {
                    current.pop();
                }
                *contents = read_dir(&current);
            }
            _ => {}
        }
    }
}
