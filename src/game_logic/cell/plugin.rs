use std::cell;
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
use bevy_prototype_lyon::prelude::*;

use crate::game_logic::math::*;
use crate::game_logic::sprites::*;

use super::*;

//pub const SPLIT_ENERGY: f32 = 200.;
pub const MIN_ENERGY: f32 = 5.;

pub const FIXED_DELTA: f32 = 1./60.;

pub const MUTATION_RATE: f32 = 0.01;
pub const WEIGHT_MUTATION_RATE: f32 = 0.1;

pub const ENERGY_PENALTY: f32 = 0.01;
pub const CHLOROPLAST_PRODUCTION: f32 = 1.;

pub const INTERCELL_PUSH: f32 = 1.;

pub const MAX_CELL_COUNT: usize = 2000;

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
            .init_resource::<CellCount>()
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
                //food_spawning,
                //cell_food_intersection.before(update_radius),
                eye_sensing,
                cell_thinking,
                update_flagellum.after(cell_thinking),
                update_energy.before(update_radius),
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

#[derive(Resource, Deref, DerefMut, Default)]
pub struct CellCount(pub usize);

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
    //mut food_spawn_event_writer: EventWriter<FoodSpawnEvent>,
    //food_sprite: Option<Res<FoodSprite>>,
    light_sprite: Option<Res<LightSprite>>,
    cell_sprite: Option<Res<CellSprite>>,
    flagellum_sprite: Option<Res<FlagellumSprite>>,
    eye_sprite: Option<Res<EyeSprite>>,
    mut cell_count: ResMut<CellCount>,
) {
    //let normal = Normal::new(0., 10000.).unwrap();
    //let mut rng = rand::thread_rng();
    
    spawn_cell(
        &mut commands,
        &mut cell_spawn_event_writer, &mut flagellum_spawn_event_writer, &mut eye_spawn_event_writer,
        Vec3::new(0., 0., 0.),
        Quat::from_rotation_z(0.),
        5., 10.,
        1,
        vec![],
        vec![],
        Array2::random((0,0), Normal::new(0., 0.5).unwrap()),
        Array1::random(0, Normal::new(0., 0.5).unwrap()),
        Array1::random(0, Normal::new(0., 0.5).unwrap()),
        cell_sprite.as_deref(),
        light_sprite.as_deref(),
        flagellum_sprite.as_deref(),
        eye_sprite.as_deref(),
        cell_count.as_mut(),
    );

    /* 
    for _ in 0..20 {
        spawn_cell(
            &mut commands,
            &mut cell_spawn_event_writer, &mut flagellum_spawn_event_writer, &mut eye_spawn_event_writer,
            Vec3::new(normal.sample(&mut rng), normal.sample(&mut rng),0.),
            Quat::from_rotation_z(0.),
            100., 200.,
            0,
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
    */
    /*
    for _ in 0..10000 {
        spawn_food(
            &mut commands, 
            &mut food_spawn_event_writer,
            Vec3::new(normal.sample(&mut rng), normal.sample(&mut rng), 0.),
            food_sprite.as_deref(),
            light_sprite.as_deref(),
        );
    }
    */
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

pub fn eye_sensing(
    mut eye_query: Query<(&Parent, &mut Activation, &GlobalTransform, &Collider, &ViewParams), With<Eye>>,
    collider_query: Query<&Parent, With<CellColliderTag>>,
    cell_query: Query<(&Transform, &Radius), With<Cell>>,
    rapier_context: Res<RapierContext>,
) {
    eye_query
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::new().min_batch_size(32))
        .for_each(|(parent, mut eye_activation, eye_transform, collider, view_params)| {
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
                    if let Ok(cell) = collider_query.get(x) {
                        if parent.get() == cell.get() {
                            return true;
                        }
                        if let Ok((cell_transform, radius)) = cell_query.get(cell.get()) {
                            let center = cell_transform.translation.truncate() - eye_transform.translation().truncate();
                            if let Some(point) = nearest_intersection(center, **radius, m, n) {
                                activation = activation.max((1.-point.length()/view_params.range).max(0.).min(1.));
                            } 
                        }
                    }
                    true
                }
            );

            **eye_activation = activation;
    });
}

