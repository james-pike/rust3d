// lib.rs - WASM entry point
use bevy::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Core systems
mod core;

// Game systems
mod game;

// World systems
mod world;

// Entities
mod entities;

// UI systems
mod ui;

// Network systems
mod network;

// Visual effects & materials
mod systems;
mod materials;

// Utilities
mod utils;

use core::args::Args;
use core::states::GameState;
use bevy_asset_loader::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass}; // Fixed: Import EguiPrimaryContextPass instead
use bevy_ggrs::prelude::*;
use bevy_matchbox::prelude::PeerId;
use bevy_roll_safe::prelude::*;
use clap::Parser;
use entities::components::*;
use game::input::read_local_inputs;

// Define UiReady resource here for simplicity (or move to resources.rs) - made pub for cross-module access
#[derive(Resource, Default, PartialEq)]
pub struct UiReady(pub bool);

use ui::chat::{ChatPlugin, setup_chat_socket, ChatUIPlugin};
use ui::auth::AuthUIPlugin;
use systems::aura_effects::{setup_aura_effects, aura_effects_ui, handle_shader_reload}; // ADD

type Config = bevy_ggrs::GgrsConfig<u8, PeerId>;

#[derive(AssetCollection, Resource)]
struct ModelAssets {
    #[asset(path = "player1.glb#Scene0")]
    player_1: Handle<Scene>,
    #[asset(path = "player1.glb#Scene0")]
    player_2: Handle<Scene>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    run_app();
}

