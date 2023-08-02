mod game_logic;
mod communication;

use std::time::Duration;

use bevy::log::LogPlugin;
use communication::server::ServerPlugin;
use game_logic::cell::*;
use game_logic::physics::*;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1./60.))),
            LogPlugin::default(),
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            CellPlugin,
            PhysicsPlugin,
            ServerPlugin,
        ))    
        .run();
}