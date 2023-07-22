use std::f32::consts::E;
use std::f32::consts::PI;
use std::time::Duration;

use bevy::ecs::query::BatchingStrategy;
use bevy::{prelude::*, sprite::Anchor};
use bevy_rapier2d::prelude::*;
use ndarray::s;
use rand_distr::{Normal, Distribution};
use rand::{self, Rng};
use ndarray::{Array1, Array2};
use ndarray_rand::{RandomExt, rand_distr::Normal as ndNormal};

use crate::sprite_animation::*;
use crate::physics::*;

pub const PLAYER_SPEED: f32 = 500.;
pub const PLAYER_ANGLE_SPEED: f32 = 7.;

pub const CELL_GROUP: Group = Group::from_bits_truncate(0b0001);
pub const EYE_GROUP: Group  = Group::from_bits_truncate(0b0010);
pub const FOOD_GROUP: Group = Group::from_bits_truncate(0b0100);

pub const SPLIT_ENERGY: f32 = 200.;
pub const MIN_ENERGY: f32 = 70.;

pub const COLLISION_FLAGS: ActiveCollisionTypes = ActiveCollisionTypes::STATIC_STATIC;

pub const FIXED_DELTA: f32 = 1./60.;

pub struct CellPlugin;
impl Plugin for CellPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CellSprite>()
            .init_resource::<EyeSprite>()
            .init_resource::<FlagellumSprite>()
            .init_resource::<FoodSprites>()
            .add_systems(Startup, (
                cell_setup,
            ))
            .add_systems(Update, (
                //keyboard_movement,
                count_cells,
                food_spawning,
                dynamic_thing,
            ))
            .add_systems(FixedUpdate, (
                cell_food_intersection,
                eye_sensing,
                cell_thinking,
                decrement_energy,
                split_cells,
                fixed_thing
            ));  
    }
}

#[derive(Resource)]
pub struct CellSprite(Handle<Image>);
impl FromWorld for CellSprite {
    fn from_world(world: &mut World) -> Self {
        CellSprite(
            world.resource::<AssetServer>().load("sprites/cell_parts/cell.png")
        )
    }
}

#[derive(Resource)]
pub struct EyeSprite(Handle<TextureAtlas>);
impl FromWorld for EyeSprite {
    fn from_world(world: &mut World) -> Self {
        EyeSprite({
            let eye_texture: Handle<Image> = world.resource::<AssetServer>().load("sprites/cell_parts/eye.png");
            let eye_atlas = TextureAtlas::from_grid(eye_texture, Vec2::new(32.,32.), 8, 1, None, None);
            world.resource_mut::<Assets<TextureAtlas>>().add(eye_atlas)
        })
    }
}
#[derive(Resource)]
pub struct FlagellumSprite(Handle<TextureAtlas>);
impl FromWorld for FlagellumSprite {
    fn from_world(world: &mut World) -> Self {
        FlagellumSprite({
            let flagellum_texture: Handle<Image> = world.resource::<AssetServer>().load("sprites/cell_parts/flagella.png");
            let flagellum_atlas = TextureAtlas::from_grid(flagellum_texture, Vec2::new(88.,110.), 8, 1, None, None);
            world.resource_mut::<Assets<TextureAtlas>>().add(flagellum_atlas)
        })
    }
}

#[derive(Resource)]
pub struct FoodSprites(Vec<Handle<Image>>);
impl FromWorld for FoodSprites {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        FoodSprites(
            (1..14).map(|n| asset_server.load(format!("sprites/food/{:0>2}.png", n))).collect()
        )
    }
}

#[derive(Component)]
pub struct Cell {
    pub flagella: Vec<Entity>,
    pub eyes: Vec<Entity>,
    pub energy: f32,
    pub flagella_params: Vec<(f32, f32)>,
    pub eye_params: Vec<f32>,
    pub weights: Array2<f32>,
    pub biases: Array1<f32>,
    pub state: Array1<f32>,
}

#[derive(Component)]
pub struct Flagellum {
    pub activation: f32,
    pub angle: f32,
}

#[derive(Component)]
pub struct Eye {
    pub activation: f32,
}

#[derive(Component)]
pub struct Food {
}

#[derive(Component)]
pub struct ThinkingTimer(Timer);

#[derive(Resource)]
pub struct FoodTimer(Timer);

#[derive(Resource)]
pub struct DebugTimer(Timer);

#[derive(Resource)]
pub struct TimeCounter(f32, f32);

