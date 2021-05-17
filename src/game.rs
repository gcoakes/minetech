use std::{
    f32::consts::PI,
    ops::{Mul, Neg},
};

use bevy::prelude::*;
use bevy_advanced_input::{
    input_id::InputID,
    plugin::InputBindingPlugin,
    user_input::{InputAxisType, MouseAxisType, UserInputHandle, UserInputSet},
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(InputBindingPlugin::<InputType, Bindings>::default())
            .add_startup_system(setup_input.system())
            .add_startup_system(spawn_player.system())
            .add_startup_system(spawn_scene.system())
            .add_system(player_control.system())
            .add_system(movement.system());
    }
}

struct FPSCameraMovement {
    pitch: f32,
    yaw: f32,
    velocity: Vec3,
}

struct Player;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum InputType {
    Game,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Bindings {
    Longitudinal,
    Latitudinal,
    Elevation,
    Pitch,
    Yaw,
}

fn setup_input(mut input_bindings: ResMut<UserInputHandle<InputType, Bindings>>) {
    let mut input_set = UserInputSet::new();

    input_set
        .begin_axis(Bindings::Longitudinal)
        .add(
            InputAxisType::GamepadAxis(GamepadAxisType::LeftStickY),
            None,
        )
        .add(InputAxisType::KeyboardButton(KeyCode::W), Some(1.0))
        .add(InputAxisType::KeyboardButton(KeyCode::S), Some(-1.0));

    input_set
        .begin_axis(Bindings::Latitudinal)
        .add(
            InputAxisType::GamepadAxis(GamepadAxisType::LeftStickX),
            None,
        )
        .add(InputAxisType::KeyboardButton(KeyCode::D), Some(1.0))
        .add(InputAxisType::KeyboardButton(KeyCode::A), Some(-1.0));

    input_set
        .begin_axis(Bindings::Elevation)
        .add(InputAxisType::KeyboardButton(KeyCode::E), Some(1.0))
        .add(InputAxisType::KeyboardButton(KeyCode::Q), Some(-1.0));

    input_set
        .begin_axis(Bindings::Pitch)
        .add(InputAxisType::MouseAxisDiff(MouseAxisType::Y), None)
        .add(
            InputAxisType::GamepadAxis(GamepadAxisType::RightStickY),
            None,
        );

    input_set
        .begin_axis(Bindings::Yaw)
        .add(InputAxisType::MouseAxisDiff(MouseAxisType::X), None)
        .add(
            InputAxisType::GamepadAxis(GamepadAxisType::RightStickX),
            None,
        );

    input_bindings.add_input(InputType::Game, input_set);
}

fn spawn_scene(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let texture: Handle<Texture> = asset_server.load("dirt.png");
    let mesh = meshes.add(Mesh::from(shape::Cube { size: 0.45 }));
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        base_color: Color::hex("ffffff").unwrap(),
        metallic: 1.0,
        roughness: 1.0,
        ..Default::default()
    });

    for y in -2..=1 {
        for x in -5..=5 {
            cmds.spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform: Transform::from_xyz(x as f32, y as f32, 0.0),
                ..Default::default()
            });
        }
    }
}

fn spawn_player(
    mut cmds: Commands,
    mut input_bindings: ResMut<UserInputHandle<InputType, Bindings>>,
) {
    cmds.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 5.0, 5.0)),
        ..Default::default()
    });
    cmds.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 8.0))
            .looking_at(Vec3::Y, Vec3::Z),
        ..Default::default()
    })
    .insert(FPSCameraMovement {
        pitch: 0.0,
        yaw: 0.0,
        velocity: Vec3::ZERO,
    })
    .insert(Player)
    .insert(input_bindings.create_input_id(InputType::Game));
}

const MAX_PITCH: f32 = PI / 2.0;
const MIN_PITCH: f32 = -PI / 2.0;

fn player_control(
    input_bindings: Res<UserInputHandle<InputType, Bindings>>,
    mut controlled: Query<(&mut FPSCameraMovement, &Transform, &InputID)>,
) {
    for (mut movement, transform, input_id) in controlled.iter_mut() {
        if let Some(input_handle) = input_bindings.to_handle(input_id) {
            let walk_unit = transform
                .local_z()
                .mul(Vec3::X + Vec3::Z)
                .neg()
                .normalize_or_zero();
            let right_unit = transform
                .local_x()
                .mul(Vec3::X + Vec3::Z)
                .normalize_or_zero();
            let walkness = input_handle
                .get_axis_value(Bindings::Longitudinal)
                .unwrap_or(0.0);
            let rightness = input_handle
                .get_axis_value(Bindings::Latitudinal)
                .unwrap_or(0.0);
            let jumpness = input_handle
                .get_axis_value(Bindings::Elevation)
                .unwrap_or(0.0);
            movement.velocity =
                (walk_unit * walkness + right_unit * rightness + Vec3::Y * jumpness)
                    .normalize_or_zero();

            let delta = -0.001
                * input_handle
                    .get_axis_value(Bindings::Yaw)
                    .zip(input_handle.get_axis_value(Bindings::Pitch))
                    .map(|(y, p)| Vec3::new(y, p, 0.0))
                    .unwrap_or(Vec3::ZERO);
            movement.yaw += delta.x;
            movement.pitch = (delta.y + movement.pitch).clamp(MIN_PITCH, MAX_PITCH);
        }
    }
}

fn movement(time: Res<Time>, mut players: Query<(&mut Transform, &FPSCameraMovement)>) {
    for (mut transform, movement) in players.iter_mut() {
        transform.translation += movement.velocity * time.delta_seconds();
        transform.rotation = Quat::from_rotation_ypr(movement.yaw, movement.pitch, 0.0);
    }
}
