use std::net::{Ipv4Addr, IpAddr};

use bevy::utils::HashMap;

use bevy::prelude::*;
use bevy_quinnet::client::certificate::CertificateVerificationMode;
use bevy_quinnet::client::connection::ConnectionConfiguration;
use bevy_quinnet::client::{QuinnetClientPlugin, Client};
use ndarray::{Array2, Array1};

use crate::communication::shared::messages::{ServerMessage, EntityId, CellParams, CellState, Tick};
use crate::game_logic::cell::{spawn_cell, CellSpawnEvent, FlagellumSpawnEvent, EyeSpawnEvent, CellDespawnEvent, despawn_cell, FoodSpawnEvent, spawn_food, FoodDespawnEvent, despawn_food, Cell, DelayedDespawnQueue, Energy};
use crate::game_logic::physics::{Velocity, Acceleration, AngularVelocity, AngularAcceleration};
use crate::game_logic::sprites::*;

pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(QuinnetClientPlugin::default())
            .add_systems(Startup, init)
            .add_systems(Update, (
                read_messages,
                add_ticks_to_cells,
            ));
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct EntityMap(HashMap<EntityId, Entity>);

#[derive(Component, Deref, DerefMut)]
pub struct LastTickUpdated(u64);

fn init(
    mut commands: Commands,
    mut client: ResMut<Client>,
) {
    commands.insert_resource(EntityMap(HashMap::new()));

    client.open_connection(
        ConnectionConfiguration::from_ips(
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 45)), 
            30800, 
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 
            0, 
        ),
        CertificateVerificationMode::SkipVerification,
    ).expect("Could not connect to server.");
}

fn add_ticks_to_cells(
    mut commands: Commands,
    new_cell_query: Query<Entity, Added<Cell>>,
) {
    for entity in new_cell_query.iter() {
        commands.entity(entity).insert(LastTickUpdated(0));
    }
}

fn read_messages(
    mut commands: Commands,
    mut despawn_queue: ResMut<DelayedDespawnQueue>,
    mut client: ResMut<Client>,
    mut entity_map: ResMut<EntityMap>,
    mut cell_spawn_event_writer: EventWriter<CellSpawnEvent>,
    mut cell_despawn_event_writer: EventWriter<CellDespawnEvent>,
    mut flagellum_spawn_event_writer: EventWriter<FlagellumSpawnEvent>,
    mut eye_spawn_event_writer: EventWriter<EyeSpawnEvent>,
    mut food_spawn_event_writer: EventWriter<FoodSpawnEvent>,
    mut food_despawn_event_writer: EventWriter<FoodDespawnEvent>,
    mut cell_query: Query<(&mut LastTickUpdated, &mut Transform, &mut Velocity, &mut Acceleration, &mut AngularVelocity, &mut AngularAcceleration, &mut Energy), With<Cell>>,
    cell_sprite: Option<Res<CellSprite>>,
    light_sprite: Option<Res<LightSprite>>,
    flagellum_sprite: Option<Res<FlagellumSprite>>,
    eye_sprite: Option<Res<EyeSprite>>,
    food_sprite: Option<Res<FoodSprite>>,
) {
    let connection = client.connection_mut();
    while let Some(message) = connection.try_receive_message::<ServerMessage>() {
        match message {
            ServerMessage::CellUpdate(tick, entity, cell_state) => cell_update_handler(
                &entity_map, 
                &mut cell_query, 
                tick, entity, &cell_state
            ),
            ServerMessage::CellSpawn(entity, cell_params, cell_state) => cell_spawn_handler(
                &mut commands, 
                &mut entity_map, 
                &mut cell_spawn_event_writer, 
                &mut flagellum_spawn_event_writer, 
                &mut eye_spawn_event_writer, 
                entity, cell_params, cell_state,
                cell_sprite.as_deref(),
                light_sprite.as_deref(),
                flagellum_sprite.as_deref(),
                eye_sprite.as_deref(),
            ),
            ServerMessage::CellDespawn(entity) => cell_despawn_handler(
                &mut despawn_queue, 
                &mut entity_map, 
                &mut cell_despawn_event_writer, 
                entity
            ),
            ServerMessage::FoodSpawn(entity, position) => food_spawn_handler(
                &mut commands, 
                &mut entity_map, 
                &mut food_spawn_event_writer, 
                entity, position,
                food_sprite.as_deref(),
                light_sprite.as_deref(),
            ),
            ServerMessage::FoodDespawn(entity) => food_despawn_handler(
                &mut despawn_queue, 
                &mut entity_map, 
                &mut food_despawn_event_writer, 
                entity,
            ),
        }
    }
}


