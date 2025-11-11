// camera.rs (fixed)
use bevy::prelude::*;
use bevy_ggrs::LocalPlayers;
use crate::components::Player;

pub fn camera_follow(
    local_players: Res<LocalPlayers>,
    players: Query<(&Player, &Transform)>,
    mut cameras: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    for (player, player_transform) in &players {
        if !local_players.0.contains(&player.handle) {
            continue;
        }

        let pos = player_transform.translation;

        for mut camera_transform in &mut cameras {
            // Follow player from behind and above
            let offset = Vec3::new(0.0, 15.0, 15.0);
            camera_transform.translation = pos + offset;
            camera_transform.look_at(pos, Vec3::Y);
        }
    }
}