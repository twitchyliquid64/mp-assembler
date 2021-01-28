use bevy::prelude::*;
use bevy_mod_picking::*;

use crate::gizmo::TranslateHandle;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(Selection::default())
            .add_event::<ParentClickedEvent>()
            .add_event::<ReleaseEvent>()
            .add_system(get_picks.system())
            .add_system(update_selection.system());
    }
}

fn get_picks(
    pick_state: Res<PickState>,
    mouse_inputs: Res<Input<MouseButton>>,
    mut parent_query: Query<(&Parent, Option<&TranslateHandle>)>,
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

fn update_selection(
    ev_clicked: Res<Events<ParentClickedEvent>>,
    mut clicked_reader: Local<EventReader<ParentClickedEvent>>,
    ev_released: Res<Events<ReleaseEvent>>,
    mut released_reader: Local<EventReader<ReleaseEvent>>,

    mut selection: ResMut<Selection>,
    selection_query: Query<&Transform, With<Selectable>>,
) {
    // Handle any 'parent clicked' event, updating the Selection resource.
    for ev in clicked_reader.iter(&ev_clicked) {
        if let Ok(transform) = selection_query.get(ev.0) {
            selection.entity = Some(ev.0);
            selection.handle = ev.1;
            selection.dragging_gizmo = ev.1.is_some();
        // data.transform = transform.clone();
        } else {
            *selection = Selection::default();
        }
    }
    for ev in released_reader.iter(&ev_released) {
        selection.dragging_gizmo = false;
    }
}

#[derive(Debug)]
struct ParentClickedEvent(pub Entity, pub Option<TranslateHandle>);

#[derive(Debug)]
struct ReleaseEvent;

#[derive(Default, Debug)]
pub struct Selectable;

#[derive(Default, Debug)]
pub struct Selection {
    pub entity: Option<Entity>,
    pub handle: Option<TranslateHandle>,
    pub dragging_gizmo: bool,
}

// fn writeback_ui(
//     data: Res<Inspector>,
//     selection: Res<Selection>,
//     mut query: Query<&mut Transform, With<Selectable>>,
// ) {
//     // Updates component values from the UI.
//     if let Some(eid) = selection.entity {
//         if let Ok(mut t) = query.get_mut(eid) {
//             *t = data.transform;
//         }
//     }
// }
