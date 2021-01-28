use bevy::prelude::*;
use bevy_egui::*;

use crate::parts;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(EguiPlugin)
            .add_resource(GUIState::default())
            .add_system(ui.system());
    }
}

#[derive(Debug)]
struct GUIState {
    pub spawn_selected: usize,
    pub spawn_mm: u32,
}

impl Default for GUIState {
    fn default() -> Self {
        Self {
            spawn_selected: 0,
            spawn_mm: 12,
        }
    }
}

fn ui(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<GUIState>,
    sel: Res<crate::selection::Selection>,
    mut sel_query: Query<
        (&mut Transform, Option<&crate::parts::Screw>),
        With<crate::selection::Selectable>,
    >,
) {
    let selected = match sel.entity {
        Some(e) => {
            if let Ok(e) = sel_query.get_mut(e) {
                Some(e)
            } else {
                None
            }
        }
        None => None,
    };

    let ctx = &mut egui_context.ctx;
    let screen = ctx.available_rect();
    let rt = egui::Rect::from_min_max(egui::pos2(screen.right() - 256.0, 0.), screen.max);
    egui::Window::new("mp-assembler")
        .fixed_rect(rt)
        .show(ctx, |ui| {
            egui::CollapsingHeader::new("Selection")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Object:");
                        ui.label(match selected {
                            Some((_, screw)) => {
                                if let Some(screw) = screw {
                                    format!("{:?}", screw)
                                } else {
                                    "pcb".to_string()
                                }
                            }
                            None => "<none>".to_string(),
                        });
                    });
                    match sel.handle {
                        Some(h) => {
                            ui.horizontal(|ui| {
                                use crate::gizmo::TranslateHandle;
                                ui.selectable_label(h == TranslateHandle::X, "X");
                                ui.selectable_label(h == TranslateHandle::Y, "Y");
                                ui.selectable_label(h == TranslateHandle::Z, "Z");
                            });
                        }
                        None => (),
                    }
                });

            egui::CollapsingHeader::new("Spawn")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.selectable_label(state.spawn_selected == 0, "m3").clicked {
                            state.spawn_selected = 0;
                        };
                        if ui.selectable_label(state.spawn_selected == 1, "m5").clicked {
                            state.spawn_selected = 1;
                        };
                        if ui
                            .selectable_label(state.spawn_selected == 2, "panel")
                            .clicked
                        {
                            state.spawn_selected = 2;
                        };
                    });
                    if state.spawn_selected < 2 {
                        ui.add(
                            egui::Slider::u32(&mut state.spawn_mm, 6..=60)
                                .smallest_positive(2.0)
                                .text("mm"),
                        );
                        if ui.add(egui::Button::new("spawn")).clicked {
                            parts::spawn_screw(
                                match state.spawn_selected {
                                    0 => parts::Screw::M3,
                                    1 => parts::Screw::M5,
                                    _ => unreachable!(),
                                },
                                commands,
                                &asset_server,
                                materials,
                                meshes,
                                Transform::from_translation(Vec3::new(0., 10., 0.)),
                                state.spawn_mm as usize,
                            );
                        };
                    }
                });
        });
}
