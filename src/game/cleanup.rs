// cleanup.rs - Cleanup systems for game entities
use bevy::prelude::*;
use crate::entities::components::GameEntity;

/// Cleanup all game entities when leaving InGame state
pub fn cleanup_game_entities(
    mut commands: Commands,
    game_entities: Query<Entity, With<GameEntity>>,
) {
    info!("Cleaning up {} game entities", game_entities.iter().count());

    for entity in &game_entities {
        commands.entity(entity).despawn();
    }
}
