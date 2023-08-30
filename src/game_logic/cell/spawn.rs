use bevy::{prelude::*, sprite::Anchor};
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use ndarray::{Array2, Array1};

use crate::game_logic::sprites::*;
use super::*;

pub fn spawn_cell(
    commands: &mut Commands,
    cell_spawn_event_writer: &mut EventWriter<CellSpawnEvent>,
    flagellum_spawn_event_writer: &mut EventWriter<FlagellumSpawnEvent>,
    eye_spawn_event_writer: &mut EventWriter<EyeSpawnEvent>,
    position: Vec3,
    rotation: Quat,
    energy: f32,
    flagella_params: Vec<(f32, f32)>,
    eye_params: Vec<f32>,
    weights: Array2<f32>,
    biases: Array1<f32>,
    state: Array1<f32>,
    cell_sprite: Option<&CellSprite>,
    light_sprite: Option<&LightSprite>,
    flagellum_sprite: Option<&FlagellumSprite>,
    eye_sprite: Option<&EyeSprite>,
) -> Entity {
    let radius = 5. * energy.sqrt();

    let flagella: Vec<Entity> = flagella_params.iter().map(
        |(pos, ang)| spawn_flagellum(commands, flagellum_spawn_event_writer, *pos, *ang, radius, flagellum_sprite)
    ).collect();
    let eyes: Vec<Entity> = eye_params.iter().map(
        |pos| spawn_eye(commands, eye_spawn_event_writer, *pos, radius, eye_sprite)
    ).collect(); 
    let collider = commands.spawn((
        SpatialBundle::default(),
        Collider::ball(radius),
    )).id();
    let sprites = {
        let mut vec: Vec<Entity> = Vec::new();
        if let Some(sprite) = cell_sprite {
            vec.push(commands.spawn(SpriteBundle {
                texture: sprite.0.clone(),
                sprite: Sprite{
                    custom_size: Some(Vec2::new(radius*2., radius*2.)),
                    color: Color::hex("8db5fb").unwrap(),
                    ..default()
                },
                ..default()
            }).id());
        }
        if let Some(sprite) = light_sprite {
            vec.push(commands.spawn(SpriteBundle {
                texture: sprite.0.clone(),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(radius*20.,radius*20.)),
                    color: Color::rgba_u8(100,200,255,50),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0.,0.,-100.)),
                ..default()
            }).id());
        }
        vec
    };

    let cell = commands.spawn((
        CellBundle::new( 
            flagella.clone(),
            eyes.clone(),
            collider,
            sprites.clone(),
            flagella_params,
            eye_params,
            energy, false,
            weights, biases, state,
            position, rotation,
        ),
    )).id();

    commands.entity(cell).push_children(&flagella);
    commands.entity(cell).push_children(&eyes);
    commands.entity(cell).push_children(&[collider]);
    commands.entity(cell).push_children(&sprites);
    cell_spawn_event_writer.send(CellSpawnEvent(cell));
    cell
}

