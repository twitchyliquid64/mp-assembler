use bevy::{prelude::*, render::camera::PerspectiveProjection};
use bevy_4x_camera::{CameraRig, CameraRigBundle, FourXCameraPlugin, KeyboardConf, MouseConf};
use bevy_egui::EguiContext;
use bevy_mod_picking::*;

mod grid;
mod gui;
mod parts;
mod selection;

fn interaction_state(
    egui: Res<EguiContext>,
    mut cameras: Query<&mut CameraRig>,
    mut pick_state: ResMut<PickState>,
) {
    let using_gui = egui.ctx.wants_mouse_input();
    for mut c in cameras.iter_mut() {
        c.disable = using_gui;
    }
    pick_state.enabled = !using_gui;
}

fn startup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
            cb.with(PickSource::default());
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
        .add_plugin(PickingPlugin)
        .add_plugin(bevy_stl::StlPlugin)
        .add_plugin(FourXCameraPlugin)
        .add_startup_system(startup.system())
        .add_system(interaction_state.system())
        .add_plugin(grid::Plugin)
        .add_plugin(selection::Plugin)
        .add_plugin(gui::Plugin)
        .run();
}
