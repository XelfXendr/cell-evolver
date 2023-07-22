use std::time::Duration;

use bevy::prelude::*;

use crate::cell::*;

pub struct SpriteAnimationPlugin;
impl Plugin for SpriteAnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                animate_sprite,
                flagellum_animation_speed,
                eye_focus_animation,
            ));
    }
}

#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}

pub fn flagellum_animation_speed(
    mut flag_query: Query<(&Flagellum, &mut AnimationTimer)>, 
) {
    for (flagellum, mut timer) in flag_query.iter_mut() {
        timer.set_duration(
            Duration::from_secs_f32(0.033 / f32::max(0.1, flagellum.activation))
        );
    }
}

pub fn eye_focus_animation(
    mut eye_query: Query<(&Eye, &AnimationIndices, &mut TextureAtlasSprite)>, 
) {
    for (eye, indices, mut sprite) in eye_query.iter_mut() {
        sprite.index = (((indices.last as f32) * eye.activation).ceil() as usize).max(indices.first).min(indices.last);
    }
}