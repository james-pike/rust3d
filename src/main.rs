use args::Args;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPlugin,
    egui::{self, Align2, Color32, FontId, RichText},
};
use bevy_ggrs::{ggrs::DesyncDetection, prelude::*, *};
use bevy_matchbox::prelude::*;
use bevy_roll_safe::prelude::*;
use bevy::scene::SceneRoot;
use clap::Parser;
use components::*;
use input::*;
use rand::{Rng, SeedableRng, rng};
use rand_xoshiro::Xoshiro256PlusPlus;

mod args;
mod components;
mod input;

type Config = bevy_ggrs::GgrsConfig<u8, PeerId>;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum GameState {
    #[default]
    AssetLoading,
    Matchmaking,
    InGame,
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum RollbackState {
    #[default]
    InRound,
    RoundEnd,
}

#[derive(Resource, Clone, Deref, DerefMut)]
struct RoundEndTimer(Timer);

#[derive(Resource, Default, Clone, Copy, Debug)]
struct Scores(u32, u32);

impl Default for RoundEndTimer {
    fn default() -> Self {
        RoundEndTimer(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

#[derive(Resource, Default, Clone, Copy, Debug, Deref, DerefMut)]
struct SessionSeed(u64);

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
        .init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .load_collection::<ModelAssets>()
                .continue_to_state(GameState::Matchmaking),
        )
        .init_ggrs_state::<RollbackState>()
        .rollback_resource_with_clone::<RoundEndTimer>()
        .rollback_resource_with_copy::<Scores>()
        .rollback_component_with_clone::<Transform>()
        .rollback_component_with_copy::<Bullet>()
        .rollback_component_with_copy::<BulletReady>()
        .rollback_component_with_copy::<Player>()
        .rollback_component_with_copy::<Wall>()
        .rollback_component_with_copy::<MoveDir>()
        .rollback_component_with_copy::<DistanceTraveled>()
        .checksum_component::<Transform>(checksum_transform)
        .insert_resource(args)
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .init_resource::<RoundEndTimer>()
        .init_resource::<Scores>()
        .add_systems(
            OnEnter(GameState::Matchmaking),
            (setup, start_matchbox_socket.run_if(p2p_mode)),
        )
        .add_systems(
            Update,
            (
                (
                    wait_for_players.run_if(p2p_mode),
                    start_synctest_session.run_if(synctest_mode),
                )
                    .run_if(in_state(GameState::Matchmaking)),
                camera_follow.run_if(in_state(GameState::InGame)),
                update_score_ui.run_if(in_state(GameState::InGame)),
                handle_ggrs_events.run_if(in_state(GameState::InGame)),
            ),
        )
        .add_systems(ReadInputs, read_local_inputs)
        .add_systems(
            OnEnter(RollbackState::InRound),
            (generate_map, spawn_players.after(generate_map)),
        )
        .add_systems(
            RollbackUpdate,
            (
                move_players,
                resolve_wall_collisions.after(move_players),
                reload_bullet,
                fire_bullets
                    .after(move_players)
                    .after(reload_bullet)
                    .after(resolve_wall_collisions),
                move_bullet.after(fire_bullets),
                bullet_wall_collisions.after(move_bullet),
                kill_players.after(move_bullet).after(move_players),
            )
                .run_if(in_state(RollbackState::InRound))
                .after(bevy_roll_safe::apply_state_transition::<RollbackState>),
        )
        .add_systems(
            RollbackUpdate,
            round_end_timeout
                .run_if(in_state(RollbackState::RoundEnd))
                .ambiguous_with(kill_players),
        )
        .run();
}

const MAP_SIZE: i32 = 41;
const WALL_HEIGHT: f32 = 3.0;

fn synctest_mode(args: Res<Args>) -> bool {
    args.synctest
}

fn p2p_mode(args: Res<Args>) -> bool {
    !args.synctest
}

fn setup(
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
    ));

    // Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 3D Camera with third-person view
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 15.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));
}

