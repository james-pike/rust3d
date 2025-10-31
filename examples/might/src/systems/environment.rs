use bevy::prelude::*;
use std::f32::consts::FRAC_PI_4;

/// Setup environment (directional light and ground plane)
pub fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Directional light with shadows
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 0.0, -FRAC_PI_4)),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::BLACK,
            perceptual_roughness: 1.0,
            double_sided: false,
            unlit: true,
            ..default()
        })),
    ));
}