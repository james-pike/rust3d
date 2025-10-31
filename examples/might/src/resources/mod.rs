use bevy::{gltf::Gltf, prelude::*};

/// Marker resource for our knight asset
#[derive(Resource, PartialEq, Eq)]
pub struct Knight {
    pub handle: Handle<Gltf>,
}

/// Marker resource to track whether or not we can spawn the loaded knight asset
#[derive(Resource, PartialEq, Eq)]
pub struct WasLoaded(pub bool);