fn generate_map(
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

fn spawn_players(
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
    bullets: Query<Entity, With<Bullet>>,
    scores: Res<Scores>,
    session_seed: Res<SessionSeed>,
    models: Res<ModelAssets>,
) {
    info!("Spawning players");

    for player in &players {
        commands.entity(player).despawn();
    }

    for bullet in &bullets {
        commands.entity(bullet).despawn();
    }

    let mut rng = Xoshiro256PlusPlus::seed_from_u64((scores.0 + scores.1) as u64 ^ **session_seed);
    let half = MAP_SIZE as f32 / 2.;
    let p1_pos = Vec3::new(
        rng.random_range(-half..half),
        PLAYER_HEIGHT / 2.,
        rng.random_range(-half..half),
    );
    let p2_pos = Vec3::new(
        rng.random_range(-half..half),
        PLAYER_HEIGHT / 2.,
        rng.random_range(-half..half),
    );

    let initial_dir = -Vec2::X;
    let forward = Vec3::new(initial_dir.x, 0.0, initial_dir.y).normalize_or_zero();
    let initial_rotation = if forward != Vec3::ZERO {
        Quat::from_rotation_arc(Vec3::X, forward)
    } else {
        Quat::IDENTITY
    };

    // Player 1
    commands
        .spawn((
            Player { handle: 0 },
            BulletReady(true),
            MoveDir(initial_dir),
            DistanceTraveled(0.0),
            SceneRoot(models.player_1.clone()),
            Transform::from_translation(p1_pos).with_rotation(initial_rotation),
            Visibility::default(),
        ))
        .add_rollback();

    // Player 2
    commands
        .spawn((
            Player { handle: 1 },
            BulletReady(true),
            MoveDir(initial_dir),
            DistanceTraveled(0.0),
            SceneRoot(models.player_2.clone()),
            Transform::from_translation(p2_pos).with_rotation(initial_rotation),
            Visibility::default(),
        ))
        .add_rollback();
}

fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "ws://127.0.0.1:3536/extreme_bevy?next=2";
    info!("connecting to matchbox server: {room_url}");
    commands.insert_resource(MatchboxSocket::new_unreliable(room_url));
}

fn wait_for_players(
    mut commands: Commands,
    mut socket: ResMut<MatchboxSocket>,
    mut next_state: ResMut<NextState<GameState>>,
    args: Res<Args>,
) {
    if socket.get_channel(0).is_err() {
        return;
    }

    socket.update_peers();
    let players = socket.players();

    let num_players = 2;
    if players.len() < num_players {
        return;
    }

    info!("All peers have joined, going in-game");

    let id = socket.id().expect("no peer id assigned").0.as_u64_pair();
    let mut seed = id.0 ^ id.1;
    for peer in socket.connected_peers() {
        let peer_id = peer.0.as_u64_pair();
        seed ^= peer_id.0 ^ peer_id.1;
    }
    commands.insert_resource(SessionSeed(seed));

    let mut session_builder = ggrs::SessionBuilder::<Config>::new()
        .with_num_players(num_players)
        .with_desync_detection_mode(DesyncDetection::On { interval: 1 })
        .with_input_delay(args.input_delay);

    for (i, player) in players.into_iter().enumerate() {
        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }

    let socket = socket.take_channel(0).unwrap();

    let ggrs_session = session_builder
        .start_p2p_session(socket)
        .expect("failed to start session");

    commands.insert_resource(bevy_ggrs::Session::P2P(ggrs_session));
    next_state.set(GameState::InGame);
}

fn start_synctest_session(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    info!("Starting synctest session");
    let num_players = 2;

    let mut session_builder = ggrs::SessionBuilder::<Config>::new().with_num_players(num_players);

    for i in 0..num_players {
        session_builder = session_builder
            .add_player(PlayerType::Local, i)
            .expect("failed to add player");
    }

    let ggrs_session = session_builder
        .start_synctest_session()
        .expect("failed to start session");

    commands.insert_resource(bevy_ggrs::Session::SyncTest(ggrs_session));
    commands.insert_resource(SessionSeed(rng().random()));
    next_state.set(GameState::InGame);
}

fn handle_ggrs_events(mut session: ResMut<Session<Config>>) {
    if let Session::P2P(s) = session.as_mut() {
        for event in s.events() {
            match event {
                GgrsEvent::Disconnected { .. } | GgrsEvent::NetworkInterrupted { .. } => {
                    warn!("GGRS event: {event:?}")
                }
                GgrsEvent::DesyncDetected {
                    local_checksum,
                    remote_checksum,
                    frame,
                    ..
                } => {
                    error!(
                        "Desync on frame {frame}. Local checksum: {local_checksum:X}, remote checksum: {remote_checksum:X}"
                    );
                }
                _ => info!("GGRS event: {event:?}"),
            }
        }
    }
}

