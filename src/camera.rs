// camera.rs (fixed)
use bevy::prelude::*;
use bevy_ggrs::LocalPlayers;
use crate::components::Player;
use crate::states::GameState;

pub fn camera_follow(
    local_players: Res<LocalPlayers>,
    players: Query<(&Player, &Transform)>,
    mut cameras: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    game_state: Res<State<GameState>>,
) {
    for (player, player_transform) in &players {
        if !local_players.0.contains(&player.handle) {
            continue;
        }

        let pos = player_transform.translation;

        for mut camera_transform in &mut cameras {
            // Different camera offset based on game state
            let offset = match game_state.get() {
                GameState::Matchmaking => {
                    // Zoomed in view for inventory inspection
                    Vec3::new(0.0, 6.0, 6.0)
                }
                GameState::InGame => {
                    // Normal gameplay view
                    Vec3::new(0.0, 15.0, 15.0)
                }
                _ => {
                    // Default view for other states
                    Vec3::new(0.0, 15.0, 15.0)
                }
            };

            camera_transform.translation = pos + offset;
            camera_transform.look_at(pos, Vec3::Y);
        }
    }
}