pub use crate::interaction::Selectable;
use bevy::prelude::*;
use bevy::render::texture::{Extent3d, TextureDimension, TextureFormat};
use maker_panel::{Panel, SpecErr};
use serde::{Deserialize, Serialize};

use crate::inspector_gui::SpawnPartEvent;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(spawner.system());
    }
}

#[derive(Debug, Clone)]
pub struct ScrewLength(pub usize);

impl Default for ScrewLength {
    fn default() -> Self {
        ScrewLength(8)
    }
}

/// Component that is present on all screw entities
#[derive(Serialize, Deserialize, Debug, Clone)]
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
    length: ScrewLength,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

/// Component that is present on all washer entities
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Washer {
    M3,
    M5,
}

impl Default for Washer {
    fn default() -> Self {
        Washer::M3
    }
}

/// Bundle to make it easy to construct washer entities.
#[derive(Bundle, Debug, Default)]
pub struct WasherBundle {
    washer: Washer,
    selectable: Selectable,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

/// Component that is present on all nut entities
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Nut {
    M3,
    M5,
}

impl Default for Nut {
    fn default() -> Self {
        Nut::M3
    }
}

/// Bundle to make it easy to construct nut entities.
#[derive(Bundle, Debug, Default)]
pub struct NutBundle {
    nut: Nut,
    selectable: Selectable,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(Debug, Clone)]
pub struct PanelDecorations {
    pub color: [f32; 3],
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

