# Diablo 2 Resurrected - Inspired Improvements

## Quick Wins - Implement These First

### 1. Skill Hotbar System
**File**: `src/ui/skills.rs`

```rust
// Add to bottom of screen, next to health/energy orbs
// 8 slots for skills (keys 1-8)
// Show skill icon, cooldown overlay, mana cost
// Click or press key to use skill
```

### 2. Damage Numbers
**File**: `src/game/damage_numbers.rs`

```rust
// Floating combat text above enemies
// Color-coded: white (normal), yellow (crit), orange (fire), etc.
// Animate upward and fade out
// Multiple numbers stack vertically
```

### 3. Item Rarity Colors
**Enhancement to**: `src/ui/inventory.rs`

```rust
pub enum ItemRarity {
    Normal,      // White
    Magic,       // Blue
    Rare,        // Yellow
    Unique,      // Gold/Orange
    Set,         // Green
    Runeword,    // Orange/Gold
}

// Apply colors to item names in inventory/tooltips
```

### 4. Grid-Based Inventory
**Enhancement to**: `src/ui/inventory.rs`

```rust
// 10x4 grid instead of list
// Items take up 1x1, 1x2, 2x2, etc. spaces
// Drag and drop to rearrange
// Visual grid highlighting on hover
```

### 5. Mini-map
**File**: `src/ui/minimap.rs`

```rust
// Top right corner
// Shows explored map area
// Player dot, enemy dots
// Clickable to move (optional)
```

### 6. Enhanced HUD Layout

```
┌─────────────────────────────────────────────┐
│  [HP Orb]              MINIMAP    [MP Orb] │
│                                              │
│                   GAME AREA                  │
│                                              │
│  [Exp Bar]  [Skill Hotbar 1-8]  [Belt]     │
└─────────────────────────────────────────────┘
```

## Implementation Guide

### Step 1: Create Damage Numbers System

```rust
// src/game/damage_numbers.rs
use bevy::prelude::*;

#[derive(Component)]
pub struct DamageNumber {
    pub value: f32,
    pub color: Color,
    pub lifetime: f32,
    pub velocity: Vec2,
}

pub fn spawn_damage_number(
    commands: &mut Commands,
    position: Vec3,
    damage: f32,
    is_crit: bool,
) {
    let color = if is_crit {
        Color::srgb(1.0, 1.0, 0.0) // Yellow for crits
    } else {
        Color::WHITE
    };

    commands.spawn((
        Text2d::new(format!("{:.0}", damage)),
        TextFont {
            font_size: if is_crit { 24.0 } else { 18.0 },
            ..default()
        },
        TextColor(color),
        Transform::from_translation(position),
        DamageNumber {
            value: damage,
            color,
            lifetime: 1.5,
            velocity: Vec2::new(0.0, 50.0),
        },
    ));
}

pub fn update_damage_numbers(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut DamageNumber)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut number) in &mut query {
        number.lifetime -= time.delta_secs();

        if number.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            // Float upward
            transform.translation.x += number.velocity.x * time.delta_secs();
            transform.translation.y += number.velocity.y * time.delta_secs();

            // Fade out
            // Update alpha based on lifetime
        }
    }
}
```

### Step 2: Add Skill Hotbar

