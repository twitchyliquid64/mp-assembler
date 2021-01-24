use bevy::prelude::*;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(grid.system());
    }
}

fn grid(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let gap = 15.0;
    let count = 10;

    (0..count + 1)
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
