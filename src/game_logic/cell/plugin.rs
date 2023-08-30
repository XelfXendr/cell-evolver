use std::f32::consts::PI;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use bevy::ecs::query::BatchingStrategy;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use ndarray::s;
use rand_distr::{Normal, Distribution};
use rand;
use ndarray::{Array1, Array2};
use ndarray_rand::RandomExt;
use bevy_prototype_lyon::prelude::*;

use crate::game_logic::math::*;
use crate::game_logic::sprites::*;

use super::*;

pub const SPLIT_ENERGY: f32 = 200.;
pub const MIN_ENERGY: f32 = 50.;

pub const FIXED_DELTA: f32 = 1./60.;

pub const MUTATION_RATE: f32 = 0.01;
pub const WEIGHT_MUTATION_RATE: f32 = 0.1;

pub struct CellCorePlugin;
impl Plugin for CellCorePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<CellSpawnEvent>()
            .add_event::<CellDespawnEvent>()
            .add_event::<FlagellumSpawnEvent>()
            .add_event::<EyeSpawnEvent>()
            .add_event::<FoodSpawnEvent>()
            .add_event::<FoodDespawnEvent>()
            .add_systems(Startup, resource_init)
            .add_systems(Update, (
                count_cells,
                dynamic_thing,
                delayed_despawn,
                update_radius,
            ))
            .add_systems(FixedUpdate, 
                fixed_thing
            );  
    }
}

pub struct CellServerPlugin;
impl Plugin for CellServerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (
                cell_setup,
            ))
            .add_systems(FixedUpdate, (
                food_spawning,
                cell_food_intersection.before(update_radius),
                eye_sensing,
                kill_intersections.before(eye_sensing),
                cell_thinking,
                update_flagellum.after(cell_thinking),
                decrement_energy.before(update_radius),
                split_cells,
            ));  
    }
}

pub struct CellClientPlugin;
impl Plugin for CellClientPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ShapePlugin);  
    }
}

pub fn resource_init(mut commands: Commands) {
    commands.insert_resource(FoodTimer(Timer::new(Duration::from_secs_f32(0.05), TimerMode::Repeating)));
    commands.insert_resource(DebugTimer(Timer::new(Duration::from_secs_f32(1.), TimerMode::Repeating)));
    commands.insert_resource(TimeCounter(0., 0.));
    commands.insert_resource(DelayedDespawnQueue::new());
}

pub fn delayed_despawn(
    mut commands: Commands, 
    mut despawn_queue: ResMut<DelayedDespawnQueue>
) {
    despawn_queue.despawn(&mut commands)
}

pub fn cell_setup(
    mut commands: Commands, 
    mut cell_spawn_event_writer: EventWriter<CellSpawnEvent>,
    mut flagellum_spawn_event_writer: EventWriter<FlagellumSpawnEvent>,
    mut eye_spawn_event_writer: EventWriter<EyeSpawnEvent>,
    mut food_spawn_event_writer: EventWriter<FoodSpawnEvent>,
    food_sprite: Option<Res<FoodSprite>>,
    light_sprite: Option<Res<LightSprite>>,
    cell_sprite: Option<Res<CellSprite>>,
    flagellum_sprite: Option<Res<FlagellumSprite>>,
    eye_sprite: Option<Res<EyeSprite>>,
) {
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
            cell_sprite.as_deref(),
            light_sprite.as_deref(),
            flagellum_sprite.as_deref(),
            eye_sprite.as_deref(),
        );
    }
    
    for _ in 0..10000 {
        spawn_food(
            &mut commands, 
            &mut food_spawn_event_writer,
            Vec3::new(normal.sample(&mut rng), normal.sample(&mut rng), 0.),
            food_sprite.as_deref(),
            light_sprite.as_deref(),
        );
    }
}

