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

/// Resource to store player's display name and profile
#[derive(Resource, Clone)]
pub struct PlayerProfile {
    pub display_name: String,
    pub is_ready: bool,
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            display_name: "Knight".to_string(),
            is_ready: false,
        }
    }
}

/// Resource to track lobby notifications
#[derive(Resource, Default)]
pub struct LobbyNotifications {
    pub messages: Vec<LobbyNotification>,
}

#[derive(Clone)]
pub struct LobbyNotification {
    pub message: String,
    pub timestamp: f64,
}

impl LobbyNotifications {
    pub fn add(&mut self, message: String, time: f64) {
        self.messages.push(LobbyNotification {
            message,
            timestamp: time,
        });

        // Keep only last 10 notifications
        if self.messages.len() > 10 {
            self.messages.remove(0);
        }
    }
}

/// Setup lobby resources
pub fn setup_lobby_resources(
    mut notifications: ResMut<LobbyNotifications>,
    profile: Res<PlayerProfile>,
    time: Res<Time>,
) {
    info!("Setting up lobby resources");

    // Add notification that player entered lobby
    notifications.add(
        format!("{} entered the lobby", profile.display_name),
        time.elapsed_secs_f64(),
    );
}

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

/// Lobby UI with player profile, notifications, and matchmaking button
pub fn lobby_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut profile: ResMut<PlayerProfile>,
    notifications: Res<LobbyNotifications>,
    time: Res<Time>,
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

    // Main lobby panel - top center
    egui::Window::new("lobby_main")
        .title_bar(false)
        .resizable(false)
        .fixed_pos([screen_rect.center().x - 250.0, 20.0])
        .fixed_size([500.0, 220.0])
        .frame(egui::Frame::default()
            .fill(egui::Color32::from_rgba_unmultiplied(20, 15, 10, 250))
            .stroke(egui::Stroke::new(3.0, egui::Color32::from_rgb(100, 80, 50)))
            .inner_margin(20.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // Title
                ui.label(egui::RichText::new("⚔ KNIGHT LOBBY ⚔")
                    .size(26.0)
                    .color(egui::Color32::from_rgb(200, 180, 140))
                    .strong());

                ui.add_space(10.0);

                // Display Name Input
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Display Name:")
                        .size(14.0)
                        .color(egui::Color32::from_rgb(180, 160, 120)));

                    let name_response = ui.add(
                        egui::TextEdit::singleline(&mut profile.display_name)
                            .desired_width(200.0)
                            .char_limit(20)
                    );

                    if name_response.changed() {
                        info!("Display name changed to: {}", profile.display_name);
                    }
                });

                ui.add_space(8.0);

                // Player Status
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Status:")
                        .size(14.0)
                        .color(egui::Color32::from_rgb(180, 160, 120)));

                    let status_color = if profile.is_ready {
                        egui::Color32::from_rgb(100, 255, 100)
                    } else {
                        egui::Color32::from_rgb(255, 200, 100)
                    };

                    ui.label(egui::RichText::new(if profile.is_ready { "✓ Ready" } else { "⚙ Equipping" })
                        .size(14.0)
                        .color(status_color));

                    ui.add_space(10.0);

                    if ui.button(if profile.is_ready { "Not Ready" } else { "Ready Up" }).clicked() {
                        profile.is_ready = !profile.is_ready;
                        info!("Player ready status: {}", profile.is_ready);
                    }
                });

                ui.add_space(10.0);

                ui.label(egui::RichText::new("Press 'I' to open inventory and equip gear")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(150, 130, 100)));

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                // Start Matchmaking button
                let button = egui::Button::new(
                    egui::RichText::new("⚔ START MATCHMAKING ⚔")
                        .size(18.0)
                        .color(egui::Color32::WHITE)
                        .strong()
                )
                .fill(egui::Color32::from_rgb(60, 100, 60))
                .min_size(egui::vec2(350.0, 45.0));

                if ui.add(button).clicked() {
                    info!("🎮 START MATCHMAKING BUTTON CLICKED!");
                    info!("Player: {} | Ready: {}", profile.display_name, profile.is_ready);
                    next_state.set(GameState::Matchmaking);
                }
            });
        });

    // Lobby notifications panel - bottom left
    egui::Window::new("lobby_notifications")
        .title_bar(false)
        .resizable(false)
        .fixed_pos([20.0, screen_rect.height() - 250.0])
        .fixed_size([350.0, 180.0])
        .frame(egui::Frame::default()
            .fill(egui::Color32::from_rgba_unmultiplied(20, 15, 10, 230))
            .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 60, 40)))
            .inner_margin(15.0))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("LOBBY ACTIVITY")
                .size(16.0)
                .color(egui::Color32::from_rgb(200, 180, 140))
                .strong());

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(5.0);

            egui::ScrollArea::vertical()
                .max_height(110.0)
                .show(ui, |ui| {
                    if notifications.messages.is_empty() {
                        ui.label(egui::RichText::new("No recent activity...")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(120, 100, 80))
                            .italics());
                    } else {
                        for notification in notifications.messages.iter().rev() {
                            let time_elapsed = time.elapsed_secs_f64() - notification.timestamp;
                            let time_str = if time_elapsed < 60.0 {
                                format!("{}s ago", time_elapsed as u32)
                            } else {
                                format!("{}m ago", (time_elapsed / 60.0) as u32)
                            };

                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("•")
                                    .size(14.0)
                                    .color(egui::Color32::from_rgb(100, 200, 100)));

                                ui.label(egui::RichText::new(&notification.message)
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(220, 220, 220)));

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(egui::RichText::new(&time_str)
                                        .size(10.0)
                                        .color(egui::Color32::from_rgb(120, 100, 80)));
                                });
                            });

                            ui.add_space(3.0);
                        }
                    }
                });
        });

    // Player info panel - top right
    egui::Window::new("player_info")
        .title_bar(false)
        .resizable(false)
        .fixed_pos([screen_rect.width() - 270.0, 20.0])
        .fixed_size([250.0, 140.0])
        .frame(egui::Frame::default()
            .fill(egui::Color32::from_rgba_unmultiplied(20, 15, 10, 230))
            .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 60, 40)))
            .inner_margin(15.0))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("PLAYER INFO")
                .size(16.0)
                .color(egui::Color32::from_rgb(200, 180, 140))
                .strong());

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Name:")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(150, 130, 100)));
                ui.label(egui::RichText::new(&profile.display_name)
                    .size(12.0)
                    .color(egui::Color32::WHITE)
                    .strong());
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Connection:")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(150, 130, 100)));
                ui.label(egui::RichText::new("Local")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(100, 255, 100)));
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Ready:")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(150, 130, 100)));
                ui.label(egui::RichText::new(if profile.is_ready { "Yes" } else { "No" })
                    .size(12.0)
                    .color(if profile.is_ready {
                        egui::Color32::from_rgb(100, 255, 100)
                    } else {
                        egui::Color32::from_rgb(255, 200, 100)
                    }));
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Location:")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(150, 130, 100)));
                ui.label(egui::RichText::new("Lobby")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(200, 180, 140)));
            });
        });
}
