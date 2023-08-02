use naia_bevy_client::events::MessageEvents;
use naia_bevy_client::transport::webrtc;
use naia_bevy_client::{Plugin as NaiaClientPlugin, ClientConfig, Client};

use bevy::prelude::*;
use naia_bevy_shared::ReceiveEvents;

use crate::communication::shared::messages::{TestChannel, TestMessage, Auth};
use crate::communication::shared::protocol::protocol;


pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(NaiaClientPlugin::new(ClientConfig::default(), protocol()))
            .add_systems(Startup, init)
            .add_systems(Update, (receive_message).in_set(ReceiveEvents));
    }
}

fn init(
    mut _commands: Commands,
    mut client: Client,
) {
    client.auth(Auth{key: "1234".to_string()});
    let socket = webrtc::Socket::new("http://127.0.0.1:30800", client.socket_config());
    client.connect(socket)
}

fn receive_message(
    mut event_reader: EventReader<MessageEvents>,
) {
    for event in event_reader.iter() {
        for message in event.read::<TestChannel, TestMessage>() {
            println!("{}", message.test);
        }
    }
}