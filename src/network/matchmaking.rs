// matchmaking.rs - CORRECT VERSION (no dual channels, keep original)
use bevy::prelude::*;
use bevy_ggrs::{ggrs::{DesyncDetection, PlayerType}, *};
use bevy_matchbox::prelude::*;
use rand::{rng, Rng};
use crate::{core::args::Args, Config, core::states::GameState, core::resources::SessionSeed};

pub fn synctest_mode(args: Res<Args>) -> bool {
    args.synctest
}

pub fn p2p_mode(args: Res<Args>) -> bool {
    !args.synctest
}

pub fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "ws://127.0.0.1:3536/extreme_bevy?next=2";
    info!("connecting to matchbox server: {room_url}");
    commands.insert_resource(MatchboxSocket::new_unreliable(room_url));
}

pub fn wait_for_players(
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

pub fn start_synctest_session(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    info!("Starting synctest session");
    let num_players = 2;

    let mut session_builder = ggrs::SessionBuilder::<Config>::new()
        .with_num_players(num_players);

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