# Complete Game Flow with Leaderboard

## Overview

The game now has a complete flow from lobby → matchmaking → in-game → game end screen → back to lobby with updated leaderboard stats.

---

## State Machine

```
WalletAuth → AssetLoading → Lobby ⇄ Matchmaking → InGame → GameEnd → Lobby
                              ↑                                         ↓
                              └─────────────────────────────────────────┘
```

### State Descriptions

1. **WalletAuth**: Player connects their Kasware wallet
2. **AssetLoading**: Load game assets (models, textures)
3. **Lobby**: Player customizes gear, sees leaderboard, starts matchmaking
4. **Matchmaking**: Waiting for opponent via P2P connection
5. **InGame**: Active gameplay, tracking kills
6. **GameEnd**: Shows match results for 5 seconds, submits stats to API
7. **Lobby** (return): Leaderboard auto-refreshes with new stats

---

## Complete Game Flow

### 1. Lobby Phase
**Location:** `src/ui/lobby.rs`

**What happens:**
- 3D knight preview with customizable gear
- Player profile (display name, ready status)
- **Leaderboard panel** showing top 10 players by K/D ratio
- Leaderboard auto-fetches every 30 seconds
- "Start Matchmaking" button

**UI Layout:**
- Top Center: Main lobby panel (name input, ready button, matchmaking button)
- Top Right: Player info panel
- **Right Side (below player info): Leaderboard panel** ← NEW!
- Bottom Left: Lobby notifications

---

### 2. Matchmaking Phase
**Location:** `src/network/matchmaking.rs`

**What happens:**
- Connects to matchbox P2P server
- Waits for 2 players to join
- **Maps player handles (0, 1) to Kaspa wallet addresses**
- Generates session seed from peer IDs
- Creates GGRS rollback session
- Transitions to **InGame**

**Player Address Mapping:**
```rust
PlayerAddressMapping {
    local_player_handle: Some(0 or 1),
    player0_address: Some("kaspa:qz123..."),
    player1_address: Some("kaspa:qz456..."),
}
```

---

### 3. In-Game Phase
**Location:** `src/game/`, `src/world/collisions.rs`

**What happens:**
- Active 2-player combat
- Scores tracked: `Scores(p1_kills, p2_kills)`
- **Every frame: checks if any player reached 10 kills**
- When score hits 10 → transitions to **GameEnd** state

**Win Condition:**
```rust
pub const WINNING_SCORE: u32 = 10;  // First to 10 kills wins
```

**Game End Detection:** `src/game/leaderboard.rs:41`
```rust
pub fn check_game_end(scores: Res<Scores>, mut next_state: ResMut<NextState<GameState>>) {
    if scores.0 >= WINNING_SCORE || scores.1 >= WINNING_SCORE {
        info!("Game ended! Transitioning to GameEnd state.");
        next_state.set(GameState::GameEnd);
    }
}
```

---

### 4. Game End Phase ← NEW!
**Location:** `src/ui/game_end.rs`

**What happens:**

#### On Enter (OnEnter(GameState::GameEnd)):
1. **Cleanup game entities** (`cleanup_game_entities`):
   - Despawns all entities marked with `GameEntity` component
   - Removes: players, walls, ground plane, game lighting
   - Camera persists but will be repositioned in lobby

2. **Setup game end data** (`setup_game_end`):
   - Captures final scores
   - Determines winner (player 0 or 1)
   - Stores local player handle
   - Creates 20-second countdown timer

3. **Submit stats to API** (`submit_stats_on_game_end`):
   - Sends POST request to `/api/stats`
   - Payload: `{player1_address, player2_address, player1_score, player2_score, session_seed}`
   - Backend updates both players' stats in database
   - Desktop mode: logs stats to console

#### During GameEnd State:
- **Game End UI** displays:
  - **VICTORY** (gold) or **DEFEAT** (red) banner
  - Winner announcement: "Player 1 Wins!" or "Player 2 Wins!"
  - Final score table showing both players' kills
  - Highlights local player as "(You)"
  - Shows 👑 crown icon next to winner
  - **Auto-countdown**: "Returning to lobby in 4.3s..."
  - **Manual button**: "Return to Lobby Now"

- **5-second timer** counts down (`game_end_timer`):
  - When finished → transitions to **Lobby**

