pub use crate::interaction::Selectable;
use bevy::prelude::*;
use maker_panel::{Panel, SpecErr};

use crate::gui::SpawnPanelEvent;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(spawner.system());
    }
}

/// Component that is present on all screw entities
#[derive(Debug)]
pub enum Screw {
    M3,
    M5,
}

impl Default for Screw {
    fn default() -> Self {
        Screw::M3
    }
}

/// Bundle to make it easy to construct screw entities.
#[derive(Bundle, Debug, Default)]
pub struct ScrewBundle {
    screw: Screw,
    selectable: Selectable,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(Debug, Clone)]
pub struct PanelInfo {
    spec: String,
    path: String,
    convex_hull: bool,
    err: Option<SpecErr>,
}

impl PanelInfo {
    pub fn new(path: String, data: String) -> Self {
        let err = Panel::new().push_spec(&data).err();

        Self {
            path,
            err,
            spec: data,
            convex_hull: false,
        }
    }

    pub fn name(&self) -> String {
        self.path.split("/").last().unwrap().to_string()
    }

    pub fn well_formed(&self) -> bool {
        self.err.is_none()
    }

    pub fn panel(&self) -> Panel {
        let mut panel = Panel::new();
        panel.push_spec(&self.spec);
        panel
    }
}

/// Component that is present on all PCB entities
#[derive(Default, Debug)]
pub struct Pcb;

/// Bundle to make it easy to construct PCB entities with
/// all the usual components present.
#[derive(Bundle, Debug)]
pub struct PcbBundle {
    panel: PanelInfo,
    pcb: Pcb,
    selectable: Selectable,
    pub transform: Transform,
    pub global_transform: GlobalTransform,

    geometry: Geometry,
}

impl PcbBundle {
    pub fn new_with_panel(panel: PanelInfo) -> Self {
        let path = panel.path.clone();
        Self {
            panel,
            pcb: Pcb::default(),
            selectable: Selectable::default(),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            geometry: Geometry::Spec(path),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Geometry {
    Stl(&'static str),
    Spec(String),
}

pub fn spawn_pcb(
    commands: &mut Commands,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    pcb: PcbBundle,
    geo: PbrBundle,
) {
    let mesh2 = geo.mesh.clone();

    commands.spawn(pcb).with_children(|parent| {
        crate::gizmo::spawn_translate(parent, &mut meshes, &mut materials);

        parent
            .spawn(geo)
            .with(bevy_mod_picking::PickableMesh::default().with_bounding_sphere(mesh2));
    });
}

pub fn spawn_screw(
    screw: Screw,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut transform: Transform,
    length: usize,
) {
    // let texture_handle = asset_server.load("worn-shiny-metal-albedo.png");
    let thread_2mm = asset_server.load("m3-2mm.stl");
    let stainless = materials.add(StandardMaterial {
        albedo: Color::rgb(0.79, 0.8, 0.81).into(),
        // albedo_texture: Some(texture_handle.clone()),
        ..Default::default()
    });

    if let Screw::M5 = screw {
        transform.scale = Vec3::new(1. / 3. * 5., 1. / 3. * 5., 1.);
    }

    commands
        .spawn(ScrewBundle {
            transform,
            screw,
            ..ScrewBundle::default()
        })
        .with_children(|parent| {
            crate::gizmo::spawn_translate(parent, &mut meshes, &mut materials);

            let transform = Transform::from_translation(Vec3::new(0., 0., length as f32));
            let pan_head = asset_server.load("m3-pan_head.stl");
            parent
                .spawn(PbrBundle {
                    mesh: pan_head.clone(),
                    material: stainless.clone(),
                    transform,
                    ..Default::default()
                })
                .with(bevy_mod_picking::PickableMesh::default().with_bounding_sphere(pan_head));

            for i in 0..(length / 2) {
                parent
                    .spawn(PbrBundle {
                        mesh: thread_2mm.clone(),
                        material: stainless.clone(),
                        transform: Transform::from_translation(Vec3::new(0., 0., i as f32 * 2.0)),
                        ..Default::default()
                    })
                    .with(
                        bevy_mod_picking::PickableMesh::default()
                            .with_bounding_sphere(thread_2mm.clone()),
                    );
            }
        });
}

fn build_panel_mesh(tessellation: (Vec<[f64; 3]>, Vec<u16>)) -> Mesh {
    use bevy::render::{
        mesh::{Indices, VertexAttributeValues},
        pipeline::PrimitiveTopology,
    };
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let (verts, inds) = tessellation;
    let v_conv = |idx: u16| {
        let v = verts[idx as usize];
        [v[0] as f32, v[1] as f32, v[2] as f32]
    };

    let mut vertexes = Vec::with_capacity(inds.len());
    let mut normals = Vec::with_capacity(inds.len());
    let mut indices = Vec::with_capacity(inds.len());

    for (i, tri) in inds.chunks_exact(3).enumerate() {
        let verts = [v_conv(tri[0]), v_conv(tri[1]), v_conv(tri[2])];
        let u = [
            verts[1][0] - verts[0][0],
            verts[1][1] - verts[0][1],
            verts[1][2] - verts[0][2],
        ];
        let v = [
            verts[2][0] - verts[0][0],
            verts[2][1] - verts[0][1],
            verts[2][2] - verts[0][2],
        ];
        let normal = [
            (u[1] * v[2]) - (u[2] * v[1]),
            (u[2] * v[0]) - (u[0] * v[2]),
            (u[0] * v[1]) - (u[1] * v[0]),
        ];

        normals.push(normal);
        normals.push(normal);
        normals.push(normal);

        for j in 0..3 {
            vertexes.push(verts[j]);
            indices.push((i * 3 + j) as u16);
        }
    }

    let uvs = vec![[0.0, 0.0, 0.0]; vertexes.len()];
    mesh.set_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float3(vertexes),
    );
    mesh.set_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float3(normals),
    );
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float3(uvs));
    mesh.set_indices(Some(Indices::U16(indices)));

    mesh
}

fn spawner(
    ev_spawn: Res<Events<SpawnPanelEvent>>,
    mut spawn_reader: Local<EventReader<SpawnPanelEvent>>,

    mut commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for ev in spawn_reader.iter(&ev_spawn) {
        let mut panel = ev.0.panel();
        panel.convex_hull(ev.1);

        let mesh = meshes.add(build_panel_mesh(panel.tessellate_3d().unwrap()));
        let material = materials.add(Color::rgb(ev.2[0], ev.2[1], ev.2[2]).into());

        spawn_pcb(
            &mut commands,
            &mut materials,
            &mut meshes,
            PcbBundle::new_with_panel(ev.0.clone()),
            PbrBundle {
                mesh,
                material,
                transform: Transform::identity(),
                ..Default::default()
            },
        )
    }
}
