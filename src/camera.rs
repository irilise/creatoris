use bevy::prelude::*;

#[derive(Component)]
pub struct GimbalCamera {
    pub distance: f32,
    pub pitch: f32,
    pub yaw: f32,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, camera_controls);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        GimbalCamera {
            distance: 25_000.0,
            pitch: 0.5,
            yaw: -0.5,
        },
        Transform::default(),
    ));
}

fn camera_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut GimbalCamera)>,
) {
    for (mut transform, mut gimbal) in query.iter_mut() {
        if keyboard.pressed(KeyCode::ArrowLeft) { gimbal.yaw -= 0.05; }
        if keyboard.pressed(KeyCode::ArrowRight) { gimbal.yaw += 0.05; }
        if keyboard.pressed(KeyCode::ArrowUp) { gimbal.pitch -= 0.05; }
        if keyboard.pressed(KeyCode::ArrowDown) { gimbal.pitch += 0.05; }

        // Clamp pitch to prevent gimbal lock
        gimbal.pitch = gimbal.pitch.clamp(-1.5, 1.5);

        // Convert spherical coords to cartesian (centered at FloatingOrigin 0,0,0)
        let y = gimbal.pitch.sin() * gimbal.distance;
        let xz_len = gimbal.pitch.cos() * gimbal.distance;
        let x = gimbal.yaw.sin() * xz_len;
        let z = gimbal.yaw.cos() * xz_len;

        transform.translation = Vec3::new(x, y, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}