pub fn cell_setup(
    mut commands: Commands, 
    cell_image: Res<CellSprite>,
    eye_image: Res<EyeSprite>,
    flagellum_image: Res<FlagellumSprite>,
    food_image: Res<FoodSprites>,
) {
    commands.insert_resource(FoodTimer(Timer::new(Duration::from_secs_f32(0.05), TimerMode::Repeating)));
    commands.insert_resource(DebugTimer(Timer::new(Duration::from_secs_f32(1.), TimerMode::Repeating)));

    let normal = Normal::new(0., 10000.).unwrap();
    let mut rng = rand::thread_rng();

    for _ in 0..20 {
        spawn_cell(
            &mut commands,
            Vec3::new(normal.sample(&mut rng), normal.sample(&mut rng),0.),
            Quat::from_rotation_z(0.),
            100.,
            &cell_image,
            &eye_image,
            &flagellum_image,
            vec![(PI/2., -PI/4.), (0., 0.), (-PI/2.,  PI/4.)],
            vec![PI*5.2/6., PI, PI*6.8/6.],
            Array2::random((100,100), Normal::new(0., 0.5).unwrap()),
            Array1::random(100, Normal::new(0., 0.5).unwrap()),
            Array1::random(100, Normal::new(0., 0.5).unwrap()),
        );
    }
    
    for _ in 0..10000 {
        spawn_food(
            &mut commands, 
            food_image.0[rng.gen_range(0..13)].clone(), 
            Vec3::new(normal.sample(&mut rng), normal.sample(&mut rng), 0.)
        );
    }
}

pub fn spawn_cell(
    commands: &mut Commands,
    position: Vec3,
    rotation: Quat,
    energy: f32,
    cell_image: &CellSprite,
    eye_image: &EyeSprite,
    flagellum_image: &FlagellumSprite,
    flagella_params: Vec<(f32, f32)>,
    eye_params: Vec<f32>,
    weights: Array2<f32>,
    biases: Array1<f32>,
    state: Array1<f32>,
) {
    commands.insert_resource(TimeCounter(0., 0.));

    let flagella: Vec<Entity> = flagella_params.iter().map(
        |(pos, ang)| spawn_flagellum(commands, flagellum_image.0.clone(), *pos, *ang)
    ).collect();
    let eyes: Vec<Entity> = eye_params.iter().map(
        |pos| spawn_eye(commands, eye_image.0.clone(), *pos)
    ).collect(); 

    let cell = commands.spawn((
        Cell { 
            flagella: flagella.clone(),
            eyes: eyes.clone(),
            energy: energy,
            flagella_params: flagella_params,
            eye_params: eye_params,
            weights: weights, biases: biases, state: state,
        },
        PhysicsBody {
            velocity: Vec2::ZERO, 
            acceleration: Vec2::ZERO,
            angular_velocity: 0.,
            angular_acceleration: 0.,
            drag: 2.,
            angular_drag: 2.,
        },
        SpatialBundle::from_transform(
            Transform::from_translation(position)
                .with_rotation(rotation)
        ),
        Collider::ball(50.),
        ThinkingTimer(Timer::from_seconds(1./20., TimerMode::Repeating)),
    )).with_children(|parent| {
        parent.spawn(SpriteBundle {
            texture: cell_image.0.clone(),
            sprite: Sprite{
                custom_size: Some(Vec2::new(100., 100.)),
                ..default()
            },
            ..default()
        });
    }).id();

    commands.entity(cell).push_children(&flagella);
    commands.entity(cell).push_children(&eyes);
}

pub fn spawn_flagellum(
    commands: &mut Commands,
    atlas_handle: Handle<TextureAtlas>,
    position: f32,
    angle: f32,
) -> Entity{
    let vert = -position.cos() * 50.;
    let horiz = position.sin() * 50.;

    commands.spawn((
        Flagellum{
            activation: 0.,
            angle: angle,
        },
        SpriteSheetBundle {
            texture_atlas: atlas_handle.clone(),
            sprite: {
                let mut flagella_sprite = TextureAtlasSprite::new(0);
                flagella_sprite.anchor = Anchor::Custom(Vec2::new(0., 0.46));
                flagella_sprite.custom_size = Some(Vec2::new(50.,50./88.*110.));
                flagella_sprite
            },
            transform: Transform::from_rotation(Quat::from_rotation_z(position + angle))
                .with_translation(Vec3::new(horiz, vert, 2.)),
            ..default()
        },
        AnimationIndices {first: 0, last: 7},
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    )).id()
}

