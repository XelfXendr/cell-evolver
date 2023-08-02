use std::time::Duration;

use naia_bevy_shared::{Protocol, LinkConditionerConfig, ChannelDirection, ChannelMode, ReliableSettings};

use super::messages::{TestMessage, TestChannel, Auth};

pub fn protocol() -> Protocol {
    Protocol::builder()
        .tick_interval(Duration::from_millis(50))
        .link_condition(LinkConditionerConfig::good_condition())
        .add_message::<TestMessage>()
        .add_message::<Auth>()
        .add_channel::<TestChannel>(ChannelDirection::ServerToClient, ChannelMode::OrderedReliable(ReliableSettings::default()))
        .build()
}