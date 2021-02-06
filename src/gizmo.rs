use crate::interaction::{Selectable, Selection};
use bevy::prelude::*;
use bevy_mod_raycast::{Intersection, Primitive3d};

/// Component that is present on all gizmo children.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gizmo {
    X,
    Y,
    Z,
}

impl Gizmo {
    fn arm_transform(&self) -> (Vec3, Quat) {
        match self {
            Gizmo::X => (
                Vec3::new(4.5, 0.0, 0.0),
                Quat::from_rotation_y(std::f32::consts::PI / 2.),
            ),
            Gizmo::Y => (
                Vec3::new(0.0, 4.5, 0.0),
                Quat::from_rotation_y(-std::f32::consts::PI / 2.)
                    .mul_quat(Quat::from_rotation_x(-std::f32::consts::PI / 2.)),
            ),
            Gizmo::Z => (Vec3::new(0.0, 0.0, 4.5), Quat::identity()),
        }
    }

    fn handle_transform(&self) -> (Vec3, Quat) {
        match self {
            Gizmo::X => (
                Vec3::new(10.0, 0.0, 0.0),
                Quat::from_rotation_y(std::f32::consts::PI / 2.),
            ),
            Gizmo::Y => (
                Vec3::new(0.0, 10.0, 0.0),
                Quat::from_rotation_x(-std::f32::consts::PI / 2.),
            ),
            Gizmo::Z => (
                Vec3::new(0.0, 0.0, 10.0),
                Quat::from_rotation_z(std::f32::consts::PI / 2.),
            ),
        }
    }
}

/// Component that is present on translate gizmo handles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TranslateHandle {
    X,
    Y,
    Z,
}

impl TranslateHandle {
    pub fn intersection_plane(&self, transform: Transform) -> (Primitive3d, Primitive3d) {
        let (normal, p): (Vec3, Vec3) = match self {
            TranslateHandle::X => (
                [0., -1., 0.].into(),
                Vec3::new(0., transform.translation.y, transform.translation.z),
            ),
            TranslateHandle::Y => (
                [0., 0., -1.].into(),
                Vec3::new(transform.translation.x, 0., transform.translation.z),
            ),
            TranslateHandle::Z => (
                [1., 0., 0.].into(),
                Vec3::new(transform.translation.x, transform.translation.y, 0.),
            ),
        };

        (
            Primitive3d::Plane {
                point: p,
                normal: normal,
            },
            Primitive3d::Plane {
                point: p,
                normal: normal * Vec3::from([-1., -1., -1.]),
            },
        )
    }

    pub fn calc_position(&self, mut transform: Transform, intersection: Intersection) -> Transform {
        let (axis, p): (Vec3, Vec3) = match self {
            TranslateHandle::X => (
                Vec3::unit_x(),
                Vec3::new(0., transform.translation.y, transform.translation.z),
            ),
            TranslateHandle::Y => (
                Vec3::unit_y(),
                Vec3::new(transform.translation.x, 0., transform.translation.z),
            ),
            TranslateHandle::Z => (
                Vec3::unit_z(),
                Vec3::new(transform.translation.x, transform.translation.y, 0.),
            ),
        };

        transform.translation = p + (intersection.position() - Vec3::splat(10.)) * axis;
        transform
    }
}

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(gizmo_update_visibility.system())
            .add_system_to_stage(stage::POST_UPDATE, gizmo_update_pos.system());
    }
}

fn gizmo_update_visibility(
    selection: Res<Selection>,
    mut gizmos: Query<(&mut Visible, &Parent), With<Gizmo>>,
) {
    let entity = selection.entity();
    for (mut vis, parent) in gizmos.iter_mut() {
        vis.is_visible = Some(parent.0) == entity;
    }
}

fn gizmo_update_pos(
    mut gizmos: Query<(
        &mut GlobalTransform,
        &Parent,
        &Gizmo,
        Option<&TranslateHandle>,
    )>,
    parent_query: Query<&Transform, Without<Gizmo>>,
) {
    for (mut transform, parent, gizmo, handle) in gizmos.iter_mut() {
        let (t, r) = if handle.is_some() {
            gizmo.handle_transform()
        } else {
            gizmo.arm_transform()
        };
        transform.rotation = r;
        if let Ok(base) = parent_query.get(parent.0) {
            transform.translation = t + base.translation;
        }
    }
}