pub fn spawn_eye(
    commands: & mut Commands,
    atlas_handle: Handle<TextureAtlas>,
    position: f32,
) -> Entity{
    let vert = -position.cos() * 50.;
    let horiz = position.sin() * 50.;

    commands.spawn((
        Eye{
            activation: 0.,
        },
        SpriteSheetBundle {
            texture_atlas: atlas_handle.clone(),
            sprite: {
                let mut eye_sprite = TextureAtlasSprite::new(0);
                eye_sprite.custom_size = Some(Vec2::new(50./88.*32.,50./88.*32.));
                eye_sprite
            },
            transform: Transform::from_rotation(Quat::from_rotation_z(position))
                .with_translation(Vec3::new(horiz, vert, 2.)),
            ..default()
        },
        AnimationIndices {first: 0, last: 7},
        Collider::convex_polyline(vec![
            Vec2::new(-10., -5.),
            Vec2::new(10., -5.), 
            Vec2::new(300., -1000.),
            Vec2::new(-300., -1000.), 
            ]).unwrap(),
    )).id()
}

pub fn spawn_food(
    commands: &mut Commands,
    texture: Handle<Image>,
    position: Vec3,
) -> Entity {
    commands.spawn((
        Food {},
        SpriteBundle{
            transform: Transform::from_translation(position),
            texture: texture,
            sprite: Sprite{
                custom_size: Some(Vec2::new(20., 20.)),
                ..default()
            },
            ..default()
        },
        Collider::ball(10.),
    )).id()
}

pub fn cell_food_intersection(
    mut cell_query: Query<(Entity, &mut Cell, &Collider, &GlobalTransform)>,
    food_query: Query<&Food>,
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
) {
    for (cell_entity, mut cell, collider, transform) in cell_query.iter_mut() {
        let direction = quat_to_direction(transform.to_scale_rotation_translation().1);
        let angle = (-direction.x).atan2(direction.y);
        let pos = transform.translation();
        rapier_context.intersections_with_shape(
            Vec2::new(pos.x, pos.y), 
            angle, 
            collider, 
            QueryFilter::default(), 
            |x| {
                if food_query.contains(x) {
                    commands.entity(x).despawn();
                    cell.energy += 10.
                }
                true
            }
        )
    }
}

pub fn eye_sensing(
    mut eye_query: Query<(&GlobalTransform, &mut Eye, &Collider)>,
    food_query: Query<&GlobalTransform, With<Food>>,
    rapier_context: Res<RapierContext>,
) {
    eye_query
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::new().min_batch_size(32))
        .for_each_mut(|(eye_transform, mut eye, collider)| {

        let mut activation: f32 = 0.;
        let direction = quat_to_direction(eye_transform.to_scale_rotation_translation().1);
        let angle = (-direction.x).atan2(direction.y);
        let pos = eye_transform.translation();
        rapier_context.intersections_with_shape(
            Vec2::new(pos.x, pos.y), 
            angle, 
            collider, 
            QueryFilter::default(), 
            |x| {
                if let Ok(food_transform) = food_query.get(x) {
                    //commands.entity(x).despawn();
                    let distance = eye_transform.translation().distance(food_transform.translation());
                    activation = activation.max((1.-distance/1000.).max(0.).min(1.));
                }
                true
            }
        );

        eye.activation = activation;
    });
}

/*
pub fn manual_eye_thing(
    cell_query: Query<&Cell>,
    mut flag_query: Query<&mut Flagellum>,
    eye_query: Query<&Eye>,
 ) {
    for cell in cell_query.iter() {
        let activations: Vec<f32> = cell.eyes.iter().map(|eye| eye_query.get(*eye).unwrap().activation).collect();
        let activations = Array1::from_vec(activations);

        let mut hidden = activations.dot(&cell.w1) + &cell.b1;
        hidden.map_inplace(tanh_inplace);
        let mut activations = hidden.dot(&cell.w2) + &cell.b2;
        activations.map_inplace(sigmoid_inplace);
        
        for (f, a) in cell.flagella.iter().zip(activations) {
            let mut flagellum = flag_query.get_mut(*f).unwrap();
            flagellum.activation = a;
        }
        
    }
}
*/
pub fn cell_thinking(
    mut cell_query: Query<(&mut Cell, &mut ThinkingTimer)>,
    mut flag_query: Query<&mut Flagellum>,
    eye_query: Query<&Eye>,
) {
    for (mut cell, mut timer) in cell_query.iter_mut() {
        timer.0.tick(Duration::from_secs_f32(FIXED_DELTA));
        if timer.0.finished() {
            let activations: Vec<f32> = cell.eyes.iter().map(|eye| eye_query.get(*eye).unwrap().activation).collect();
            for (i, act) in activations.iter().enumerate() {
                cell.state[i] = *act;
            }
            
            cell.state = cell.state.dot(&cell.weights) + &cell.biases;
            let activation_range = s![cell.flagella.len()..cell.state.shape()[0]-cell.eyes.len()];
            cell.state.slice_mut(activation_range).map_inplace(tanh_inplace);
            let activation_range = s![cell.state.shape()[0]-cell.eyes.len()..];
            cell.state.slice_mut(activation_range).map_inplace(sigmoid_inplace);
            
            for (f, a) in cell.flagella.iter().zip(cell.state.slice(activation_range)) {
                let mut flagellum = flag_query.get_mut(*f).unwrap();
                flagellum.activation = *a;
            }
        }
    }
}

