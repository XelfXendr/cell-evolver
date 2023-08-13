use std::collections::VecDeque;
use std::net::Ipv4Addr;

use bevy::prelude::*;

use bevy_quinnet::server::certificate::CertificateRetrievalMode;
use bevy_quinnet::server::{QuinnetServerPlugin, Server, ServerConfiguration, ConnectionEvent};
use bevy_quinnet::shared::QuinnetError;
use bevy_quinnet::shared::channel::ChannelId;

use crate::communication::shared::messages::ServerMessage;
use crate::game_logic::cell::{Cell, CellDespawnEvent, Food, FoodDespawnEvent, FlagellaParams, EyeParams, Energy};
use crate::game_logic::physics::{Velocity, Acceleration, AngularVelocity, AngularAcceleration};

#[derive(Resource, Deref, DerefMut)]
pub struct TickCounter(u64);

pub enum Recipient {
    Broadcast,
    User(u64),
}
#[derive(Resource, Deref, DerefMut, Default)]
pub struct MessageQueue(VecDeque<(ServerMessage, Recipient)>);
impl MessageQueue {
    pub fn add(&mut self, recipient: Recipient, message: ServerMessage) {
        self.push_back((message, recipient));
    }
    pub fn try_send_all(&mut self, server: &Server) {
        let endpoint = server.endpoint();
        while let Some((message, recipient)) = self.front() {
            let result = match recipient {
                Recipient::Broadcast => endpoint.broadcast_message_on::<ServerMessage>(
                    ChannelId::OrderedReliable(1),
                    message.clone(),
                ),
                Recipient::User(id) => endpoint.send_message_on::<ServerMessage>(
                    *id, 
                    ChannelId::OrderedReliable(1),
                    message.clone()
                ),
            };
            match result {
                Err(qe) => match qe {
                    QuinnetError::FullQueue => return, //keep message until channel clears
                    _ => self.pop_front(), //something wrong with connection, drop message
                },
                _ => self.pop_front(), //message sent successfully
            };
        }
    }
}

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
                send_reliable_messages,
            ));
    }
}

fn init(mut commands: Commands, mut server: ResMut<Server>) {
    commands.insert_resource(TickCounter(0));
    commands.insert_resource(MessageQueue::default());

    server.start_endpoint(
        ServerConfiguration::from_ip(
            std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            30800
        ), 
        CertificateRetrievalMode::GenerateSelfSigned { server_hostname: "127.0.0.1".to_string() },
    ).expect("Unable to start server!");
}

fn send_reliable_messages(
    server: Res<Server>,
    mut message_queue: ResMut<MessageQueue>,
) {
    message_queue.try_send_all(&server);
}

fn connect_event_handler(
    mut message_queue: ResMut<MessageQueue>,
    cell_query: Query<(Entity, &FlagellaParams, &EyeParams, &Transform, &Velocity, &Acceleration, &AngularVelocity, &AngularAcceleration, &Energy), With<Cell>>,
    food_query: Query<(Entity, &Transform), With<Food>>,
    mut event_reader: EventReader<ConnectionEvent>,
) {
    for ConnectionEvent{id} in event_reader.iter() {
        info!("Client id {} connected.", id);
        for (entity, flagella_params, eye_params, transform, velocity, acceleration, ang_velocity, ang_acceleration, energy) in cell_query.iter() {
            message_queue.add(
                Recipient::User(*id), 
                ServerMessage::cell_spawn(entity, flagella_params, eye_params, transform, *velocity, *acceleration, *ang_velocity, *ang_acceleration, *energy)
            );
        }
        for (food_entity, food_transform) in food_query.iter() {
            message_queue.add(
                Recipient::User(*id),
                ServerMessage::food_spawn(food_entity, food_transform)
            )
        }
    }
}

fn cell_spawn_handler(
    mut message_queue: ResMut<MessageQueue>,
    new_cell_query: Query<(Entity, &FlagellaParams, &EyeParams, &Transform, &Velocity, &Acceleration, &AngularVelocity, &AngularAcceleration, &Energy), Added<Cell>>,
    mut despawn_event_reader: EventReader<CellDespawnEvent>,
) {
    for (entity, flagella_params, eye_params, transform, velocity, acceleration, ang_velocity, ang_acceleration, energy) in new_cell_query.iter() {
        message_queue.add(
            Recipient::Broadcast, 
            ServerMessage::cell_spawn(entity, flagella_params, eye_params, transform, *velocity, *acceleration, *ang_velocity, *ang_acceleration, *energy)
        );
    }
    for cell_entity in despawn_event_reader.iter() {
        message_queue.add(
            Recipient::Broadcast,
            ServerMessage::cell_despawn(**cell_entity)
        );
    }
}

fn food_spawn_handler(
    mut message_queue: ResMut<MessageQueue>,
    new_food_query: Query<(Entity, &Transform), Added<Food>>,
    mut despawn_event_reader: EventReader<FoodDespawnEvent>,
) {
    for (food_entity, food_transform) in new_food_query.iter() {
        message_queue.add(
            Recipient::Broadcast,
            ServerMessage::food_spawn(food_entity, food_transform)
        );
    }
    for food_entity in despawn_event_reader.iter() {
        message_queue.add(
            Recipient::Broadcast,
            ServerMessage::food_despawn(**food_entity)
        );
    }
}

fn update_cells(
    server: Res<Server>,
    mut tick: ResMut<TickCounter>,
    cell_query: Query<(Entity, &Transform, &Velocity, &Acceleration, &AngularVelocity, &AngularAcceleration, &Energy)>,
    ) {
    let endpoint = server.endpoint();
    for (entity, transform, velocity, acceleration, ang_velocity, ang_acceleration, energy) in cell_query.iter() {
        let _ = endpoint.broadcast_message_on::<ServerMessage>(
            ChannelId::Unreliable, 
            ServerMessage::cell_update(**tick, entity, transform, *velocity, *acceleration, *ang_velocity, *ang_acceleration, *energy)
        );
    }
    **tick += 1;
}