use bevy::{input::mouse::MouseMotion, prelude::*};

#[repr(transparent)]
struct Velocity(Vec3);

/// Yaw and pitch rotation in terms of radians.
#[repr(transparent)]
struct YPRotation(Vec2);

struct Player;

pub fn run() {
    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .query_selector("#game")
        .unwrap()
        .unwrap();

    let mut builder = App::build();
    let mut win_desc = WindowDescriptor {
        resizable: true,
        width: canvas.client_width() as f32,
        height: canvas.client_height() as f32,
        cursor_locked: true,
        ..Default::default()
    };

    #[cfg(target_arch = "wasm32")]
    {
        win_desc.canvas = Some("#game".to_string());
    }

    builder
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(win_desc)
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(input.system())
        .add_system(movement.system())
        .add_system(mouse_look.system());

    #[cfg(target_arch = "wasm32")]
    {
        builder.add_plugin(bevy_webgl2::WebGL2Plugin);
    }

    builder.run();
}

fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for y in -2..=2 {
        for x in -5..=5 {
            let x01 = (x + 5) as f32 / 10.0;
            let y01 = (y + 2) as f32 / 4.0;
            // sphere
            cmds.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: 0.45,
                    subdivisions: 32,
                })),
                material: materials.add(StandardMaterial {
                    base_color: Color::hex("ffd891").unwrap(),
                    // vary key PBR parameters on a grid of spheres to show the effect
                    metallic: y01,
                    roughness: x01,
                    ..Default::default()
                }),
                transform: Transform::from_xyz(x as f32, y as f32, 0.0),
                ..Default::default()
            });
        }
    }
    cmds.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, -5.0)),
        ..Default::default()
    });
    cmds.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -8.0))
            .looking_at(Vec3::Y, Vec3::Z),
        ..Default::default()
    })
    .insert(Velocity(Vec3::ZERO))
    .insert(YPRotation(Vec2::ZERO))
    .insert(Player);
}

fn input(keys: Res<Input<KeyCode>>, mut velocities: Query<&mut Velocity, With<Player>>) {
    for mut velocity in velocities.iter_mut() {
        velocity.0.z =
            (keys.pressed(KeyCode::S) as i32 + -(keys.pressed(KeyCode::W) as i32)) as f32;
        velocity.0.x =
            (keys.pressed(KeyCode::D) as i32 + -(keys.pressed(KeyCode::A) as i32)) as f32;
    }
}

fn mouse_look(
    mut mouse_events: EventReader<MouseMotion>,
    mut transforms_rotations: Query<(&mut Transform, &mut YPRotation), With<Player>>,
) {
    let delta: Vec2 = mouse_events
        .iter()
        .map(|ev| ev.delta)
        .fold(Vec2::ZERO, |acc, delta| acc + delta);

    for (mut transform, mut ypr) in transforms_rotations.iter_mut() {
        ypr.0 += delta / 1000.0;
        let rotation = Quat::from_rotation_ypr(ypr.0.x, ypr.0.y, 0.0);
        transform.look_at(Vec3::Y, Vec3::Z);
        transform.rotate(rotation);
    }
}

fn movement(time: Res<Time>, mut players: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in players.iter_mut() {
        if velocity.0 != Vec3::ZERO {
            let delta = transform.rotation * velocity.0;
            transform.translation += delta * time.delta_seconds();
        }
    }
}
