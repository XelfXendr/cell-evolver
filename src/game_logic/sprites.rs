use std::time::Duration;

use bevy::{prelude::*, render::texture::ImageSampler, sprite::Anchor};
use rand::Rng;

use super::cell::*;

pub struct SpritesPlugin;
impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut App) {
        app            
            .init_resource::<CellSprite>()
            .init_resource::<EyeSprite>()
            .init_resource::<FlagellumSprite>()
            .init_resource::<FoodSprites>()
            .add_systems(Update, (
                texture_fixer,
                animate_sprite,
                flagellum_animation_speed,
                eye_focus_animation,
                cell_sprite_adder,
                flagellum_sprite_adder,
                eye_sprite_adder,
                food_sprite_adder,
            ));
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct EyeSprite(pub Handle<TextureAtlas>);
impl FromWorld for EyeSprite {
    fn from_world(world: &mut World) -> Self {
        EyeSprite({
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
        FlagellumSprite({
            let flagellum_texture: Handle<Image> = world.resource::<AssetServer>().load("sprites/cell_parts/flagella.png");
            let flagellum_atlas = TextureAtlas::from_grid(flagellum_texture, Vec2::new(88.,110.), 8, 1, None, None);
            world.resource_mut::<Assets<TextureAtlas>>().add(flagellum_atlas)
        })
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct FoodSprites(pub Vec<Handle<Image>>);
impl FromWorld for FoodSprites {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        FoodSprites(
            (1..14).map(|n| asset_server.load(format!("sprites/food/{:0>2}.png", n))).collect()
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

#[derive(Resource, Deref, DerefMut)]
pub struct CellSprite(pub Handle<Image>);
impl FromWorld for CellSprite {
    fn from_world(world: &mut World) -> Self {
        CellSprite(
            world.resource::<AssetServer>().load("sprites/cell_parts/cell.png")
        )
    }
}


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

pub fn cell_sprite_adder(
    mut commands: Commands,
    new_cell_query: Query<Entity, Added<Cell>>,
    cell_sprite: Res<CellSprite>,
) {
    for cell_entity in new_cell_query.iter() {
        if let Some(mut entity) = commands.get_entity(cell_entity) {
            let sprite = entity.commands().spawn(SpriteBundle {
                texture: cell_sprite.clone(),
                sprite: Sprite{
                    custom_size: Some(Vec2::new(100., 100.)),
                    ..default()
                },
                ..default()
            }).id();
            entity.add_child(sprite);
        }
    }
}

pub fn flagellum_sprite_adder(
    mut commands: Commands,
    new_flagellum_query: Query<Entity, Added<Flagellum>>,
    flagellum_sprite: Res<FlagellumSprite>,
) {
    for flagellum_entity in new_flagellum_query.iter() {
        if let Some(mut entity) = commands.get_entity(flagellum_entity) {
            let sprite = entity.commands().spawn((
                FlagellumSpriteTag,
                SpriteSheetBundle {
                    texture_atlas: flagellum_sprite.clone(),
                    sprite: {
                        let mut flagella_sprite = TextureAtlasSprite::new(0);
                        flagella_sprite.anchor = Anchor::Custom(Vec2::new(0., 0.46));
                        flagella_sprite.custom_size = Some(Vec2::new(50.,50./88.*110.));
                        flagella_sprite
                    },
                    ..default()
                },
                AnimationIndices {first: 0, last: 7},
                AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            )).id();
            entity.add_child(sprite);
        }
    }
}

pub fn eye_sprite_adder(
    mut commands: Commands,
    new_eye_query: Query<Entity, Added<Eye>>,
    eye_sprite: Res<EyeSprite>,
) {
    for eye_entity in new_eye_query.iter() {
        if let Some(mut entity) = commands.get_entity(eye_entity) {
            let sprite = entity.commands().spawn((
                EyeSpriteTag,
                SpriteSheetBundle {
                    texture_atlas: eye_sprite.clone(),
                    sprite: {
                        let mut eye_sprite = TextureAtlasSprite::new(0);
                        eye_sprite.custom_size = Some(Vec2::new(50./88.*32.,50./88.*32.));
                        eye_sprite
                    },
                    ..default()
                },
                AnimationIndices {first: 0, last: 7},
            )).id();
            entity.add_child(sprite);
        }
    }
}

pub fn food_sprite_adder(
    mut commands: Commands,
    new_food_query: Query<Entity, Added<Food>>,
    food_sprites: Res<FoodSprites>,
) {
    for food_entity in new_food_query.iter() {
        if let Some(mut entity) = commands.get_entity(food_entity) {
            let rand_index = rand::thread_rng().gen_range(0..food_sprites.len());
            let sprite = entity.commands().spawn(SpriteBundle{
                texture: food_sprites[rand_index].clone(),
                sprite: Sprite{
                    custom_size: Some(Vec2::new(20., 20.)),
                    ..default()
                },
                ..default()
            }).id();
            entity.add_child(sprite);
        }
    }
}