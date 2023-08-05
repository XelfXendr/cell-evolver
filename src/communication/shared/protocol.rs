use std::time::Duration;

use naia_bevy_shared::{Protocol, LinkConditionerConfig};

use super::messages::MessageProtocol;

pub fn protocol() -> Protocol {
    Protocol::builder()
        .tick_interval(Duration::from_millis(50))
        .link_condition(LinkConditionerConfig::good_condition())
        .add_plugin(MessageProtocol)
        .build()
}