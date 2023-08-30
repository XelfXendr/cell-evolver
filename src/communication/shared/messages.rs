use bevy::prelude::{Deref, DerefMut, Entity, Transform, Vec2};
use serde::{Serialize, Deserialize};

use crate::game_logic::{
    cell::{Energy, FlagellaParams, EyeParams}, 
    physics::{quat_to_direction, Force, AngularVelocity, AngularForce, Velocity}
};

#[derive(Serialize, Deserialize, Deref, DerefMut, Eq, PartialEq, Hash, Clone, Copy)]
pub struct EntityId(u32);
impl EntityId {
    pub fn new(entity: Entity) -> Self {
        Self(entity.index())
    }
}

#[derive(Serialize, Deserialize, Deref, DerefMut, Clone, Copy)]
pub struct Tick(u64);
impl Tick {
    pub fn new(tick: u64) -> Self {
        Self(tick)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ServerMessage {
    CellUpdate(Tick, EntityId, CellState),
    CellSpawn(EntityId, CellParams, CellState),
    CellDespawn(EntityId),
    FoodSpawn(EntityId, Vec2),
    FoodDespawn(EntityId)
}
impl ServerMessage {
    pub fn cell_update(tick: u64, 
        entity: Entity, 
        transform: &Transform, 
        velocity: Velocity, 
        force: Force, 
        ang_velocity: AngularVelocity, 
        ang_force: AngularForce,
        energy: Energy) -> Self {
        Self::CellUpdate(
            Tick::new(tick),
            EntityId::new(entity),
            CellState::new(transform, velocity, force, ang_velocity, ang_force, energy),
        )
    }
    pub fn cell_spawn(entity: Entity, 
        flagella_params: &FlagellaParams,
        eye_params: &EyeParams,
        transform: &Transform, 
        velocity: Velocity, 
        force: Force, 
        ang_velocity: AngularVelocity, 
        ang_force: AngularForce,
        energy: Energy) -> Self {
        Self::CellSpawn(
            EntityId::new(entity),
            CellParams::new(flagella_params, eye_params),
            CellState::new(transform, velocity, force, ang_velocity, ang_force, energy),
        )
    }
    pub fn cell_despawn(entity: Entity) -> Self {
        Self::CellDespawn(EntityId::new(entity))
    }
    pub fn food_spawn(entity: Entity, transform: &Transform) -> Self {
        Self::FoodSpawn(EntityId::new(entity), transform.translation.truncate())
    }
    pub fn food_despawn(entity: Entity) -> Self {
        Self::FoodDespawn(EntityId::new(entity))
    }
}

// Information structs
#[derive(Serialize, Deserialize, Clone)]
pub struct CellState {
    pub position: Vec2,
    pub velocity: Vec2,
    pub force: Vec2,
    pub rotation: f32,
    pub angular_velocity: f32,
    pub angular_force: f32,
    pub energy: f32,
}
impl CellState {
    pub fn new(
        transform: &Transform, 
        velocity: Velocity, 
        force: Force, 
        ang_velocity: AngularVelocity, 
        ang_force: AngularForce,
        energy: Energy
    ) -> Self {
        Self {
            position: transform.translation.truncate(),
            velocity: *velocity,
            force: *force,
            rotation: {
                let direction = quat_to_direction(transform.rotation);
                (-direction.x).atan2(direction.y)
            },
            angular_velocity: *ang_velocity,
            angular_force: *ang_force,
            energy: *energy,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CellParams {
    pub flagella_params: Vec<(f32, f32)>,
    pub eye_params: Vec<f32>,
}
impl CellParams {
    pub fn new(flagella_params: &FlagellaParams, eye_params: &EyeParams) -> Self {
        Self {
            flagella_params: (**flagella_params).clone(),
            eye_params: (**eye_params).clone(),
        }
    }
}