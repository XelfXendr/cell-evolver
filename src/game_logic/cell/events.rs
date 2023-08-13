use bevy::prelude::*;

#[derive(Event, Deref, DerefMut)]
pub struct CellSpawnEvent(pub Entity);

#[derive(Event, Deref, DerefMut)]
pub struct CellDespawnEvent(pub Entity);

#[derive(Event, Deref, DerefMut)]
pub struct FlagellumSpawnEvent(pub Entity);

#[derive(Event, Deref, DerefMut)]
pub struct EyeSpawnEvent(pub Entity);

#[derive(Event, Deref, DerefMut)]
pub struct FoodSpawnEvent(pub Entity);

#[derive(Event, Deref, DerefMut)]
pub struct FoodDespawnEvent(pub Entity);