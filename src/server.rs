mod cell;
mod camera_controll;
mod physics;

use crate::cell::*;
use crate::physics::*;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            CellPlugin,
            PhysicsPlugin,
        ))
        .run();
}