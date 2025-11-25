// resources.rs
use bevy::prelude::*;

#[derive(Resource, Clone, Deref, DerefMut)]
pub struct RoundEndTimer(pub Timer);

impl Default for RoundEndTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct Scores(pub u32, pub u32);

#[derive(Resource, Default, Clone, Copy, Debug, Deref, DerefMut)]
pub struct SessionSeed(pub u64);

/// Maps player handles (0 or 1) to their Kaspa wallet addresses and display names
/// This is populated at the start of each match
#[derive(Resource, Default, Clone, Debug)]
pub struct PlayerAddressMapping {
    pub local_player_handle: Option<usize>,
    pub player0_address: Option<String>,
    pub player1_address: Option<String>,
    pub player0_display_name: Option<String>,
    pub player1_display_name: Option<String>,
}

impl PlayerAddressMapping {
    pub fn get_address_by_handle(&self, handle: usize) -> Option<&String> {
        match handle {
            0 => self.player0_address.as_ref(),
            1 => self.player1_address.as_ref(),
            _ => None,
        }
    }

    pub fn get_display_name_by_handle(&self, handle: usize) -> Option<&String> {
        match handle {
            0 => self.player0_display_name.as_ref(),
            1 => self.player1_display_name.as_ref(),
            _ => None,
        }
    }

    pub fn get_local_address(&self) -> Option<&String> {
        self.local_player_handle
            .and_then(|handle| self.get_address_by_handle(handle))
    }

    pub fn get_local_display_name(&self) -> Option<&String> {
        self.local_player_handle
            .and_then(|handle| self.get_display_name_by_handle(handle))
    }

    pub fn get_opponent_address(&self) -> Option<&String> {
        self.local_player_handle.and_then(|handle| {
            let opponent_handle = if handle == 0 { 1 } else { 0 };
            self.get_address_by_handle(opponent_handle)
        })
    }

    pub fn get_opponent_display_name(&self) -> Option<&String> {
        self.local_player_handle.and_then(|handle| {
            let opponent_handle = if handle == 0 { 1 } else { 0 };
            self.get_display_name_by_handle(opponent_handle)
        })
    }
}