fn move_players(
    mut players: Query<(&mut Transform, &mut MoveDir, &mut DistanceTraveled, &Player)>,
    inputs: Res<PlayerInputs<Config>>,
    time: Res<Time>,
) {
    for (mut transform, mut move_direction, mut distance, player) in &mut players {
        let (input, _) = inputs[player.handle];

        let direction = direction(input);

        if direction == Vec2::ZERO {
            continue;
        }

        move_direction.0 = direction;

        // Set rotation to face movement direction
        let forward = Vec3::new(direction.x, 0.0, direction.y).normalize_or_zero();
        if forward != Vec3::ZERO {
            transform.rotation = Quat::from_rotation_arc(Vec3::X, forward);
        }

        let move_speed = 6.;
        let move_delta = direction * move_speed * time.delta_secs();

        let old_pos = transform.translation;
        let limit = MAP_SIZE as f32 / 2. - 0.5;
        
        // Move in XZ plane (horizontal plane in 3D)
        let new_x = (old_pos.x + move_delta.x).clamp(-limit, limit);
        let new_z = (old_pos.z + move_delta.y).clamp(-limit, limit);

        transform.translation.x = new_x;
        transform.translation.z = new_z;

        distance.0 += move_delta.length();
    }
}

fn resolve_wall_collisions(
    mut players: Query<&mut Transform, With<Player>>,
    walls: Query<&Transform, (With<Wall>, Without<Player>)>,
) {
    for mut player_transform in &mut players {
        for wall_transform in &walls {
            let wall_scale = wall_transform.scale;
            let wall_size = Vec3::new(wall_scale.x, WALL_HEIGHT, wall_scale.z);
            let wall_pos = wall_transform.translation;
            let player_pos = player_transform.translation;

            // Work in XZ plane
            let wall_pos_xz = Vec2::new(wall_pos.x, wall_pos.z);
            let player_pos_xz = Vec2::new(player_pos.x, player_pos.z);
            let wall_size_xz = Vec2::new(wall_size.x, wall_size.z);

            let wall_to_player = player_pos_xz - wall_pos_xz;
            let wall_to_player_abs = wall_to_player.abs();
            let wall_corner_to_player_center = wall_to_player_abs - wall_size_xz / 2.;

            let corner_to_corner = wall_corner_to_player_center - Vec2::splat(PLAYER_RADIUS);

            if corner_to_corner.x > 0. || corner_to_corner.y > 0. {
                continue;
            }

            if corner_to_corner.x > corner_to_corner.y {
                player_transform.translation.x -= wall_to_player.x.signum() * corner_to_corner.x;
            } else {
                player_transform.translation.z -= wall_to_player.y.signum() * corner_to_corner.y;
            }
        }
    }
}

fn reload_bullet(
    inputs: Res<PlayerInputs<Config>>,
    mut players: Query<(&mut BulletReady, &Player)>,
) {
    for (mut can_fire, player) in players.iter_mut() {
        let (input, _) = inputs[player.handle];
        if !fire(input) {
            can_fire.0 = true;
        }
    }
}

fn fire_bullets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    inputs: Res<PlayerInputs<Config>>,
    mut players: Query<(&Transform, &Player, &mut BulletReady, &MoveDir)>,
) {
    for (transform, player, mut bullet_ready, move_dir) in &mut players {
        let (input, _) = inputs[player.handle];
        if fire(input) && bullet_ready.0 {
            let player_pos = transform.translation;
            
            // Muzzle offset in 3D space
            let muzzle_offset = match move_dir.octant() {
                0 => Vec3::new(0.5, 0.0, 0.0),
                1 => Vec3::new(0.5, 0.0, 0.25),
                2 => Vec3::new(0.25, 0.0, 0.5),
                3 => Vec3::new(-0.4, 0.0, 0.3),
                4 => Vec3::new(-0.5, 0.0, 0.0),
                5 => Vec3::new(-0.4, 0.0, -0.25),
                6 => Vec3::new(-0.25, 0.0, -0.5),
                7 => Vec3::new(0.25, 0.0, -0.25),
                _ => unreachable!(),
            };
            
            let pos = player_pos + muzzle_offset;
            
            // Calculate rotation to face direction
            let forward = Vec3::new(move_dir.0.x, 0.0, move_dir.0.y).normalize_or_zero();
            let rotation = if forward != Vec3::ZERO {
                Quat::from_rotation_arc(Vec3::X, forward)
            } else {
                Quat::IDENTITY
            };
            
            commands
                .spawn((
                    Bullet,
                    *move_dir,
                    Mesh3d(meshes.add(Capsule3d::new(BULLET_RADIUS, 0.3))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 1.0, 0.0),
                        emissive: LinearRgba::new(1.0, 1.0, 0.0, 1.0),
                        ..default()
                    })),
                    Transform::from_translation(pos).with_rotation(rotation),
                    Visibility::default(),
                ))
                .add_rollback();
            bullet_ready.0 = false;
        }
    }
}

