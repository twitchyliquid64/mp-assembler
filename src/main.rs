use bevy::{prelude::*, render::camera::PerspectiveProjection};
use bevy_4x_camera::{CameraRig, CameraRigBundle, FourXCameraPlugin, KeyboardConf, MouseConf};
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

mod parts;

#[derive(Inspectable, Debug, Default)]
struct Data {
    #[inspectable(label = "Part position")]
    transform: Transform,
}

fn data_ui(
    data: Res<Data>,
    egui: Res<bevy_egui::EguiContext>,
    mut pcbs: Query<&mut Transform, With<parts::Pcb>>,
    mut cameras: Query<&mut CameraRig>,
) {
    // Updates component values from the UI.
    for mut t in pcbs.iter_mut() {
        *t = data.transform;
    }

    for mut c in cameras.iter_mut() {
        c.disable = egui.ctx.wants_mouse_input();
    }
}


fn grid(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let gap = 15.0;
    let count = 10;

    (0..count+1)
        .map(|i| {
            let mut transform = Transform::from_translation(Vec3::new(
                i as f32 * gap - (gap * count as f32 / 2.),
                0.,
                0.,
            ));
            transform.rotate(Quat::from_rotation_x(std::f32::consts::PI / -2.));
            commands.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
                    0.05,
                    gap * count as f32,
                )))),
                material: materials.add(Color::rgb(0.07, 0.06, 0.04).into()),
                transform,
                ..Default::default()
            });

            let mut transform = Transform::from_translation(Vec3::new(
                0.,
                0.,
                i as f32 * gap - (gap * count as f32 / 2.),
            ));
            transform.rotate(Quat::from_rotation_x(std::f32::consts::PI / -2.));
            commands.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
                    gap * count as f32,
                    0.05,
                )))),
                material: materials.add(Color::rgb(0.07, 0.06, 0.04).into()),
                transform,
                ..Default::default()
            });
        })
        .for_each(drop);
}

fn startup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    grid(commands, meshes, &mut materials);
    commands
        // lights
        .spawn(LightBundle {
            light: Light {
                color: Color::rgb(0.8, 0.8, 0.8),
                depth: 0.1..90.0,
                fov: f32::to_radians(20.0),
            },
            transform: Transform::from_translation(Vec3::new(20.0, 80.0, 40.0)),
            ..Default::default()
        })
        .spawn(LightBundle {
            light: Light {
                color: Color::rgb(0.7, 0.7, 0.7),
                depth: 0.1..90.0,
                fov: f32::to_radians(20.0),
            },
            transform: Transform::from_translation(Vec3::new(-40.0, -180.0, 40.0)),
            ..Default::default()
        })
        // camera
        .spawn(CameraRigBundle {
            camera_rig: CameraRig {
                keyboard: KeyboardConf {
                    forward: Box::new([KeyCode::W]),
                    backward: Box::new([KeyCode::S]),
                    left: Box::new([KeyCode::A]),
                    right: Box::new([KeyCode::D]),
                    move_sensitivity: (2.3, 0.9),
                    ..KeyboardConf::default()
                },
                mouse: MouseConf {
                    zoom_sensitivity: 9.,
                    drag_sensitivity: (13.3, std::f32::consts::PI / 42.),
                    ..MouseConf::default()
                },
                ..CameraRig::default()
            },
            ..CameraRigBundle::default()
        })
        .with_children(|cb| {
            cb.spawn(Camera3dBundle {
                // I recommend setting the fov to a low value to get a
                // a pseudo-orthographic perspective
                perspective_projection: PerspectiveProjection {
                    fov: 0.27,
                    near: 0.1,
                    far: 1500.0,
                    aspect_ratio: 1.0,
                },
                transform: Transform::from_translation(Vec3::new(-300.0, 30., 0.0))
                    .looking_at(Vec3::zero(), Vec3::unit_y()),
                ..Default::default()
            });
        });

    // Actual PCB
    parts::spawn_pcb(
        commands,
        &asset_server,
        &mut materials,
        parts::PcbBundle::new_with_stl("train_base.stl"),
    );

    parts::spawn_m3_screw(
        commands,
        &asset_server,
        materials,
        Transform::from_translation(Vec3::new(0., 10., 0.)),
        10,
    );
    println!("ye");
}

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_stl::StlPlugin)
        .add_plugin(FourXCameraPlugin)
        .add_startup_system(startup.system())
        .add_system(data_ui.system())
        .add_plugin(InspectorPlugin::<Data>::new())
        .run();
}
