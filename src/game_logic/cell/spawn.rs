use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use ndarray::{Array2, Array1};

use crate::game_logic::physics::*;
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
) -> Entity {
    let flagella: Vec<Entity> = flagella_params.iter().map(
        |(pos, ang)| spawn_flagellum(commands, flagellum_spawn_event_writer, *pos, *ang)
    ).collect();
    let eyes: Vec<Entity> = eye_params.iter().map(
        |pos| spawn_eye(commands, eye_spawn_event_writer, *pos)
    ).collect(); 

    let cell = commands.spawn((
        CellBundle::new( 
            flagella.clone(),
            eyes.clone(),
            flagella_params,
            eye_params,
            energy, false,
            weights, biases, state,
        ),
        PhysicsBundle::from_drag(2., 2.),
        SpatialBundle::from_transform(
            Transform::from_translation(position)
                .with_rotation(rotation)
        ),
        Collider::ball(50.),
        ThinkingTimer(Timer::from_seconds(1./20., TimerMode::Repeating)),
    )).id();

    commands.entity(cell).push_children(&flagella);
    commands.entity(cell).push_children(&eyes);
    cell_spawn_event_writer.send(CellSpawnEvent(cell));
    cell
}

pub fn spawn_flagellum(
    commands: &mut Commands,
    flagellum_spawn_event_writer: &mut EventWriter<FlagellumSpawnEvent>,
    position: f32,
    angle: f32,
) -> Entity{
    let vert = -position.cos() * 50.;
    let horiz = position.sin() * 50.;

    let flagellum = commands.spawn((
        FlagellumBundle::new(0., angle),
        SpatialBundle::from_transform(
            Transform::from_rotation(Quat::from_rotation_z(position + angle))
                .with_translation(Vec3::new(horiz, vert, 2.))
        ),
    )).id();

    flagellum_spawn_event_writer.send(FlagellumSpawnEvent(flagellum));
    flagellum
}

pub fn spawn_eye(
    commands: & mut Commands,
    eye_spawn_event_writer: &mut EventWriter<EyeSpawnEvent>,
    position: f32,
) -> Entity{
    let vert = -position.cos() * 50.;
    let horiz = position.sin() * 50.;

    let mut path_builder = PathBuilder::new();
    path_builder.move_to(Vec2::new(-10., -5.));
    path_builder.line_to(Vec2::new(10., -5.));
    path_builder.line_to(Vec2::new(300., -1000.));
    path_builder.line_to(Vec2::new(-300., -1000.));
    path_builder.line_to(Vec2::new(-10., -5.));
    path_builder.close();
    let path = path_builder.build();

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
        c.spawn((
            ShapeBundle{
                path: path,
                transform: Transform::from_xyz(0., 0., -1.),
                ..default()
            },
            Fill::color(Color::rgba_u8(255, 255, 255, 10)),
        ));
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
) -> Entity {
    let food = commands.spawn((
        FoodBundle::new(),
        SpatialBundle::from_transform(Transform::from_translation(position)),
        Collider::ball(10.),
    )).id();

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