pub fn spawn_translate(
    commands: &mut ChildBuilder,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let cone = meshes.add(cone());
    let arm = meshes.add(cylinder());

    let red = materials.add(Color::rgb(1.0, 0.0, 0.0).into());
    let green = materials.add(Color::rgb(0.0, 1.0, 0.0).into());
    let blue = materials.add(Color::rgb(0.0, 0.0, 1.0).into());

    commands
        // X handle
        .spawn(PbrBundle {
            mesh: cone.clone(),
            material: red.clone(),
            transform: Transform {
                translation: Vec3::new(10.0, 0.0, 0.0),
                rotation: Quat::from_rotation_y(std::f32::consts::PI / 2.),
                scale: Vec3::new(2.6, 2.6, 1.),
            },
            visible: Visible {
                is_visible: false,
                is_transparent: false,
            },
            ..Default::default()
        })
        .with(TranslateHandle::X)
        .with(Gizmo::X)
        .with(bevy_mod_picking::PickableMesh::default())
        .with(Selectable)
        // X arm
        .spawn(PbrBundle {
            mesh: arm.clone(),
            material: red.clone(),
            transform: Transform {
                translation: Vec3::new(4.5, 0.0, 0.0),
                rotation: Quat::from_rotation_y(std::f32::consts::PI / 2.),
                scale: Vec3::new(1.2, 1.2, 4.5),
            },
            visible: Visible {
                is_visible: false,
                is_transparent: false,
            },
            ..Default::default()
        })
        .with(Gizmo::X)
        // Y handle
        .spawn(PbrBundle {
            mesh: cone.clone(),
            material: green.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 10.0, 0.0),
                rotation: Quat::from_rotation_x(-std::f32::consts::PI / 2.),
                scale: Vec3::new(2.6, 2.6, 1.),
            },
            visible: Visible {
                is_visible: false,
                is_transparent: false,
            },
            ..Default::default()
        })
        .with(TranslateHandle::Y)
        .with(Gizmo::Y)
        .with(bevy_mod_picking::PickableMesh::default())
        .with(Selectable)
        // Y arm
        .spawn(PbrBundle {
            mesh: arm.clone(),
            material: green.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 4.5, 0.0),
                rotation: Quat::from_rotation_y(-std::f32::consts::PI / 2.)
                    .mul_quat(Quat::from_rotation_x(-std::f32::consts::PI / 2.)),
                scale: Vec3::new(1.2, 1.2, 4.5),
            },
            visible: Visible {
                is_visible: false,
                is_transparent: false,
            },
            ..Default::default()
        })
        .with(Gizmo::Y)
        // Z handle
        .spawn(PbrBundle {
            mesh: cone.clone(),
            material: blue.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 10.0),
                rotation: Quat::from_rotation_z(std::f32::consts::PI / 2.),
                scale: Vec3::new(2.6, 2.6, 1.),
            },
            visible: Visible {
                is_visible: false,
                is_transparent: false,
            },
            ..Default::default()
        })
        .with(TranslateHandle::Z)
        .with(Gizmo::Z)
        .with(bevy_mod_picking::PickableMesh::default())
        .with(Selectable)
        // Z arm
        .spawn(PbrBundle {
            mesh: arm.clone(),
            material: blue.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 4.5),
                rotation: Quat::identity(),
                scale: Vec3::new(1.2, 1.2, 4.5),
            },
            visible: Visible {
                is_visible: false,
                is_transparent: false,
            },
            ..Default::default()
        })
        .with(Gizmo::Z);
}

use bevy::render::mesh::Indices;
use bevy::render::pipeline::*;
use genmesh::{generators::*, Triangulate};

fn cone() -> Mesh {
    let s = Cone::new(12);
    let positions: Vec<[f32; 3]> = s
        .shared_vertex_iter()
        .map(|v| [v.pos.x, v.pos.y, v.pos.z])
        .collect();
    let normals: Vec<[f32; 3]> = s
        .shared_vertex_iter()
        .map(|v| [v.normal.x, v.normal.y, v.normal.z])
        .collect();
    let uvs: Vec<[f32; 2]> = (0..s.shared_vertex_count())
        .into_iter()
        .map(|_| [0., 0.])
        .collect();

    let indices = Indices::U32(
        s.indexed_polygon_iter()
            .triangulate()
            .map(|tr| vec![tr.x as u32, tr.y as u32, tr.z as u32])
            .flatten()
            .collect(),
    );

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(indices));
    mesh
}

fn cylinder() -> Mesh {
    let s = Cylinder::new(12);

    let positions: Vec<[f32; 3]> = s
        .shared_vertex_iter()
        .map(|v| [v.pos.x, v.pos.y, v.pos.z])
        .collect();
    let normals: Vec<[f32; 3]> = s
        .shared_vertex_iter()
        .map(|v| [v.normal.x, v.normal.y, v.normal.z])
        .collect();
    let uvs: Vec<[f32; 2]> = (0..s.shared_vertex_count())
        .into_iter()
        .map(|_| [0., 0.])
        .collect();

    let indices = Indices::U32(
        s.indexed_polygon_iter()
            .triangulate()
            .map(|tr| vec![tr.x as u32, tr.y as u32, tr.z as u32])
            .flatten()
            .collect(),
    );

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(indices));
    mesh
}
