// setup.rs
use bevy::prelude::*;
use crate::{core::constants::MAP_SIZE, entities::components::GameEntity};

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(MAP_SIZE as f32, MAP_SIZE as f32))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.2),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::default(),
        GameEntity, // Mark for cleanup
    ));

    // Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        GameEntity, // Mark for cleanup
    ));

    // Camera is already spawned in auth_ui.rs during WalletAuth state
}