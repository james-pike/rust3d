# Step-by-Step Refactoring Guide

## Safety First!

Before starting, create a git commit:
```bash
git add -A
git commit -m "Pre-refactoring checkpoint"
```

## Phase 1: Move Files (Do this manually or with script)

### Step 1: Core Systems
```bash
cd /home/james/dk/src
mv states.rs core/
mv constants.rs core/
mv args.rs core/
mv resources.rs core/
```

### Step 2: Game Systems
```bash
mv player.rs game/
mv combat.rs game/
mv input.rs game/
mv camera.rs game/
mv round.rs game/
```

### Step 3: World Systems
```bash
mv map.rs world/
mv collisions.rs world/
```

### Step 4: Entities
```bash
mv components.rs entities/
mv bullet.rs entities/
```

### Step 5: UI Systems
```bash
mv hud_system.rs ui/hud.rs
mv inventory_system.rs ui/inventory.rs
mv lobby.rs ui/
mv ui.rs ui/score.rs

# Chat subsystem
mv chat.rs ui/chat/network.rs
mv chat_ui.rs ui/chat/ui.rs

# Auth subsystem
mv auth.rs ui/auth/system.rs
mv auth_ui.rs ui/auth/ui.rs
```

### Step 6: Network Systems
```bash
mv matchmaking.rs network/
mv networking.rs network/session.rs
```

### Step 7: Utilities
```bash
mv checksum.rs utils/
mv setup.rs utils/
mv utils.rs utils/helpers.rs
```

## Phase 2: Create mod.rs Files

### core/mod.rs
```rust
pub mod states;
pub mod constants;
pub mod args;
pub mod resources;

pub use states::*;
pub use constants::*;
pub use resources::*;
```

### game/mod.rs
```rust
pub mod player;
pub mod combat;
pub mod input;
pub mod camera;
pub mod round;

pub use player::*;
pub use input::*;
```

### world/mod.rs
```rust
pub mod map;
pub mod collisions;

pub use map::*;
pub use collisions::*;
```

### entities/mod.rs
```rust
pub mod components;
pub mod bullet;

pub use components::*;
pub use bullet::*;
```

### ui/mod.rs
```rust
pub mod hud;
pub mod inventory;
pub mod lobby;
pub mod score;
pub mod chat;
pub mod auth;

pub use hud::*;
pub use inventory::*;
pub use lobby::*;
```

### ui/chat/mod.rs
```rust
pub mod network;
pub mod ui;

pub use network::*;
pub use ui::ChatUIPlugin;
```

### ui/auth/mod.rs
```rust
pub mod system;
pub mod ui;

pub use system::*;
pub use ui::AuthUIPlugin;
```

### network/mod.rs
```rust
pub mod matchmaking;
pub mod session;

pub use matchmaking::*;
pub use session::*;
```

### utils/mod.rs
```rust
pub mod checksum;
pub mod setup;
pub mod helpers;

pub use checksum::*;
pub use setup::*;
pub use helpers::*;
```

## Phase 3: Update lib.rs

Replace the old mod declarations with:

```rust
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

// Visual effects
mod systems;
mod materials;

// Utilities
mod utils;

// Re-exports for convenience
use core::*;
use game::*;
use world::*;
use entities::*;
use network::*;
use utils::*;
```

## Phase 4: Fix Imports

After moving files, you'll need to update imports. Here's a search-replace guide:

### Find and Replace Examples:

1. **States**
   - Find: `use crate::states::`
   - Replace: `use crate::core::states::`

2. **Constants**
   - Find: `use crate::constants::`
   - Replace: `use crate::core::constants::`

3. **Components**
   - Find: `use crate::components::`
   - Replace: `use crate::entities::components::`

4. **Player**
   - Find: `use crate::player::`
   - Replace: `use crate::game::player::`

5. **Lobby**
   - Find: `use crate::lobby::`
   - Replace: `use crate::ui::lobby::`

6. **Chat**
   - Find: `use crate::chat::`
   - Replace: `use crate::ui::chat::`

7. **HUD**
   - Find: `use crate::hud_system::`
   - Replace: `use crate::ui::hud::`

8. **Inventory**
   - Find: `use crate::inventory_system::`
   - Replace: `use crate::ui::inventory::`

## Phase 5: Test Compilation

After each phase, test:
```bash
cargo check
```

If errors occur, use the compiler messages to guide fixes.

## Phase 6: Clean Up

Remove any unused imports:
```bash
cargo clippy --fix --allow-dirty
```

Format code:
```bash
cargo fmt
```

## Rollback if Needed

If things break badly:
```bash
git reset --hard HEAD
```

## Pro Tips

1. **Move one module at a time** - Don't try to move everything at once
2. **Test after each move** - Run `cargo check` frequently
3. **Use IDE refactoring** - If using VS Code with rust-analyzer, it can update imports automatically
4. **Keep a checklist** - Mark off each file as you move it
5. **Commit frequently** - After each successful module move

## Automated Script (Optional)

Create `refactor.sh`:
```bash
#!/bin/bash

# Create directories
mkdir -p src/{core,game,world,entities,ui/{chat,auth},network,utils}

# Move core
mv src/states.rs src/core/
mv src/constants.rs src/core/
mv src/args.rs src/core/
mv src/resources.rs src/core/

# Move game
mv src/player.rs src/game/
mv src/combat.rs src/game/
mv src/input.rs src/game/
mv src/camera.rs src/game/
mv src/round.rs src/game/

# ... etc

# Test
cargo check

echo "Refactoring complete! Check cargo check output for errors."
```

Run with:
```bash
chmod +x refactor.sh
./refactor.sh
```
