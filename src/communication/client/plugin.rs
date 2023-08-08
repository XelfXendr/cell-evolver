use std::net::{Ipv4Addr, IpAddr};

use bevy::utils::HashMap;

use bevy::prelude::*;
use bevy_quinnet::client::certificate::CertificateVerificationMode;
use bevy_quinnet::client::connection::ConnectionConfiguration;
use bevy_quinnet::client::{QuinnetClientPlugin, Client};
use ndarray::{Array2, Array1};

use crate::communication::shared::messages::{ServerMessage, EntityId, CellParams, CellState, Tick};
use crate::game_logic::cell::{spawn_cell, CellSpawnEvent, FlagellumSpawnEvent, EyeSpawnEvent, CellDespawnEvent, despawn_cell, FoodSpawnEvent, spawn_food, FoodDespawnEvent, despawn_food, Cell, DelayedDespawnQueue};
use crate::game_logic::physics::PhysicsBody;
use crate::game_logic::sprites::{cell_sprite_adder, eye_sprite_adder, flagellum_sprite_adder, food_sprite_adder};

pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(QuinnetClientPlugin::default())
            .add_systems(Startup, init)
            .add_systems(Update, (
                read_messages
                    .before(cell_sprite_adder)
                    .before(eye_sprite_adder)
                    .before(flagellum_sprite_adder)
                    .before(food_sprite_adder),
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
            IpAddr::V4(Ipv4Addr::new(127,0,0,1)),//Ipv4Addr::new(192, 168, 1, 45)), 
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
    mut cell_query: Query<(&mut Cell, &mut Transform, &mut PhysicsBody, &mut LastTickUpdated)>
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
                entity, cell_params, cell_state
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
                entity, position
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
) {
    if entity_map.contains_key(&entity) {
        return;
    }
    entity_map.insert(entity, 
        spawn_food(
            commands, 
            food_spawn_event_writer, 
            position.extend(0.),
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
    cell_query: &mut Query<(&mut Cell, &mut Transform, &mut PhysicsBody, &mut LastTickUpdated)>,
    tick: Tick,
    entity: EntityId,
    cell_state: &CellState,
) {
    if let Some((mut cell, mut cell_transform, mut cell_body, mut last_tick)) = entity_map.get(&entity).and_then(|e| cell_query.get_mut(*e).ok()) {
        if *tick <= **last_tick {
            return;
        }

        cell_transform.translation = cell_state.position.extend(0.);
        cell_body.velocity = cell_state.velocity;
        cell_body.acceleration = cell_state.acceleration;
        cell_transform.rotation = Quat::from_rotation_z(cell_state.rotation);
        cell_body.angular_velocity = cell_state.angular_velocity;
        cell_body.angular_acceleration = cell_state.angular_acceleration;
        cell.energy = cell_state.energy;
        **last_tick = *tick;
    }
}