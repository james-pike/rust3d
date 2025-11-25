// leaderboard.rs - Stats submission and game completion
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{
    core::resources::{Scores, SessionSeed, PlayerAddressMapping},
    core::states::GameState,
};

/// Win condition: first player to reach this score wins
pub const WINNING_SCORE: u32 = 3;

/// Resource to track if stats have been submitted for this game
#[derive(Resource, Default)]
pub struct StatsSubmitted(pub bool);

/// Resource to track the game end timer
#[derive(Resource)]
pub struct GameEndTimer {
    pub timer: Timer,
}

impl Default for GameEndTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(20.0, TimerMode::Once),
        }
    }
}

/// Resource to store game end data for display
#[derive(Resource, Default, Clone, Debug)]
pub struct GameEndData {
    pub player0_score: u32,
    pub player1_score: u32,
    pub winner_handle: Option<usize>,
    pub local_player_handle: Option<usize>,
}

/// Check if the game has ended (someone reached winning score)
/// If so, transition to GameEnd state to show results
pub fn check_game_end(
    scores: Res<Scores>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if scores.0 >= WINNING_SCORE || scores.1 >= WINNING_SCORE {
        info!("Game ended! Final scores: {:?}. Transitioning to GameEnd state.", *scores);
        next_state.set(GameState::GameEnd);
    }
}

/// Setup game end data when entering GameEnd state
pub fn setup_game_end(
    mut commands: Commands,
    scores: Res<Scores>,
    address_mapping: Res<PlayerAddressMapping>,
) {
    let winner_handle = if scores.0 >= WINNING_SCORE {
        Some(0)
    } else if scores.1 >= WINNING_SCORE {
        Some(1)
    } else {
        None
    };

    let game_end_data = GameEndData {
        player0_score: scores.0,
        player1_score: scores.1,
        winner_handle,
        local_player_handle: address_mapping.local_player_handle,
    };

    info!("Game ended - P0: {} | P1: {} | Winner: {:?}",
          scores.0, scores.1, winner_handle);

    commands.insert_resource(game_end_data);
    commands.insert_resource(GameEndTimer::default());
}

/// Timer to automatically return to lobby after showing results
pub fn game_end_timer(
    mut timer: ResMut<GameEndTimer>,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
) {
    timer.timer.tick(time.delta());

    if timer.timer.just_finished() {
        info!("GameEnd timer finished, returning to lobby");
        next_state.set(GameState::Lobby);
    }
}

/// Submit stats to the backend API when entering GameEnd state
#[cfg(target_arch = "wasm32")]
pub fn submit_stats_on_game_end(
    scores: Res<Scores>,
    session_seed: Res<SessionSeed>,
    address_mapping: Res<PlayerAddressMapping>,
    mut stats_submitted: ResMut<StatsSubmitted>,
) {
    use wasm_bindgen_futures::spawn_local;

    // Only submit once
    if stats_submitted.0 {
        return;
    }

    let player0_addr = address_mapping.player0_address.clone();
    let player1_addr = address_mapping.player1_address.clone();

    // Only submit if we have at least one known address
    if player0_addr.is_none() && player1_addr.is_none() {
        warn!("Cannot submit stats: no player addresses available");
        return;
    }

    // Prevent duplicate submissions when both players have the same address (testing scenario)
    if player0_addr.is_some() && player1_addr.is_some() && player0_addr == player1_addr {
        warn!("Both players have the same address - skipping stats submission to prevent duplicates");
        return;
    }

    let player0_name = address_mapping.player0_display_name.clone();
    let player1_name = address_mapping.player1_display_name.clone();

    let payload = MatchResult {
        player1_address: player0_addr.unwrap_or_else(|| "unknown".to_string()),
        player2_address: player1_addr.unwrap_or_else(|| "unknown".to_string()),
        player1_display_name: player0_name,
        player2_display_name: player1_name,
        player1_score: scores.0,
        player2_score: scores.1,
        session_seed: format!("{}", session_seed.0),
    };

    info!("Submitting stats to API: {:?}", payload);

    stats_submitted.0 = true;

    spawn_local(async move {
        if let Err(e) = submit_stats_async(payload).await {
            error!("Failed to submit stats: {}", e);
        } else {
            info!("Stats submitted successfully!");
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn submit_stats_on_game_end(
    scores: Res<Scores>,
    address_mapping: Res<PlayerAddressMapping>,
    mut stats_submitted: ResMut<StatsSubmitted>,
) {
    if stats_submitted.0 {
        return;
    }

    stats_submitted.0 = true;

    info!("Stats submission skipped (not running in browser)");
    info!("Game ended with scores: {:?}", *scores);
    info!("Player addresses: {:?}", *address_mapping);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MatchResult {
    player1_address: String,
    player2_address: String,
    player1_display_name: Option<String>,
    player2_display_name: Option<String>,
    player1_score: u32,
    player2_score: u32,
    session_seed: String,
}

#[cfg(target_arch = "wasm32")]
async fn submit_stats_async(payload: MatchResult) -> Result<(), String> {
    use wasm_bindgen::{JsValue, JsCast};
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, RequestMode, Response, Headers};

    let api_url = "https://dk-leaderboard-api.dagknights.workers.dev/api/stats";

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    let headers = Headers::new().map_err(|e| format!("Headers error: {:?}", e))?;
    headers.set("Content-Type", "application/json")
        .map_err(|e| format!("Header set error: {:?}", e))?;
    opts.headers(&headers);

    let json_str = serde_json::to_string(&payload)
        .map_err(|e| format!("Serialization error: {}", e))?;
    opts.body(Some(&JsValue::from_str(&json_str)));

    let request = Request::new_with_str_and_init(api_url, &opts)
        .map_err(|e| format!("Request creation error: {:?}", e))?;

    let window = web_sys::window().ok_or("No window object")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;

    let resp: Response = resp_value.dyn_into()
        .map_err(|_| "Response cast error")?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("HTTP error: {}", resp.status()))
    }
}
