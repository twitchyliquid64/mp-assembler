use bevy::{input::keyboard::KeyboardInput, prelude::*, render::camera::Camera};
use bevy_mod_picking::*;

use crate::gizmo::TranslateHandle;

#[derive(Default, Debug)]
pub struct Selectable;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DraggingKind {
    None,
    Gizmo,
    Hotkey,
}

impl Default for DraggingKind {
    fn default() -> Self {
        DraggingKind::None
    }
}

#[derive(Debug)]
pub enum Selection {
    None,
    Focused(Entity, Transform),
    AxisFocused {
        entity: Entity,
        handle: TranslateHandle,
        start_transform: Transform,
        dragging: DraggingKind,
    },
}

impl Default for Selection {
    fn default() -> Self {
        Selection::None
    }
}

impl Selection {
    pub fn entity(&self) -> Option<Entity> {
        match self {
            Selection::None => None,
            Selection::Focused(e, _) => Some(e.clone()),
            Selection::AxisFocused { entity, .. } => Some(entity.clone()),
        }
    }

    pub fn gizmo_handle(&self) -> Option<TranslateHandle> {
        match self {
            Selection::AxisFocused { handle, .. } => Some(handle.clone()),
            _ => None,
        }
    }

    pub fn is_dragging(&self) -> bool {
        match self {
            Selection::AxisFocused { dragging, .. } => *dragging != DraggingKind::None,
            _ => false,
        }
    }
}

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(Selection::default())
            .add_event::<ParentClickedEvent>()
            .add_event::<ReleaseEvent>()
            .add_system(get_picks.system())
            .add_event::<HotkeyEvent>()
            .add_system(get_keyboard.system())
            .add_event::<DragEvent>()
            .add_resource(AxisEntity::default())
            .add_system(gcd.system())
            .add_event::<EntityDragEvent>()
            .add_system(compute_drag.system())
            .add_system(update_from_drag.system());
    }
}

#[derive(Debug)]
struct ParentClickedEvent(pub Entity, pub Option<TranslateHandle>);

#[derive(Debug)]
struct ReleaseEvent;

// get_picks emits clicked + released events based on mouse movement and
// what PickableMesh is under the cursor.
fn get_picks(
    pick_state: Res<PickState>,
    mouse_inputs: Res<Input<MouseButton>>,
    parent_query: Query<(&Parent, Option<&TranslateHandle>)>,
    mut ev_clicked: ResMut<Events<ParentClickedEvent>>,
    mut ev_released: ResMut<Events<ReleaseEvent>>,
) {
    if mouse_inputs.just_pressed(MouseButton::Left) {
        let top = pick_state.top(Group::default());
        if let Some(top) = top {
            if let Ok((parent, hnd)) = parent_query.get(top.0) {
                ev_clicked.send(ParentClickedEvent(parent.0, hnd.map(|t| t.clone())));
            }
        }
    } else if mouse_inputs.just_released(MouseButton::Left) {
        ev_released.send(ReleaseEvent);
    }
}

#[derive(Debug)]
pub enum HotkeyEvent {
    Escape,
    Delete,
    AxisX,
    AxisY,
    AxisZ,
    Edit,
    Open,
}

fn get_keyboard(
    ev_keys: Res<Events<KeyboardInput>>,
    mut keys_reader: Local<EventReader<KeyboardInput>>,

    mut ev_hotkey: ResMut<Events<HotkeyEvent>>,
) {
    let mut keys: Vec<HotkeyEvent> = Vec::new();
    for event in keys_reader.iter(&ev_keys) {
        let event = match event.key_code {
            Some(KeyCode::Escape) => Some(HotkeyEvent::Escape),
            Some(KeyCode::Delete) => Some(HotkeyEvent::Delete),
            Some(KeyCode::F1) => Some(HotkeyEvent::AxisX),
            Some(KeyCode::F2) => Some(HotkeyEvent::AxisY),
            Some(KeyCode::F3) => Some(HotkeyEvent::AxisZ),
            Some(KeyCode::F5) => Some(HotkeyEvent::Open),
            Some(KeyCode::R) => Some(HotkeyEvent::Edit),
            _ => None,
        };

        if let Some(e) = event {
            keys.push(e);
        }
    }

    for hotkey in keys.into_iter() {
        ev_hotkey.send(hotkey);
    }
}

