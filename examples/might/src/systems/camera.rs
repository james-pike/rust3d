use bevy::prelude::*;
use shadplay::camera::PanOrbitCamera;

/// Setup camera with orbit controls and environment mapping
pub fn setup_camera(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(2.0, 6.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 1.0,
            ..default()
        },
    ));
}
