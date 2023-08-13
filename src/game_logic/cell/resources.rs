use bevy::prelude::*;

#[derive(Resource, Deref, DerefMut)]
pub struct FoodTimer(pub Timer);

#[derive(Resource, Deref, DerefMut)]
pub struct DebugTimer(pub Timer);

#[derive(Resource)]
pub struct TimeCounter(pub f32, pub f32);

#[derive(Resource)]
pub struct DelayedDespawnQueue {
    pending: Vec<Entity>,
    current: Vec<Entity>,
}

impl DelayedDespawnQueue {
    pub fn new() -> Self {
        Self { pending: Vec::new(), current: Vec::new() }
    }
    pub fn add(&mut self, entity: Entity) {
        self.pending.push(entity);
    }
    pub fn despawn(&mut self, commands: &mut Commands) {
        for entity in self.current.iter() {
            if let Some(entity_commands) = commands.get_entity(*entity) {
                entity_commands.despawn_recursive();
            }
        }
        self.current.clear();
        std::mem::swap(&mut self.current, &mut self.pending);
    }
}