```rust
// src/ui/skills.rs
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

#[derive(Resource)]
pub struct SkillBar {
    pub slots: [Option<Skill>; 8],
}

pub struct Skill {
    pub name: String,
    pub icon_path: String,
    pub cooldown: f32,
    pub max_cooldown: f32,
    pub mana_cost: f32,
}

pub fn render_skill_bar(
    mut contexts: EguiContexts,
    skill_bar: Res<SkillBar>,
    vitals: Res<PlayerVitals>,
) {
    let ctx = contexts.ctx_mut();
    let screen_rect = ctx.viewport_rect();

    let bar_width = 400.0;
    let bar_x = screen_rect.center().x - bar_width / 2.0;
    let bar_y = screen_rect.max.y - 150.0;

    egui::Window::new("skill_bar")
        .title_bar(false)
        .fixed_pos([bar_x, bar_y])
        .fixed_size([bar_width, 60.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, skill) in skill_bar.slots.iter().enumerate() {
                    render_skill_slot(ui, i, skill, &vitals);
                }
            });
        });
}

fn render_skill_slot(
    ui: &mut egui::Ui,
    index: usize,
    skill: &Option<Skill>,
    vitals: &PlayerVitals,
) {
    let slot_size = egui::vec2(48.0, 48.0);
    let (rect, response) = ui.allocate_exact_size(slot_size, egui::Sense::click());

    // Draw slot background
    ui.painter().rect_filled(
        rect,
        3.0,
        egui::Color32::from_rgb(30, 25, 20),
    );

    if let Some(skill) = skill {
        // Draw skill icon (would load from skill.icon_path)

        // Draw cooldown overlay if on cooldown
        if skill.cooldown > 0.0 {
            let cd_percent = skill.cooldown / skill.max_cooldown;
            let cd_height = rect.height() * cd_percent;
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(rect.min.x, rect.max.y - cd_height),
                    egui::vec2(rect.width(), cd_height),
                ),
                0.0,
                egui::Color32::from_rgba_premultiplied(0, 0, 0, 150),
            );
        }

        // Draw mana cost if not enough mana
        if vitals.energy < skill.mana_cost {
            ui.painter().rect_stroke(
                rect,
                3.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 0, 0)),
                egui::epaint::StrokeKind::Outside,
            );
        }

        // Show tooltip on hover
        if response.hovered() {
            egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new(format!("skill_{}", index)), |ui| {
                ui.label(&skill.name);
                ui.label(format!("Mana: {}", skill.mana_cost));
                ui.label(format!("Cooldown: {:.1}s", skill.max_cooldown));
            });
        }
    }

    // Draw hotkey number
    ui.painter().text(
        egui::pos2(rect.min.x + 4.0, rect.min.y + 4.0),
        egui::Align2::LEFT_TOP,
        (index + 1).to_string(),
        egui::FontId::proportional(12.0),
        egui::Color32::WHITE,
    );

    // Handle click
    if response.clicked() {
        // Activate skill
    }
}
```

### Step 3: Enhanced Item System

```rust
// src/entities/item.rs
use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct Item {
    pub name: String,
    pub rarity: ItemRarity,
    pub item_type: ItemType,
    pub stats: ItemStats,
    pub grid_size: (u32, u32), // Width x Height in inventory grid
}

#[derive(Clone, Copy, PartialEq)]
pub enum ItemRarity {
    Normal,
    Magic,
    Rare,
    Unique,
    Set,
}

impl ItemRarity {
    pub fn color(&self) -> egui::Color32 {
        match self {
            ItemRarity::Normal => egui::Color32::WHITE,
            ItemRarity::Magic => egui::Color32::from_rgb(100, 100, 255),
            ItemRarity::Rare => egui::Color32::from_rgb(255, 255, 100),
            ItemRarity::Unique => egui::Color32::from_rgb(200, 150, 50),
            ItemRarity::Set => egui::Color32::from_rgb(100, 255, 100),
        }
    }
}

#[derive(Clone)]
pub enum ItemType {
    Weapon,
    Armor,
    Helmet,
    Boots,
    Gloves,
    Ring,
    Amulet,
    Potion,
}

#[derive(Clone)]
pub struct ItemStats {
    pub damage: Option<(f32, f32)>,  // Min-Max damage
    pub defense: Option<f32>,
    pub required_level: u32,
    pub required_strength: u32,
    pub required_dexterity: u32,
    pub modifiers: Vec<ItemModifier>,
}

#[derive(Clone)]
pub struct ItemModifier {
    pub stat: String,
    pub value: f32,
}
```

## Quick Implementation Checklist

- [ ] Add damage numbers system
- [ ] Create skill hotbar UI
- [ ] Implement item rarity colors
- [ ] Add grid-based inventory
- [ ] Create minimap
- [ ] Add potion belt
- [ ] Implement skill cooldowns
- [ ] Add item tooltips with stats
- [ ] Create experience bar
- [ ] Add sound effects
- [ ] Implement item drops
- [ ] Add character stats panel
- [ ] Create skill tree UI

## Visual Polish

### Colors (D2R Palette)
```rust
// Use these colors for consistency
const D2R_GOLD: Color = Color::srgb(0.8, 0.6, 0.2);
const D2R_DARK_BG: Color = Color::srgb(0.08, 0.06, 0.04);
const D2R_BORDER: Color = Color::srgb(0.4, 0.3, 0.2);
const D2R_TEXT_LIGHT: Color = Color::srgb(0.9, 0.85, 0.75);
const D2R_TEXT_DARK: Color = Color::srgb(0.5, 0.45, 0.4);
```

### Fonts
- Use a gothic/medieval style font for titles
- Monospace for numbers
- Sans-serif for body text

### Animations
- Smooth transitions (easing functions)
- Item pickup animations
- Skill activation effects
- Damage numbers floating up
- Level-up burst effect
