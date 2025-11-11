// checksum.rs
use bevy::prelude::*;

pub fn checksum_transform(transform: &Transform) -> u64 {
    // Simple checksum for transform
    let pos = transform.translation;
    let rot = transform.rotation;
    ((pos.x as u64) ^ (pos.y as u64) ^ (pos.z as u64) ^ (rot.x as u64) ^ (rot.y as u64) ^ (rot.z as u64) ^ (rot.w as u64)) as u64
}