#### UI Example:
```
┌──────────────────────────────────────────────────┐
│                                                  │
│               ⚔ VICTORY ⚔                        │
│             Player 1 Wins!                       │
│                                                  │
│ ─────────────────────────────────────────────── │
│                                                  │
│              FINAL SCORES                        │
│                                                  │
│   PLAYER          KILLS         STATUS           │
│                                                  │
│   Player 1 (You)    10       👑 WINNER           │
│   Player 2           7                           │
│                                                  │
│ ─────────────────────────────────────────────── │
│                                                  │
│      Returning to lobby in 3.2s                  │
│                                                  │
│      [Return to Lobby Now]                       │
│                                                  │
└──────────────────────────────────────────────────┘
```

---

### 5. Return to Lobby
**Location:** `src/lib.rs:156-168`

**What happens on OnEnter(GameState::Lobby):**

1. **Reset game stats** (`reset_game_stats`):
   - Scores → (0, 0)
   - StatsSubmitted → false
   - PlayerAddressMapping → cleared
   - GameEndData → cleared

2. **Force leaderboard refresh** (`force_leaderboard_refresh`):
   - Sets `last_fetch = 0.0`
   - Triggers immediate fetch on next frame
   - **Player sees updated stats with their new K/D ratio!**

3. **Respawn lobby entities**:
   - 3D knight preview
   - Camera and lighting
   - UI panels

**Result:** Player is back in lobby with fresh leaderboard showing their improved stats!

---

## Stats Submission Flow

### What Gets Submitted
```json
{
  "player1_address": "kaspa:qz123...",
  "player2_address": "kaspa:qz456...",
  "player1_score": 10,
  "player2_score": 7,
  "session_seed": "12345678901234567890"
}
```

### Backend Processing (Cloudflare Worker)
1. Receives POST `/api/stats`
2. Determines winner (higher score)
3. Creates match history record
4. Updates both players' aggregate stats:
   - `total_kills += score`
   - `total_deaths += opponent_score`
   - `games_played += 1`
   - `wins += 1` (if winner) or `losses += 1` (if loser)
5. Returns success

### Database Updates
```sql
-- Player 1 (winner: 10 kills, 7 deaths)
UPDATE players SET
  total_kills = total_kills + 10,
  total_deaths = total_deaths + 7,
  games_played = games_played + 1,
  wins = wins + 1

-- Player 2 (loser: 7 kills, 10 deaths)
UPDATE players SET
  total_kills = total_kills + 7,
  total_deaths = total_deaths + 10,
  games_played = games_played + 1,
  losses = losses + 1
```

---

## Leaderboard Refresh Behavior

### Auto-Refresh (Every 30 seconds)
**Location:** `src/ui/leaderboard.rs:55`

```rust
pub fn fetch_leaderboard(mut leaderboard: ResMut<LeaderboardData>, time: Res<Time>) {
    let current_time = time.elapsed_secs_f64();

    if current_time - leaderboard.last_fetch > 30.0 {
        // Fetch new data from API
    }
}
```

### Force Refresh (On Lobby Entry)
**Location:** `src/lib.rs:80`

```rust
fn force_leaderboard_refresh(mut leaderboard: ResMut<LeaderboardData>) {
    leaderboard.force_refresh();  // Sets last_fetch = 0.0
}
```

**Result:** When player returns from GameEnd → Lobby, leaderboard immediately fetches new data showing their updated stats!

---

## File Changes Summary

### New Files
- `src/ui/game_end.rs` - Game end UI screen
- `src/game/cleanup.rs` - Game entity cleanup system
- `GAME_FLOW_SUMMARY.md` - This document

### Modified Files
- `src/core/states.rs` - Added `GameEnd` state
- `src/entities/components.rs` - Added `GameEntity` marker component
- `src/game/leaderboard.rs` - Added:
  - `GameEndData` resource
  - `GameEndTimer` resource
  - `setup_game_end()` system
  - `game_end_timer()` system
  - `submit_stats_on_game_end()` system
  - Updated `check_game_end()` to transition to GameEnd
- `src/ui/mod.rs` - Export `game_end` module
- `src/ui/leaderboard.rs` - Added `force_refresh()` method
- `src/game/mod.rs` - Export `cleanup` module
- `src/game/player.rs` - Mark players with `GameEntity`
- `src/world/map.rs` - Mark walls with `GameEntity`
- `src/utils/setup.rs` - Mark ground plane and light with `GameEntity`
- `src/lib.rs` - Added:
  - GameEnd state systems
  - `cleanup_game_entities()` on GameEnd entry
  - `force_leaderboard_refresh()` on lobby entry
  - `reset_game_stats()` updated to clear GameEndData

