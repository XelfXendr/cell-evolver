use bevy::prelude::*;

#[derive(Bundle, Default)]
pub struct PhysicsBundle {
    pub velocity: Velocity,
    pub acceleration: Acceleration,
    pub drag: Drag,
    pub angular_velocity: AngularVelocity,
    pub angular_acceleration: AngularAcceleration,
    pub angular_drag: AngularDrag,
}
impl PhysicsBundle {
    pub fn from_drag(drag: f32, angular_drag: f32) -> Self {
        Self {
            drag: Drag(drag),
            angular_drag: AngularDrag(angular_drag),
            ..default()
        }
    }
}

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct Velocity(pub Vec2);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct Acceleration(pub Vec2);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct Drag(pub f32);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct AngularVelocity(pub f32);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct AngularAcceleration(pub f32);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct AngularDrag(pub f32);