fn cell_spawn_handler(
    commands: &mut Commands,
    entity_map: &mut EntityMap,
    cell_spawn_event_writer: &mut EventWriter<CellSpawnEvent>,
    flagellum_spawn_event_writer: &mut EventWriter<FlagellumSpawnEvent>,
    eye_spawn_event_writer: &mut EventWriter<EyeSpawnEvent>,
    entity: EntityId,
    cell_params: CellParams,
    cell_state: CellState,
    cell_sprite: Option<&CellSprite>,
    light_sprite: Option<&LightSprite>,
    flagellum_sprite: Option<&FlagellumSprite>,
    eye_sprite: Option<&EyeSprite>,
) {
    if entity_map.contains_key(&entity) {
        return;
    }
    entity_map.insert(entity,
        spawn_cell(commands, 
            cell_spawn_event_writer, 
            flagellum_spawn_event_writer, 
            eye_spawn_event_writer,
            cell_state.position.extend(0.),
            Quat::from_rotation_z(cell_state.rotation),
            cell_state.energy,
            cell_params.flagella_params,
            cell_params.eye_params,
            Array2::default((0,0)),
            Array1::default(0),
            Array1::default(0),
            cell_sprite,
            light_sprite,
            flagellum_sprite,
            eye_sprite,
        )
    );
}

fn cell_despawn_handler(
    despawn_queue: &mut DelayedDespawnQueue,
    entity_map: &mut EntityMap,
    cell_despawn_event_writer: &mut EventWriter<CellDespawnEvent>,
    entity: EntityId,
) {
    if let Some(cell_entity) = entity_map.remove(&entity) {
        despawn_cell(despawn_queue, cell_despawn_event_writer, cell_entity);
    }
}

fn food_spawn_handler(
    commands: &mut Commands,
    entity_map: &mut EntityMap,
    food_spawn_event_writer: &mut EventWriter<FoodSpawnEvent>,
    entity: EntityId,
    position: Vec2,
    food_sprite: Option<&FoodSprite>,
    light_sprite: Option<&LightSprite>,
) {
    if entity_map.contains_key(&entity) {
        return;
    }
    entity_map.insert(entity, 
        spawn_food(
            commands, 
            food_spawn_event_writer, 
            position.extend(0.),
            food_sprite,
            light_sprite,
        )
    );
}

fn food_despawn_handler(
    despawn_queue: &mut DelayedDespawnQueue,
    entity_map: &mut EntityMap,
    food_despawn_event_writer: &mut EventWriter<FoodDespawnEvent>,
    entity: EntityId,
) {
    if let Some(food_entity) = entity_map.remove(&entity) {
        despawn_food(despawn_queue, food_despawn_event_writer, food_entity);
    }
}

fn cell_update_handler(
    entity_map: &EntityMap,
    cell_query: &mut Query<(&mut LastTickUpdated, &mut Transform, &mut Velocity, &mut Acceleration, &mut AngularVelocity, &mut AngularAcceleration, &mut Energy), With<Cell>>,
    tick: Tick,
    entity: EntityId,
    cell_state: &CellState,
) {
    if let Some((mut last_tick, mut transform, mut velocity, mut acceleration, mut angular_velocity, mut angular_acceleration, mut energy)) = entity_map.get(&entity).and_then(|e| cell_query.get_mut(*e).ok()) {
        if *tick <= **last_tick {
            return;
        }

        transform.translation = cell_state.position.extend(0.);
        **velocity = cell_state.velocity;
        **acceleration = cell_state.acceleration;
        transform.rotation = Quat::from_rotation_z(cell_state.rotation);
        **angular_velocity = cell_state.angular_velocity;
        **angular_acceleration = cell_state.angular_acceleration;
        **energy = cell_state.energy;
        **last_tick = *tick;
    }
}