    pub fn split(self) -> (String, String, bool) {
        (self.path, self.spec, self.convex_hull)
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
    decorations: PanelDecorations,
    pcb: Pcb,
    selectable: Selectable,
    pub transform: Transform,
    pub global_transform: GlobalTransform,

    geometry: Geometry,
}

impl PcbBundle {
    pub fn new_with_panel(
        panel: PanelInfo,
        transform: Transform,
        decorations: PanelDecorations,
    ) -> Self {
        let path = panel.path.clone();
        Self {
            panel,
            transform,
            decorations,
            pcb: Pcb::default(),
            selectable: Selectable::default(),
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

fn spawn_pcb(
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

fn spawn_screw(
    screw: Screw,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    mut meshes: &mut ResMut<Assets<Mesh>>,
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
            length: ScrewLength(length),
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

fn spawn_washer(
    washer: Washer,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut transform: Transform,
) {
    let stainless = materials.add(StandardMaterial {
        albedo: Color::rgb(0.79, 0.8, 0.81).into(),
        ..Default::default()
    });

    if let Washer::M5 = washer {
        transform.scale = Vec3::new(1. / 3. * 5., 1. / 3. * 5., 1.);
    }

    commands
        .spawn(WasherBundle {
            transform,
            washer,
            ..WasherBundle::default()
        })
        .with_children(|parent| {
            crate::gizmo::spawn_translate(parent, &mut meshes, &mut materials);

            let w_mesh = asset_server.load("m3-washer.stl");
            parent
                .spawn(PbrBundle {
                    mesh: w_mesh.clone(),
                    material: stainless.clone(),
                    ..Default::default()
                })
                .with(bevy_mod_picking::PickableMesh::default().with_bounding_sphere(w_mesh));
        });
}

fn spawn_nut(
    nut: Nut,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut transform: Transform,
) {
    let stainless = materials.add(StandardMaterial {
        albedo: Color::rgb(0.79, 0.8, 0.81).into(),
        ..Default::default()
    });

    if let Nut::M5 = nut {
        transform.scale = Vec3::new(1. / 3. * 5., 1. / 3. * 5., 1.);
    }

    commands
        .spawn(NutBundle {
            transform,
            nut,
            ..NutBundle::default()
        })
        .with_children(|parent| {
            crate::gizmo::spawn_translate(parent, &mut meshes, &mut materials);

            let w_mesh = asset_server.load("m3-nut.stl");
            parent
                .spawn(PbrBundle {
                    mesh: w_mesh.clone(),
                    material: stainless.clone(),
                    ..Default::default()
                })
                .with(bevy_mod_picking::PickableMesh::default().with_bounding_sphere(w_mesh));
        });
}

fn build_panel_texture(
    atoms: Vec<maker_panel::features::InnerAtom>,
    color: &[f32; 3],
    vertexes: &Vec<[f64; 3]>,
) -> Texture {
    use maker_panel::features::InnerAtom;
    use raqote::*;
    use std::f32::{consts::PI, MAX};

    let ((x_min, x_max), (y_min, y_max)) = vertexes.iter().fold(
        ((MAX, -MAX), (MAX, -MAX)),
        |((x_min, x_max), (y_min, y_max)), v| {
            let (x, y) = (v[0] as f32, v[1] as f32);
            ((x_min.min(x), x_max.max(x)), (y_min.min(y), y_max.max(y)))
        },
    );

    let avg_dims = ((x_max - x_min) + (y_max - y_min)) / 2.;
    let (width, height) = if avg_dims < 31. {
        (500, 500)
    } else if avg_dims < 48. {
        (1536, 1536)
    } else if avg_dims < 60. {
        (2048, 2048)
    } else {
        (3072, 2048)
    };
    let mut dt = DrawTarget::new(width, height);
    dt.clear(SolidSource::from_unpremultiplied_argb(
        255,
        (color[0] * 255.).ceil() as u8,
        (color[1] * 255.).ceil() as u8,
        (color[2] * 255.).ceil() as u8,
    ));

    // let (dx, dy) = ((xb.1 - xb.0).ceil() as usize, (yb.1 - yb.0).ceil() as usize);
    let map = |x: f32, y: f32| {
        (
            ((x - x_min) / (x_max - x_min)) * (width as f32 - 0.) + 0.,
            ((y - y_min) / (y_max - y_min)) * (height as f32 - 0.) + 0.,
        )
    };

    for a in atoms {
        match a {
            InnerAtom::Rect { rect, layer } => {
                if layer == maker_panel::Layer::FrontMask {
                    let (x1, y1) = rect.min().x_y();
                    let (x1, y1) = map(x1 as f32, y1 as f32);
                    let (x2, y2) = rect.max().x_y();
                    let (x2, y2) = map(x2 as f32, y2 as f32);

                    let mut pb = PathBuilder::new();
                    pb.rect(x1, y1, x2 - x1, y2 - y1);
                    dt.fill(
                        &pb.finish(),
                        &Source::Solid(SolidSource {
                            r: 151,
                            g: 149,
                            b: 152,
                            a: 0xff,
                        }),
                        &DrawOptions::new(),
                    );
                }
            }
            InnerAtom::Circle {
                center,
                radius,
                layer,
            } => {
                if layer == maker_panel::Layer::FrontMask {
                    let mut pb = PathBuilder::new();

                    for i in 0..=48 {
                        let angle = i as f32 * PI / 24.;
                        let (sin_theta, cos_theta) = angle.sin_cos();
                        let dx = radius as f32;
                        let dy = 0.;
                        let (x2, y2) = map(
                            dx * cos_theta - dy * sin_theta + center.x as f32,
                            dx * sin_theta + dy * cos_theta + center.y as f32,
                        );
                        if i == 0 {
                            pb.move_to(x2, y2);
                        } else {
                            pb.line_to(x2, y2);
                        }
                    }
                    pb.close();

                    // pb.move_to(center.0, center.1);
                    // pb.arc(center.0, center.1, size.0 / 2., 0., 2. * std::f32::consts::PI);
                    dt.fill(
                        &pb.finish(),
                        &Source::Solid(SolidSource {
                            r: 151,
                            g: 149,
                            b: 152,
                            a: 0xff,
                        }),
                        &DrawOptions::new(),
                    );
                }
            }
            _ => {}
        }
    }

    let data: Vec<u8> = dt
        .into_vec()
        .into_iter()
        .map(|p| {
            vec![
                (p & 0xff) as u8,
                ((p >> 8) & 0xff) as u8,
                ((p >> 16) & 0xff) as u8,
                ((p >> 24) & 0xff) as u8,
            ]
        })
        .flatten()
        .collect();

    Texture::new(
        Extent3d::new(width as u32, height as u32, 1),
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
    )
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

    use std::f32::MAX;
    let ((x_min, x_max), (y_min, y_max)) = vertexes.iter().fold(
        ((MAX, -MAX), (MAX, -MAX)),
        |((x_min, x_max), (y_min, y_max)), v| {
            let (x, y) = (v[0], v[1]);
            ((x_min.min(x), x_max.max(x)), (y_min.min(y), y_max.max(y)))
        },
    );
    let uvs = vertexes
        .iter()
        .map(|v| {
            [
                (v[0] - x_min) / (x_max - x_min),
                (v[1] - y_min) / (y_max - y_min),
            ]
        })
        .collect();

    mesh.set_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float3(vertexes),
    );
    mesh.set_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float3(normals),
    );
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float2(uvs));
    mesh.set_indices(Some(Indices::U16(indices)));

    mesh
}

fn spawner(
    ev_spawn: Res<Events<SpawnPartEvent>>,
    mut spawn_reader: Local<EventReader<SpawnPartEvent>>,

    mut commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut textures: ResMut<Assets<Texture>>,
) {
    for ev in spawn_reader.iter(&ev_spawn) {
        match ev {
            SpawnPartEvent::Panel(panel, convex_hull, color, transform) => {
                let mut p = panel.panel();
                p.convex_hull(*convex_hull);

                let tessellation = p.tessellate_3d().unwrap();
                let t = build_panel_texture(p.interior_geometry(), color, &tessellation.0);
                let mesh = meshes.add(build_panel_mesh(tessellation));
                let material = materials.add(StandardMaterial {
                    albedo_texture: Some(textures.add(t)),
                    ..StandardMaterial::default()
                });

                let transform = if let Some(t) = transform {
                    t.clone()
                } else {
                    Transform::identity()
                };

                spawn_pcb(
                    &mut commands,
                    &mut materials,
                    &mut meshes,
                    PcbBundle::new_with_panel(
                        panel.clone(),
                        transform,
                        PanelDecorations {
                            color: color.clone(),
                        },
                    ),
                    PbrBundle {
                        mesh,
                        material,
                        ..Default::default()
                    },
                )
            }
            SpawnPartEvent::Screw(screw, length, transform) => {
                let transform = if let Some(t) = transform {
                    t.clone()
                } else {
                    Transform::from_translation(Vec3::new(0., 10., 0.))
                };

                spawn_screw(
                    screw.clone(),
                    &mut commands,
                    &asset_server,
                    &mut materials,
                    &mut meshes,
                    transform,
                    *length,
                );
            }
            SpawnPartEvent::Washer(washer, transform) => {
                let transform = if let Some(t) = transform {
                    t.clone()
                } else {
                    Transform::from_translation(Vec3::new(0., 10., 0.))
                };

                spawn_washer(
                    washer.clone(),
                    &mut commands,
                    &asset_server,
                    &mut materials,
                    &mut meshes,
                    transform,
                );
            }
            SpawnPartEvent::Nut(nut, transform) => {
                let transform = if let Some(t) = transform {
                    t.clone()
                } else {
                    Transform::from_translation(Vec3::new(0., 10., 0.))
                };

                spawn_nut(
                    nut.clone(),
                    &mut commands,
                    &asset_server,
                    &mut materials,
                    &mut meshes,
                    transform,
                );
            }
        }
    }
}
