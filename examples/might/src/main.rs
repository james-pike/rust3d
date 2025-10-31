use bevy::prelude::*;
use shadplay::camera::PanOrbitCameraPlugin;

mod components;
mod resources;
mod systems;
mod materials;

use resources::{Knight, WasLoaded};
use systems::{
    camera::setup_camera,
    environment::setup_environment,
    knight::{load_knight, spawn_knight},
    light::animate_light_direction,
    input::quit_listener,
};
use materials::aura::AuraMaterial;

fn main() {
    // Verify the model exists
    let knight_model_path = std::path::Path::new("../assets/scenes/knight_uncompressed.glb");
    if !knight_model_path.exists() {
        dbg!(
            "knight.glb does not exist at {:?}.\n\
             Free non-rigged version: https://sketchfab.com/3d-models/elysia-knight-d099f11914f445afbe727fe5c3ddd39d\n\
             Rigged version (paid): https://www.artstation.com/marketplace/p/RGmbB/medieval-armor-set-unreal-engine-rigged",
            knight_model_path
        );
    }

    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin))
        .add_plugins(MaterialPlugin::<AuraMaterial>::default())
        
        .add_systems(Startup, (setup_camera, setup_environment, load_knight))
        .add_systems(Update, (animate_light_direction, quit_listener))
        
        // Knight spawn pipeline
        .add_systems(
            Update,
            spawn_knight
                .after(load_knight)
                .run_if(resource_exists::<Knight>)
                .run_if(resource_exists_and_equals::<WasLoaded>(WasLoaded(false))),
        )
        
        .run();
}