---

## Testing the Flow

### Desktop Mode (cargo run)
```bash
cargo run
```

**Expected behavior:**
1. Wallet auth screen → Skip or connect mock wallet
2. Lobby with mock leaderboard (3 test players)
3. Start Matchmaking → Synctest mode (2 local players)
4. In-game → Score 10 kills (control both players with WASD + Space)
5. **Game End screen appears:**
   - Game map entities cleaned up
   - Shows "VICTORY" or "DEFEAT"
   - Displays final scores
   - 20-second countdown
6. **Auto-returns to lobby:**
   - Lobby scene fully restored (knight preview, lobby camera)
   - Mock leaderboard refreshed
   - Stats logged to console

### WASM Mode (trunk build)
```bash
trunk build --release
trunk serve
```

**Expected behavior:**
1. Connect Kasware wallet
2. Lobby with **real leaderboard** from API
3. Start Matchmaking → Wait for opponent
4. In-game → First to 10 kills wins
5. **Game End screen:**
   - Game map cleaned up
   - Shows results
   - **Stats submitted to API** (check console)
   - 20-second countdown
6. **Returns to lobby:**
   - **Lobby scene fully restored**
   - **Leaderboard refreshes automatically**
   - **Your new K/D ratio visible!**

---

## Configuration Options

### Win Score
Change in `src/game/leaderboard.rs:10`:
```rust
pub const WINNING_SCORE: u32 = 10;  // Change to 5, 15, 20, etc.
```

### Game End Timer Duration
Change in `src/game/leaderboard.rs:25`:
```rust
impl Default for GameEndTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(20.0, TimerMode::Once),  // Currently 20 seconds
        }
    }
}
```

### Leaderboard Auto-Refresh Interval
Change in `src/ui/leaderboard.rs:51`:
```rust
if current_time - leaderboard.last_fetch > 30.0 {  // Change to 10.0, 60.0, etc.
```

---

## Troubleshooting

### Game doesn't end at 10 kills
- **Check:** `check_game_end()` system is running in InGame state
- **Check:** Console logs: "Game ended! Transitioning to GameEnd state."
- **Fix:** Verify WINNING_SCORE constant is set correctly

### Game End screen doesn't appear
- **Check:** GameEnd state is in state machine
- **Check:** `render_game_end_ui()` is in EguiPrimaryContextPass
- **Fix:** Look for state transition logs

### Stats not submitting
- **Check:** Browser console for fetch errors
- **Check:** API URL is correct in `leaderboard.rs:120`
- **Check:** CORS headers in Cloudflare Worker
- **Fix:** Verify Turso database is accessible

### Leaderboard doesn't update after game
- **Check:** `force_leaderboard_refresh()` is called on lobby entry
- **Check:** Console logs: "Forcing leaderboard refresh on lobby entry"
- **Check:** API returns updated stats (may take a few seconds)
- **Fix:** Manually refresh by leaving and re-entering lobby

### Timer doesn't count down
- **Check:** `game_end_timer()` system is running in GameEnd state
- **Check:** `GameEndTimer` resource is initialized
- **Fix:** Verify timer is being ticked with `time.delta()`

---

## Next Steps

### Recommended Improvements
1. **Opponent Address Exchange**: Implement P2P protocol to share Kaspa addresses
2. **Better Win Condition**: Add "best of 5 rounds" or "match point" system
3. **Sound Effects**: Victory/defeat sounds on GameEnd screen
4. **Animations**: Fade transitions between states
5. **Player Highlights**: Show which kills were yours vs opponent's
6. **Match Replays**: Save session seed for replay functionality

### Optional Enhancements
- Show both players' K/D ratios on GameEnd screen
- Add "Rematch" button (search for same opponent)
- Show rank change: "You moved up 3 places!"
- Add victory poses/emotes for winner
- Implement surrender/forfeit option

---

## Success! ✅

Your game now has a complete flow:
- ✅ Lobby with leaderboard
- ✅ Matchmaking with address mapping
- ✅ In-game with win condition (10 kills)
- ✅ **Game End screen showing results**
- ✅ **Automatic stats submission to API**
- ✅ **5-second countdown timer**
- ✅ **Return to lobby with refreshed leaderboard**

Players can now see their K/D ratio improve after each game!
