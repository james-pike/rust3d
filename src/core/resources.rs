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