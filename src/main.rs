mod cell;
mod camera_controll;
mod sprite_animation;
mod physics;

use crate::cell::*;
use crate::camera_controll::*;
use crate::sprite_animation::*;
use crate::physics::*;

use bevy::prelude::*;
use bevy::render::texture::ImageSampler;
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            RapierDebugRenderPlugin::default(),
            crate::cell::CellPlugin,
            PhysicsPlugin,
            SpriteAnimationPlugin,
            CamControllPlugin,
        ))
        .add_systems(Update, texture_fixer)
        .run();
}

pub fn texture_fixer(
    mut texture_event: EventReader<AssetEvent<Image>>,
    mut assets: ResMut<Assets<Image>>
) {
    for ev in texture_event.iter() {
        if let AssetEvent::Created {handle} = ev {
            if let Some(texture) = assets.get_mut(&handle) {
                texture.sampler_descriptor = ImageSampler::nearest();
            }
        }
    }
}