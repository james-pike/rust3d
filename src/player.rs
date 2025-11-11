// player.rs
use bevy::prelude::*;
use bevy_ggrs::{prelude::*, AddRollbackCommandExtension};
use rand_xoshiro::Xoshiro256PlusPlus;
use rand::{Rng, SeedableRng};
use crate::{Config, constants::*, components::{Player, BulletReady, MoveDir, DistanceTraveled, Bullet}, ModelAssets, resources::{Scores, SessionSeed}, input::direction};

pub fn spawn_players(
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
    bullets: Query<Entity, With<Bullet>>,
    scores: Res<Scores>,
    session_seed: Res<SessionSeed>,
    models: Res<ModelAssets>,
) {
    info!("Spawning players");

    for player in &players {
        commands.entity(player).despawn();
    }

    for bullet in &bullets {
        commands.entity(bullet).despawn();
    }

    let mut rng = Xoshiro256PlusPlus::seed_from_u64((scores.0 + scores.1) as u64 ^ **session_seed);
    let half = MAP_SIZE as f32 / 2.;
    let p1_pos = Vec3::new(
        rng.random_range(-half..half),
        PLAYER_HEIGHT / 2.,
        rng.random_range(-half..half),
    );
    let p2_pos = Vec3::new(
        rng.random_range(-half..half),
        PLAYER_HEIGHT / 2.,
        rng.random_range(-half..half),
    );

    let initial_dir = -Vec2::X;
    let forward = Vec3::new(initial_dir.x, 0.0, initial_dir.y).normalize_or_zero();
    let initial_rotation = if forward != Vec3::ZERO {
        Quat::from_rotation_arc(Vec3::X, forward)
    } else {
        Quat::IDENTITY
    };

    // Player 1
    commands
        .spawn((
            Player { handle: 0 },
            BulletReady(true),
            MoveDir(initial_dir),
            DistanceTraveled(0.0),
            SceneRoot(models.player_1.clone()),
            Transform::from_translation(p1_pos).with_rotation(initial_rotation),
            Visibility::default(),
        ))
        .add_rollback();

    // Player 2
    commands
        .spawn((
            Player { handle: 1 },
            BulletReady(true),
            MoveDir(initial_dir),
            DistanceTraveled(0.0),
            SceneRoot(models.player_2.clone()),
            Transform::from_translation(p2_pos).with_rotation(initial_rotation),
            Visibility::default(),
        ))
        .add_rollback();
}

pub fn move_players(
    mut players: Query<(&mut Transform, &mut MoveDir, &mut DistanceTraveled, &Player)>,
    inputs: Res<PlayerInputs<Config>>,
    time: Res<Time>,
) {
    for (mut transform, mut move_direction, mut distance, player) in &mut players {
        let (input, _) = inputs[player.handle];

        let direction = direction(input);

        if direction == Vec2::ZERO {
            continue;
        }

        move_direction.0 = direction;

        // Set rotation to face movement direction
        let forward = Vec3::new(direction.x, 0.0, direction.y).normalize_or_zero();
        if forward != Vec3::ZERO {
            transform.rotation = Quat::from_rotation_arc(Vec3::X, forward);
        }

        let move_speed = 6.;
        let move_delta = direction * move_speed * time.delta_secs();

        let old_pos = transform.translation;
        let limit = MAP_SIZE as f32 / 2. - 0.5;
        
        // Move in XZ plane (horizontal plane in 3D)
        let new_x = (old_pos.x + move_delta.x).clamp(-limit, limit);
        let new_z = (old_pos.z + move_delta.y).clamp(-limit, limit);

        transform.translation.x = new_x;
        transform.translation.z = new_z;

        distance.0 += move_delta.length();
    }
}