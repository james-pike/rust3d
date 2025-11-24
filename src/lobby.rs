// lobby.rs - Lobby system for knight preview and gear selection
use bevy::prelude::*;
use bevy::math::primitives::Cylinder;
use bevy::light::NotShadowCaster;
use bevy_egui::{egui, EguiContexts};
use crate::{
    ModelAssets,
    materials::aura::{AuraMaterial, EFFECT_FIRE},
    systems::aura_effects::AuraDisc,
    states::GameState,
    inventory_system::KnightCharacter,
};

/// Component to mark lobby entities that should be despawned when leaving lobby
#[derive(Component)]
pub struct LobbyEntity;

/// Spawn the local player's knight in the lobby for preview
pub fn spawn_lobby_knight(
    mut commands: Commands,
    models: Res<ModelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut aura_materials: ResMut<Assets<AuraMaterial>>,
) {
    info!("Spawning lobby knight for preview");

    // Spawn a single knight at origin for preview
    let knight_pos = Vec3::new(0.0, 0.0, 0.0);
    let initial_rotation = Quat::IDENTITY;

    // Create aura disc
    let disc_mesh = meshes.add(Cylinder::new(2.0, 0.05).mesh());
    let aura_mat_handle = aura_materials.add(AuraMaterial {
        effect_type: EFFECT_FIRE,
        intensity: 1.0,
        color_r: 1.0,
        color_g: 0.5,
        color_b: 0.0,
        _padding1: 0.0,
        _padding2: 0.0,
        _padding3: 0.0,
    });

    let aura_bundle = (
        Mesh3d(disc_mesh),
        MeshMaterial3d(aura_mat_handle),
        AuraDisc,
        Transform::from_scale(Vec3::new(1.0, 1.0, 1.0)),
        Visibility::Inherited,
        NotShadowCaster,
    );

    // Spawn lobby knight
    commands
        .spawn((
            SceneRoot(models.player_1.clone()),
            Transform::from_translation(knight_pos).with_rotation(initial_rotation),
            Visibility::default(),
            LobbyEntity, // Mark for cleanup
            KnightCharacter, // Mark as the knight for inventory system
        ))
        .with_children(|parent| {
            parent.spawn(aura_bundle);
        });

    info!("Lobby knight spawned successfully");
}

/// Spawn camera for lobby view
pub fn spawn_lobby_camera(mut commands: Commands) {
    info!("Spawning lobby camera");

    // Close-up camera for knight inspection
    let camera_pos = Vec3::new(0.0, 6.0, 6.0);
    let look_at = Vec3::new(0.0, 1.0, 0.0); // Look at knight's center

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(camera_pos).looking_at(look_at, Vec3::Y),
        LobbyEntity, // Mark for cleanup
    ));
}

/// Spawn lighting for lobby
pub fn spawn_lobby_lighting(mut commands: Commands) {
    info!("Spawning lobby lighting");

    // Main directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
        LobbyEntity,
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        affects_lightmapped_meshes: false,
    });
}

/// Cleanup lobby entities when leaving lobby
pub fn cleanup_lobby(
    mut commands: Commands,
    lobby_entities: Query<Entity, With<LobbyEntity>>,
) {
    info!("Cleaning up lobby entities");

    for entity in &lobby_entities {
        commands.entity(entity).despawn();
    }
}

/// Lobby UI with start matchmaking button
pub fn lobby_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    // Debug: Log current state
    if current_state.is_changed() {
        info!("Current game state: {:?}", current_state.get());
    }

    let ctx = match contexts.ctx_mut() {
        Ok(c) => c,
        Err(_) => {
            warn!("No Egui context available in lobby");
            return;
        }
    };
    let screen_rect = ctx.viewport_rect();

    // Top center panel with lobby title and start button
    egui::Window::new("lobby_controls")
        .title_bar(false)
        .resizable(false)
        .fixed_pos([screen_rect.center().x - 200.0, 20.0])
        .fixed_size([400.0, 120.0])
        .frame(egui::Frame::default()
            .fill(egui::Color32::from_rgba_unmultiplied(20, 15, 10, 250))
            .stroke(egui::Stroke::new(3.0, egui::Color32::from_rgb(100, 80, 50)))
            .inner_margin(20.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("KNIGHT LOBBY")
                    .size(24.0)
                    .color(egui::Color32::from_rgb(200, 180, 140))
                    .strong());

                ui.add_space(10.0);

                ui.label(egui::RichText::new("Equip your knight with gear from the inventory (Press I)")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(180, 160, 120)));

                ui.add_space(10.0);

                // Start Matchmaking button
                let button = egui::Button::new(
                    egui::RichText::new("⚔ START MATCHMAKING ⚔")
                        .size(18.0)
                        .color(egui::Color32::WHITE)
                        .strong()
                )
                .fill(egui::Color32::from_rgb(60, 100, 60))
                .min_size(egui::vec2(300.0, 40.0));

                if ui.add(button).clicked() {
                    info!("🎮 START MATCHMAKING BUTTON CLICKED!");
                    info!("Transitioning from {:?} to Matchmaking", current_state.get());
                    next_state.set(GameState::Matchmaking);
                }
            });
        });
}
