use bevy::prelude::*;
use bevy_egui::*;

use crate::gizmo::TranslateHandle;
use crate::parts::{self, PanelInfo};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(EguiPlugin)
            .add_event::<SpawnPartEvent>()
            .add_event::<FocusUIEvent>()
            .add_resource(GUIState::default())
            .add_resource(WidgetIDs::default())
            .add_system(ui.system());
    }
}

pub struct Library(pub Vec<PanelInfo>);

#[derive(Debug)]
pub enum SpawnPartEvent {
    Panel(PanelInfo, bool, [f32; 3], Option<Transform>),
    Screw(parts::Screw, usize, Option<Transform>),
    Washer(parts::Washer, Option<Transform>),
    Nut(parts::Nut, Option<Transform>),
}

#[derive(Debug)]
pub enum FocusUIEvent {
    TranslateInput,
}

#[derive(Debug)]
struct GUIState {
    pub spawn_selected: usize,
    pub spawn_mm: u32,

    pub spawn_panel_hull: bool,
    pub spawn_panel_color: [f32; 3],

    pub translation: Vec3,
    pub rotation: Vec4,
    pub cur_axis: Option<TranslateHandle>,
}

impl Default for GUIState {
    fn default() -> Self {
        Self {
            spawn_selected: 0,
            spawn_mm: 12,
            spawn_panel_hull: false,
            spawn_panel_color: [0.2, 0.5, 0.2],
            translation: Vec3::default(),
            rotation: Vec4::default(),
            cur_axis: None,
        }
    }
}

#[derive(Debug)]
struct WidgetIDs {
    translate: egui::Id,
}

impl Default for WidgetIDs {
    fn default() -> Self {
        Self {
            translate: egui::Id::new(1u8),
        }
    }
}

enum RotationAction {
    None,
    Reset,
    Negate,
    Sub,
    Add,
}

