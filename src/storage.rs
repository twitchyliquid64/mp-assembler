use bevy::{input::keyboard::KeyboardInput, prelude::*, render::camera::Camera};
use bevy_mod_picking::*;
use serde::{Deserialize, Serialize};

use crate::interaction::Selectable;
use crate::parts::{Nut, PanelInfo, Pcb, Screw, ScrewLength, Washer};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<StorageEvent>()
            .add_system_to_stage(stage::POST_UPDATE, saver.system());
    }
}

pub enum StorageEvent {
    Save,
}

fn saver(
    ev_action: Res<Events<StorageEvent>>,
    mut action_reader: Local<EventReader<StorageEvent>>,

    q: Query<
        (
            &Transform,
            Option<&Screw>,
            Option<&ScrewLength>,
            Option<&Washer>,
            Option<&Nut>,
            Option<&PanelInfo>,
        ),
        With<Selectable>,
    >,
    mut ev_storage: ResMut<Events<crate::dialog_gui::DialogHotkeyEvent>>,
) {
    for ev in action_reader.iter(&ev_action) {
        match ev {
            StorageEvent::Save => {
                let objs: Vec<serde_json::Value> = q
                    .iter()
                    .map(|obj| {
                        let rep: ObjectRep = obj.into();
                        rep
                    })
                    .filter(|rep| !matches!(rep, ObjectRep::None))
                    .map(|rep| serde_json::to_value(&rep).unwrap())
                    .collect();

                let obj = serde_json::Value::Array(objs);
                ev_storage.send(crate::dialog_gui::DialogHotkeyEvent::SaveScene(obj));
            }
        }
    }
}

pub(crate) fn decode_scene(json: &Vec<u8>) -> Vec<ObjectRep> {
    let decode: Result<Vec<ObjectRep>, _> = serde_json::from_slice(json);
    if let Ok(objs) = decode {
        objs
    } else {
        vec![]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Pos {
    x: f32,
    y: f32,
    z: f32,
    quat: [f32; 4],
}

impl From<&Transform> for Pos {
    fn from(i: &Transform) -> Self {
        Self {
            x: i.translation.x,
            y: i.translation.y,
            z: i.translation.z,
            quat: i.rotation.into(),
        }
    }
}

impl Into<Transform> for Pos {
    fn into(self: Self) -> Transform {
        Transform {
            translation: Vec3::new(self.x, self.y, self.z),
            rotation: self.quat.into(),
            ..Transform::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum ObjectRep {
    Screw {
        pos: Pos,
        screw: Screw,
        length: usize,
    },
    Nut {
        pos: Pos,
        nut: Nut,
    },
    Washer {
        pos: Pos,
        washer: Washer,
    },
    Panel {
        pos: Pos,
        path: String,
        spec: String,
        convex_hull: bool,
    },
    None,
}

impl
    From<(
        &Transform,
        Option<&Screw>,
        Option<&ScrewLength>,
        Option<&Washer>,
        Option<&Nut>,
        Option<&PanelInfo>,
    )> for ObjectRep
{
    fn from(
        info: (
            &Transform,
            Option<&Screw>,
            Option<&ScrewLength>,
            Option<&Washer>,
            Option<&Nut>,
            Option<&PanelInfo>,
        ),
    ) -> Self {
        let (transform, screw, length, washer, nut, panel) = info;
        if let Some(screw) = screw {
            return ObjectRep::Screw {
                pos: transform.into(),
                screw: screw.clone(),
                length: length.unwrap().0,
            };
        }
        if let Some(nut) = nut {
            return ObjectRep::Nut {
                pos: transform.into(),
                nut: nut.clone(),
            };
        }
        if let Some(washer) = washer {
            return ObjectRep::Washer {
                pos: transform.into(),
                washer: washer.clone(),
            };
        }
        if let Some(panel) = panel {
            let (path, spec, convex_hull) = panel.clone().split();
            return ObjectRep::Panel {
                path,
                spec,
                convex_hull,
                pos: transform.into(),
            };
        }

        ObjectRep::None
    }
}
