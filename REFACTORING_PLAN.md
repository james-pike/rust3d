# Project Refactoring Plan - Diablo 2 Resurrected Style

## Current Issues
- 29+ files in src/ root - hard to navigate
- No clear separation of concerns
- Mixed responsibilities (UI, gameplay, networking)
- Difficult to find related code

## Proposed Directory Structure

```
src/
├── main.rs
├── lib.rs
│
├── core/                    # Core engine/game systems
│   ├── mod.rs
│   ├── states.rs           # Game state machine
│   ├── constants.rs        # Global constants
│   ├── args.rs             # CLI arguments
│   └── resources.rs        # Game resources
│
├── game/                    # Gameplay mechanics
│   ├── mod.rs
│   ├── player.rs           # Player movement, spawning
│   ├── combat.rs           # Combat system
│   ├── input.rs            # Input handling
│   ├── camera.rs           # Camera system
│   └── round.rs            # Round management
│
├── world/                   # Environment systems
│   ├── mod.rs
│   ├── map.rs              # Map generation
│   └── collisions.rs       # Collision detection
│
├── entities/                # Game entities & components
│   ├── mod.rs
│   ├── components.rs       # ECS components
│   └── bullet.rs           # Projectile system
│
├── ui/                      # User Interface
│   ├── mod.rs
│   ├── hud.rs              # Health/Energy orbs (from hud_system.rs)
│   ├── inventory.rs        # Inventory drawer (from inventory_system.rs)
│   ├── lobby.rs            # Lobby interface
│   ├── chat/               # Chat subsystem
│   │   ├── mod.rs
│   │   ├── input.rs        # Chat input handling
│   │   └── ui.rs           # Chat UI rendering
│   ├── auth/               # Authentication UI
│   │   ├── mod.rs
│   │   ├── system.rs       # Auth logic
│   │   └── ui.rs           # Auth UI
│   └── score.rs            # Score display (from ui.rs)
│
├── network/                 # Networking layer
│   ├── mod.rs
│   ├── matchmaking.rs      # P2P matchmaking
│   ├── session.rs          # GGRS session (from networking.rs)
│   └── chat.rs             # Chat networking
│
├── systems/                 # Visual effects & systems
│   ├── mod.rs
│   └── aura_effects.rs     # Aura shader effects
│
├── materials/               # Render materials
│   ├── mod.rs
│   └── aura.rs             # Aura material
│
└── utils/                   # Utilities & helpers
    ├── mod.rs
    ├── checksum.rs         # Rollback checksums
    └── setup.rs            # Initial setup

```

## Diablo 2 Resurrected Inspired Improvements

### 1. **Enhanced Inventory System**
- Grid-based inventory (D2R style)
- Item stacking
- Item rarity colors (white/magic/rare/unique)
- Hover tooltips with stats
- Right-click to equip/unequip

### 2. **Improved HUD**
- Corner minimap
- Skill hotbar at bottom
- Potion belt (quick slots)
- Experience bar with level-up effects
- Character stats panel

### 3. **Better Combat Feel**
- Hit reactions and stun
- Damage numbers floating above enemies
- Critical hit effects
- Death animations
- Corpse system

### 4. **Skill System**
- Skill tree UI
- Active skills on hotbar (1-8 keys)
- Cooldown visualizations
- Mana/resource costs
- Skill synergies

### 5. **Loot System**
- Item drops from enemies
- Rarity-based colors
- Auto-pickup for gold/potions
- Item comparison tooltips
- Stash system

### 6. **Audio & Polish**
- Hit sounds
- Ability sound effects
- UI click sounds
- Ambient music
- Level-up sound

### 7. **Multiplayer Lobby**
- Show all players in lobby
- Ready status for each player
- Kick/invite controls
- Game difficulty selection
- Character preview for all players

## Implementation Priority

### Phase 1: Code Organization (Now)
1. Create directory structure
2. Move files to new locations
3. Update all imports and mod.rs files
4. Verify compilation

### Phase 2: Core Improvements
1. Enhanced inventory with grid system
2. Skill hotbar
3. Improved combat feedback
4. Damage numbers

### Phase 3: Advanced Features
1. Skill system
2. Loot system
3. Audio integration
4. Advanced multiplayer features

## Migration Notes

- Keep backwards compatibility during refactoring
- Test after each major move
- Update documentation as we go
- Preserve git history where possible