fn ui(
    commands: &mut Commands,
    library: Res<Library>,

    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<GUIState>,
    mut widgets: ResMut<WidgetIDs>,
    ev_focus: Res<Events<FocusUIEvent>>,
    mut focus_reader: Local<EventReader<FocusUIEvent>>,

    sel: Res<crate::interaction::Selection>,
    mut sel_query: Query<
        (
            &mut Transform,
            Option<&crate::parts::PanelInfo>,
            Option<&crate::parts::Screw>,
            Option<&crate::parts::Washer>,
            Option<&crate::parts::Nut>,
        ),
        With<crate::interaction::Selectable>,
    >,

    mut spawner: ResMut<Events<SpawnPartEvent>>,
    mut ev_dialog: ResMut<Events<crate::dialog_gui::DialogHotkeyEvent>>,
    mut ev_storage: ResMut<Events<crate::storage::StorageEvent>>,
) {
    let selected = match sel.entity() {
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
    if sel.is_dragging() {
        if let Some(h) = sel.gizmo_handle() {
            state.cur_axis = Some(h);
        }
    }

    let mut reset_rotation = false;
    let mut rotation_action_x = RotationAction::None;
    let mut rotation_action_y = RotationAction::None;
    let mut rotation_action_z = RotationAction::None;

    let ctx = &mut egui_context.ctx;
    let screen = ctx.available_rect();
    let rt = egui::Rect::from_min_max(
        egui::pos2(screen.right() - 292.0, 0.),
        screen.max - egui::Vec2::new(10., 10.),
    );

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
                            Some((_, _, Some(screw), _, _)) => {
                                format!("{:?} screw", screw)
                            }
                            Some((_, _, _, Some(washer), _)) => {
                                format!("{:?} washer", washer)
                            }
                            Some((_, _, _, _, Some(nut))) => {
                                format!("{:?} nut", nut)
                            }
                            Some((_, Some(pcb), _, _, _)) => {
                                format!("{}", pcb.name())
                            }
                            _ => "<none>".to_string(),
                        });
                    });

                    ui.separator();
                    ui.columns(3, |columns| {
                        columns[0].allocate_space(egui::Vec2::new(0., 1.));
                        columns[0].label("Position");

                        let mut dummy = 0.;
                        use bevy_egui::egui::Widget;
                        widgets.translate = egui::widgets::DragValue::f32(match state.cur_axis {
                            Some(TranslateHandle::X) => &mut state.translation.x,
                            Some(TranslateHandle::Y) => &mut state.translation.y,
                            Some(TranslateHandle::Z) => &mut state.translation.z,
                            _ => &mut dummy,
                        })
                        .ui(&mut columns[1])
                        .id;

                        columns[2].with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                            ui.horizontal(|ui| {
                                ui.label("  ");
                                if ui
                                    .selectable_label(
                                        state.cur_axis == Some(TranslateHandle::Z),
                                        "Z",
                                    )
                                    .clicked()
                                {
                                    state.cur_axis = Some(TranslateHandle::Z);
                                }
                                if ui
                                    .selectable_label(
                                        state.cur_axis == Some(TranslateHandle::Y),
                                        "Y",
                                    )
                                    .clicked()
                                {
                                    state.cur_axis = Some(TranslateHandle::Y);
                                }
                                if ui
                                    .selectable_label(
                                        state.cur_axis == Some(TranslateHandle::X),
                                        "X",
                                    )
                                    .clicked()
                                {
                                    state.cur_axis = Some(TranslateHandle::X);
                                }
                            });
                        });
                    });

                    ui.allocate_space(egui::Vec2::new(0., 5.));
                    ui.columns(4, |columns| {
                        // columns[0].allocate_space(egui::Vec2::new(0., 1.));
                        columns[0].label("Rotation");
                        if columns[3].small_button("reset all").clicked() {
                            reset_rotation = true;
                        }
                    });
                    ui.allocate_space(egui::Vec2::new(0., 1.));

                    rotation_component_ui(ui, "X", &mut rotation_action_x, &mut state.rotation.x);
                    rotation_component_ui(ui, "Y", &mut rotation_action_y, &mut state.rotation.y);
                    rotation_component_ui(ui, "Z", &mut rotation_action_z, &mut state.rotation.z);

                    ui.allocate_space(egui::Vec2::new(0., 1.));
                    ui.separator();
                    ui.allocate_space(egui::Vec2::new(0., 1.));

                    if ui.button("Delete").clicked() {
                        if let Some(sel) = sel.entity() {
                            state.cur_axis = None;
                            commands.despawn_recursive(sel);
                        }
                    }
                    ui.allocate_space(egui::Vec2::new(0., 4.));
                });

            egui::CollapsingHeader::new("Spawn")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(state.spawn_selected == 0, "panel")
                            .clicked()
                        {
                            state.spawn_selected = 0;
                        };
                        if ui
                            .selectable_label(state.spawn_selected == 1, "m3")
                            .clicked()
                        {
                            state.spawn_selected = 1;
                        };
                        if ui
                            .selectable_label(state.spawn_selected == 2, "m5")
                            .clicked()
                        {
                            state.spawn_selected = 2;
                        };
                    });

                    if state.spawn_selected >= 1 {
                        ui.add(
                            egui::Slider::u32(&mut state.spawn_mm, 6..=60)
                                .smallest_positive(2.0)
                                .text("mm"),
                        );
                        ui.columns(3, |columns| {
                            if columns[0].add(egui::Button::new("screw")).clicked() {
                                spawner.send(SpawnPartEvent::Screw(
                                    match state.spawn_selected {
                                        1 => parts::Screw::M3,
                                        2 => parts::Screw::M5,
                                        _ => unreachable!(),
                                    },
                                    state.spawn_mm as usize,
                                    None,
                                ));
                            };
                            if columns[1].add(egui::Button::new("washer")).clicked() {
                                spawner.send(SpawnPartEvent::Washer(
                                    match state.spawn_selected {
                                        1 => parts::Washer::M3,
                                        2 => parts::Washer::M5,
                                        _ => unreachable!(),
                                    },
                                    None,
                                ));
                            };
                            if columns[2].add(egui::Button::new("nut")).clicked() {
                                spawner.send(SpawnPartEvent::Nut(
                                    match state.spawn_selected {
                                        1 => parts::Nut::M3,
                                        2 => parts::Nut::M5,
                                        _ => unreachable!(),
                                    },
                                    None,
                                ));
                            };
                        });
                    } else {
                        for panel in library.0.iter() {
                            ui.columns(2, |columns| {
                                columns[0].label(format!("{}", panel.name()));
                                columns[1].with_layout(
                                    egui::Layout::top_down(egui::Align::Max),
                                    |ui| {
                                        if panel.well_formed() {
                                            if ui.small_button("+").clicked() {
                                                spawner.send(SpawnPartEvent::Panel(
                                                    panel.clone(),
                                                    state.spawn_panel_hull,
                                                    state.spawn_panel_color,
                                                    None,
                                                ));
                                            }
                                        } else {
                                            ui.colored_label(egui::Color32::RED, ":/");
                                        }
                                    },
                                );
                            });
                        }
                        ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                            if ui.button("Load spec").clicked() {
                                ev_dialog.send(crate::dialog_gui::DialogHotkeyEvent::AddSpec);
                            }
                        });
                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.checkbox(&mut state.spawn_panel_hull, "Convex hull");
                            ui.color_edit_button_rgb(&mut state.spawn_panel_color);
                        });
                    }
                });

            egui::CollapsingHeader::new("Assembly")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("From file").clicked() {
                            ev_dialog.send(crate::dialog_gui::DialogHotkeyEvent::LoadScene);
                        }
                        if ui.button("Save As").clicked() {
                            ev_storage.send(crate::storage::StorageEvent::Save);
                        }
                    });
                });
        });

    if let Some(mut selected) = selected {
        selected.0.translation = state.translation.clone();
        if reset_rotation {
            selected.0.rotation = Quat::identity();
        } else {
            let rotation: Quat = state.rotation.clone().into();
            selected.0.rotation = rotation.normalize();
            if rotation.is_nan() {
                selected.0.rotation = Quat::identity();
            }
        }

        match rotation_action_x {
            RotationAction::Add => {
                selected.0.rotation *= Quat::from_rotation_x(std::f32::consts::PI / 20.);
            }
            RotationAction::Sub => {
                selected.0.rotation *= Quat::from_rotation_x(-std::f32::consts::PI / 20.);
            }
            RotationAction::Negate => {
                selected.0.rotation *= Quat::from_xyzw(1., 0., 0., 0.);
            }
            RotationAction::Reset => {
                let mut tmp: [f32; 4] = selected.0.rotation.into();
                tmp[0] = 0.;
                selected.0.rotation = Quat::from(tmp).normalize();
            }
            _ => {}
        }
        match rotation_action_y {
            RotationAction::Add => {
                selected.0.rotation *= Quat::from_rotation_y(std::f32::consts::PI / 20.);
            }
            RotationAction::Sub => {
                selected.0.rotation *= Quat::from_rotation_y(-std::f32::consts::PI / 20.);
            }
            RotationAction::Negate => {
                selected.0.rotation *= Quat::from_xyzw(0., 1., 0., 0.);
            }
            RotationAction::Reset => {
                let mut tmp: [f32; 4] = selected.0.rotation.into();
                tmp[1] = 0.;
                selected.0.rotation = Quat::from(tmp).normalize();
            }
            _ => {}
        }
        match rotation_action_z {
            RotationAction::Add => {
                selected.0.rotation *= Quat::from_rotation_z(std::f32::consts::PI / 20.);
            }
            RotationAction::Sub => {
                selected.0.rotation *= Quat::from_rotation_z(-std::f32::consts::PI / 20.);
            }
            RotationAction::Negate => {
                selected.0.rotation *= Quat::from_xyzw(0., 0., 1., 0.);
            }
            RotationAction::Reset => {
                let mut tmp: [f32; 4] = selected.0.rotation.into();
                tmp[2] = 0.;
                selected.0.rotation = Quat::from(tmp).normalize();
            }
            _ => {}
        }
    }

    for ev in focus_reader.iter(&ev_focus) {
        match ev {
            FocusUIEvent::TranslateInput => {
                ctx.memory().request_kb_focus(widgets.translate);
            }
        }
    }
}

fn rotation_component_ui(
    ui: &mut egui::Ui,
    label: &str,
    action: &mut RotationAction,
    val: &mut f32,
) {
    ui.columns(4, |columns| {
        columns[0].allocate_space(egui::Vec2::new(0., 1.));
        columns[0].label(label);
        columns[1].drag_angle(val);
        columns[2].horizontal(|ui| {
            if ui.small_button("-").clicked() {
                *action = RotationAction::Sub;
            }
            if ui.small_button("+").clicked() {
                *action = RotationAction::Add;
            }
        });
        columns[3].horizontal(|ui| {
            if ui.button("N").clicked() {
                *action = RotationAction::Negate;
            }
            if ui.button("R").clicked() {
                *action = RotationAction::Reset;
            }
        });
    });
}
