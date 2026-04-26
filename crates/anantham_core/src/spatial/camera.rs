use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

use leafwing_input_manager::prelude::ActionState;

use crate::prelude::CoreAction;

#[derive(Component)]
pub struct PlayerCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        PlayerCamera {
            yaw: 0.0,
            pitch: 0.0,
            fov: std::f32::consts::FRAC_PI_4,
        },
        Transform::from_xyz(0.0, 10.0, 0.0),
    ));
    info!("Camera spawned.");
}

pub fn camera_movement_system(
    time: Res<Time>,
    action_state: Res<ActionState<CoreAction>>,
    mut query: Query<(&mut Transform, &mut PlayerCamera)>,
) {
    let Ok((mut transform, mut camera)) = query.single_mut() else {
        return;
    };

    let look_axis = action_state.axis_pair(&CoreAction::LookAround);
    let delta = look_axis.xy();
    let sensitivity = 0.002;

    camera.yaw -= delta.x * sensitivity;
    camera.pitch -= delta.y * sensitivity;
    camera.pitch = camera.pitch.clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);

    transform.rotation = Quat::from_euler(EulerRot::YXZ, camera.yaw, camera.pitch, 0.0);

    let move_axis = action_state.axis_pair(&CoreAction::MoveCamera);
    let delta = move_axis.xy();
    let speed = 25.0 * time.delta_secs();

    let forward = transform.forward().as_vec3();
    let right = transform.right().as_vec3();

    transform.translation += forward * delta.y * speed;
    transform.translation += right * delta.x * speed;

    if action_state.pressed(&CoreAction::FlyUp) {
        let up = transform.up().as_vec3();
        transform.translation += up * speed;
    }
    if action_state.pressed(&CoreAction::FlyDown) {
        let down = transform.down().as_vec3();
        transform.translation += down * speed;
    }
}
