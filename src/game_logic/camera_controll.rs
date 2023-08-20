use bevy::{prelude::*, input::mouse::{MouseWheel, MouseScrollUnit}, window::PrimaryWindow};
pub struct CamControllPlugin;
impl Plugin for CamControllPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ClearColor(Color::hex("0f0f0f").unwrap()))
            .add_systems(Startup, camera_setup)
            .add_systems(Update, camera_grab_system);
    }
}

#[derive(Resource, Deref, DerefMut)]
struct CursorPosition(Vec2);

pub fn camera_setup(mut commands: Commands) {
    commands.insert_resource(CursorPosition(Vec2::default()));
    commands.spawn(Camera2dBundle::default());
}

fn camera_grab_system(
    mut cameras: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
    btn: Res<Input<MouseButton>>,
    mut scroll_evr: EventReader<MouseWheel>,
    mut cursor_evr: EventReader<CursorMoved>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cursor_pos: ResMut<CursorPosition>
) {
    if btn.just_pressed(MouseButton::Left) {
        if let Some(pos) = windows.single().cursor_position() {
            **cursor_pos = pos;
        }
    }

    if btn.pressed(MouseButton::Left) && !cursor_evr.is_empty() {
        for ev in cursor_evr.iter() {
            let delta = ev.position - **cursor_pos;
            **cursor_pos = ev.position;
            for (mut cam_transform, projection) in cameras.iter_mut() {
                cam_transform.translation = cam_transform.translation + Vec3::new(-delta.x, delta.y, 0.) * projection.scale;
            }
        }
    }

    if !scroll_evr.is_empty() {
        let scroll_distance: f32 = scroll_evr.iter()
            .map(|ev| match ev.unit {
                MouseScrollUnit::Line => -ev.y,
                MouseScrollUnit::Pixel => -ev.y / 16.,
            })
            .sum();
        for (_, mut projection) in cameras.iter_mut() {
            projection.scale *= f32::powf(1.1, scroll_distance);            
        }
    }
}