#[derive(Debug)]
struct DragEvent(Entity, Transform, DraggingKind, TranslateHandle);

#[derive(Default, Debug)]
struct AxisEntity(Option<Entity>);

impl AxisEntity {
    fn build(
        transform: Transform,
        handle: TranslateHandle,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> (Handle<Mesh>, Handle<StandardMaterial>) {
        let mesh = meshes.add(Mesh::from(match handle {
            TranslateHandle::X => shape::Box {
                min_x: -99999.,
                max_x: 99999.,
                min_y: transform.translation.y - 0.5,
                max_y: transform.translation.y + 0.5,
                min_z: transform.translation.z - 0.5,
                max_z: transform.translation.z + 0.5,
            },
            TranslateHandle::Y => shape::Box {
                min_x: transform.translation.x - 0.5,
                max_x: transform.translation.x + 0.5,
                min_y: -99999.,
                max_y: 99999.,
                min_z: transform.translation.z - 0.5,
                max_z: transform.translation.z + 0.5,
            },
            TranslateHandle::Z => shape::Box {
                min_x: transform.translation.x - 0.5,
                max_x: transform.translation.x + 0.5,
                min_y: transform.translation.y - 0.5,
                max_y: transform.translation.y + 0.5,
                min_z: -99999.,
                max_z: 99999.,
            },
        }));
        let color = materials.add(match handle {
            TranslateHandle::X => Color::rgba(1.0, 0.0, 0.0, 0.2).into(),
            TranslateHandle::Y => Color::rgba(0.0, 1.0, 0.0, 0.2).into(),
            TranslateHandle::Z => Color::rgba(0.0, 0.0, 1.0, 0.2).into(),
        });

        (mesh, color)
    }
}

/// gcd:
///  - emits dragging events
///  - updates the current selected entity.
///  - handles hotkeys
fn gcd(
    ev_clicked: Res<Events<ParentClickedEvent>>,
    mut clicked_reader: Local<EventReader<ParentClickedEvent>>,
    ev_released: Res<Events<ReleaseEvent>>,
    mut released_reader: Local<EventReader<ReleaseEvent>>,
    ev_hotkey: Res<Events<HotkeyEvent>>,
    mut hotkey_reader: Local<EventReader<HotkeyEvent>>,

    mut selection: ResMut<Selection>,
    selection_query: Query<&Transform, With<Selectable>>,
    commands: &mut Commands,

    mut axis_entity: ResMut<AxisEntity>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ev_dragging: ResMut<Events<DragEvent>>,
    mut ev_focus: ResMut<Events<crate::inspector_gui::FocusUIEvent>>,
) {
    // Handle any 'parent clicked' event, updating the Selection resource.
    for ev in clicked_reader.iter(&ev_clicked) {
        if let Ok(transform) = selection_query.get(ev.0) {
            if let Some(handle) = ev.1 {
                // Handle clicked
                *selection = Selection::AxisFocused {
                    handle,
                    entity: ev.0,
                    dragging: DraggingKind::Gizmo,
                    start_transform: transform.clone(),
                };
            } else {
                // Entity focused
                *selection = Selection::Focused(ev.0, transform.clone());
            }
        } else {
            *selection = Selection::None;
        }
    }

    for _ev in released_reader.iter(&ev_released) {
        // If the mouse was released while dragging a translate axis.
        if let Selection::AxisFocused {
            ref mut dragging, ..
        } = *selection
        {
            *dragging = DraggingKind::None;
        }
    }

    for ev in hotkey_reader.iter(&ev_hotkey) {
        match ev {
            HotkeyEvent::Escape => {
                *selection = Selection::None;
            }
            HotkeyEvent::Delete => {
                if let Some(sel) = selection.entity() {
                    commands.despawn_recursive(sel);
                }
                *selection = Selection::None;
            }

            HotkeyEvent::AxisX | HotkeyEvent::AxisY | HotkeyEvent::AxisZ => {
                let entity = match *selection {
                    Selection::Focused(entity, _) => Some(entity),
                    Selection::AxisFocused { entity, .. } => Some(entity),
                    _ => None,
                };
                if let Some(entity) = entity {
                    if let Ok(transform) = selection_query.get(entity) {
                        *selection = Selection::AxisFocused {
                            entity,
                            start_transform: transform.clone(),
                            handle: match ev {
                                HotkeyEvent::AxisX => TranslateHandle::X,
                                HotkeyEvent::AxisY => TranslateHandle::Y,
                                HotkeyEvent::AxisZ => TranslateHandle::Z,
                                _ => unreachable!(),
                            },
                            dragging: DraggingKind::Hotkey,
                        };
                        if let Some(entity) = axis_entity.0 {
                            // Theres already an axis visualization, yeet it on
                            // outta here.
                            commands.despawn(entity);
                            *axis_entity = AxisEntity(None);
                        }
                    }
                }
            }
            HotkeyEvent::Edit => {
                if let Selection::AxisFocused { .. } = &*selection {
                    ev_focus.send(crate::inspector_gui::FocusUIEvent::TranslateInput);
                }
            }
            HotkeyEvent::Open => (),
        }
    }

    if let Selection::AxisFocused {
        dragging,
        entity,
        handle,
        start_transform,
    } = *selection
    {
        if dragging != DraggingKind::None {
            ev_dragging.send(DragEvent(entity, start_transform, dragging, handle));
        }
        match (dragging == DraggingKind::Hotkey, axis_entity.0) {
            (true, None) => {
                // We are in an axis hotkey mode but no entity for the visuals exists.
                let (mesh, material) =
                    AxisEntity::build(start_transform, handle, &mut meshes, &mut materials);
                *axis_entity = AxisEntity(Some(
                    commands
                        .spawn(PbrBundle {
                            mesh,
                            material,
                            visible: Visible {
                                is_visible: true,
                                is_transparent: true,
                            },
                            ..Default::default()
                        })
                        .current_entity()
                        .unwrap(),
                ))
            }
            (false, Some(entity)) => {
                commands.despawn(entity);
                *axis_entity = AxisEntity(None);
            }
            _ => {}
        }
    } else if let Some(entity) = axis_entity.0 {
        // If there is an axis visualization but we arent axis focused.
        commands.despawn(entity);
        *axis_entity = AxisEntity(None);
    }
}

#[derive(Debug)]
struct EntityDragEvent(pub Entity, pub Transform);

fn compute_drag(
    ev_dragging: Res<Events<DragEvent>>,
    mut drag_reader: Local<EventReader<DragEvent>>,
    ev_cursor: Res<Events<CursorMoved>>,
    mut cursor_reader: Local<EventReader<CursorMoved>>,

    windows: Res<Windows>,
    mut camera_query: Query<(&GlobalTransform, &Camera)>,

    mut ev_entity_dragging: ResMut<Events<EntityDragEvent>>,
) {
    use bevy_mod_raycast::RayCastSource;

    for ev in drag_reader.iter(&ev_dragging) {
        let start_transform = ev.1;
        for event in cursor_reader.iter(&ev_cursor) {
            for (global_transform, camera) in &mut camera_query.iter_mut() {
                let p: [f32; 2] = event.position.into();
                let source: RayCastSource<()> = RayCastSource::new().with_screenspace_ray(
                    p.into(),
                    &windows,
                    camera,
                    global_transform,
                );

                let (hit_plane_t, hit_plane_b) = ev.3.intersection_plane(start_transform);
                let cast_result = if let Some(i) = source.intersect_primitive(hit_plane_t) {
                    Some(i)
                } else {
                    source.intersect_primitive(hit_plane_b)
                };
                if let Some(i) = cast_result {
                    ev_entity_dragging.send(EntityDragEvent(
                        ev.0,
                        ev.3.calc_position(start_transform, i, ev.2 == DraggingKind::Gizmo),
                    ));
                }
            }
        }
    }
}

fn update_from_drag(
    ev_dragging: Res<Events<EntityDragEvent>>,
    mut drag_reader: Local<EventReader<EntityDragEvent>>,
    mut selection_query: Query<&mut Transform, With<Selectable>>,
) {
    for ev in drag_reader.iter(&ev_dragging) {
        if let Ok(mut transform) = selection_query.get_mut(ev.0) {
            *transform = ev.1;
        }
    }
}
