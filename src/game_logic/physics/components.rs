use bevy::prelude::*;

#[derive(Bundle, Default)]
pub struct PhysicsBundle {
    pub velocity: Velocity,
    pub force: Force,
    pub angular_velocity: AngularVelocity,
    pub angular_force: AngularForce,
}
impl PhysicsBundle {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct Velocity(pub Vec2);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct Force(pub Vec2);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct AngularVelocity(pub f32);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct AngularForce(pub f32);