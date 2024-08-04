use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_debug_text_overlay::screen_print;

use crate::schedule::{CustomLast, CustomPostUpdate};

const MAX_BOUNCES: u8 = 5;
const SKIN_WIDTH: f32 = 0.005;

#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Clone)]
pub struct CharacterControllerSet;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            CustomPostUpdate,
            move_character_controllers.in_set(CharacterControllerSet),
        )
        .add_systems(CustomLast, print_collisions);
    }
}

#[derive(Component, Default)]
pub struct CharacterController {
    pub velocity: Vec3, // todo: this is a Vec3 but do we support vertical movement?
}

fn print_collisions(
    mut collision_event_reader: EventReader<Collision>,
    character_controllers: Query<&CharacterController>,
) {
    let has_collision = collision_event_reader.read().any(|Collision(contacts)| {
        character_controllers.contains(contacts.entity1)
            || character_controllers.contains(contacts.entity2)
    });

    screen_print!(
        "{}",
        if has_collision {
            "Colliding"
        } else {
            "Not colliding"
        }
    );
}

fn move_character_controllers(
    mut query: Query<(Entity, &CharacterController, &Collider, &mut Transform)>,
    spatial_query: SpatialQuery,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    for (entity, character_controller, collider, mut transform) in &mut query {
        let mut direction_result = Dir3::new(character_controller.velocity);
        let mut distance = character_controller.velocity.length() * time.delta_seconds();

        let Ok(start_direction) = direction_result else {
            continue;
        };

        let mut bounce_count = 0;
        let mut hit_count = 0;
        let mut planes = Vec::new();

        for _ in 0..MAX_BOUNCES {
            bounce_count += 1;

            if let Ok(direction) = direction_result {
                gizmos.ray(
                    transform.translation,
                    direction.as_vec3(),
                    Color::linear_rgb(1.0, 0.0, 0.0),
                );

                if let Some(hit) = spatial_query.cast_shape(
                    collider,
                    transform.translation,
                    transform.rotation,
                    direction,
                    distance + SKIN_WIDTH,
                    true,
                    SpatialQueryFilter::from_excluded_entities([entity]),
                ) {
                    hit_count += 1;

                    screen_print!("normal: {}", hit.normal1);

                    let hit_point = *transform * hit.point2;

                    gizmos.sphere(hit_point, Quat::IDENTITY, 0.1, Color::WHITE);

                    if hit.time_of_impact >= distance {
                        transform.translation +=
                            direction * (hit.time_of_impact - SKIN_WIDTH).max(0.0);
                        break;
                    }

                    if hit.time_of_impact >= SKIN_WIDTH {
                        transform.translation += direction * (hit.time_of_impact - SKIN_WIDTH)
                    }

                    let extra_distance = distance - (hit.time_of_impact - SKIN_WIDTH).max(0.0);
                    let extra_velocity = direction * extra_distance;

                    let mut projected_velocity =
                        extra_velocity - (extra_velocity.dot(hit.normal1) * hit.normal1);

                    if projected_velocity.dot(*start_direction) <= 0.0 {
                        break;
                    }

                    for plane in &planes {
                        if hit.normal1.dot(*plane) > 0.99 {
                            projected_velocity += hit.normal1 * 0.01;
                        }
                    }

                    planes.push(hit.normal1);

                    direction_result = Dir3::new(projected_velocity);
                    distance = projected_velocity.length();
                } else {
                    transform.translation += direction * distance;
                    break;
                }
            } else {
                break;
            }
        }

        screen_print!("bounces: {}", bounce_count);
        screen_print!("hit count: {}", hit_count);
    }
}
