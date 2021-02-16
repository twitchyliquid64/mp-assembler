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

pub(crate) enum FileNavIntent {
    SpecSelection,
}

impl FileNavIntent {
    fn title(&self) -> &'static str {
        match self {
            FileNavIntent::SpecSelection => &"Add panel from spec",
        }
    }
    fn filter(&self, entry: &(fs::DirEntry, fs::Metadata)) -> bool {
        match self {
            FileNavIntent::SpecSelection => {
                (entry.1.is_dir()
                    && !entry
                        .0
                        .path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .starts_with("."))
                    || entry.0.path().extension() == Some(&std::ffi::OsStr::new("spec"))
            }
        }
    }
}

pub(crate) enum DialogState {
    None,
    Open {
        intent: FileNavIntent,
        current: path::PathBuf,
        contents: Vec<(fs::DirEntry, fs::Metadata)>,
    },
}

impl Default for DialogState {
    fn default() -> Self {
        DialogState::None
    }
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

enum UiAction {
    TopbarNavigate(usize),
    DirEntryNavigate((path::PathBuf, fs::Metadata)),
}

pub enum DialogHotkeyEvent {
    Open,
    Escape,
}

fn ui(
    cmd_args: Res<crate::CmdArgs>,
    mut library: ResMut<Library>,

    ev_hotkey: Res<Events<DialogHotkeyEvent>>,
    mut hotkey_reader: Local<EventReader<DialogHotkeyEvent>>,

    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<DialogState>,
) {
    let mut action: Option<UiAction> = None;

    for ev in hotkey_reader.iter(&ev_hotkey) {
        match ev {
            DialogHotkeyEvent::Open => {
                let current = if cmd_args.0.spec_dirs.len() == 0 {
                    directories::BaseDirs::new()
                        .unwrap()
                        .home_dir()
                        .to_path_buf()
                } else {
                    cmd_args.0.spec_dirs[0].clone().into()
                };
                let contents = read_dir(&current);

                *state = DialogState::Open {
                    current,
                    contents,
                    intent: FileNavIntent::SpecSelection,
                };
            }
            DialogHotkeyEvent::Escape => {
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
                                action = Some(UiAction::TopbarNavigate(i));
                            };
                        }
                    });
                    ui.separator();
                    ui.allocate_space(egui::Vec2::new(0., 4.));

                    egui::containers::ScrollArea::auto_sized()
                        .id_source("files")
                        .show(ui, |ui| {
                            for entry in contents.iter().filter(|e| intent.filter(e)) {
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
                                        if ui.button(name.to_str().unwrap()).clicked() {
                                            action = Some(UiAction::DirEntryNavigate((
                                                entry.0.path(),
                                                entry.1.clone(),
                                            )));
                                        }
                                    }
                                });
                            }
                        });

                    ui.allocate_space(egui::Vec2::new(
                        10.,
                        ui.available_rect_before_wrap_finite().height() - 25.,
                    ));
                });
        }
    }

    if let Some(action) = action {
        match (&mut *state, action) {
            (
                DialogState::Open {
                    current, contents, ..
                },
                UiAction::TopbarNavigate(idx),
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
            (
                DialogState::Open {
                    current,
                    contents,
                    intent,
                    ..
                },
                UiAction::DirEntryNavigate((ref path, ref meta)),
            ) => {
                if meta.is_dir() {
                    *current = path.to_path_buf();
                    *contents = read_dir(&current);
                } else {
                    match intent {
                        FileNavIntent::SpecSelection => match std::fs::read(path) {
                            Ok(contents) => {
                                let spec = crate::parts::PanelInfo::new(
                                    path.to_str().unwrap().to_string(),
                                    String::from_utf8_lossy(&contents).to_string(),
                                );
                                library.0.push(spec);
                                *state = DialogState::None;
                            }
                            Err(e) => {
                                eprintln!("Failed reading {:?}: {:?}", path, e);
                            }
                        },
                    }
                }
            }
            _ => {}
        }
    }
}
