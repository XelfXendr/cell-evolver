use std::net::Ipv4Addr;

use bevy::prelude::*;

use bevy_quinnet::server::certificate::CertificateRetrievalMode;
use bevy_quinnet::server::{QuinnetServerPlugin, Server, ServerConfiguration, ConnectionEvent};
use bevy_quinnet::shared::channel::ChannelId;

use crate::communication::shared::messages::ServerMessage;
use crate::game_logic::cell::{Cell, CellDespawnEvent, Food, FoodDespawnEvent};
use crate::game_logic::physics::PhysicsBody;

#[derive(Resource, Deref, DerefMut)]
pub struct TickCounter(u64);

pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(QuinnetServerPlugin::default())
            .add_systems(Startup, init)
            .add_systems(Update, (
                connect_event_handler,
                cell_spawn_handler.after(connect_event_handler),
                food_spawn_handler.after(connect_event_handler),
                update_cells,
            ));
    }
}

fn init(mut commands: Commands, mut server: ResMut<Server>) {
    commands.insert_resource(TickCounter(0));
    server.start_endpoint(
        ServerConfiguration::from_ip(
            std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            30800
        ), 
        CertificateRetrievalMode::GenerateSelfSigned { server_hostname: "127.0.0.1".to_string() },
    ).expect("Unable to start server!");
}

fn connect_event_handler(
    server: Res<Server>,
    cell_query: Query<(Entity, &Cell, &Transform, &PhysicsBody)>,
    food_query: Query<(Entity, &Transform), With<Food>>,
    mut event_reader: EventReader<ConnectionEvent>,
) {
    let endpoint = server.endpoint();
    for ConnectionEvent{id} in event_reader.iter() {
        info!("Client id {} connected.", id);
        for (cell_entity, cell, cell_transform, cell_body) in cell_query.iter() {
            let res = endpoint.send_message_on::<ServerMessage>(*id, 
                ChannelId::OrderedReliable(1), 
                ServerMessage::cell_spawn(cell_entity, cell, cell_transform, cell_body)
            );
            if let Err(e) = res { 
                info!("{}", e);
            }
        }
        for (food_entity, food_transform) in food_query.iter() {
            let res =  endpoint.send_message_on::<ServerMessage>(*id, 
                ChannelId::OrderedReliable(1), 
                ServerMessage::food_spawn(food_entity, food_transform)
            );
            if let Err(e) = res { 
                info!("{}", e);
            }
        }
    }
}

fn cell_spawn_handler(
    server: Res<Server>,
    new_cell_query: Query<(Entity, &Cell, &Transform, &PhysicsBody), Added<Cell>>,
    mut despawn_event_reader: EventReader<CellDespawnEvent>,
) {
    let endpoint = server.endpoint();
    for (cell_entity, cell, cell_transform, cell_body) in new_cell_query.iter() {
        let _ = endpoint.broadcast_message_on::<ServerMessage>(
            ChannelId::OrderedReliable(1), 
            ServerMessage::cell_spawn(cell_entity, cell, cell_transform, cell_body)
        );
    }
    for cell_entity in despawn_event_reader.iter() {
        let _ = endpoint.broadcast_message_on::<ServerMessage>(
            ChannelId::OrderedReliable(1), 
            ServerMessage::cell_despawn(**cell_entity)
        );
    }
}

fn food_spawn_handler(
    server: Res<Server>,
    new_food_query: Query<(Entity, &Transform), Added<Food>>,
    mut despawn_event_reader: EventReader<FoodDespawnEvent>,
) {
    let endpoint = server.endpoint();
    for (food_entity, food_transform) in new_food_query.iter() {
        let _ = endpoint.broadcast_message_on::<ServerMessage>(
            ChannelId::OrderedReliable(1), 
            ServerMessage::food_spawn(food_entity, food_transform)
        );
    }
    for food_entity in despawn_event_reader.iter() {
        let _ = endpoint.broadcast_message_on::<ServerMessage>(
            ChannelId::OrderedReliable(1), 
            ServerMessage::food_despawn(**food_entity)
        );
    }
}

fn update_cells(
    server: Res<Server>,
    mut tick: ResMut<TickCounter>,
    cell_query: Query<(Entity, &Cell, &Transform, &PhysicsBody)>,

    ) {
    let endpoint = server.endpoint();
    for (cell_entity, cell, cell_transform, cell_body) in cell_query.iter() {
        let _ = endpoint.broadcast_message_on::<ServerMessage>(
            ChannelId::Unreliable, 
            ServerMessage::cell_update(**tick, cell_entity, cell, cell_transform, cell_body)
        );
    }
    **tick += 1;
}