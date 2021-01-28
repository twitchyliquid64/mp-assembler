use crate::selection::{Selectable, Selection};
use bevy::prelude::*;

/// Component that is present on all gizmo children.
#[derive(Debug, Default)]
pub struct Gizmo;

/// Component that is present on translate gizmo handles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TranslateHandle {
    X,
    Y,
    Z,
}

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(gizmo_visibility.system());
    }
}

fn gizmo_visibility(
    selection: Res<Selection>,
    mut gizmos: Query<(&mut Visible, &Parent), With<Gizmo>>,
) {
    for (mut vis, parent) in gizmos.iter_mut() {
        vis.is_visible = Some(parent.0) == selection.entity;
    }
}

pub fn spawn_translate(
    commands: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    mut meshes: &mut ResMut<Assets<Mesh>>,
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
                scale: Vec3::new(2.6, 2.6, 2.6),
            },
            visible: Visible {
                is_visible: false,
                is_transparent: false,
            },
            ..Default::default()
        })
        .with(TranslateHandle::X)
        .with(Gizmo)
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
        .with(Gizmo)
        // Y handle
        .spawn(PbrBundle {
            mesh: cone.clone(),
            material: green.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 10.0, 0.0),
                rotation: Quat::from_rotation_x(-std::f32::consts::PI / 2.),
                scale: Vec3::new(2.6, 2.6, 2.6),
            },
            visible: Visible {
                is_visible: false,
                is_transparent: false,
            },
            ..Default::default()
        })
        .with(TranslateHandle::Y)
        .with(Gizmo)
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
        .with(Gizmo)
        // Z handle
        .spawn(PbrBundle {
            mesh: cone.clone(),
            material: blue.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 10.0),
                rotation: Quat::from_rotation_z(std::f32::consts::PI / 2.),
                scale: Vec3::new(2.6, 2.6, 2.6),
            },
            visible: Visible {
                is_visible: false,
                is_transparent: false,
            },
            ..Default::default()
        })
        .with(TranslateHandle::Z)
        .with(Gizmo)
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
        .with(Gizmo);
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