pub fn cell_thinking(
    mut cell_query: Query<(&mut NeuronState, &NeuronWeights, &NeuronBiases, &mut ThinkingTimer, &CellEyes, &CellFlagella)>,
    eye_query: Query<&Activation, With<Eye>>,
) {
    cell_query.par_iter_mut()
        .batching_strategy(BatchingStrategy::new().min_batch_size(100))
        .for_each(|(mut state, weights, biases, mut timer, eyes, flagella)| {
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
                //state.slice_mut(activation_range).map_inplace(sigmoid_inplace);
                state.slice_mut(activation_range).map_inplace(tanh_inplace);
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

pub fn update_energy(
    mut despawn_queue: ResMut<DelayedDespawnQueue>,
    mut cell_query: Query<(Entity, &mut Energy, &SplitEnergy, &Chloroplasts), With<Cell>>,
    mut cell_despawn_event_writer: EventWriter<CellDespawnEvent>,
    mut cell_count: ResMut<CellCount>
) {
    for (cell_entity, mut energy, split_energy, chloroplasts) in cell_query.iter_mut() {
        **energy += (chloroplasts.0 as f32 * CHLOROPLAST_PRODUCTION - energy.0 * ENERGY_PENALTY) * FIXED_DELTA;
        if energy.0 < split_energy.0 / 4. {
            despawn_cell(&mut despawn_queue, &mut cell_despawn_event_writer, cell_entity, cell_count.as_mut());
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
    mut cell_query: Query<(Entity, &mut Dead, &Energy, &SplitEnergy, &Chloroplasts, &NeuronWeights, &NeuronBiases, &NeuronState, &FlagellaParams, &EyeParams, &Transform), With<Cell>>,
    cell_sprite: Option<Res<CellSprite>>,
    light_sprite: Option<Res<LightSprite>>,
    flagellum_sprite: Option<Res<FlagellumSprite>>,
    eye_sprite: Option<Res<EyeSprite>>,
    mut cell_count: ResMut<CellCount>,
) {
    for (
        cell_entity, mut dead, 
        energy, split_energy, chloroplasts, 
        weights, biases, state, 
        flagella_params, eye_params, 
        cell_transform
    ) in cell_query.iter_mut().filter(
        |(_, dead, energy, split_energy, _, _, _, _, _, _, _)| energy.0 >= split_energy.0 && !dead.0
    ) {
        **dead = true;

        let position = cell_transform.translation;
        let rotation = cell_transform.rotation;
        let (weights, biases, state) = (&**weights, &**biases, &**state);
        
        let normal = Normal::new(0., MUTATION_RATE).unwrap();
        let weight_normal = Normal::new(0., WEIGHT_MUTATION_RATE).unwrap();
        let mut rng = rand::thread_rng();

        despawn_cell(&mut despawn_queue, &mut cell_despawn_event_writer, cell_entity, cell_count.as_mut());
        spawn_cell(&mut commands, 
            &mut cell_spawn_event_writer, &mut flagellum_spawn_event_writer, &mut eye_spawn_event_writer,
            position, 
            rotation * Quat::from_rotation_z(0.1), 
            **energy/2.,
            (**split_energy + 10. * normal.sample(&mut rng)).max(MIN_ENERGY*2.),
            **chloroplasts,
            flagella_params.iter().map(|(pos, ang)| (pos + normal.sample(&mut rng), (ang + normal.sample(&mut rng)).clamp(-PI/2., PI/2.))).collect(),
            eye_params.iter().map(|pos| pos + normal.sample(&mut rng)).collect(),
            weights.map(|x| x + weight_normal.sample(&mut rng)),
            biases.map(|x| x + weight_normal.sample(&mut rng)),
            state.clone(),
            cell_sprite.as_deref(),
            light_sprite.as_deref(),
            flagellum_sprite.as_deref(),
            eye_sprite.as_deref(),
            cell_count.as_mut(),
            );
        if cell_count.0 >= MAX_CELL_COUNT {
            continue;
        }
        spawn_cell(&mut commands, 
            &mut cell_spawn_event_writer, &mut flagellum_spawn_event_writer, &mut eye_spawn_event_writer,
            position, 
            rotation * Quat::from_rotation_z(-0.1), 
            **energy/2., 
            (**split_energy + 10. * normal.sample(&mut rng)).max(MIN_ENERGY*2.),
            **chloroplasts,
            flagella_params.iter().map(|(pos, ang)| (pos + normal.sample(&mut rng), (ang + normal.sample(&mut rng)).clamp(-PI/2., PI/2.))).collect(),
            eye_params.iter().map(|pos| pos + normal.sample(&mut rng)).collect(),
            weights.map(|x| x + weight_normal.sample(&mut rng)),
            biases.map(|x| x + weight_normal.sample(&mut rng)),
            state.clone(),
            cell_sprite.as_deref(),
            light_sprite.as_deref(),
            flagellum_sprite.as_deref(),
            eye_sprite.as_deref(),
            cell_count.as_mut(),
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

/*
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
}*/

pub fn count_cells(cell_query: Query<&Cell>, food_query: Query<&Food>, mut timer: ResMut<DebugTimer>, time: Res<Time>, cell_count: Res<CellCount>) {
    timer.tick(time.delta());
    if timer.finished() {
        print!("FPS: {}, cell_count: {}, food count: {}, resource_count: {}\n", (1./time.delta_seconds()).round(), cell_query.iter().count(), food_query.iter().count(), cell_count.0);
    }
}

pub fn dynamic_thing(time: Res<Time>, mut cnter: ResMut<TimeCounter>) {
    cnter.0 += time.delta_seconds();
}

pub fn fixed_thing(mut cnter: ResMut<TimeCounter>) {
    cnter.1 += 1.;
    if cnter.0 >= 1. {
        print!("FixedFPS: {:?} \n", cnter.1);
        cnter.0 -= 1.;
        cnter.1 = 0.;
    }
}