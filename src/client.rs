mod game_logic;
mod communication;

use communication::client::ClientPlugin;
use game_logic::camera_controll::*;
use game_logic::sprites::*;
use game_logic::physics::*;
use game_logic::cell::CellCorePlugin;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            }),
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            RapierDebugRenderPlugin::default(),
            CellCorePlugin,
            PhysicsPlugin,
            SpritesPlugin,
            CamControllPlugin,
            ClientPlugin,
        ))
        .run();
}

