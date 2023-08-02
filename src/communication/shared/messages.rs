use naia_bevy_shared::{Message, Channel};

#[derive(Message)]
pub struct TestMessage {
    pub test: String,
}

#[derive(Message)]
pub struct Auth {
    pub key: String,
}

#[derive(Channel)]
pub struct TestChannel;