fn run_app() {
    let args = Args::parse();
    eprintln!("{args:?}");

    App::new()
        // Core plugins first
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            GgrsPlugin::<Config>::default(),
            RollbackSchedulePlugin::new_ggrs(),
            EguiPlugin::default(),
            MaterialPlugin::<materials::aura::AuraMaterial>::default(),
        ))
        // Custom plugins
        .add_plugins(ChatPlugin)
        .add_plugins(ChatUIPlugin)
        .add_plugins(AuthUIPlugin)
        // State machine
        .init_state::<GameState>()
        .insert_state(GameState::WalletAuth)
        // Loading state
        .add_loading_state(
            LoadingState::new(core::states::GameState::AssetLoading)
                .load_collection::<ModelAssets>()
                .continue_to_state(core::states::GameState::Lobby), // Go to Lobby after loading
        )
        // GGRS rollback setup
        .init_ggrs_state::<core::states::RollbackState>()
        .rollback_resource_with_clone::<resources::RoundEndTimer>()
        .rollback_resource_with_copy::<resources::Scores>()
        .rollback_component_with_clone::<Transform>()
        .rollback_component_with_copy::<Bullet>()
        .rollback_component_with_copy::<BulletReady>()
        .rollback_component_with_copy::<Player>()
        .rollback_component_with_copy::<Wall>()
        .rollback_component_with_copy::<MoveDir>()
        .rollback_component_with_copy::<DistanceTraveled>()
        .checksum_component::<Transform>(utils::checksum_transform)
        // Resources
        .insert_resource(args)
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .init_resource::<resources::RoundEndTimer>()
        .init_resource::<resources::Scores>()
        .init_resource::<UiReady>()
        .init_resource::<lobby::PlayerProfile>()
        .init_resource::<lobby::LobbyNotifications>()
        // REMOVED: OnEnter(GameState::WalletAuth) - handled by AuthUIPlugin

        // Flip UiReady after first frame (runs ONCE when false)
        .add_systems(
            Update,
            set_ui_ready
                .run_if(resource_equals(UiReady(false))),
        )
        // REMOVED: Fallback system (unneeded - set_ui_ready works)
        // Lobby entry - spawn knight, camera, lighting, and setup systems
        .add_systems(
            OnEnter(core::states::GameState::Lobby),
            (
                lobby::setup_lobby_resources,
                lobby::spawn_lobby_knight,
                lobby::spawn_lobby_camera,
                lobby::spawn_lobby_lighting,
                inventory_system::setup_inventory_system,
                hud_system::setup_player_vitals,
            ),
        )
        // Lobby exit - cleanup
        .add_systems(
            OnExit(core::states::GameState::Lobby),
            lobby::cleanup_lobby,
        )
        // Matchmaking entry
        .add_systems(
            OnEnter(core::states::GameState::Matchmaking),
            (
                setup::setup,
                matchmaking::start_matchbox_socket.run_if(matchmaking::p2p_mode),
                chat::setup_chat_socket.run_if(matchmaking::p2p_mode),
            ),
        )
        // Aura systems (visual only, no rollback needed)
        .add_systems(Startup, setup_aura_effects)
        .add_systems(
            EguiPrimaryContextPass, // Fixed: Add to EguiPrimaryContextPass schedule
            (
                aura_effects_ui,
                lobby::lobby_ui.run_if(in_state(core::states::GameState::Lobby)),
            ),
        )
        .add_systems(
            Update,
            handle_shader_reload,
        )
        // Update systems
        .add_systems(
            Update,
            (
                // Inventory systems in Lobby
                (
                    inventory_system::handle_inventory_input,
                    inventory_system::animate_inventory_drawer,
                    inventory_system::update_texture_cache,
                    inventory_system::inventory_ui,
                    inventory_system::attach_to_bones,
                )
                    .run_if(in_state(core::states::GameState::Lobby)),
                // HUD systems in Lobby
                (
                    hud_system::render_diablo_hud,
                    hud_system::simulate_vitals_changes,
                )
                    .run_if(in_state(core::states::GameState::Lobby)),
                // Matchmaking systems
                (
                    matchmaking::wait_for_players.run_if(matchmaking::p2p_mode),
                    matchmaking::start_synctest_session.run_if(matchmaking::synctest_mode),
                )
                    .run_if(in_state(core::states::GameState::Matchmaking)),
                // InGame systems
                camera::camera_follow.run_if(in_state(core::states::GameState::InGame)),
                ui::update_score_ui.run_if(in_state(core::states::GameState::InGame)),
                auth_ui::update_wallet_display.run_if(in_state(core::states::GameState::InGame)),
                networking::handle_ggrs_events.run_if(in_state(core::states::GameState::InGame)),
                // Inventory systems during gameplay
                (
                    inventory_system::handle_inventory_input,
                    inventory_system::animate_inventory_drawer,
                    inventory_system::update_texture_cache,
                    inventory_system::inventory_ui,
                    inventory_system::attach_to_bones,
                )
                    .run_if(in_state(core::states::GameState::InGame)),
                // HUD systems during gameplay
                (
                    hud_system::render_diablo_hud,
                    hud_system::simulate_vitals_changes,
                )
                    .run_if(in_state(core::states::GameState::InGame)),
            ),
        )
        // Input reading
        .add_systems(ReadInputs, read_local_inputs)
        // Round entry
        .add_systems(
            OnEnter(core::states::RollbackState::InRound),
            (map::generate_map, player::spawn_players.after(map::generate_map)),
        )
        // Rollback logic
        .add_systems(
            RollbackUpdate,
            (
                player::move_players,
                collisions::resolve_wall_collisions.after(player::move_players),
                bullet::reload_bullet,
                bullet::fire_bullets
                    .after(player::move_players)
                    .after(bullet::reload_bullet)
                    .after(collisions::resolve_wall_collisions),
                bullet::move_bullet.after(bullet::fire_bullets),
                collisions::bullet_wall_collisions.after(bullet::move_bullet),
                collisions::kill_players.after(bullet::move_bullet).after(player::move_players),
            )
                .run_if(in_state(core::states::RollbackState::InRound))
                .after(bevy_roll_safe::apply_state_transition::<core::states::RollbackState>),
        )
        .add_systems(
            RollbackUpdate,
            round::round_end_timeout
                .run_if(in_state(core::states::RollbackState::RoundEnd))
                .ambiguous_with(collisions::kill_players),
        )
        .run();
}

// System to enable UI after first frame (runs correctly)
fn set_ui_ready(mut ui_ready: ResMut<UiReady>) {
    ui_ready.0 = true;
    info!("✅ UiReady set to true - Egui should now draw");
}

fn transition_to_asset_loading(mut next_state: ResMut<NextState<core::states::GameState>>) {
    next_state.set(core::states::GameState::AssetLoading);
}