pub fn decrement_energy(
    mut cell_query: Query<(Entity, &mut Cell)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (cell_entity, mut cell) in cell_query.iter_mut() {
        cell.energy -= FIXED_DELTA;
        if cell.energy < MIN_ENERGY {
            commands.entity(cell_entity).despawn_recursive();
        }
    }
}

pub fn split_cells(
    cell_query: Query<(Entity, &Cell, &Transform)>,
    mut commands: Commands,
    cell_image: Res<CellSprite>,
    eye_image: Res<EyeSprite>,
    flagellum_image: Res<FlagellumSprite>,
) {
    for (cell_entity, cell, cell_transform) in cell_query.iter().filter(|x| x.1.energy >= SPLIT_ENERGY) {
        let position = cell_transform.translation;
        let rotation = cell_transform.rotation;
        let (weights, biases, state) = (&cell.weights, &cell.biases, &cell.state);
        
        let normal = Normal::new(0., 0.01).unwrap();
        let weight_normal = Normal::new(0., 0.1).unwrap();
        let mut rng = rand::thread_rng();

        commands.entity(cell_entity).despawn_recursive();
        spawn_cell(&mut commands, 
            position, 
            rotation * Quat::from_rotation_z(0.1), 
            100., 
            &cell_image, &eye_image, &flagellum_image,
            cell.flagella_params.iter().map(|(pos, ang)| (pos + normal.sample(&mut rng), (ang + normal.sample(&mut rng)).clamp(-PI/2., PI/2.))).collect(),
            cell.eye_params.iter().map(|pos| pos + normal.sample(&mut rng)).collect(),
            weights.map(|x| x + weight_normal.sample(&mut rng)),
            biases.map(|x| x + weight_normal.sample(&mut rng)),
            state.clone(),
            );
        spawn_cell(&mut commands, 
            position, 
            rotation * Quat::from_rotation_z(-0.1), 
            100., 
            &cell_image, &eye_image, &flagellum_image,
            cell.flagella_params.iter().map(|(pos, ang)| (pos + normal.sample(&mut rng), (ang + normal.sample(&mut rng)).clamp(-PI/2., PI/2.))).collect(),
            cell.eye_params.iter().map(|pos| pos + normal.sample(&mut rng)).collect(),
            weights.map(|x| x + weight_normal.sample(&mut rng)),
            biases.map(|x| x + weight_normal.sample(&mut rng)),
            state.clone(),
        );
    }
}

pub fn food_spawning(
    mut commands: Commands,
    mut timer: ResMut<FoodTimer>,
    food_image: Res<FoodSprites>,
) {
    timer.0.tick(Duration::from_secs_f32(FIXED_DELTA));
    if timer.0.finished() {
        let normal = Normal::new(0., 15000.).unwrap();
        let mut rng = rand::thread_rng();
        
        spawn_food(
            &mut commands, 
            food_image.0[rng.gen_range(0..13)].clone(), 
            Vec3::new(normal.sample(&mut rng), normal.sample(&mut rng), 0.)
        );
    }
}

pub fn sigmoid(x: f32) -> f32 {
    1. / (1. + E.powf(-x))
}

pub fn sigmoid_inplace(x: &mut f32) {
    *x = sigmoid(*x);
}

pub fn tanh_inplace(x: &mut f32) {
    *x = f32::tanh(*x);
}

pub fn count_cells(cell_query: Query<&Cell>, food_query: Query<&Food>, mut timer: ResMut<DebugTimer>, time: Res<Time>) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        println!("FPS: {}, cell_count: {}, food count: {}", (1./time.delta_seconds()).round(), cell_query.iter().count(), food_query.iter().count());
    }
}

pub fn dynamic_thing(time: Res<Time>, mut cnter: ResMut<TimeCounter>) {
    cnter.0 += time.delta_seconds();
}

pub fn fixed_thing(mut cnter: ResMut<TimeCounter>) {
    cnter.1 += 1.;
    if cnter.0 >= 1. {
        println!("FixedFPS: {:?}", cnter.1);
        cnter.0 -= 1.;
        cnter.1 = 0.;
    }
}