use naia_bevy_shared::{Message, Channel, Serde, ProtocolPlugin, Protocol, ChannelDirection, ChannelMode, ReliableSettings};

pub struct MessageProtocol;
impl ProtocolPlugin for MessageProtocol {
    fn build(&self, protocol: &mut Protocol) {
        protocol
            .add_message::<Auth>()
            .add_message::<CellSpawn>()
            .add_message::<CellDespawn>()
            .add_message::<CellUpdate>()
            .add_message::<FoodSpawn>()
            .add_message::<FoodDespawn>()
            .add_channel::<ReliableUpdateChannel>(ChannelDirection::ServerToClient, ChannelMode::UnorderedUnreliable)//ChannelMode::OrderedReliable(ReliableSettings::default()))
            .add_channel::<UnreliableUpdateChannel>(ChannelDirection::ServerToClient, ChannelMode::UnorderedUnreliable);
    }
}

#[derive(Channel)]
pub struct UnreliableUpdateChannel;

#[derive(Channel)]
pub struct ReliableUpdateChannel;

#[derive(Message)]
pub struct Auth {
    pub key: String,
}

#[derive(Serde, PartialEq, Clone)]
pub struct CellState {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub acceleration: [f32; 2],
    pub rotation: f32,
    pub angular_velocity: f32,
    pub angular_acceleration: f32,
    pub energy: f32,
}

#[derive(Serde, PartialEq, Clone)]
pub struct CellParams {
    pub flagella_params: Vec<(f32, f32)>,
    pub eye_params: Vec<f32>,
}

#[derive(Message)]
pub struct CellUpdate {
    pub entity: u32,
    pub cell_state: CellState,
}

#[derive(Message)]
pub struct CellSpawn {
    pub entity: u32,
    pub cell_params: CellParams,
    pub cell_state: CellState,
}

#[derive(Message)]
pub struct CellDespawn {
    pub entity: u32,
}

#[derive(Message)]
pub struct FoodSpawn {
    pub entity: u32,
    pub position: [f32; 2],
}

#[derive(Message)]
pub struct FoodDespawn {
    pub entity: u32,
}