use bevy::prelude::*;

/// Listen for quit keys (Q or Escape)
pub fn quit_listener(input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::KeyQ) || input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}
