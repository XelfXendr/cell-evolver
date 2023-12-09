use bevy::prelude::*;
use bevy_rapier2d::prelude::Collider;
use ndarray::{Array2, Array1};

use crate::game_logic::physics::PhysicsBundle;

#[derive(Bundle)]
pub struct CellBundle {
    pub cell: Cell,
    pub flagella: CellFlagella,
    pub eyes: CellEyes,
    pub collider: CellCollider,
    pub sprites: CellSprites,
    pub energy: Energy,
    pub split_energy: SplitEnergy,
    pub radius: Radius,
    pub dead: Dead,
    pub chloroplasts: Chloroplasts,
    pub weights: NeuronWeights,
    pub biases: NeuronBiases,
    pub state: NeuronState,
    pub flagella_params: FlagellaParams,
    pub eye_params: EyeParams,
    #[bundle()]
    pub physics_bundle: PhysicsBundle,
    #[bundle()]
    pub spatial_bundle: SpatialBundle,
    pub thinking_timer: ThinkingTimer,
}
impl CellBundle {
    pub fn new(
        flagella: Vec<Entity>,
        eyes: Vec<Entity>,
        collider: Entity,
        sprites: Vec<Entity>,
        flagella_params: Vec<(f32, f32)>,
        eye_params: Vec<f32>,
        energy: f32,
        split_energy: f32,
        chloroplasts: u8,        
        weights: Array2<f32>,
        biases: Array1<f32>,
        state: Array1<f32>,
        position: Vec3,
        rotation: Quat,
    ) -> Self {
        Self {
            cell: Cell{},
            flagella: CellFlagella(flagella),
            eyes: CellEyes(eyes),
            collider: CellCollider(collider),
            sprites: CellSprites(sprites),
            energy: Energy(energy),
            split_energy: SplitEnergy(split_energy),
            radius: Radius(5. * energy.sqrt()),
            dead: Dead(false),
            chloroplasts: Chloroplasts(chloroplasts),
            weights: NeuronWeights(weights),
            biases: NeuronBiases(biases),
            state: NeuronState(state),
            flagella_params: FlagellaParams(flagella_params),
            eye_params: EyeParams(eye_params),
            physics_bundle: PhysicsBundle::new(),
            spatial_bundle: SpatialBundle::from_transform(
                Transform::from_translation(position)
                    .with_rotation(rotation)
            ),
            thinking_timer: ThinkingTimer(Timer::from_seconds(1./20., TimerMode::Repeating)),
        }
    }
}

#[derive(Bundle, Default)]
pub struct FlagellumBundle {
    flagellum: Flagellum,
    activation: Activation,
    angle: Angle,
}
impl FlagellumBundle {
    pub fn new(activation: f32, angle: f32) -> Self {
        Self {
            activation: Activation(activation),
            angle: Angle(angle),
            ..default()
        }
    }
}

#[derive(Bundle, Default)]
pub struct EyeBundle {
    eye: Eye,
    view_params: ViewParams,
    activation: Activation,
    #[bundle()]
    spatial_bundle: SpatialBundle,
    collider: Collider,
}
impl EyeBundle {
    pub fn new(position: f32, radius: f32, m_normal: Vec2, n_normal: Vec2, range: f32, path: &[Vec2]) -> Self {
        let vert = -position.cos() * radius;
        let horiz = position.sin() * radius;

        Self {
            eye: Eye{},
            view_params: ViewParams { 
                m_normal: m_normal,
                n_normal: n_normal, 
                range: range, 
            },
            activation: Activation(0.),
            spatial_bundle: SpatialBundle::from_transform(
                Transform::from_rotation(Quat::from_rotation_z(position))
                    .with_translation(Vec3::new(horiz, vert, 2.))
            ),
            collider: Collider::convex_polyline(Vec::from(path)).unwrap(),
        }
    }
}

#[derive(Bundle, Default)]
pub struct FoodBundle {
    food: Food,
    dead: Dead,
}
impl FoodBundle {
    pub fn new() -> Self {
        FoodBundle { food: Food{}, dead: Dead(false) }
    }
}

#[derive(Component, Default, Clone, Copy)]
pub struct Cell;

#[derive(Component, Default, Clone, Copy)]
pub struct Flagellum;

#[derive(Component, Default, Clone, Copy)]
pub struct Eye;

#[derive(Component, Default, Clone, Copy)]
pub struct ViewParams {
    pub m_normal: Vec2,
    pub n_normal: Vec2,
    pub range: f32,
}

#[derive(Component, Default, Clone, Copy)]
pub struct Food;

#[derive(Component, Deref, DerefMut, Default)]
pub struct CellFlagella(pub Vec<Entity>);

#[derive(Component, Deref, DerefMut, Default)]
pub struct CellEyes(pub Vec<Entity>);

#[derive(Component, Deref, DerefMut)]
pub struct CellCollider(pub Entity);

#[derive(Component)]
pub struct CellColliderTag;

#[derive(Component, Deref, DerefMut)]
pub struct CellSprites(pub Vec<Entity>);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct Energy(pub f32);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct SplitEnergy(pub f32);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct Dead(pub bool);

#[derive(Component, Deref, DerefMut, Default)]
pub struct NeuronWeights(pub Array2<f32>);

#[derive(Component, Deref, DerefMut, Default)]
pub struct NeuronBiases(pub Array1<f32>);

#[derive(Component, Deref, DerefMut, Default)]
pub struct NeuronState(pub Array1<f32>);

#[derive(Component, Deref, DerefMut, Default)]
pub struct Chloroplasts(pub u8);

#[derive(Component, Deref, DerefMut, Default)]
pub struct FlagellaParams(pub Vec<(f32,f32)>);

#[derive(Component, Deref, DerefMut, Default)]
pub struct EyeParams(pub Vec<f32>);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct Activation(pub f32);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct Angle(pub f32);

#[derive(Component, Deref, DerefMut, Default)]
pub struct ThinkingTimer(pub Timer);

#[derive(Component, Deref, DerefMut, Default)]
pub struct Radius(pub f32);