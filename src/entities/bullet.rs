// bullet.rs
use bevy::prelude::*;
use bevy_ggrs::AddRollbackCommandExtension;
use crate::{Config, PlayerInputs, components::{Bullet, BulletReady, MoveDir, Player}, constants::BULLET_RADIUS, input::fire};

pub fn reload_bullet(
    inputs: Res<PlayerInputs<Config>>,
    mut players: Query<(&mut BulletReady, &Player)>,
) {
    for (mut can_fire, player) in players.iter_mut() {
        let (input, _) = inputs[player.handle];
        if !fire(input) {
            can_fire.0 = true;
        }
    }
}

pub fn fire_bullets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    inputs: Res<PlayerInputs<Config>>,
    mut players: Query<(&Transform, &Player, &mut BulletReady, &MoveDir)>,
) {
    for (transform, player, mut bullet_ready, move_dir) in &mut players {
        let (input, _) = inputs[player.handle];
        if fire(input) && bullet_ready.0 {
            let player_pos = transform.translation;
            
            // Muzzle offset in 3D space
            let muzzle_offset = match move_dir.octant() {
                0 => Vec3::new(0.5, 0.0, 0.0),
                1 => Vec3::new(0.5, 0.0, 0.25),
                2 => Vec3::new(0.25, 0.0, 0.5),
                3 => Vec3::new(-0.4, 0.0, 0.3),
                4 => Vec3::new(-0.5, 0.0, 0.0),
                5 => Vec3::new(-0.4, 0.0, -0.25),
                6 => Vec3::new(-0.25, 0.0, -0.5),
                7 => Vec3::new(0.25, 0.0, -0.25),
                _ => unreachable!(),
            };
            
            let pos = player_pos + muzzle_offset;
            
            // Calculate rotation to face direction
            let forward = Vec3::new(move_dir.0.x, 0.0, move_dir.0.y).normalize_or_zero();
            let rotation = if forward != Vec3::ZERO {
                Quat::from_rotation_arc(Vec3::X, forward)
            } else {
                Quat::IDENTITY
            };
            
            commands
                .spawn((
                    Bullet,
                    *move_dir,
                    Mesh3d(meshes.add(Capsule3d::new(BULLET_RADIUS, 0.3))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 1.0, 0.0),
                        emissive: LinearRgba::new(1.0, 1.0, 0.0, 1.0),
                        ..default()
                    })),
                    Transform::from_translation(pos).with_rotation(rotation),
                    Visibility::default(),
                ))
                .add_rollback();
            bullet_ready.0 = false;
        }
    }
}

pub fn move_bullet(mut bullets: Query<(&mut Transform, &MoveDir), With<Bullet>>, time: Res<Time>) {
    for (mut transform, dir) in &mut bullets {
        let speed = 20.;
        let delta = Vec3::new(dir.0.x, 0.0, dir.0.y) * speed * time.delta_secs();
        transform.translation += delta;
    }
}