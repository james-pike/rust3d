use crate::Config;
use bevy::{platform::collections::HashMap, prelude::*};
use bevy_ggrs::{LocalInputs, LocalPlayers};

// Input bit flags
const INPUT_UP: u8 = 1 << 0;
const INPUT_DOWN: u8 = 1 << 1;
const INPUT_LEFT: u8 = 1 << 2;
const INPUT_RIGHT: u8 = 1 << 3;
const INPUT_FIRE: u8 = 1 << 4;      // Used for both fire and attack
const INPUT_DODGE: u8 = 1 << 5;     // Dodge/Roll
const INPUT_BLOCK: u8 = 1 << 6;     // Block
const INPUT_SPRINT: u8 = 1 << 7;    // Sprint (optional)

pub fn read_local_inputs(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    local_players: Res<LocalPlayers>,
    chat_input: Option<Res<crate::chat::ChatInput>>,
) {
    let mut local_inputs = HashMap::new();

    // Don't process gameplay inputs if chat is focused
    let chat_is_focused = chat_input.as_ref().map(|c| c.is_focused).unwrap_or(false);

    for handle in &local_players.0 {
        let mut input = 0u8;

        if !chat_is_focused {
            // Movement inputs
            if keys.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
                input |= INPUT_UP;
            }
            if keys.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
                input |= INPUT_DOWN;
            }
            if keys.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
                input |= INPUT_LEFT
            }
            if keys.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
                input |= INPUT_RIGHT;
            }

            // Combat inputs
            if keys.any_pressed([KeyCode::Space, KeyCode::Enter]) {
                input |= INPUT_FIRE;
            }
            if keys.pressed(KeyCode::ShiftLeft) {
                input |= INPUT_DODGE;
            }
            if keys.pressed(KeyCode::ControlLeft) {
                input |= INPUT_BLOCK;
            }
            if keys.pressed(KeyCode::ShiftRight) {
                input |= INPUT_SPRINT;
            }
        }

        local_inputs.insert(*handle, input);
    }

    commands.insert_resource(LocalInputs::<Config>(local_inputs));
}

// Helper functions
pub fn direction(input: u8) -> Vec2 {
    let mut direction = Vec2::ZERO;
    if input & INPUT_UP != 0 {
        direction.y -= 1.;  // W moves up on screen (negative Z)
    }
    if input & INPUT_DOWN != 0 {
        direction.y += 1.;  // S moves down on screen (positive Z)
    }
    if input & INPUT_RIGHT != 0 {
        direction.x += 1.;
    }
    if input & INPUT_LEFT != 0 {
        direction.x -= 1.;
    }
    direction.normalize_or_zero()
}

pub fn fire(input: u8) -> bool {
    input & INPUT_FIRE != 0
}

pub fn attack(input: u8) -> bool {
    input & INPUT_FIRE != 0  // Same as fire
}

pub fn dodge(input: u8) -> bool {
    input & INPUT_DODGE != 0
}

pub fn block(input: u8) -> bool {
    input & INPUT_BLOCK != 0
}

pub fn sprint(input: u8) -> bool {
    input & INPUT_SPRINT != 0
}