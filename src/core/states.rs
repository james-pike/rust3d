// states.rs
use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum GameState {
    WalletAuth,  // Initial auth state (no #[default] needed with insert_state)
    #[default]
    AssetLoading,
    Lobby,       // NEW: Lobby for gear selection before matchmaking
    Matchmaking,
    InGame,
    GameEnd,     // Game over screen showing results
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum RollbackState {
    #[default]
    InRound,
    RoundEnd,
}