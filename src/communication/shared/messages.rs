use bevy::prelude::{Deref, DerefMut, Entity, Transform, Vec2};
use serde::{Serialize, Deserialize};

use crate::game_logic::{cell::Cell, physics::{PhysicsBody, quat_to_direction}};

#[derive(Serialize, Deserialize, Deref, DerefMut, Eq, PartialEq, Hash)]
pub struct EntityId(u32);
impl EntityId {
    pub fn new(entity: Entity) -> Self {
        Self(entity.index())
    }
}

#[derive(Serialize, Deserialize, Deref, DerefMut)]
pub struct Tick(u64);
impl Tick {
    pub fn new(tick: u64) -> Self {
        Self(tick)
    }
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    CellUpdate(Tick, EntityId, CellState),
    CellSpawn(EntityId, CellParams, CellState),
    CellDespawn(EntityId),
    FoodSpawn(EntityId, Vec2),
    FoodDespawn(EntityId)
}
impl ServerMessage {
    pub fn cell_update(tick: u64, entity: Entity, cell: &Cell, cell_transform: &Transform, cell_body: &PhysicsBody) -> Self {
        Self::CellUpdate(
            Tick::new(tick),
            EntityId::new(entity),
            CellState::new(cell, cell_transform, cell_body)
        )
    }
    pub fn cell_spawn(entity: Entity, cell: &Cell, cell_transform: &Transform, cell_body: &PhysicsBody) -> Self {
        Self::CellSpawn(
            EntityId::new(entity),
            CellParams::new(cell),
            CellState::new(cell, cell_transform, cell_body)
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
#[derive(Serialize, Deserialize)]
pub struct CellState {
    pub position: Vec2,
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub rotation: f32,
    pub angular_velocity: f32,
    pub angular_acceleration: f32,
    pub energy: f32,
}
impl CellState {
    pub fn new(cell: &Cell, cell_transform: &Transform, cell_body: &PhysicsBody) -> Self {
        Self {
            position: cell_transform.translation.truncate(),
            velocity: cell_body.velocity,
            acceleration: cell_body.acceleration,
            rotation: {
                let direction = quat_to_direction(cell_transform.rotation);
                (-direction.x).atan2(direction.y)
            },
            angular_velocity: cell_body.angular_velocity,
            angular_acceleration: cell_body.angular_acceleration,
            energy: cell.energy,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CellParams {
    pub flagella_params: Vec<(f32, f32)>,
    pub eye_params: Vec<f32>,
}
impl CellParams {
    pub fn new(cell: &Cell) -> Self {
        Self {
            flagella_params: cell.flagella_params.clone(),
            eye_params: cell.eye_params.clone(),
        }
    }
}