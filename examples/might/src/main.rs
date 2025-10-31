use bevy::{
    gltf::Gltf,
    log,
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

use shadplay::camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use std::f32::consts::*;

/// marker component for our knight
#[derive(Resource, PartialEq, Eq)]
struct Knight {
    handle: Handle<Gltf>,
}

/// marker resource to track whether or not we can spawn the loaded knight asset
#[derive(Resource, PartialEq, Eq)]
struct WasLoaded(bool);

fn main() {
    // -----------------------------------------------------------------
    // 1. Verify the model exists
    // -----------------------------------------------------------------
    let knight_model_path = std::path::Path::new("../assets/scenes/knight_uncompressed.glb");
    if !knight_model_path.exists() {
        dbg!(
            "knight.glb does not exist at {:?}.\n\
             Free non-rigged version: https://sketchfab.com/3d-models/elysia-knight-d099f11914f445afbe727fe5c3ddd39d\n\
             Rigged version (paid): https://www.artstation.com/marketplace/p/RGmbB/medieval-armor-set-unreal-engine-rigged",
            knight_model_path
        );
    }

    App::new()
        // -----------------------------------------------------------------
        // 2. NO SHADOW MAP RESOURCE NEEDED (Bevy 0.17 auto-sizes)
        // -----------------------------------------------------------------
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin))

        .add_systems(Startup, setup)
        .add_systems(Update, (animate_light_direction, quit_listener))

        // -----------------------------------------------------------------
        // 3. Load → spawn pipeline
        // -----------------------------------------------------------------
        .add_systems(Startup, load_knight)
        .add_systems(
            Update,
            spawn_knight
                .after(load_knight)
                .run_if(resource_exists::<Knight>)
                .run_if(resource_exists_and_equals::<WasLoaded>(WasLoaded(false))),
        )

        // -----------------------------------------------------------------
        // 4. Custom aura material
        // -----------------------------------------------------------------
        .add_plugins(MaterialPlugin::<AuraMaterial>::default())
        .run();
}

// ---------------------------------------------------------------------
// 5. Scene setup (camera + light + ground)
// ---------------------------------------------------------------------
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera + orbit + env-map
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

    // Directional light – **no cascade config needed**
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

// ---------------------------------------------------------------------
// 6. Load the GLTF
// ---------------------------------------------------------------------
fn load_knight(mut commands: Commands, asset_server: Res<AssetServer>) {
    let hndl = asset_server.load("scenes/knight.glb");
    commands.insert_resource(Knight { handle: hndl });
    commands.insert_resource(WasLoaded(false));
    log::info!("load_knight done.");
}

// ---------------------------------------------------------------------
// 7. Spawn knight + aura disc
// ---------------------------------------------------------------------
fn spawn_knight(
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

// ---------------------------------------------------------------------
// 8. Animate light
// ---------------------------------------------------------------------
fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut tf in &mut query {
        tf.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_secs() * PI / 100.0,
            -FRAC_PI_4,
        );
    }
}

// ---------------------------------------------------------------------
// 9. Aura material
// ---------------------------------------------------------------------
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct AuraMaterial {
    #[uniform(100)]
    inner: f32,
}

impl Material for AuraMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/aura2.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

// ---------------------------------------------------------------------
// 10. Quit on Q/Esc
// ---------------------------------------------------------------------
fn quit_listener(input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::KeyQ) || input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}