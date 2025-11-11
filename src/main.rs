// main.rs (updated)
use args::Args;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_ggrs::prelude::*;
use bevy_matchbox::prelude::PeerId;
use bevy_roll_safe::prelude::*;
use clap::Parser;
use components::*;
use input::*;
use constants::*;

mod args;
mod components;
mod input;
mod ui;
mod camera;
mod networking;
mod matchmaking;
mod bullet;
mod constants;
mod map;
mod player;
mod collisions;
mod resources;
mod states;
mod utils;
mod setup;
mod round;

type Config = bevy_ggrs::GgrsConfig<u8, PeerId>;

#[derive(AssetCollection, Resource)]
struct ModelAssets {
    #[asset(path = "player1.glb#Scene0")]
    player_1: Handle<Scene>,
    #[asset(path = "player2.glb#Scene0")]
    player_2: Handle<Scene>,
}

fn main() {
    let args = Args::parse();
    eprintln!("{args:?}");

    App::new()
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
        ))
        .init_state::<states::GameState>()
        .add_loading_state(
            LoadingState::new(states::GameState::AssetLoading)
                .load_collection::<ModelAssets>()
                .continue_to_state(states::GameState::Matchmaking),
        )
        .init_ggrs_state::<states::RollbackState>()
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
        .insert_resource(args)
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .init_resource::<resources::RoundEndTimer>()
        .init_resource::<resources::Scores>()
        .add_systems(
            OnEnter(states::GameState::Matchmaking),
            (setup::setup, matchmaking::start_matchbox_socket.run_if(matchmaking::p2p_mode)),
        )
        .add_systems(
            Update,
            (
                (
                    matchmaking::wait_for_players.run_if(matchmaking::p2p_mode),
                    matchmaking::start_synctest_session.run_if(matchmaking::synctest_mode),
                )
                    .run_if(in_state(states::GameState::Matchmaking)),
                camera::camera_follow.run_if(in_state(states::GameState::InGame)),
                ui::update_score_ui.run_if(in_state(states::GameState::InGame)),
                networking::handle_ggrs_events.run_if(in_state(states::GameState::InGame)),
            ),
        )
        .add_systems(ReadInputs, read_local_inputs)
        .add_systems(
            OnEnter(states::RollbackState::InRound),
            (map::generate_map, player::spawn_players.after(map::generate_map)),
        )
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
                .run_if(in_state(states::RollbackState::InRound))
                .after(bevy_roll_safe::apply_state_transition::<states::RollbackState>),
        )
        .add_systems(
            RollbackUpdate,
            round::round_end_timeout
                .run_if(in_state(states::RollbackState::RoundEnd))
                .ambiguous_with(collisions::kill_players),
        )
        .run();
}