// networking.rs (fixed)
use bevy::prelude::*;
use bevy_ggrs::{ggrs::GgrsEvent, Session};
use crate::Config;

pub fn handle_ggrs_events(mut session: ResMut<Session<Config>>) {
    if let Session::P2P(s) = session.as_mut() {
        for event in s.events() {
            match event {
                GgrsEvent::Disconnected { .. } | GgrsEvent::NetworkInterrupted { .. } => {
                    warn!("GGRS event: {event:?}")
                }
                GgrsEvent::DesyncDetected {
                    local_checksum,
                    remote_checksum,
                    frame,
                    ..
                } => {
                    error!(
                        "Desync on frame {frame}. Local checksum: {local_checksum:X}, remote checksum: {remote_checksum:X}"
                    );
                }
                _ => info!("GGRS event: {event:?}"),
            }
        }
    }
}