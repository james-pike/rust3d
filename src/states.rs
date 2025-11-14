// states.rs
use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum GameState {
    WalletAuth,  // Initial auth state (no #[default] needed with insert_state)
    #[default]
    AssetLoading,
    Matchmaking,
    InGame,
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum RollbackState {
    #[default]
    InRound,
    RoundEnd,
}