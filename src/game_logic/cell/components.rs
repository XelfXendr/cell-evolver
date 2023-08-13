use bevy::prelude::*;
use ndarray::{Array2, Array1};

#[derive(Bundle, Default)]
pub struct CellBundle {
    pub cell: Cell,
    pub flagella: CellFlagella,
    pub eyes: CellEyes,
    pub energy: Energy,
    pub dead: Dead,
    pub weights: NeuronWeights,
    pub biases: NeuronBiases,
    pub state: NeuronState,
    pub flagella_params: FlagellaParams,
    pub eye_params: EyeParams,
}
impl CellBundle {
    pub fn new(
        flagella: Vec<Entity>,
        eyes: Vec<Entity>,
        flagella_params: Vec<(f32, f32)>,
        eye_params: Vec<f32>,
        energy: f32,
        dead: bool,
        weights: Array2<f32>,
        biases: Array1<f32>,
        state: Array1<f32>,
    ) -> Self {
        Self {
            cell: Cell{},
            flagella: CellFlagella(flagella),
            eyes: CellEyes(eyes),
            energy: Energy(energy),
            dead: Dead(dead),
            weights: NeuronWeights(weights),
            biases: NeuronBiases(biases),
            state: NeuronState(state),
            flagella_params: FlagellaParams(flagella_params),
            eye_params: EyeParams(eye_params),
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
    activation: Activation,
}
impl EyeBundle {
    pub fn new(activation: f32) -> Self {
        Self {
            activation: Activation(activation),
            ..default()
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
pub struct Food;

#[derive(Component, Deref, DerefMut, Default)]
pub struct CellFlagella(pub Vec<Entity>);

#[derive(Component, Deref, DerefMut, Default)]
pub struct CellEyes(pub Vec<Entity>);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct Energy(pub f32);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct Dead(pub bool);

#[derive(Component, Deref, DerefMut, Default)]
pub struct NeuronWeights(pub Array2<f32>);

#[derive(Component, Deref, DerefMut, Default)]
pub struct NeuronBiases(pub Array1<f32>);

#[derive(Component, Deref, DerefMut, Default)]
pub struct NeuronState(pub Array1<f32>);

#[derive(Component, Deref, DerefMut, Default)]
pub struct FlagellaParams(pub Vec<(f32,f32)>);

#[derive(Component, Deref, DerefMut, Default)]
pub struct EyeParams(pub Vec<f32>);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct Activation(pub f32);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
pub struct Angle(pub f32);

#[derive(Component, Deref, DerefMut)]
pub struct ThinkingTimer(pub Timer);