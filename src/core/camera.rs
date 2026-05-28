//! Camera setup and follow system.

use crate::core::components::Player;
use bevy::prelude::*;

/// Spawns a 2D camera.
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Follows the player entity each frame, converting grid Position to world coords.
pub fn camera_follow(
    player_query: Query<&Transform, (With<Player>, Changed<Transform>)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let Ok(player_tf) = player_query.single() else {
        return;
    };
    let Ok(mut cam_tf) = camera_query.single_mut() else {
        return;
    };

    cam_tf.translation.x = player_tf.translation.x;
    cam_tf.translation.y = player_tf.translation.y;
}
