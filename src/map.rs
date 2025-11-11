// map.rs
use bevy::prelude::*;
use rand_xoshiro::Xoshiro256PlusPlus;
use rand::{Rng, SeedableRng};
use crate::{constants::*, components::Wall, resources::{Scores, SessionSeed}};

pub fn generate_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    walls: Query<Entity, With<Wall>>,
    scores: Res<Scores>,
    session_seed: Res<SessionSeed>,
) {
    // despawn walls from previous round (if any)
    for wall in &walls {
        commands.entity(wall).despawn();
    }

    let mut rng = Xoshiro256PlusPlus::seed_from_u64((scores.0 + scores.1) as u64 ^ **session_seed);

    for _ in 0..20 {
        let max_box_size = MAP_SIZE / 4;
        let width = rng.random_range(1..max_box_size);
        let depth = rng.random_range(1..max_box_size);

        let cell_x = rng.random_range(0..=(MAP_SIZE - width));
        let cell_z = rng.random_range(0..=(MAP_SIZE - depth));

        let size = Vec3::new(width as f32, WALL_HEIGHT, depth as f32);

        commands.spawn((
            Wall,
            Mesh3d(meshes.add(Cuboid::from_size(size))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.4, 0.4, 0.4),
                ..default()
            })),
            Transform::from_translation(Vec3::new(
                cell_x as f32 + size.x / 2. - MAP_SIZE as f32 / 2.,
                WALL_HEIGHT / 2.,
                cell_z as f32 + size.z / 2. - MAP_SIZE as f32 / 2.,
            )),
            Visibility::default(),
        ));
    }
}