use bevy::prelude::*;

use bevy::utils::HashSet;
use naia_bevy_server::events::{ConnectEvent, DisconnectEvent, AuthEvents};
use naia_bevy_server::{transport::webrtc, Server};
use naia_bevy_server::{Plugin as NaiaServerPlugin, ServerConfig, RoomKey, ReceiveEvents, UserKey, };

use crate::communication::shared::messages::{TestChannel, TestMessage, Auth};
use crate::communication::shared::protocol::protocol;

pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(NaiaServerPlugin::new(ServerConfig::default(), protocol()))
            .add_systems(Startup, init)
            .add_systems(Update, test_event )
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

fn test_event(mut server: Server, keys: Res<UserKeys>) {
    for key in keys.iter() {
        println!("trying");
        server.send_message::<TestChannel, TestMessage>(key, &TestMessage { test: "Yo?".to_string() });
        println!("sent hello");
    }
}

fn auth_event_handler(mut server: Server, mut event_reader: EventReader<AuthEvents>) {
    for events in event_reader.iter() {
        for (user_key, auth) in events.read::<Auth>() {
            println!("Connecting with auth key: {}", auth.key);
            server.accept_connection(&user_key);
        }
    }
}

fn connect_event_handler(
    mut keys: ResMut<UserKeys>,
    mut event_reader: EventReader<ConnectEvent>,
) {
    for ConnectEvent(key) in event_reader.iter() {
        println!("Accepted, inserting");
        keys.insert(*key);
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