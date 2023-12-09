mod game_logic;
mod communication;

use bevy::render::texture::ImageFilterMode;
use bevy::render::texture::ImageSamplerDescriptor;
use communication::client::ClientPlugin;
use game_logic::camera_controll::*;
use game_logic::sprites::*;
use game_logic::physics::*;
use game_logic::cell::*;

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
            }).set(ImagePlugin {
                default_sampler: ImageSamplerDescriptor {
                    mag_filter: ImageFilterMode::Nearest,
                    min_filter: ImageFilterMode::Nearest,
                    ..default()
                },
            }),
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            //RapierDebugRenderPlugin::default(),
            PhysicsPlugin,
            SpritesPlugin,
            CamControllPlugin,
            CellCorePlugin,
            CellClientPlugin,
            ClientPlugin

        ))
        .run();
}

