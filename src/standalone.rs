mod game_logic;

use bevy::render::render_resource::FilterMode;
use bevy::render::render_resource::SamplerDescriptor;
use game_logic::cell::*;
use game_logic::camera_controll::*;
use game_logic::sprites::*;
use game_logic::physics::*;

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
                default_sampler: SamplerDescriptor {
                    mag_filter: FilterMode::Nearest,
                    min_filter: FilterMode::Nearest,
                    ..default()
                },
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

