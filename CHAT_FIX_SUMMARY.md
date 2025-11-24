# Chat System Fix - Working Across All States

## Problem

The chat system was not working in Lobby or after games (GameEnd state) because:

1. **Chat socket was initialized too late** - `setup_chat_socket` was only called when entering Matchmaking state
2. **Chat UI was not enabled in GameEnd state** - UI systems had state guards that excluded GameEnd
3. **Duplicate chat UI creation** - Chat UI was recreated every time player returned to Lobby, causing duplicates

## Solution

### 1. **Move Chat Socket Initialization to Lobby** ✓

**Changed in:** `src/lib.rs:167`

**Before:**
```rust
// Matchmaking entry
.add_systems(
    OnEnter(GameState::Matchmaking),
    (
        utils::setup::setup,
        network::matchmaking::start_matchbox_socket.run_if(network::matchmaking::p2p_mode),
        setup_chat_socket.run_if(network::matchmaking::p2p_mode),  // ❌ Too late!
    ),
)
```

**After:**
```rust
// Lobby entry
.add_systems(
    OnEnter(core::states::GameState::Lobby),
    (
        ui::lobby::setup_lobby_resources,
        ui::lobby::spawn_lobby_knight,
        ui::lobby::spawn_lobby_camera,
        ui::lobby::spawn_lobby_lighting,
        ui::inventory::setup_inventory_system,
        ui::hud::setup_player_vitals,
        reset_game_stats,
        force_leaderboard_refresh,
        setup_chat_socket.run_if(network::matchmaking::p2p_mode),  // ✅ Now available in Lobby!
    ),
)
```

**Result:** Players can now chat immediately when they enter the lobby, before starting matchmaking.

---

### 2. **Enable Chat UI in GameEnd State** ✓

**Changed in:** `src/ui/chat/ui.rs:22-34`

**Before:**
```rust
update_chat_ui
    .run_if(
        in_state(GameState::Lobby)
            .or(in_state(GameState::Matchmaking))
            .or(in_state(GameState::InGame))
    ),  // ❌ GameEnd excluded!
```

**After:**
```rust
update_chat_ui
    .run_if(
        in_state(GameState::Lobby)
            .or(in_state(GameState::Matchmaking))
            .or(in_state(GameState::InGame))
            .or(in_state(GameState::GameEnd))  // ✅ Now included!
    ),
```

**Result:** Players can chat during the 20-second GameEnd screen, allowing them to say "gg" or discuss the match results.

---

### 3. **Prevent Duplicate Chat UI Creation** ✓

**Changed in:** `src/ui/chat/ui.rs:55-63`

**Before:**
```rust
fn setup_chat_ui(mut commands: Commands) {
    info!("Setting up chat UI");
    // Always creates new chat UI ❌
    commands.spawn((
        // ... chat UI nodes
    ));
}
```

**After:**
```rust
fn setup_chat_ui(
    mut commands: Commands,
    existing_chat: Query<Entity, With<ChatInputContainer>>,
) {
    // Only create chat UI if it doesn't already exist
    if !existing_chat.is_empty() {
        info!("Chat UI already exists, skipping setup");
        return;  // ✅ Prevents duplicates
    }

    info!("Setting up chat UI");
    commands.spawn((
        // ... chat UI nodes
    ));
}
```

**Result:** Chat UI is created once on first Lobby entry and persists across all states. No duplicate UI elements.

---

## How Chat Now Works

### State Flow

```
WalletAuth
    ↓
AssetLoading
    ↓
Lobby (Chat socket initialized ✅, Chat UI created ✅)
    ↓ [Players can chat here!]
    ↓
Matchmaking (Chat continues working ✅)
    ↓ [Players can chat while waiting!]
    ↓
InGame (Chat continues working ✅)
    ↓ [Players can chat during game!]
    ↓
GameEnd (Chat continues working ✅)
    ↓ [Players can chat during results screen!]
    ↓
Back to Lobby (Chat persists ✅)
    ↓ [Chat UI reused, no duplicates!]
```

### Chat Features

**How to Use:**
1. **Click the chat input box** OR **Press Enter** to focus
2. **Type your message** (text appears as `> your text`)
3. **Press Enter** to send
4. **Press Escape** OR **Click outside** to close

**Status Indicator:**
- Green box in top-left corner shows chat status
- "Chat: Click to type" - not focused
- "Chat: ACTIVE (ESC or click away to close)" - focused

