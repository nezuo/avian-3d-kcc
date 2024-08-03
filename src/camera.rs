use std::f32::consts::PI;

use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_atmosphere::plugin::AtmosphereCamera;

use crate::character_controller::CharacterController;

const CAMERA_DISTANCE: f32 = 10.0;
const SENSITIVITY: f32 = 0.005;
const PITCH_MIN: f32 = -PI / 2.0;
const PITCH_MAX: f32 = PI / 2.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraRotation>()
            .add_systems(Startup, setup_camera)
            .add_systems(Update, (rotate_camera, grab_cursor))
            .add_systems(
                PostUpdate,
                transform_camera
                    // .after(CharacterControllerSet) todo: when to schedule camera
                    .before(TransformSystem::TransformPropagate),
            );
    }
}

#[derive(Resource, Default)]
pub struct CameraRotation {
    pub pitch: f32,
    pub yaw: f32,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 60.0_f32.to_radians(),
                ..default()
            }),
            ..default()
        },
        AtmosphereCamera::default(),
    ));
}

fn grab_cursor(
    mut windows: Query<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    let mut window = windows.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}

fn rotate_camera(
    window: Query<&Window, With<PrimaryWindow>>,
    mut camera_rotation: ResMut<CameraRotation>,
    mut mouse_motion: EventReader<MouseMotion>,
) {
    let window = window.single();

    if window.cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }

    for event in mouse_motion.read() {
        camera_rotation.pitch =
            (camera_rotation.pitch - SENSITIVITY * event.delta.y).clamp(PITCH_MIN, PITCH_MAX);
        camera_rotation.yaw -= SENSITIVITY * event.delta.x;
    }
}

fn transform_camera(
    camera_rotation: Res<CameraRotation>,
    mut camera: Query<&mut Transform, With<Camera>>,
    player: Query<&Transform, (With<CharacterController>, Without<Camera>)>,
) {
    let player_transform = player.single();
    let mut camera_transform = camera.single_mut();

    let rotation =
        Quat::from_rotation_y(camera_rotation.yaw) * Quat::from_rotation_x(camera_rotation.pitch);
    let rotation_matrix = Mat3::from_quat(rotation);

    camera_transform.rotation = rotation;
    camera_transform.translation = player_transform.translation
        + rotation_matrix.mul_vec3(Vec3::new(0.0, 0.0, CAMERA_DISTANCE));
}
