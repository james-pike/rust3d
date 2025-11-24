// collisions.rs
use bevy::prelude::*;
use crate::{core::constants::*, entities::components::{Wall, Bullet, Player}, core::states::RollbackState, core::resources::Scores};

pub fn resolve_wall_collisions(
    mut players: Query<&mut Transform, With<Player>>,
    walls: Query<&Transform, (With<Wall>, Without<Player>)>,
) {
    for mut player_transform in &mut players {
        for wall_transform in &walls {
            let wall_scale = wall_transform.scale;
            let wall_size = Vec3::new(wall_scale.x, WALL_HEIGHT, wall_scale.z);
            let wall_pos = wall_transform.translation;
            let player_pos = player_transform.translation;

            // Work in XZ plane
            let wall_pos_xz = Vec2::new(wall_pos.x, wall_pos.z);
            let player_pos_xz = Vec2::new(player_pos.x, player_pos.z);
            let wall_size_xz = Vec2::new(wall_size.x, wall_size.z);

            let wall_to_player = player_pos_xz - wall_pos_xz;
            let wall_to_player_abs = wall_to_player.abs();
            let wall_corner_to_player_center = wall_to_player_abs - wall_size_xz / 2.;

            let corner_to_corner = wall_corner_to_player_center - Vec2::splat(PLAYER_RADIUS);

            if corner_to_corner.x > 0. || corner_to_corner.y > 0. {
                continue;
            }

            if corner_to_corner.x > corner_to_corner.y {
                player_transform.translation.x -= wall_to_player.x.signum() * corner_to_corner.x;
            } else {
                player_transform.translation.z -= wall_to_player.y.signum() * corner_to_corner.y;
            }
        }
    }
}

pub fn bullet_wall_collisions(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    walls: Query<&Transform, (With<Wall>, Without<Bullet>)>,
) {
    let map_limit = MAP_SIZE as f32 / 2.;

    for (bullet_entity, bullet_transform) in &bullets {
        let bullet_pos = bullet_transform.translation;

        if bullet_pos.x.abs() > map_limit || bullet_pos.z.abs() > map_limit {
            commands.entity(bullet_entity).despawn();
            continue;
        }

        for wall_transform in &walls {
            let wall_scale = wall_transform.scale;
            let wall_size = Vec3::new(wall_scale.x, WALL_HEIGHT, wall_scale.z);
            let wall_pos = wall_transform.translation;
            
            let wall_pos_xz = Vec2::new(wall_pos.x, wall_pos.z);
            let bullet_pos_xz = Vec2::new(bullet_pos.x, bullet_pos.z);
            let wall_size_xz = Vec2::new(wall_size.x, wall_size.z);
            
            let center_to_center = wall_pos_xz - bullet_pos_xz;
            let center_to_center = center_to_center.abs();
            let corner_to_center = center_to_center - wall_size_xz / 2.;
            
            if corner_to_center.x < 0. && corner_to_center.y < 0. {
                commands.entity(bullet_entity).despawn();
                break;
            }
        }
    }
}

pub fn kill_players(
    mut commands: Commands,
    players: Query<(Entity, &Transform, &Player), Without<Bullet>>,
    bullets: Query<&Transform, With<Bullet>>,
    mut next_state: ResMut<NextState<RollbackState>>,
    mut scores: ResMut<Scores>,
) {
    for (player_entity, player_transform, player) in &players {
        for bullet_transform in &bullets {
            let player_pos = player_transform.translation;
            let bullet_pos = bullet_transform.translation;

            let distance = player_pos.distance(bullet_pos);

            if distance < PLAYER_RADIUS + BULLET_RADIUS {
                commands.entity(player_entity).despawn();
                next_state.set(RollbackState::RoundEnd);

                if player.handle == 0 {
                    scores.1 += 1;
                } else {
                    scores.0 += 1;
                }
                info!("player died: {scores:?}")
            }
        }
    }
}