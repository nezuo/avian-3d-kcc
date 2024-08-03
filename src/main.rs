mod camera;
mod character_controller;
mod schedule;

use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use avian3d::prelude::*;
use bevy::{color::palettes, prelude::*};
use bevy_atmosphere::prelude::*;
use bevy_debug_text_overlay::{screen_print, OverlayPlugin};
use clap::Parser;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use crate::{
    camera::{CameraPlugin, CameraRotation},
    character_controller::{
        CharacterController, CharacterControllerPlugin, CharacterControllerSet,
    },
    light_consts::lux::AMBIENT_DAYLIGHT,
    schedule::{step_custom_schedule, CustomPreUpdate, CustomUpdate, SchedulePlugin},
};

const PLAYER_SPEED: f32 = 15.0;

#[derive(Resource)]
struct CustomStepping {
    enabled: bool,
}

#[derive(Resource, Default)]
struct FrameCount(u32);

#[derive(Resource, Default, Serialize, Deserialize)]
struct RecordedVelocities(HashMap<u32, Vec3>);

#[derive(Parser, Resource)]
struct Cli {
    #[arg(short)]
    playback: Option<PathBuf>,
}

fn main() -> AppExit {
    let args = Cli::parse();

    let recorded_velocities = match &args.playback {
        Some(playback_path) => {
            ron::de::from_str(&fs::read_to_string(playback_path).unwrap()).unwrap()
        }
        None => RecordedVelocities::default(),
    };

    App::new()
        .add_plugins((
            DefaultPlugins,
            AtmospherePlugin,
            PhysicsPlugins::new(CustomUpdate),
            SchedulePlugin,
            CameraPlugin,
            CharacterControllerPlugin,
            OverlayPlugin {
                font_size: 24.0,
                ..default()
            },
        ))
        .init_resource::<FrameCount>()
        .insert_resource(recorded_velocities)
        .insert_resource(CustomStepping {
            enabled: args.playback.is_some(),
        })
        .insert_resource(args)
        .init_resource::<AtmosphereModel>()
        .add_systems(Startup, (setup_level, setup_character, setup_sun))
        .add_systems(
            CustomPreUpdate,
            (increment_frame, set_velocity)
                .chain()
                .before(CharacterControllerSet),
        )
        .add_systems(Update, (toggle_system_stepping, step))
        .add_systems(Last, serialize_captured_input_on_exit)
        .run()
}

fn toggle_system_stepping(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut custom_stepping: ResMut<CustomStepping>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        custom_stepping.enabled = !custom_stepping.enabled
    }
}

fn step(world: &mut World) {
    let keyboard_input = world.resource::<ButtonInput<KeyCode>>();

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        step_custom_schedule(world);
    }
}

fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // commands.spawn((
    //     RigidBody::Static,
    //     Collider::cuboid(100.0, 0.1, 100.0),
    //     PbrBundle {
    //         mesh: meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.0))),
    //         material: materials.add(Color::Srgba(palettes::css::BLACK)),
    //         ..default()
    //     },
    // ));

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(2.0, 5.0, 10.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::from_size(Vec3::new(2.0, 5.0, 10.0))),
            transform: Transform::from_xyz(15.0, 2.5, 0.0),
            material: materials.add(Color::Srgba(palettes::css::BLACK)),
            ..default()
        },
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(10.0, 5.0, 2.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::from_size(Vec3::new(10.0, 5.0, 2.0))),
            transform: Transform::from_xyz(-15.0, 2.5, 0.0)
                .with_rotation(Quat::from_rotation_y(15.0_f32.to_radians())),
            material: materials.add(Color::Srgba(palettes::css::BLACK)),
            ..default()
        },
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(10.0, 5.0, 2.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::from_size(Vec3::new(10.0, 5.0, 2.0))),
            transform: Transform::from_xyz(0.0, 2.5, 15.0)
                .with_rotation(Quat::from_rotation_y(-15.0_f32.to_radians())),
            material: materials.add(Color::Srgba(palettes::css::BLACK)),
            ..default()
        },
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(2.0, 5.0, 10.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::from_size(Vec3::new(2.0, 5.0, 10.0))),
            transform: Transform::from_xyz(0.0, 2.5, 15.0)
                .with_rotation(Quat::from_rotation_y(35.0_f32.to_radians())),
            material: materials.add(Color::Srgba(palettes::css::BLACK)),
            ..default()
        },
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(2.0, 5.0, 10.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::from_size(Vec3::new(2.0, 5.0, 10.0))),
            transform: Transform::from_xyz(0.0, 2.5, 30.0)
                .with_rotation(Quat::from_rotation_x(35.0_f32.to_radians())),
            material: materials.add(Color::Srgba(palettes::css::BLACK)),
            ..default()
        },
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(10.0, 2.0, 30.0),
        PbrBundle {
            mesh: meshes.add(Cuboid::from_size(Vec3::new(10.0, 2.0, 30.0))),
            transform: Transform::from_xyz(-15.0, -2.0, 30.0)
                .with_rotation(Quat::from_rotation_x(35.0_f32.to_radians())),
            material: materials.add(Color::linear_rgba(0.0, 0.0, 0.0, 0.2)),
            ..default()
        },
    ));
}

fn setup_sun(mut commands: Commands, mut atmosphere: AtmosphereMut<Nishita>) {
    let t: f32 = 1.0;

    atmosphere.sun_position = Vec3::new(0., t.sin(), t.cos());

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_rotation_x(-t)),
        directional_light: DirectionalLight {
            illuminance: t.sin().max(0.0).powf(2.0) * AMBIENT_DAYLIGHT,
            ..default()
        },
        ..default()
    });
}

fn setup_character(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        CharacterController::default(),
        RigidBody::Kinematic,
        Collider::cylinder(0.5, 2.0),
        PbrBundle {
            mesh: meshes.add(Cylinder::new(0.5, 2.0)),
            material: materials.add(Color::Srgba(Srgba::new(1.0, 0.0, 0.0, 0.5))),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        },
    ));
}

fn increment_frame(mut frame_count: ResMut<FrameCount>) {
    frame_count.0 += 1;
    screen_print!("FRAME: {}", frame_count.0);
}

fn set_velocity(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    camera_rotation: Res<CameraRotation>,
    mut query: Query<&mut CharacterController>,
    mut recorded_velocities: ResMut<RecordedVelocities>,
    frame_count: Res<FrameCount>,
    cli: Res<Cli>,
) {
    if cli.playback.is_some() {
        let velocity = recorded_velocities.0.get(&frame_count.0);

        if velocity.is_none() {
            return;
        }

        for mut character_controller in &mut query {
            character_controller.velocity = *velocity.unwrap();
        }

        return;
    }

    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction.z -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.z += 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    let camera_rotation = Mat3::from_quat(Quat::from_rotation_y(camera_rotation.yaw));
    let move_direction = camera_rotation.mul_vec3(direction);

    for mut character_controller in &mut query {
        character_controller.velocity = move_direction.normalize_or_zero() * PLAYER_SPEED;
        recorded_velocities
            .0
            .insert(frame_count.0, character_controller.velocity);
    }
}

fn serialize_captured_input_on_exit(
    app_exit_events: EventReader<AppExit>,
    recorded_velocities: Res<RecordedVelocities>,
    cli: Res<Cli>,
) {
    if !app_exit_events.is_empty() && cli.playback.is_none() {
        serialize_timestamped_inputs(&recorded_velocities);
    }
}

fn serialize_timestamped_inputs(recorded_velocities: &RecordedVelocities) {
    let file_path = Path::new("out.ron");

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(file_path)
        .expect("Could not open file.");

    write!(
        file,
        "{}",
        ron::ser::to_string_pretty(recorded_velocities, PrettyConfig::default())
            .expect("Could not convert captured input to a string.")
    )
    .expect("Could not write string to file.");
}
