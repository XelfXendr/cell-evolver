use std::time::Duration;

use bevy::prelude::*;

use super::cell::*;

pub struct SpritesPlugin;
impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut App) {
        app            
            .init_resource::<CellSprite>()
            .init_resource::<EyeSprite>()
            .init_resource::<FlagellumSprite>()
            .init_resource::<FoodSprite>()
            .init_resource::<LightSprite>()
            .add_systems(Update, (
                animate_sprite,
                flagellum_animation_speed,
                eye_focus_animation,
            ));
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct CellSprite(pub Handle<Image>);
impl FromWorld for CellSprite {
    fn from_world(world: &mut World) -> Self {
        Self(
            world.resource::<AssetServer>().load("sprites/cell_parts/cell.png")
        )
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct EyeSprite(pub Handle<TextureAtlas>);
impl FromWorld for EyeSprite {
    fn from_world(world: &mut World) -> Self {
        Self({
            let eye_texture: Handle<Image> = world.resource::<AssetServer>().load("sprites/cell_parts/eye.png");
            let eye_atlas = TextureAtlas::from_grid(eye_texture, Vec2::new(32.,32.), 8, 1, None, None);
            world.resource_mut::<Assets<TextureAtlas>>().add(eye_atlas)
        })
    }
}
#[derive(Resource, Deref, DerefMut)]
pub struct FlagellumSprite(pub Handle<TextureAtlas>);
impl FromWorld for FlagellumSprite {
    fn from_world(world: &mut World) -> Self {
        Self({
            let flagellum_texture: Handle<Image> = world.resource::<AssetServer>().load("sprites/cell_parts/flagella.png");
            let flagellum_atlas = TextureAtlas::from_grid(flagellum_texture, Vec2::new(88.,110.), 8, 1, None, None);
            world.resource_mut::<Assets<TextureAtlas>>().add(flagellum_atlas)
        })
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct FoodSprite(pub Handle<Image>);
impl FromWorld for FoodSprite {
    fn from_world(world: &mut World) -> Self {
        Self(
            world.resource::<AssetServer>().load("sprites/food/food.png")
        )
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct LightSprite(pub Handle<Image>);
impl FromWorld for LightSprite {
    fn from_world(world: &mut World) -> Self {
        Self(
            world.resource::<AssetServer>().load("sprites/lights/normal_light.png")
        )
    }
}

#[derive(Component)]
pub struct FlagellumSpriteTag;

#[derive(Component)]
pub struct EyeSpriteTag;

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
    mut sprite_query: Query<(&Parent, &mut AnimationTimer), With<FlagellumSpriteTag>>,
    flag_query: Query<&Activation, With<Flagellum>>, 
) {
    for (parent, mut timer) in sprite_query.iter_mut() {
        if let Ok(activation) = flag_query.get(parent.get()) {
            timer.set_duration(
                Duration::from_secs_f32(0.033 / f32::max(0.1, **activation))
            );
        } 
    }
}

pub fn eye_focus_animation(
    mut sprite_query: Query<(&Parent, &AnimationIndices, &mut TextureAtlasSprite), With<EyeSpriteTag>>,
    eye_query: Query<&Activation, With<Eye>> 
) {
    for (parent, indices, mut sprite) in sprite_query.iter_mut() {
        if let Ok(activation) = eye_query.get(parent.get()) {
            sprite.index = (((indices.last as f32) * **activation).ceil() as usize).max(indices.first).min(indices.last);
        }
    }
}