**Message Display:**
- Shows last 8 messages
- Format: `PlayerName: message text`
- Messages received from other players appear in real-time
- Messages persist across state transitions

---

## Technical Details

### Chat Socket

**Connection:**
- Uses separate WebSocket channel from game GGRS session
- URL: `ws://127.0.0.1:3536/extreme_bevy_chat?next=2`
- Reliable channel (guaranteed delivery)

**Initialized:** OnEnter(Lobby) - early enough for pre-game chat

**Persists:** Throughout Lobby → Matchmaking → InGame → GameEnd → Lobby cycle

### Chat UI

**Created:** Once on first OnEnter(Lobby)

**Persists:** Across all states (never despawned)

**Active States:** Lobby, Matchmaking, InGame, GameEnd

**Inactive States:** WalletAuth, AssetLoading (no socket yet)

### Chat Network

**Plugin:** `ChatPlugin` in `src/ui/chat/network.rs`

**Systems run in:** `Update` schedule (no state guards - always active)

**Systems:**
- `handle_chat_input` - Keyboard input handling
- `send_chat_messages` - Send on Enter press
- `receive_chat_messages` - Receive from peers

---

## Testing

### Desktop Mode (cargo run)

```bash
cargo run
```

**Expected behavior:**
1. Enter Lobby
2. **Click chat box or press Enter**
3. **Type a message** (appears as `> your message`)
4. **Press Enter** to send
5. Message appears in chat history
6. Start matchmaking → Chat still works
7. In game → Chat still works
8. Game ends → **Chat works during 20s results screen**
9. Return to lobby → Chat still works, no duplicates

### P2P Mode (two instances)

**Terminal 1:**
```bash
cargo run
```

**Terminal 2:**
```bash
cargo run
```

**Expected behavior:**
1. Both players enter lobby
2. **Player 1 sends message in lobby** → Player 2 sees it
3. Both start matchmaking
4. **Player 2 sends message while waiting** → Player 1 sees it
5. Game starts
6. **Players chat during game** → Both see messages
7. Game ends at 10 kills
8. **Players say "gg" during results screen** → Both see messages
9. Both return to lobby
10. **Chat continues working** with message history intact

---

## File Changes Summary

### Modified Files

1. **`src/lib.rs`**
   - Moved `setup_chat_socket` from Matchmaking entry to Lobby entry
   - Removed duplicate call from Matchmaking

2. **`src/ui/chat/ui.rs`**
   - Added GameEnd state to chat UI run conditions
   - Added duplicate prevention check in `setup_chat_ui()`

---

## Configuration

### Chat Socket URL

Change in `src/ui/chat/network.rs:62`:
```rust
let chat_room_url = "ws://127.0.0.1:3536/extreme_bevy_chat?next=2";
```

### Message History Limit

Change in `src/ui/chat/ui.rs:190`:
```rust
let start = chat_messages.messages.len().saturating_sub(8);  // Show last 8 messages
```

### Chat Box Size

Change in `src/ui/chat/ui.rs:99-100`:
```rust
width: Val::Px(400.0),   // Chat box width
height: Val::Px(200.0),  // Chat box height
```

---

## Troubleshooting

### Chat not appearing
- **Check:** Is chat UI being created? Look for "Setting up chat UI" log
- **Fix:** Verify you've entered Lobby state

### Can't type in chat
- **Check:** Did you click the chat box or press Enter?
- **Check:** Is status indicator showing "ACTIVE"?
- **Fix:** Click directly on the gray chat input box

### Messages not sending
- **Check:** Is chat socket connected? Look for "Sending chat message" log
- **Check:** Are you in P2P mode (not synctest)?
- **Fix:** Make sure matchbox server is running on port 3536

### Messages not received
- **Check:** Is peer connected? Look for "Received chat from" log
- **Check:** Are both players using same chat room URL?
- **Fix:** Verify both players are in P2P mode and connected

### Duplicate chat boxes
- **Should not happen anymore!**
- Fixed by duplicate prevention check
- If still occurs: Check for multiple `ChatInputContainer` entities

---

## Success! ✅

Your chat system now works:
- ✅ **Before games** (in Lobby)
- ✅ **During matchmaking**
- ✅ **During games**
- ✅ **After games** (in GameEnd screen)
- ✅ **No duplicate UI elements**
- ✅ **Message history persists** across states

Players can now communicate throughout the entire game session!
