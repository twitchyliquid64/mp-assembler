use bevy::{input::keyboard::KeyboardInput, prelude::*, render::camera::Camera};
use bevy_mod_picking::*;

use crate::gizmo::TranslateHandle;

#[derive(Default, Debug)]
pub struct Selectable;

#[derive(Debug)]
pub enum Selection {
    None,
    Focused(Entity, Transform),
    AxisFocused {
        entity: Entity,
        handle: TranslateHandle,
        current_transform: Transform,
        dragging_gizmo: bool,
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

    pub fn is_dragging_gizmo(&self) -> bool {
        match self {
            Selection::AxisFocused { dragging_gizmo, .. } => *dragging_gizmo,
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
            .add_event::<GizmoDragEvent>()
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
enum HotkeyEvent {
    Escape,
    Delete,
}

fn get_keyboard(
    ev_keys: Res<Events<KeyboardInput>>,
    mut keys_reader: Local<EventReader<KeyboardInput>>,

    mut ev_hotkey: ResMut<Events<HotkeyEvent>>,
) {
    let mut escape_pressed = false;
    let mut delete_pressed = false;
    for event in keys_reader.iter(&ev_keys) {
        match event.key_code {
            Some(KeyCode::Escape) => {
                escape_pressed = true;
            }
            Some(KeyCode::Delete) => {
                delete_pressed = true;
            }
            _ => {}
        }
    }

    if escape_pressed {
        ev_hotkey.send(HotkeyEvent::Escape);
    }
    if delete_pressed {
        ev_hotkey.send(HotkeyEvent::Delete);
    }
}

#[derive(Debug)]
struct GizmoDragEvent(pub Entity, pub Transform, pub TranslateHandle);

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
    mut commands: &mut Commands,

    mut ev_dragging: ResMut<Events<GizmoDragEvent>>,
) {
    // Handle any 'parent clicked' event, updating the Selection resource.
    for ev in clicked_reader.iter(&ev_clicked) {
        if let Ok(transform) = selection_query.get(ev.0) {
            if let Some(handle) = ev.1 {
                // Handle clicked
                *selection = Selection::AxisFocused {
                    handle,
                    entity: ev.0,
                    dragging_gizmo: true,
                    current_transform: transform.clone(),
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
            ref mut dragging_gizmo,
            ..
        } = *selection
        {
            *dragging_gizmo = false;
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
        }
    }

    if let Selection::AxisFocused {
        dragging_gizmo,
        entity,
        handle,
        current_transform,
    } = *selection
    {
        if dragging_gizmo {
            ev_dragging.send(GizmoDragEvent(entity, current_transform, handle));
        }
    }
}

#[derive(Debug)]
struct EntityDragEvent(pub Entity, pub Transform);

fn compute_drag(
    ev_dragging: Res<Events<GizmoDragEvent>>,
    mut drag_reader: Local<EventReader<GizmoDragEvent>>,
    ev_cursor: Res<Events<CursorMoved>>,
    mut cursor_reader: Local<EventReader<CursorMoved>>,

    windows: Res<Windows>,
    mut camera_query: Query<(&GlobalTransform, &Camera)>,

    mut ev_entity_dragging: ResMut<Events<EntityDragEvent>>,
) {
    use bevy_mod_raycast::RayCastSource;

    for ev in drag_reader.iter(&ev_dragging) {
        let current_transform = ev.1;
        for event in cursor_reader.iter(&ev_cursor) {
            for (global_transform, camera) in &mut camera_query.iter_mut() {
                let p: [f32; 2] = event.position.into();
                let source: RayCastSource<()> = RayCastSource::new().with_screenspace_ray(
                    p.into(),
                    &windows,
                    camera,
                    global_transform,
                );

                let (hit_plane_t, hit_plane_b) = ev.2.intersection_plane(current_transform);
                let cast_result = if let Some(i) = source.intersect_primitive(hit_plane_t) {
                    Some(i)
                } else {
                    source.intersect_primitive(hit_plane_b)
                };
                if let Some(i) = cast_result {
                    ev_entity_dragging.send(EntityDragEvent(
                        ev.0,
                        ev.2.calc_position(current_transform, i),
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
