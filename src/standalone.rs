mod cell;
mod camera_controll;
mod sprites;
mod physics;

use crate::cell::*;
use crate::camera_controll::*;
use crate::sprites::*;
use crate::physics::*;

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
            CellPlugin,
            PhysicsPlugin,
            SpritesPlugin,
            CamControllPlugin,
        ))
        .run();
}

