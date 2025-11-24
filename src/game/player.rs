// player.rs
use bevy::prelude::*;
use bevy::math::primitives::Cylinder;
use bevy::light::NotShadowCaster;
use bevy_ggrs::{prelude::*, AddRollbackCommandExtension};
use rand_xoshiro::Xoshiro256PlusPlus;
use rand::{Rng, SeedableRng};
use crate::{
    Config,
    core::constants::*,
    entities::components::{Player, BulletReady, MoveDir, DistanceTraveled, Bullet, GameEntity},
    ModelAssets,
    core::resources::{Scores, SessionSeed},
    game::input::direction,
    materials::aura::{AuraMaterial, EFFECT_FIRE},
    systems::aura_effects::AuraDisc,
};

pub fn spawn_players(
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
    bullets: Query<Entity, With<Bullet>>,
    aura_discs: Query<Entity, With<AuraDisc>>, // ADD: To despawn auras separately
    scores: Res<Scores>,
    session_seed: Res<SessionSeed>,
    models: Res<ModelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut aura_materials: Option<ResMut<Assets<AuraMaterial>>>, // Fixed: Option to avoid panic
) {
    info!("Spawning players");

    // Despawn players
    for player in &players {
        commands.entity(player).despawn();
    }

    // Despawn bullets
    for bullet in &bullets {
        commands.entity(bullet).despawn();
    }

    // Despawn existing aura discs (safe fallback without recursive)
    for disc in &aura_discs {
        commands.entity(disc).despawn();
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

    // Common aura setup (only if materials resource exists)
    let disc_mesh = meshes.add(Cylinder::new(2.0, 0.05).mesh());
    let aura_mat_handle = if let Some(mut aura_materials) = aura_materials {
        aura_materials.add(AuraMaterial {
            effect_type: EFFECT_FIRE,
            intensity: 1.0,
            color_r: 1.0,
            color_g: 0.5,
            color_b: 0.0,
            _padding1: 0.0,
            _padding2: 0.0,
            _padding3: 0.0,
        })
    } else {
        // Fallback: Spawn without material (invisible disc, but entity exists for later UI)
        Handle::default()
    };

    let aura_bundle = (
        Mesh3d(disc_mesh),
        MeshMaterial3d(aura_mat_handle),
        AuraDisc,
        Transform::from_scale(Vec3::new(1.0, 1.0, 1.0)), // No offset needed; shader handles positioning
        Visibility::Inherited,
        NotShadowCaster, // Prevent aura from casting shadows
    );

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
            GameEntity, // Mark for cleanup
        ))
        .add_rollback()
        .with_children(|parent| {
            parent.spawn(aura_bundle.clone());
        });

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
            GameEntity, // Mark for cleanup
        ))
        .add_rollback()
        .with_children(|parent| {
            parent.spawn(aura_bundle.clone());
        });
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