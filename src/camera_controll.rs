use bevy::{prelude::*, input::mouse::{MouseMotion, MouseWheel, MouseScrollUnit}};


pub struct CamControllPlugin;
impl Plugin for CamControllPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, camera_setup)
            .add_systems(Update, camera_grab_system);
    }
}

pub fn camera_setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn camera_grab_system(
    mut cameras: Query<(&mut Transform, &mut OrthographicProjection)>,
    btn: Res<Input<MouseButton>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut scroll_evr: EventReader<MouseWheel>,
) {
    let (mut cam_transform, mut projection) = cameras.single_mut();

    if btn.pressed(MouseButton::Left) && !motion_evr.is_empty() {
        let mouse_translation: Vec2 = motion_evr.iter().map(|ev| ev.delta).sum();
        cam_transform.translation += Vec3::new(-mouse_translation.x, mouse_translation.y, 0.) * projection.scale / 16.;
    }

    if !scroll_evr.is_empty() {
        let scroll_distance: f32 = scroll_evr.iter()
            .map(|ev| match ev.unit {
                MouseScrollUnit::Line => -ev.y,
                MouseScrollUnit::Pixel => -ev.y / 16.,
            })
            .sum();
        projection.scale *= f32::powf(1.1, scroll_distance);
    }
}