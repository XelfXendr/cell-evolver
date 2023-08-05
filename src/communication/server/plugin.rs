use bevy::ecs::schedule::{ScheduleLabel, ExecutorKind};
use bevy::prelude::*;

use bevy::utils::HashSet;
use naia_bevy_server::events::{ConnectEvent, DisconnectEvent, AuthEvents};
use naia_bevy_server::{transport::webrtc, Server};
use naia_bevy_server::{Plugin as NaiaServerPlugin, ServerConfig, RoomKey, ReceiveEvents, UserKey, };

use crate::communication::shared::messages::{Auth, ReliableUpdateChannel, CellSpawn, CellParams, CellState, UnreliableUpdateChannel, CellUpdate, CellDespawn, FoodSpawn, FoodDespawn};
use crate::communication::shared::protocol::protocol;
use crate::game_logic::cell::{Cell, CellSpawnEvent, CellDespawnEvent, Food, FoodSpawnEvent, FoodDespawnEvent};
use crate::game_logic::physics::{PhysicsBody, quat_to_direction};

pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(NaiaServerPlugin::new(ServerConfig::default(), protocol()))
            .add_systems(Startup, init)
            .add_systems(Update, (
                cell_spawn_handler,
                cell_despawn_handler,
                update_cells,
                food_spawn_handler,
                food_despawn_handler,
            ))
            .add_systems(Update, (
                auth_event_handler,
                connect_event_handler,
                disconnect_event_handler,
            ).chain().in_set(ReceiveEvents));
    }
}


#[derive(Resource, Deref, DerefMut)]
pub struct MainRoomKey(RoomKey);

#[derive(Resource, Deref, DerefMut)]
pub struct UserKeys(HashSet<UserKey>);

fn init(mut commands: Commands, mut server: Server) {
    let server_adresses = webrtc::ServerAddrs::new(
        "127.0.0.1:30800".parse().expect("Could not parse Signaling address/port."),
        "127.0.0.1:30801".parse().expect("Could not parse WebRTC data address/port."),
        "http://127.0.0.1:30801"
    );
    
    let socket = webrtc::Socket::new(&server_adresses, server.socket_config());
    server.listen(socket);
    let main_room_key = server.make_room().key();
    commands.insert_resource(MainRoomKey(main_room_key));
    commands.insert_resource(UserKeys(HashSet::new()));
}

fn create_cell_params(cell: &Cell) -> CellParams {
    CellParams {
        flagella_params: cell.flagella_params.clone(),
        eye_params: cell.eye_params.clone(),
    }
}

fn create_cell_state(cell: &Cell, cell_transform: &Transform, cell_body: &PhysicsBody) -> CellState {
    CellState {
        position: [cell_transform.translation.x, cell_transform.translation.y],
        velocity: [cell_body.velocity.x, cell_body.velocity.y],
        acceleration: [cell_body.acceleration.x, cell_body.acceleration.y],
        rotation: {
            let direction = quat_to_direction(cell_transform.rotation);
            (-direction.x).atan2(direction.y)
        },
        angular_velocity: cell_body.angular_velocity,
        angular_acceleration: cell_body.angular_acceleration,
        energy: cell.energy,
    }
}

fn auth_event_handler(
    mut server: Server, 
    mut event_reader: EventReader<AuthEvents>) 
{
    for events in event_reader.iter() {
        for (user_key, auth) in events.read::<Auth>() {
            if auth.key == "1234" {
                println!("Connecting with auth key: {}", auth.key);
                server.accept_connection(&user_key);    
            }
        }
    }
}

fn connect_event_handler(
    mut server: Server,
    mut keys: ResMut<UserKeys>,
    cell_query: Query<(Entity, &Cell, &Transform, &PhysicsBody)>,
    food_query: Query<(Entity, &Transform), With<Food>>,
    mut event_reader: EventReader<ConnectEvent>,
) {
    for ConnectEvent(key) in event_reader.iter() {
        keys.insert(*key);
        for (cell_entity, cell, cell_transform, cell_body) in cell_query.iter() {
            server.send_message::<ReliableUpdateChannel, CellSpawn>(key, &CellSpawn { 
                entity: cell_entity.index(), 
                cell_params: create_cell_params(cell), 
                cell_state: create_cell_state(cell, cell_transform, cell_body) 
            })
        }
        for (food_entity, food_transform) in food_query.iter() {
            server.send_message::<ReliableUpdateChannel, FoodSpawn>(key, &FoodSpawn {
                entity: food_entity.index(),
                position: [food_transform.translation.x, food_transform.translation.y],
            });
        }
    }
}

fn disconnect_event_handler(
    mut keys: ResMut<UserKeys>,
    mut event_reader: EventReader<DisconnectEvent>,
) {
    for DisconnectEvent(key, _user) in event_reader.iter() {
        keys.remove(key);
    }
}

fn cell_spawn_handler(
    mut server: Server,
    cell_query: Query<(&Cell, &Transform, &PhysicsBody)>,
    mut spawn_event_reader: EventReader<CellSpawnEvent>,
) {
    for cell_entity in spawn_event_reader.iter() {
        if let Ok((cell, cell_transform, cell_body)) = cell_query.get(**cell_entity) {
            server.broadcast_message::<ReliableUpdateChannel, CellSpawn>(&CellSpawn { 
                entity: cell_entity.index(),
                cell_params: create_cell_params(cell), 
                cell_state: create_cell_state(cell, cell_transform, cell_body) 
            });
        }
    }
}

fn cell_despawn_handler(
    mut server: Server,
    mut despawn_event_reader: EventReader<CellDespawnEvent>,
) {
    for cell_entity in despawn_event_reader.iter() {
        server.broadcast_message::<ReliableUpdateChannel, CellDespawn>(&CellDespawn { 
            entity: cell_entity.index() 
        })
    }
}

fn food_spawn_handler(
    mut server: Server,
    food_query: Query<&Transform, With<Food>>,
    mut spawn_event_reader: EventReader<FoodSpawnEvent>,
) {
    for food_entity in spawn_event_reader.iter() {
        if let Ok(food_transform) = food_query.get(**food_entity) {
            server.broadcast_message::<ReliableUpdateChannel, FoodSpawn>(&FoodSpawn { 
                entity: food_entity.index(),
                position: [food_transform.translation.x, food_transform.translation.y],
            });
        }
    }
}

fn food_despawn_handler(
    mut server: Server,
    mut despawn_event_reader: EventReader<FoodDespawnEvent>,
) {
    for food_entity in despawn_event_reader.iter() {
        server.broadcast_message::<ReliableUpdateChannel, FoodDespawn>(&FoodDespawn { 
            entity: food_entity.index() 
        })
    }
}

fn update_cells(
    mut server: Server,
    cell_query: Query<(Entity, &Cell, &Transform, &PhysicsBody)>,
    ) {
    for (cell_entity, cell, cell_transform, cell_body) in cell_query.iter() {
        server.broadcast_message::<UnreliableUpdateChannel, CellUpdate>(&CellUpdate { 
            entity: cell_entity.index(), 
            cell_state: create_cell_state(cell, cell_transform, cell_body),
        });
    }
}
