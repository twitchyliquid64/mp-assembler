use bevy::prelude::*;
use bevy_mod_picking::*;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(Selection::default())
            .add_event::<ParentClickedEvent>()
            .add_system(get_picks.system())
            .add_system(update_selection.system());
    }
}

fn get_picks(
    pick_state: Res<PickState>,
    mouse_inputs: Res<Input<MouseButton>>,
    mut parent_query: Query<&Parent>,
    mut events: ResMut<Events<ParentClickedEvent>>,
) {
    if mouse_inputs.just_pressed(MouseButton::Left) {
        let top = pick_state.top(Group::default());
        if let Some(top) = top {
            if let Ok(parent) = parent_query.get(top.0) {
                events.send(ParentClickedEvent(parent.0));
            }
        }
    }
}

fn update_selection(
    mut selection: ResMut<Selection>,

    events: Res<Events<ParentClickedEvent>>,
    mut reader: Local<EventReader<ParentClickedEvent>>,

    selection_query: Query<&Transform, With<Selectable>>,
) {
    // Handle any 'parent clicked' event, updating the Selection resource.
    for ev in reader.iter(&events) {
        if let Ok(transform) = selection_query.get(ev.0) {
            selection.entity = Some(ev.0);
        // data.transform = transform.clone();
        } else {
            selection.entity = None;
        }
    }
}

#[derive(Debug)]
struct ParentClickedEvent(pub Entity);

#[derive(Default, Debug)]
pub struct Selectable;

#[derive(Default, Debug)]
pub struct Selection {
    pub entity: Option<Entity>,
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
