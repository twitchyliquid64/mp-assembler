use bevy::prelude::*;

/// Component that is present on all screw entities
#[derive(Default, Debug)]
pub struct Screw;

/// Bundle to make it easy to construct screw entities.
#[derive(Bundle, Debug, Default)]
pub struct ScrewBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

/// Component that is present on all PCB entities
#[derive(Default, Debug)]
pub struct Pcb;

/// Bundle to make it easy to construct PCB entities with
/// all the usual components present.
#[derive(Bundle, Debug)]
pub struct PcbBundle {
    pub pcb: Pcb,
    pub transform: Transform,
    pub global_transform: GlobalTransform,

    geometry: Geometry,
}

impl PcbBundle {
    pub fn new_with_stl(path: &'static str) -> Self {
        Self {
            pcb: Pcb::default(),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            geometry: Geometry::Stl(path),
        }
    }

    fn make_mesh(
        &self,
        asset_server: &Res<AssetServer>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> PbrBundle {
        PbrBundle {
            mesh: match self.geometry {
                Geometry::Stl(s) => asset_server.load(s),
            },
            material: materials.add(Color::rgb(0.2, 0.5, 0.2).into()),
            transform: Transform::identity(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Geometry {
    Stl(&'static str),
}

pub fn spawn_pcb(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    pcb: PcbBundle,
) {
    let mesh = pcb.make_mesh(asset_server, materials);
    commands.spawn(pcb).with_children(|parent| {
        parent.spawn(mesh);
    });
}

pub fn spawn_m3_screw(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    transform: Transform,
    length: usize,
) {
    // let texture_handle = asset_server.load("worn-shiny-metal-albedo.png");
    let thread_2mm = asset_server.load("m3-2mm.stl");
    let stainless = materials.add(StandardMaterial {
        albedo: Color::rgb(0.79, 0.8, 0.81).into(),
        // albedo_texture: Some(texture_handle.clone()),
        ..Default::default()
    });

    commands
        .spawn(ScrewBundle {
            transform,
            ..ScrewBundle::default()
        })
        .with_children(|parent| {
            let transform = Transform::from_translation(Vec3::new(0., 0., length as f32));
            parent.spawn(PbrBundle {
                mesh: asset_server.load("m3-pan_head.stl"),
                material: stainless.clone(),
                transform,
                ..Default::default()
            });

            for i in 0..(length / 2) {
                parent.spawn(PbrBundle {
                    mesh: thread_2mm.clone(),
                    material: stainless.clone(),
                    transform: Transform::from_translation(Vec3::new(0., 0., i as f32 * 2.0)),
                    ..Default::default()
                });
            }
        });
}