pub fn cell_food_intersection(
    mut despawn_queue: ResMut<DelayedDespawnQueue>,
    mut cell_query: Query<(&mut Energy, &CellCollider), With<Cell>>,
    collider_query: Query<(&Collider, &GlobalTransform)>,
    mut food_query: Query<&mut Dead, With<Food>>,
    rapier_context: Res<RapierContext>,
    mut food_despawn_event_writer: EventWriter<FoodDespawnEvent>,
) {
    for (mut energy, collider_entity) in cell_query.iter_mut() {
        if let Ok((collider, transform)) = collider_query.get(**collider_entity) {
            let direction = quat_to_direction(transform.to_scale_rotation_translation().1);
            let angle = (-direction.x).atan2(direction.y);
            rapier_context.intersections_with_shape(
                transform.translation().truncate(), 
                angle, 
                collider, 
                QueryFilter::default(), 
                |x| {
                    if let Ok(mut eaten) = food_query.get_mut(x) {
                        if !**eaten {
                            **eaten = true;
                            despawn_food(&mut despawn_queue, &mut food_despawn_event_writer, x);
                            **energy += 10.
                        }
                    }
                    true
                }
            )
        }
    }
}

#[derive(Component)]
pub struct Intersection;

pub fn eye_sensing(
    mut eye_query: Query<(&Parent, &mut Activation, &GlobalTransform, &Collider, &ViewParams), With<Eye>>,
    food_query: Query<&Transform, With<Food>>,
    collider_query: Query<&Parent, With<CellColliderTag>>,
    cell_query: Query<(&Transform, &Radius), With<Cell>>,
    rapier_context: Res<RapierContext>,
    sprite: Option<Res<CellSprite>>,
    mut commands: Commands,
) {
    let new_points: Arc<Mutex<Vec<Vec2>>> = Arc::default(); 
    eye_query
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::new().min_batch_size(32))
        .for_each_mut(|(parent, mut eye_activation, eye_transform, collider, view_params)| {
            let mut activation: f32 = 0.;
            let direction = quat_to_direction(eye_transform.to_scale_rotation_translation().1);
            let angle = (-direction.x).atan2(direction.y);

            let m = Vec2::new(
                view_params.m_normal.x*direction.y + view_params.m_normal.y*direction.x, 
                - view_params.m_normal.x*direction.x + view_params.m_normal.y*direction.y
            );
            let n = Vec2::new(
                view_params.n_normal.x*direction.y + view_params.n_normal.y*direction.x, 
                - view_params.n_normal.x*direction.x + view_params.n_normal.y*direction.y
            );

            rapier_context.intersections_with_shape(
                eye_transform.translation().truncate(), 
                angle, 
                collider, 
                QueryFilter::default(), 
                |x| {
                    if let Ok(food_transform) = food_query.get(x) {
                        let center = food_transform.translation.truncate() - eye_transform.translation().truncate();
                        if let Some(point) = nearest_intersection(center, 5., m, n) {
                            new_points.lock().unwrap().push(point + eye_transform.translation().truncate());
                            activation = activation.max((1.-point.length()/view_params.range).max(0.).min(1.));
                        } 
                    }
                    if let Ok(cell) = collider_query.get(x) {
                        if parent.get() == cell.get() {
                            return true;
                        }
                        if let Ok((cell_transform, radius)) = cell_query.get(cell.get()) {
                            let center = cell_transform.translation.truncate() - eye_transform.translation().truncate();
                            if let Some(point) = nearest_intersection(center, **radius, m, n) {
                                new_points.lock().unwrap().push(point + eye_transform.translation().truncate());
                                activation = activation.max((1.-point.length()/view_params.range).max(0.).min(1.));
                            } 
                        }
                    }
                    true
                }
            );

            **eye_activation = activation;
    });
    if let Some(sprite) = sprite {
        for point in new_points.lock().unwrap().iter() {
            commands.spawn((
                Intersection,
                SpriteBundle {
                    sprite: Sprite  {
                        custom_size: Some(Vec2::new(10.,10.)),
                        color: Color::rgb_u8(255, 255, 0),
                        ..default()
                    },
                    transform: Transform::from_translation(point.extend(5.)),
                    texture: sprite.clone(),
                    ..default()
                }
            ));
        }
    }
}