fn move_bullet(mut bullets: Query<(&mut Transform, &MoveDir), With<Bullet>>, time: Res<Time>) {
    for (mut transform, dir) in &mut bullets {
        let speed = 20.;
        let delta = Vec3::new(dir.0.x, 0.0, dir.0.y) * speed * time.delta_secs();
        transform.translation += delta;
    }
}

fn bullet_wall_collisions(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    walls: Query<&Transform, (With<Wall>, Without<Bullet>)>,
) {
    let map_limit = MAP_SIZE as f32 / 2.;

    for (bullet_entity, bullet_transform) in &bullets {
        let bullet_pos = bullet_transform.translation;

        if bullet_pos.x.abs() > map_limit || bullet_pos.z.abs() > map_limit {
            commands.entity(bullet_entity).despawn();
            continue;
        }

        for wall_transform in &walls {
            let wall_scale = wall_transform.scale;
            let wall_size = Vec3::new(wall_scale.x, WALL_HEIGHT, wall_scale.z);
            let wall_pos = wall_transform.translation;
            
            let wall_pos_xz = Vec2::new(wall_pos.x, wall_pos.z);
            let bullet_pos_xz = Vec2::new(bullet_pos.x, bullet_pos.z);
            let wall_size_xz = Vec2::new(wall_size.x, wall_size.z);
            
            let center_to_center = wall_pos_xz - bullet_pos_xz;
            let center_to_center = center_to_center.abs();
            let corner_to_center = center_to_center - wall_size_xz / 2.;
            
            if corner_to_center.x < 0. && corner_to_center.y < 0. {
                commands.entity(bullet_entity).despawn();
                break;
            }
        }
    }
}

const PLAYER_HEIGHT: f32 = 1.0;
const PLAYER_RADIUS: f32 = 0.3;
const BULLET_RADIUS: f32 = 0.05;

fn kill_players(
    mut commands: Commands,
    players: Query<(Entity, &Transform, &Player), Without<Bullet>>,
    bullets: Query<&Transform, With<Bullet>>,
    mut next_state: ResMut<NextState<RollbackState>>,
    mut scores: ResMut<Scores>,
) {
    for (player_entity, player_transform, player) in &players {
        for bullet_transform in &bullets {
            let player_pos = player_transform.translation;
            let bullet_pos = bullet_transform.translation;

            let distance = player_pos.distance(bullet_pos);

            if distance < PLAYER_RADIUS + BULLET_RADIUS {
                commands.entity(player_entity).despawn();
                next_state.set(RollbackState::RoundEnd);

                if player.handle == 0 {
                    scores.1 += 1;
                } else {
                    scores.0 += 1;
                }
                info!("player died: {scores:?}")
            }
        }
    }
}

fn camera_follow(
    local_players: Res<LocalPlayers>,
    players: Query<(&Player, &Transform)>,
    mut cameras: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    for (player, player_transform) in &players {
        if !local_players.0.contains(&player.handle) {
            continue;
        }

        let pos = player_transform.translation;

        for mut camera_transform in &mut cameras {
            // Follow player from behind and above
            let offset = Vec3::new(0.0, 15.0, 15.0);
            camera_transform.translation = pos + offset;
            camera_transform.look_at(pos, Vec3::Y);
        }
    }
}

fn round_end_timeout(
    mut timer: ResMut<RoundEndTimer>,
    mut state: ResMut<NextState<RollbackState>>,
    time: Res<Time>,
) {
    timer.tick(time.delta());

    if timer.just_finished() {
        state.set(RollbackState::InRound);
    }
}

fn update_score_ui(mut contexts: EguiContexts, scores: Res<Scores>) {
    let Scores(p1_score, p2_score) = *scores;

    egui::Area::new("score".into())
        .anchor(Align2::CENTER_TOP, (0., 25.))
        .show(contexts.ctx_mut().unwrap(), |ui| {
            ui.label(
                RichText::new(format!("{p1_score} - {p2_score}"))
                    .color(Color32::WHITE)
                    .font(FontId::proportional(72.0)),
            );
        });
}

fn checksum_transform(transform: &Transform) -> u64 {
    // Simple checksum for transform
    let pos = transform.translation;
    let rot = transform.rotation;
    ((pos.x as u64) ^ (pos.y as u64) ^ (pos.z as u64) ^ (rot.x as u64) ^ (rot.y as u64) ^ (rot.z as u64) ^ (rot.w as u64)) as u64
}