use crate::bodies::{CelestialBody, SolarSystemDirectory};
use crate::coordinates::FloatingOrigin;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

#[derive(Component)]
pub struct GimbalCamera {
    pub distance: f32,
    pub target_distance: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl Default for GimbalCamera {
    fn default() -> Self {
        Self {
            distance: 120.0,
            target_distance: 120.0,
            pitch: 0.35,
            yaw: -0.6,
        }
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, (handle_target_switching, camera_controls));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            near: 0.5,
            far: 3_000_000.0,
            fov: std::f32::consts::PI / 4.0,
            ..default()
        }),
        AmbientLight {
            color: Color::srgb(0.6, 0.7, 0.9),
            brightness: 120.0,
            affects_lightmapped_meshes: false,
        },
        GimbalCamera::default(),
        Transform::default(),
    ));
}

fn handle_target_switching(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut directory: ResMut<SolarSystemDirectory>,
    mut origin: ResMut<FloatingOrigin>,
    mut camera_query: Query<&mut GimbalCamera>,
    body_query: Query<&CelestialBody>,
) {
    let mut switch_to: Option<usize> = None;

    if keyboard.just_pressed(KeyCode::Digit0) || keyboard.just_pressed(KeyCode::Numpad0) {
        switch_to = Some(0); // Sun
    } else if keyboard.just_pressed(KeyCode::Digit1) || keyboard.just_pressed(KeyCode::Numpad1) {
        switch_to = Some(1); // Mercury
    } else if keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::Numpad2) {
        switch_to = Some(2); // Venus
    } else if keyboard.just_pressed(KeyCode::Digit3) || keyboard.just_pressed(KeyCode::Numpad3) {
        switch_to = Some(3); // Earth
    } else if keyboard.just_pressed(KeyCode::Digit4) || keyboard.just_pressed(KeyCode::Numpad4) {
        switch_to = Some(4); // Moon
    } else if keyboard.just_pressed(KeyCode::Digit5) || keyboard.just_pressed(KeyCode::Numpad5) {
        switch_to = Some(5); // Mars
    } else if keyboard.just_pressed(KeyCode::Digit6) || keyboard.just_pressed(KeyCode::Numpad6) {
        switch_to = Some(6); // Jupiter
    } else if keyboard.just_pressed(KeyCode::Digit7) || keyboard.just_pressed(KeyCode::Numpad7) {
        switch_to = Some(7); // Saturn
    } else if keyboard.just_pressed(KeyCode::Digit8) || keyboard.just_pressed(KeyCode::Numpad8) {
        switch_to = Some(8); // Uranus
    } else if keyboard.just_pressed(KeyCode::Digit9) || keyboard.just_pressed(KeyCode::Numpad9) {
        switch_to = Some(9); // Neptune
    } else if keyboard.just_pressed(KeyCode::KeyP) {
        switch_to = Some(10); // Pluto
    } else if keyboard.just_pressed(KeyCode::Tab) {
        if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            directory.cycle_prev();
            switch_to = Some(directory.current_target_index);
        } else {
            directory.cycle_next();
            switch_to = Some(directory.current_target_index);
        }
    }

    // If initial startup, align origin focus with current directory selection
    if origin.focus_entity.is_none() && !directory.targets.is_empty() {
        switch_to = Some(directory.current_target_index);
    }

    if let Some(idx) = switch_to {
        if let Some(target_entity) = directory.set_target_by_index(idx) {
            origin.focus_entity = Some(target_entity);

            // Compute an optimal framing distance based on the celestial body's display size
            if let Ok(mut gimbal) = camera_query.single_mut() {
                if let Ok(body) = body_query.get(target_entity) {
                    let ideal_dist = (body.display_radius * 4.5).clamp(40.0, 1_500.0);
                    gimbal.target_distance = ideal_dist;
                }
            }
        }
    }
}

fn camera_controls(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut query: Query<(&mut Transform, &mut GimbalCamera)>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut gimbal) in query.iter_mut() {
        // 1. Mouse Drag Rotation
        if mouse_button.pressed(MouseButton::Left) || mouse_button.pressed(MouseButton::Right) {
            gimbal.yaw -= mouse_motion.delta.x * 0.005;
            gimbal.pitch += mouse_motion.delta.y * 0.005;
        }

        // 2. Keyboard Rotation (Arrows & WASD)
        let key_turn_speed = 1.5 * dt;
        if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
            gimbal.yaw -= key_turn_speed;
        }
        if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
            gimbal.yaw += key_turn_speed;
        }
        if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
            gimbal.pitch -= key_turn_speed;
        }
        if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
            gimbal.pitch += key_turn_speed;
        }

        // 3. Mouse Wheel Zoom
        if mouse_scroll.delta.y.abs() > 0.0001 {
            let zoom_factor = 1.0 - (mouse_scroll.delta.y * 0.15);
            gimbal.target_distance = (gimbal.target_distance * zoom_factor).clamp(15.0, 1_200_000.0);
        }

        // 4. Keyboard Zoom (Q / E or PageUp / PageDown)
        if keyboard.pressed(KeyCode::KeyQ) || keyboard.pressed(KeyCode::PageUp) {
            gimbal.target_distance = (gimbal.target_distance * 0.95).clamp(15.0, 1_200_000.0);
        }
        if keyboard.pressed(KeyCode::KeyE) || keyboard.pressed(KeyCode::PageDown) {
            gimbal.target_distance = (gimbal.target_distance * 1.05).clamp(15.0, 1_200_000.0);
        }

        // Smooth camera distance interpolation
        let lerp_factor = (10.0 * dt).min(1.0);
        gimbal.distance += (gimbal.target_distance - gimbal.distance) * lerp_factor;

        // Clamp pitch to prevent flipping upside down
        gimbal.pitch = gimbal.pitch.clamp(-1.54, 1.54);

        // Convert spherical coords to cartesian (centered at FloatingOrigin 0,0,0)
        let y = gimbal.pitch.sin() * gimbal.distance;
        let xz_len = gimbal.pitch.cos() * gimbal.distance;
        let x = gimbal.yaw.sin() * xz_len;
        let z = gimbal.yaw.cos() * xz_len;

        transform.translation = Vec3::new(x, y, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}