pub fn kill_intersections(
    mut commands: Commands,
    query: Query<Entity, With<Intersection>>,
) {
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn cell_thinking(
    mut cell_query: Query<(&mut NeuronState, &NeuronWeights, &NeuronBiases, &mut ThinkingTimer, &CellEyes, &CellFlagella)>,
    eye_query: Query<&Activation, With<Eye>>,
) {
    cell_query.par_iter_mut()
        .batching_strategy(BatchingStrategy::new().min_batch_size(100))
        .for_each_mut(|(mut state, weights, biases, mut timer, eyes, flagella)| {
            timer.tick(Duration::from_secs_f32(FIXED_DELTA));
            if timer.finished() {
                //update eye neuron state from what eyes see
                let activations: Vec<f32> = eyes.iter().map(|eye| **eye_query.get(*eye).unwrap()).collect();
                for (i, act) in activations.iter().enumerate() {
                    state[i] = *act;
                }
                
                //compute state update
                **state = state.dot(&**weights) + &**biases;
                let activation_range = s![eyes.len()..state.shape()[0]-flagella.len()];
                state.slice_mut(activation_range).map_inplace(tanh_inplace);
                let activation_range = s![state.shape()[0]-flagella.len()..];
                state.slice_mut(activation_range).map_inplace(sigmoid_inplace);
            }
        });
}

pub fn update_flagellum(
    state_query: Query<(&NeuronState, &CellFlagella)>,
    mut flag_query: Query<&mut Activation, With<Flagellum>>,
) {
    for (state, flagella) in state_query.iter() {
        for (f, a) in flagella.iter().zip(state.slice(s![state.shape()[0]-flagella.len()..])) {
            let mut activation = flag_query.get_mut(*f).unwrap();
            **activation = *a;
        }
    }
}

pub fn decrement_energy(
    mut despawn_queue: ResMut<DelayedDespawnQueue>,
    mut cell_query: Query<(Entity, &mut Energy), With<Cell>>,
    mut cell_despawn_event_writer: EventWriter<CellDespawnEvent>,
) {
    for (cell_entity, mut energy) in cell_query.iter_mut() {
        **energy -= FIXED_DELTA;
        if **energy < MIN_ENERGY {
            despawn_cell(&mut despawn_queue, &mut cell_despawn_event_writer, cell_entity);
        }
    }
}

pub fn split_cells(
    mut commands: Commands,
    mut despawn_queue: ResMut<DelayedDespawnQueue>,
    mut cell_spawn_event_writer: EventWriter<CellSpawnEvent>,
    mut cell_despawn_event_writer: EventWriter<CellDespawnEvent>,
    mut flagellum_spawn_event_writer: EventWriter<FlagellumSpawnEvent>,
    mut eye_spawn_event_writer: EventWriter<EyeSpawnEvent>,
    mut cell_query: Query<(Entity, &mut Dead, &Energy, &NeuronWeights, &NeuronBiases, &NeuronState, &FlagellaParams, &EyeParams, &Transform), With<Cell>>,
    cell_sprite: Option<Res<CellSprite>>,
    light_sprite: Option<Res<LightSprite>>,
    flagellum_sprite: Option<Res<FlagellumSprite>>,
    eye_sprite: Option<Res<EyeSprite>>,
) {
    for (cell_entity, mut dead, energy, weights, biases, state, flagella_params, eye_params, cell_transform) in cell_query.iter_mut().filter(|(_, dead, energy, _, _, _, _, _, _)| ***energy >= SPLIT_ENERGY && !***dead) {
        **dead = true;

        let position = cell_transform.translation;
        let rotation = cell_transform.rotation;
        let (weights, biases, state) = (&**weights, &**biases, &**state);
        
        let normal = Normal::new(0., MUTATION_RATE).unwrap();
        let weight_normal = Normal::new(0., WEIGHT_MUTATION_RATE).unwrap();
        let mut rng = rand::thread_rng();

        despawn_cell(&mut despawn_queue, &mut cell_despawn_event_writer, cell_entity);
        spawn_cell(&mut commands, 
            &mut cell_spawn_event_writer, &mut flagellum_spawn_event_writer, &mut eye_spawn_event_writer,
            position, 
            rotation * Quat::from_rotation_z(0.1), 
            **energy/2., 
            flagella_params.iter().map(|(pos, ang)| (pos + normal.sample(&mut rng), (ang + normal.sample(&mut rng)).clamp(-PI/2., PI/2.))).collect(),
            eye_params.iter().map(|pos| pos + normal.sample(&mut rng)).collect(),
            weights.map(|x| x + weight_normal.sample(&mut rng)),
            biases.map(|x| x + weight_normal.sample(&mut rng)),
            state.clone(),
            cell_sprite.as_deref(),
            light_sprite.as_deref(),
            flagellum_sprite.as_deref(),
            eye_sprite.as_deref(),
            );
        spawn_cell(&mut commands, 
            &mut cell_spawn_event_writer, &mut flagellum_spawn_event_writer, &mut eye_spawn_event_writer,
            position, 
            rotation * Quat::from_rotation_z(-0.1), 
            **energy/2., 
            flagella_params.iter().map(|(pos, ang)| (pos + normal.sample(&mut rng), (ang + normal.sample(&mut rng)).clamp(-PI/2., PI/2.))).collect(),
            eye_params.iter().map(|pos| pos + normal.sample(&mut rng)).collect(),
            weights.map(|x| x + weight_normal.sample(&mut rng)),
            biases.map(|x| x + weight_normal.sample(&mut rng)),
            state.clone(),
            cell_sprite.as_deref(),
            light_sprite.as_deref(),
            flagellum_sprite.as_deref(),
            eye_sprite.as_deref(),
        );
    }
}

pub fn update_radius(
    mut cell_query: Query<(&Energy, &mut Radius, &CellFlagella, &CellEyes, &CellCollider, &CellSprites), With<Cell>>,
    mut transform_query: Query<&mut Transform>,
) {
    for (energy, mut radius, cell_flagella, cell_eyes, cell_collider, cell_sprites) in cell_query.iter_mut() {
        let old_radius = radius.0;
        radius.0 = 5. * energy.0.sqrt();
        let ratio = radius.0 / old_radius;
        
        cell_flagella.iter().for_each(|e| {
            if let Ok(mut t) = transform_query.get_mut(*e) {
                t.translation *= ratio;
            }
        });
        cell_eyes.iter().for_each(|e| {
            if let Ok(mut t) = transform_query.get_mut(*e) {
                t.translation *= ratio;
            }
        });
        cell_sprites.iter().for_each(|e| {
            if let Ok(mut t) = transform_query.get_mut(*e) {
                t.scale *= ratio;
            }
        });
        if let Ok(mut t) = transform_query.get_mut(**cell_collider) {
            t.scale *= ratio;
        }
    }
}

pub fn food_spawning(
    mut commands: Commands,
    mut food_spawn_event_writer: EventWriter<FoodSpawnEvent>,
    mut timer: ResMut<FoodTimer>,
    food_sprite: Option<Res<FoodSprite>>,
    light_sprite: Option<Res<LightSprite>>,
) {
    timer.tick(Duration::from_secs_f32(FIXED_DELTA));
    for _ in 0..timer.times_finished_this_tick() {
        let normal = Normal::new(0., 15000.).unwrap();
        let mut rng = rand::thread_rng();
        
        spawn_food(
            &mut commands, 
            &mut food_spawn_event_writer,
            Vec3::new(normal.sample(&mut rng), normal.sample(&mut rng), 0.),
            food_sprite.as_deref(),
            light_sprite.as_deref(),
        );
    }
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