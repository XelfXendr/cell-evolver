use std::f32::consts::E;
use std::f32::consts::PI;
use std::time::Duration;

use bevy::ecs::query::BatchingStrategy;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use ndarray::s;
use rand_distr::{Normal, Distribution};
use rand;
use ndarray::{Array1, Array2};
use ndarray_rand::RandomExt;

use crate::physics::*;

pub const PLAYER_SPEED: f32 = 500.;
pub const PLAYER_ANGLE_SPEED: f32 = 7.;

pub const SPLIT_ENERGY: f32 = 200.;
pub const MIN_ENERGY: f32 = 70.;

pub const FIXED_DELTA: f32 = 1./60.;

pub struct CellPlugin;
impl Plugin for CellPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<CellSpawnEvent>()
            .add_event::<FlagellumSpawnEvent>()
            .add_event::<EyeSpawnEvent>()
            .add_event::<FoodSpawnEvent>()
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

#[derive(Component, Deref, DerefMut)]
pub struct ThinkingTimer(Timer);

#[derive(Resource, Deref, DerefMut)]
pub struct FoodTimer(Timer);

#[derive(Resource, Deref, DerefMut)]
pub struct DebugTimer(Timer);

#[derive(Resource)]
pub struct TimeCounter(f32, f32);

#[derive(Event, Deref, DerefMut)]
pub struct CellSpawnEvent(Entity);

#[derive(Event, Deref, DerefMut)]
pub struct FlagellumSpawnEvent(Entity);

#[derive(Event, Deref, DerefMut)]
pub struct EyeSpawnEvent(Entity);

#[derive(Event, Deref, DerefMut)]
pub struct FoodSpawnEvent(Entity);

pub fn cell_setup(
    mut commands: Commands, 
    mut cell_spawn_event_writer: EventWriter<CellSpawnEvent>,
    mut flagellum_spawn_event_writer: EventWriter<FlagellumSpawnEvent>,
    mut eye_spawn_event_writer: EventWriter<EyeSpawnEvent>,
    mut food_spawn_event_writer: EventWriter<FoodSpawnEvent>,
) {
    commands.insert_resource(FoodTimer(Timer::new(Duration::from_secs_f32(0.05), TimerMode::Repeating)));
    commands.insert_resource(DebugTimer(Timer::new(Duration::from_secs_f32(1.), TimerMode::Repeating)));
    commands.insert_resource(TimeCounter(0., 0.));

    let normal = Normal::new(0., 10000.).unwrap();
    let mut rng = rand::thread_rng();
    
    for _ in 0..20 {
        spawn_cell(
            &mut commands,
            &mut cell_spawn_event_writer, &mut flagellum_spawn_event_writer, &mut eye_spawn_event_writer,
            Vec3::new(normal.sample(&mut rng), normal.sample(&mut rng),0.),
            Quat::from_rotation_z(0.),
            100.,
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
            &mut food_spawn_event_writer,
            Vec3::new(normal.sample(&mut rng), normal.sample(&mut rng), 0.)
        );
    }
}

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
) {
    let flagella: Vec<Entity> = flagella_params.iter().map(
        |(pos, ang)| spawn_flagellum(commands, flagellum_spawn_event_writer, *pos, *ang)
    ).collect();
    let eyes: Vec<Entity> = eye_params.iter().map(
        |pos| spawn_eye(commands, eye_spawn_event_writer, *pos)
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
    )).id();

    cell_spawn_event_writer.send(CellSpawnEvent(cell));
    commands.entity(cell).push_children(&flagella);
    commands.entity(cell).push_children(&eyes);
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
        Flagellum{
            activation: 0.,
            angle: angle,
        },
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

    let eye = commands.spawn((
        Eye{
            activation: 0.,
        },
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
    )).id();

    eye_spawn_event_writer.send(EyeSpawnEvent(eye));
    eye
}

pub fn spawn_food(
    commands: &mut Commands,
    food_spawn_event_writer: &mut EventWriter<FoodSpawnEvent>,
    position: Vec3,
) -> Entity {
    let food = commands.spawn((
        Food {},
        SpatialBundle::from_transform(Transform::from_translation(position)),
        Collider::ball(10.),
    )).id();

    food_spawn_event_writer.send(FoodSpawnEvent(food));
    food
}

pub fn cell_food_intersection(
    mut cell_query: Query<(&mut Cell, &Collider, &GlobalTransform)>,
    food_query: Query<&Food>,
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
) {
    for (mut cell, collider, transform) in cell_query.iter_mut() {
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
                    commands.entity(x).despawn_recursive();
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
        timer.tick(Duration::from_secs_f32(FIXED_DELTA));
        if timer.finished() {
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
    mut commands: Commands,
    mut cell_spawn_event_writer: EventWriter<CellSpawnEvent>,
    mut flagellum_spawn_event_writer: EventWriter<FlagellumSpawnEvent>,
    mut eye_spawn_event_writer: EventWriter<EyeSpawnEvent>,
    cell_query: Query<(Entity, &Cell, &Transform)>,
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
            &mut cell_spawn_event_writer, &mut flagellum_spawn_event_writer, &mut eye_spawn_event_writer,
            position, 
            rotation * Quat::from_rotation_z(0.1), 
            100., 
            cell.flagella_params.iter().map(|(pos, ang)| (pos + normal.sample(&mut rng), (ang + normal.sample(&mut rng)).clamp(-PI/2., PI/2.))).collect(),
            cell.eye_params.iter().map(|pos| pos + normal.sample(&mut rng)).collect(),
            weights.map(|x| x + weight_normal.sample(&mut rng)),
            biases.map(|x| x + weight_normal.sample(&mut rng)),
            state.clone(),
            );
        spawn_cell(&mut commands, 
            &mut cell_spawn_event_writer, &mut flagellum_spawn_event_writer, &mut eye_spawn_event_writer,
            position, 
            rotation * Quat::from_rotation_z(-0.1), 
            100., 
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
    mut food_spawn_event_writer: EventWriter<FoodSpawnEvent>,
    mut timer: ResMut<FoodTimer>,
) {
    timer.tick(Duration::from_secs_f32(FIXED_DELTA));
    for _ in 0..timer.times_finished_this_tick() {
        let normal = Normal::new(0., 15000.).unwrap();
        let mut rng = rand::thread_rng();
        
        spawn_food(
            &mut commands, 
            &mut food_spawn_event_writer,
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
    timer.tick(time.delta());
    if timer.finished() {
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