pub fn spawn_flagellum(
    commands: &mut Commands,
    flagellum_spawn_event_writer: &mut EventWriter<FlagellumSpawnEvent>,
    position: f32,
    angle: f32,
    radius: f32,
    flagellum_sprite: Option<&FlagellumSprite>,
) -> Entity{
    let vert = -position.cos() * radius;
    let horiz = position.sin() * radius;

    let flagellum = commands.spawn((
        FlagellumBundle::new(0., angle),
        SpatialBundle::from_transform(
            Transform::from_rotation(Quat::from_rotation_z(position + angle))
                .with_translation(Vec3::new(horiz, vert, 2.))
        ),
    )).with_children(|c| {
        if let Some(sprite) = flagellum_sprite {
            c.spawn((
                FlagellumSpriteTag,
                SpriteSheetBundle {
                    texture_atlas: sprite.0.clone(),
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
            ));
        }
    }).id();

    flagellum_spawn_event_writer.send(FlagellumSpawnEvent(flagellum));
    flagellum
}

pub fn spawn_eye(
    commands: & mut Commands,
    eye_spawn_event_writer: &mut EventWriter<EyeSpawnEvent>,
    position: f32,
    radius: f32,
    eye_sprite: Option<&EyeSprite>,
) -> Entity{
    let vert = -position.cos() * radius;
    let horiz = position.sin() * radius;
    
    let eye = commands.spawn((
        EyeBundle::new(0.),
        SpatialBundle::from_transform(
            Transform::from_rotation(Quat::from_rotation_z(position))
                .with_translation(Vec3::new(horiz, vert, 2.))
        ),
        Collider::convex_polyline(vec![
            Vec2::new(-10., -5.),
            Vec2::new(10., -5.), 
            Vec2::new(300., -1000.),
            Vec2::new(-300., -1000.), 
        ]).unwrap(),
    )).with_children(|c| {
        if let Some(sprite) = eye_sprite {
            c.spawn((
                EyeSpriteTag,
                SpriteSheetBundle {
                    texture_atlas: sprite.0.clone(),
                    sprite: {
                        let mut eye_sprite = TextureAtlasSprite::new(0);
                        eye_sprite.custom_size = Some(Vec2::new(50./88.*32.,50./88.*32.));
                        eye_sprite
                    },
                    ..default()
                },
                AnimationIndices {first: 0, last: 7},
            ));
            let mut path_builder = PathBuilder::new();
            path_builder.move_to(Vec2::new(-10., -5.));
            path_builder.line_to(Vec2::new(10., -5.));
            path_builder.line_to(Vec2::new(300., -1000.));
            path_builder.line_to(Vec2::new(-300., -1000.));
            path_builder.line_to(Vec2::new(-10., -5.));
            path_builder.close();
            let path = path_builder.build();
            c.spawn((
                ShapeBundle{
                    path: path,
                    transform: Transform::from_xyz(0., 0., -1.),
                    ..default()
                },
                Fill::color(Color::rgba_u8(255, 255, 255, 10)),
            ));
        }
    }).id();


    eye_spawn_event_writer.send(EyeSpawnEvent(eye));
    eye
}

pub fn despawn_cell(
    despawn_queue: &mut DelayedDespawnQueue,
    cell_despawn_event_writer: &mut EventWriter<CellDespawnEvent>,
    cell_entity: Entity
) {
    despawn_queue.add(cell_entity);
    cell_despawn_event_writer.send(CellDespawnEvent(cell_entity));
}

pub fn spawn_food(
    commands: &mut Commands,
    food_spawn_event_writer: &mut EventWriter<FoodSpawnEvent>,
    position: Vec3,
    food_sprite: Option<&FoodSprite>,
    light_sprite: Option<&LightSprite>,
) -> Entity {
    let food = commands.spawn((
        FoodBundle::new(),
        SpatialBundle::from_transform(Transform::from_translation(position)),
        Collider::ball(10.),
    )).with_children(|c| {
        if let Some(sprite) = food_sprite {
            c.spawn(SpriteBundle{
                texture: sprite.0.clone(),
                sprite: Sprite{
                    custom_size: Some(Vec2::new(20., 20.)),
                    ..default()
                },
                ..default()
            });
        }
        if let Some(sprite) = light_sprite {
            c.spawn(SpriteBundle {
                texture: sprite.0.clone(),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(200.,200.)),
                    color: Color::rgba_u8(150,255,150,100),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0.,0.,-100.)),
                ..default()
            });
        }
    }).id();

    food_spawn_event_writer.send(FoodSpawnEvent(food));
    food
}

pub fn despawn_food(
    despawn_queue: &mut DelayedDespawnQueue,
    food_despawn_event_writer: &mut EventWriter<FoodDespawnEvent>,
    food_entity: Entity,
) {
    despawn_queue.add(food_entity);
    food_despawn_event_writer.send(FoodDespawnEvent(food_entity));
}