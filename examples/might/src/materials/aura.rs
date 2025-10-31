
// ============================================================
// File: materials/aura.rs
// ============================================================
use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

/// Custom material for the aura effect
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct AuraMaterial {
    #[uniform(100)]
    pub inner: f32,
}

impl Material for AuraMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/fire.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}