use bevy::{prelude::*, render::camera::PerspectiveProjection};
use bevy_4x_camera::{CameraRig, CameraRigBundle, FourXCameraPlugin, KeyboardConf, MouseConf};
use bevy_egui::EguiContext;
use bevy_mod_picking::*;

use structopt::StructOpt;

mod gizmo;
mod grid;
mod gui;
mod parts;
mod interaction;

fn interaction_state(
    egui: Res<EguiContext>,
    sel: Res<interaction::Selection>,
    mut cameras: Query<&mut CameraRig>,
    mut pick_state: ResMut<PickState>,
) {
    let using_gui = egui.ctx.wants_mouse_input();
    for mut c in cameras.iter_mut() {
        c.disable = using_gui || sel.dragging_gizmo;
    }
    pick_state.enabled = !using_gui;
}

fn startup(commands: &mut Commands) {
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
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "mp-assembler", about = "Visualize maker-panel geometry")]
struct Opt {
    spec_dirs: Vec<String>,
}

fn load_specs(spec_dirs: &Vec<String>) -> Result<Vec<parts::PanelInfo>, std::io::Error> {
    let mut out = Vec::new();

    for dir in spec_dirs {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(std::ffi::OsStr::to_str) == Some("spec") {
                let spec = parts::PanelInfo::new(
                    entry.path().to_str().unwrap().to_string(),
                    String::from_utf8_lossy(&std::fs::read(entry.path())?).to_string(),
                );
                out.push(spec);
            }
        }
    }

    Ok(out)
}

fn main() {
    let args = Opt::from_args();
    let specs = load_specs(&args.spec_dirs).unwrap();

    App::build()
        .add_resource(gui::Library(specs))
        .add_resource(Msaa { samples: 8 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(bevy_stl::StlPlugin)
        .add_plugin(FourXCameraPlugin)
        .add_startup_system(startup.system())
        .add_system(interaction_state.system())
        .add_plugin(grid::Plugin)
        .add_plugin(gizmo::Plugin)
        .add_plugin(interaction::Plugin)
        .add_plugin(gui::Plugin)
        .add_plugin(parts::Plugin)
        .run();
}
