use bevy::utils::HashMap;
use naia_bevy_client::events::MessageEvents;
use naia_bevy_client::transport::webrtc;
use naia_bevy_client::{Plugin as NaiaClientPlugin, ClientConfig, Client};

use bevy::prelude::*;
use naia_bevy_shared::ReceiveEvents;
use ndarray::{Array2, Array1};

use crate::communication::shared::messages::{Auth, UnreliableUpdateChannel, CellUpdate, ReliableUpdateChannel, CellSpawn, CellDespawn, FoodSpawn, FoodDespawn};
use crate::communication::shared::protocol::protocol;
use crate::game_logic::cell::{spawn_cell, CellSpawnEvent, FlagellumSpawnEvent, EyeSpawnEvent, CellDespawnEvent, despawn_cell, FoodSpawnEvent, spawn_food, FoodDespawnEvent, despawn_food, Cell};
use crate::game_logic::physics::PhysicsBody;
use crate::game_logic::sprites::{food_sprite_adder, cell_sprite_adder, flagellum_sprite_adder, eye_sprite_adder};

pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(NaiaClientPlugin::new(ClientConfig::default(), protocol()))
            .add_systems(Startup, init)
            .add_systems(Update, (
                cell_spawn_handler.before(cell_sprite_adder).before(flagellum_sprite_adder).before(eye_sprite_adder),
                cell_despawn_handler,
                food_spawn_handler.before(food_sprite_adder),
                food_despawn_handler,
                cell_update_handler,
            ).in_set(ReceiveEvents));
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct EntityMap(HashMap<u32, Entity>);

fn init(
    mut commands: Commands,
    mut client: Client,
) {
    commands.insert_resource(EntityMap(HashMap::new()));

    client.auth(Auth{key: "1234".to_string()});
    let socket = webrtc::Socket::new("http://127.0.0.1:30800", client.socket_config());
    client.connect(socket)
}

fn cell_spawn_handler(
    mut commands: Commands,
    mut entity_map: ResMut<EntityMap>,
    mut message_event_reader: EventReader<MessageEvents>,
    mut cell_spawn_event_writer: EventWriter<CellSpawnEvent>,
    mut flagellum_spawn_event_writer: EventWriter<FlagellumSpawnEvent>,
    mut eye_spawn_event_writer: EventWriter<EyeSpawnEvent>,
) {
    for event in message_event_reader.iter() {
        for message in event.read::<ReliableUpdateChannel, CellSpawn>() {
            entity_map.insert(message.entity,
                spawn_cell(&mut commands, 
                    &mut cell_spawn_event_writer, 
                    &mut flagellum_spawn_event_writer, 
                    &mut eye_spawn_event_writer,
                    Vec3::new(message.cell_state.position[0], message.cell_state.position[1], 0.),
                    Quat::from_rotation_z(message.cell_state.rotation),
                    message.cell_state.energy,
                    message.cell_params.flagella_params,
                    message.cell_params.eye_params,
                    Array2::default((0,0)),
                    Array1::default(0),
                    Array1::default(0),
                )
            );
        }
    }
}

fn cell_despawn_handler(
    mut commands: Commands,
    mut entity_map: ResMut<EntityMap>,
    mut message_event_reader: EventReader<MessageEvents>,
    mut cell_despawn_event_writer: EventWriter<CellDespawnEvent>,
) {
    for event in message_event_reader.iter() {
        for message in event.read::<ReliableUpdateChannel, CellDespawn>() {
            if let Some(cell_entity) = entity_map.remove(&message.entity) {
                despawn_cell(&mut commands, &mut cell_despawn_event_writer, cell_entity)
            }
        }
    }
}

fn food_spawn_handler(
    mut commands: Commands,
    mut entity_map: ResMut<EntityMap>,
    mut message_event_reader: EventReader<MessageEvents>,
    mut food_spawn_event_writer: EventWriter<FoodSpawnEvent>,
) {
    for event in message_event_reader.iter() {
        for message in event.read::<ReliableUpdateChannel, FoodSpawn>() {
            entity_map.insert(message.entity, 
                spawn_food(
                    &mut commands, 
                    &mut food_spawn_event_writer, 
                    Vec3::new(message.position[0], message.position[1], 0.),
                )
            );
        }
    }
}

fn food_despawn_handler(
    mut commands: Commands,
    mut entity_map: ResMut<EntityMap>,
    mut message_event_reader: EventReader<MessageEvents>,
    mut food_despawn_event_writer: EventWriter<FoodDespawnEvent>,
) {
    for event in message_event_reader.iter() {
        for message in event.read::<ReliableUpdateChannel, FoodDespawn>() {
            if let Some(food_entity) = entity_map.remove(&message.entity) {
                despawn_food(&mut commands, &mut food_despawn_event_writer, food_entity)
            }
        }
    }
}

fn cell_update_handler(
    entity_map: Res<EntityMap>,
    mut message_event_reader: EventReader<MessageEvents>,
    mut cell_query: Query<(&mut Cell, &mut Transform, &mut PhysicsBody)>
) {
    for event in message_event_reader.iter() {
        for message in event.read::<UnreliableUpdateChannel, CellUpdate>() {
            if let Some((mut cell, mut cell_transform, mut cell_body)) = entity_map.get(&message.entity).and_then(|e| cell_query.get_mut(*e).ok()) {
                cell_transform.translation = Vec3::new(message.cell_state.position[0], message.cell_state.position[1], 0.);
                cell_body.velocity = Vec2::from_array(message.cell_state.velocity);
                cell_body.acceleration = Vec2::from_array(message.cell_state.acceleration);
                cell_transform.rotation = Quat::from_rotation_z(message.cell_state.rotation);
                cell_body.angular_velocity = message.cell_state.angular_velocity;
                cell_body.angular_acceleration = message.cell_state.angular_acceleration;
                cell.energy = message.cell_state.energy;
            }
        }
    }
}