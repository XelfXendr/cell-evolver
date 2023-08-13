use bevy::prelude::*;

use crate::game_logic::cell::*;
use super::*;

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(FixedUpdate, (
                flagellum_physics,
                velocity_update,
                angular_update,
            ));
    }
}

pub fn velocity_update(
    mut query: Query<(&mut Transform, &mut Velocity, &Acceleration, &Drag)>
) {
    for (mut transform, mut velocity, acceleration, drag) in query.iter_mut() {
        let acc = **acceleration * FIXED_DELTA;
        let multiplier = 1. - **drag/60.;
        **velocity += acc/2.;
        **velocity *= multiplier;
        transform.translation += Vec3::from((**velocity * FIXED_DELTA, 0.));
        **velocity += acc/2.;
    }
}

pub fn angular_update(
    mut query: Query<(&mut Transform, &mut AngularVelocity, &AngularAcceleration, &AngularDrag)>,
) {
    for (mut transform, mut velocity, acceleration, drag) in query.iter_mut() {
        let acc = **acceleration * FIXED_DELTA;
        let multiplier = 1. - **drag/60.;
        **velocity += acc/2.;
        **velocity *= multiplier;
        transform.rotate_local_z(**velocity * FIXED_DELTA);
        **velocity += acc/2.;
    }
}

pub fn flagellum_physics(
    mut cell_query: Query<(&CellFlagella, &mut Acceleration, &mut AngularAcceleration, &Transform)>,
    flag_query: Query<(&Activation, &Angle, &Transform), With<Flagellum>>
) {
    for (flagella, mut acceleration, mut angular_acceleration, cell_transform) in cell_query.iter_mut() {
        let mut direction = Vec2::ZERO;
        let mut angle_direction = 0.;

        for flagellum_entity in flagella.iter() {
            if let Ok((activation, angle, flagellum_transform)) = flag_query.get(*flagellum_entity) {
                direction += **activation * quat_to_direction(cell_transform.rotation * flagellum_transform.rotation);
                angle_direction -= **activation * angle.sin();
            }
        }
        **acceleration = direction * PLAYER_SPEED;
        **angular_acceleration = angle_direction * PLAYER_ANGLE_SPEED;
    }
}

pub fn quat_to_direction(quat: Quat) -> Vec2 {
    Vec2::new(-2.*quat.z*quat.w, 1.-2.*quat.z*quat.z)
}