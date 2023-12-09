use bevy::prelude::*;
use bevy_rapier2d::prelude::{Collider, RapierContext, QueryFilter};

use crate::game_logic::{cell::*, math::quat_to_direction};
use super::*;

const MASS_MULTIPLIER: f32 = 1./200.;
const RADIUS_MULTIPLIER: f32 = 1./50.;

pub const PLAYER_SPEED: f32 = 500.;
pub const PLAYER_ANGLE_SPEED: f32 = 7.;

pub const DRAG: f32 = 2.;
pub const ANGULAR_DRAG: f32 = 2.;

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(FixedUpdate, (
                flagellum_physics,
                cell_push,
                velocity_update,
                angular_update,
            ));
    }
}

pub fn velocity_update(
    mut query: Query<(&mut Transform, &mut Velocity, &mut Force, &Energy)>
) {
    query
        .par_iter_mut()
        .for_each_mut(|(mut transform, mut velocity, mut force, energy)| {
            //get current acceleration
            let mass = **energy * MASS_MULTIPLIER;
            let acceleration = **force * FIXED_DELTA / mass;
            **force = Vec2::ZERO;

            let multiplier = 1. - DRAG * FIXED_DELTA;
            
            //update velocity
            **velocity += acceleration/2.;
            **velocity *= multiplier;
            transform.translation += Vec3::from((**velocity * FIXED_DELTA, 0.));
            **velocity += acceleration/2.;
        });
}

pub fn angular_update(
    mut query: Query<(&mut Transform, &mut AngularVelocity, &mut AngularForce, &Energy)>,
) {
    query
        .par_iter_mut()
        .for_each_mut(|(mut transform, mut velocity, mut force, energy)| {
            //get current angular acceleration
            let mass = **energy * MASS_MULTIPLIER;
            let acceleration = **force * FIXED_DELTA / mass;
            **force = 0.;
    
            let multiplier = 1. - ANGULAR_DRAG * FIXED_DELTA;
            
            //update angular velocity
            **velocity += acceleration/2.;
            **velocity *= multiplier;
            transform.rotate_local_z(**velocity * FIXED_DELTA);
            **velocity += acceleration/2.;
        });
}

pub fn flagellum_physics(
    mut cell_query: Query<(&CellFlagella, &mut Force, &mut AngularForce, &Transform, &Radius)>,
    flag_query: Query<(&Activation, &Angle, &Transform), With<Flagellum>>
) {
    cell_query
        .par_iter_mut()
        .for_each_mut(|(flagella, mut force, mut angular_force, cell_transform, radius)| {
            for flagellum_entity in flagella.iter() {
                if let Ok((activation, angle, flagellum_transform)) = flag_query.get(*flagellum_entity) {
                    **force += **activation * quat_to_direction(cell_transform.rotation * flagellum_transform.rotation) * PLAYER_SPEED;
                    **angular_force -= **activation * angle.sin() * **radius * RADIUS_MULTIPLIER * PLAYER_ANGLE_SPEED;
                }
            }
        });
}

pub fn cell_push(
    collider_query: Query<(&Parent, &Collider), With<CellColliderTag>>,
    mut cell_query: Query<(Entity, &Transform, &Radius, &CellCollider, &mut Force), With<Cell>>,
    cell_b_query: Query<(&Transform, &Radius)>,
    rapier_context: Res<RapierContext>,
) {
    cell_query
        .par_iter_mut()
        .for_each_mut(|(entity, transform_a, radius_a, cell_collider, mut force)| {
            if let Ok((_, collider)) = collider_query.get(**cell_collider) {
                rapier_context.intersections_with_shape(
                    transform_a.translation.truncate(), 
                    0., 
                    collider, 
                    QueryFilter::default(), 
                    |x| {
                        if let Ok((parent, _)) = collider_query.get(x) {
                            if parent.get() == entity {
                                return true;
                            }
                            if let Ok((transform_b, radius_b)) = cell_b_query.get(parent.get()) {
                                let mut direction = transform_a.translation.truncate() - transform_b.translation.truncate();
                                let d = direction.length();
                                direction = match direction.try_normalize() {
                                    Some(d) => d,
                                    None => quat_to_direction(transform_a.rotation),
                                };
                                let magnitude = (radius_a.0 + radius_b.0 - d) * INTERCELL_PUSH;
                                **force += magnitude * direction;
                            }
                        }
                        true
                    }
                );
            }
    });
}