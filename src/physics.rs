use bevy::prelude::*;

use crate::cell::*;

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(FixedUpdate, (
                flagellum_physics,
                physics_update,
            ));
    }
}

#[derive(Component)]
pub struct PhysicsBody {
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub angular_velocity: f32,
    pub angular_acceleration: f32,
    pub drag: f32,
    pub angular_drag: f32,
}

pub fn physics_update(
    mut body_query: Query<(&mut PhysicsBody, &mut Transform)>,
) {
    let dt = FIXED_DELTA;
    for (mut body, mut transform) in body_query.iter_mut() {
        let acc = body.acceleration * dt;
        let multiplier = (1. - body.drag/60.).powf(60.*dt);
        body.velocity += acc/2.;
        body.velocity *= multiplier;
        transform.translation += Vec3::from((body.velocity * dt, 0.));
        body.velocity += acc/2.;

        let ang_acc = body.angular_acceleration * dt;
        let multiplier = (1. - body.angular_drag/60.).powf(60.*dt);
        body.angular_velocity += ang_acc/2.;
        body.angular_velocity *= multiplier;
        transform.rotate_local_z(body.angular_velocity * dt);
        body.angular_velocity += ang_acc/2.;

    }
}

pub fn flagellum_physics(
    mut cell_query: Query<(&Cell, &mut PhysicsBody, &Transform)>,
    flag_query: Query<(&Flagellum, &Transform)>
) {
    for (cell, mut body, cell_transform) in cell_query.iter_mut() {
        let mut direction = Vec2::ZERO;
        let mut angle_direction = 0.;

        for flagellum_entity in cell.flagella.iter() {
            if let Ok((flagellum, flagellum_transform)) = flag_query.get(*flagellum_entity) {
                direction += flagellum.activation * quat_to_direction(cell_transform.rotation * flagellum_transform.rotation);
                angle_direction -= flagellum.activation * flagellum.angle.sin();
            }
        }
        body.acceleration = direction * PLAYER_SPEED;
        body.angular_acceleration = angle_direction * PLAYER_ANGLE_SPEED;
    }
}

pub fn quat_to_direction(quat: Quat) -> Vec2 {
    Vec2::new(-2.*quat.z*quat.w, 1.-2.*quat.z*quat.z)
}