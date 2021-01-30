use bevy::prelude::*;
use bevy_egui::*;

use crate::gizmo::TranslateHandle;
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
    pub translation: Vec3,
    pub rotation: Vec4,
    pub cur_axis: Option<TranslateHandle>,
}

impl Default for GUIState {
    fn default() -> Self {
        Self {
            spawn_selected: 0,
            spawn_mm: 12,
            translation: Vec3::default(),
            rotation: Vec4::default(),
            cur_axis: None,
        }
    }
}

fn ui(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,

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
                state.translation = e.0.translation.clone();
                state.rotation = e.0.rotation.into();
                Some(e)
            } else {
                None
            }
        }
        None => None,
    };
    if let Some(h) = sel.handle {
        state.cur_axis = Some(h);
    }

    let ctx = &mut egui_context.ctx;
    let screen = ctx.available_rect();
    let rt = egui::Rect::from_min_max(egui::pos2(screen.right() - 292.0, 0.), screen.max);

    let state = &mut state;
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
                    ui.horizontal(|ui| {
                        ui.label("Position:");
                        if ui
                            .selectable_label(state.cur_axis == Some(TranslateHandle::X), "X")
                            .clicked
                        {
                            state.cur_axis = Some(TranslateHandle::X);
                        }
                        if ui
                            .selectable_label(state.cur_axis == Some(TranslateHandle::Y), "Y")
                            .clicked
                        {
                            state.cur_axis = Some(TranslateHandle::Y);
                        }
                        if ui
                            .selectable_label(state.cur_axis == Some(TranslateHandle::Z), "Z")
                            .clicked
                        {
                            state.cur_axis = Some(TranslateHandle::Z);
                        }
                        ui.label("   ");

                        let mut dummy = 0.;
                        use bevy_egui::egui::Widget;
                        let amt = egui::widgets::DragValue::f32(match state.cur_axis {
                            Some(TranslateHandle::X) => &mut state.translation.x,
                            Some(TranslateHandle::Y) => &mut state.translation.y,
                            Some(TranslateHandle::Z) => &mut state.translation.z,
                            _ => &mut dummy,
                        })
                        .ui(ui);
                    });

                    ui.label("Rotation");
                    ui.horizontal(|ui| {
                        ui.label(" X:");
                        ui.drag_angle(&mut state.rotation.x);
                        ui.label(" Y: ");
                        ui.drag_angle(&mut state.rotation.y);
                        ui.label(" Z: ");
                        ui.drag_angle(&mut state.rotation.z);
                    });
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

    if let Some(mut selected) = selected {
        selected.0.translation = state.translation.clone();
        let rotation: Quat = state.rotation.clone().into();
        selected.0.rotation = rotation.normalize();
        if rotation.is_nan() {
            selected.0.rotation = Quat::identity();
        }
    }
}
