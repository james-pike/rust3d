// matchmaking.rs - CORRECT VERSION (no dual channels, keep original)
use bevy::prelude::*;
use bevy_ggrs::{ggrs::{DesyncDetection, PlayerType}, *};
use bevy_matchbox::prelude::*;
use rand::{rng, Rng};
use crate::{
    core::args::Args,
    Config,
    core::states::GameState,
    core::resources::{SessionSeed, PlayerAddressMapping},
    ui::auth::system::WalletInfo,
    ui::lobby::PlayerProfile,
};

pub fn synctest_mode(args: Res<Args>) -> bool {
    args.synctest
}

pub fn p2p_mode(args: Res<Args>) -> bool {
    !args.synctest
}

pub fn start_matchbox_socket(mut commands: Commands) {
    // Use public matchbox demo server for P2P matchmaking
    // For production, you should deploy your own matchbox server at match.nft.cx
    let room_url = "wss://match.helsing.studio/dagknights?next=2";
    info!("connecting to matchbox server: {room_url}");
    commands.insert_resource(MatchboxSocket::new_unreliable(room_url));
}

pub fn wait_for_players(
    mut commands: Commands,
    mut socket: ResMut<MatchboxSocket>,
    mut next_state: ResMut<NextState<GameState>>,
    args: Res<Args>,
    wallet_info: Res<WalletInfo>,
    profile: Res<PlayerProfile>,
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

    // Determine local player handle by finding our peer ID in the players list
    let local_peer_id = socket.id().expect("no peer id assigned");
    let mut local_handle = None;

    for (i, player) in players.iter().enumerate() {
        if let PlayerType::Remote(peer_id) = player {
            if *peer_id == local_peer_id {
                local_handle = Some(i);
                break;
            }
        } else if let PlayerType::Local = player {
            local_handle = Some(i);
            break;
        }
    }

    // Create player address mapping with local player's info
    let mut address_mapping = PlayerAddressMapping::default();
    if let Some(handle) = local_handle {
        address_mapping.local_player_handle = Some(handle);
        let local_address = if wallet_info.connected {
            Some(wallet_info.address.clone())
        } else {
            Some(format!("guest_{}", local_peer_id.0.as_u128()))
        };

        // Set local player's address and display name
        if handle == 0 {
            address_mapping.player0_address = local_address;
            address_mapping.player0_display_name = Some(profile.display_name.clone());
        } else {
            address_mapping.player1_address = local_address;
            address_mapping.player1_display_name = Some(profile.display_name.clone());
        }

        info!("Local player assigned handle {} with address {:?} and name '{}'",
              handle,
              address_mapping.get_local_address(),
              profile.display_name);
    }

    // TODO: Exchange opponent's Kaspa address and display name via custom message protocol
    // For now, opponent address and name will be unknown
    commands.insert_resource(address_mapping);

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
    wallet_info: Res<WalletInfo>,
    profile: Res<PlayerProfile>,
) {
    info!("Starting synctest session");
    let num_players = 2;

    // In synctest mode, create mock player addresses and display names
    let mut address_mapping = PlayerAddressMapping::default();
    address_mapping.local_player_handle = Some(0);
    address_mapping.player0_address = Some(
        if wallet_info.connected {
            wallet_info.address.clone()
        } else {
            "synctest_player0".to_string()
        }
    );
    address_mapping.player1_address = Some("synctest_player1".to_string());
    address_mapping.player0_display_name = Some(profile.display_name.clone());
    address_mapping.player1_display_name = Some("Test Opponent".to_string());
    commands.insert_resource(address_mapping);

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