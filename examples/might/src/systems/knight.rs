// ============================================================
// File: systems/knight.rs
// ============================================================
use bevy::{gltf::Gltf, log, prelude::*};
use crate::resources::{Knight, WasLoaded};
use crate::materials::aura::AuraMaterial;

/// Load the GLTF knight model
pub fn load_knight(mut commands: Commands, asset_server: Res<AssetServer>) {
    let hndl = asset_server.load("scenes/knight.glb");
    commands.insert_resource(Knight { handle: hndl });
    commands.insert_resource(WasLoaded(false));
    log::info!("load_knight done.");
}

/// Spawn knight scene with aura disc
pub fn spawn_knight(
    mut commands: Commands,
    knight: Res<Knight>,
    assets_gltf: Res<Assets<Gltf>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut was_loaded: ResMut<WasLoaded>,
    mut materials: ResMut<Assets<AuraMaterial>>,
) {
    if let Some(gltf) = assets_gltf.get(&knight.handle) {
        log::info!("Spawning scene...");

        let disc = Mesh::from(Cylinder::new(1.2, 0.001));
        let aura_mat = AuraMaterial { inner: 0.0 };

        commands
            .spawn(SceneRoot(gltf.scenes[0].clone()))
            .with_children(|parent| {
                parent.spawn((
                    Mesh3d(meshes.add(disc)),
                    MeshMaterial3d(materials.add(aura_mat)),
                ));
            });

        *was_loaded = WasLoaded(true);
        log::info!("Spawn complete...");
    }
}