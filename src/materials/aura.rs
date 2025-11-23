// materials/aura.rs
use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef, // Fixed: Correct import path for ShaderRef
};

/// Custom material for the aura effect with multiple effect types
/// IMPORTANT: Keep this struct in sync with the WGSL shader!
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct AuraMaterial {
    #[uniform(100)]
    pub effect_type: u32,
    #[uniform(100)]
    pub intensity: f32,
    #[uniform(100)]
    pub color_r: f32,
    #[uniform(100)]
    pub color_g: f32,
    #[uniform(100)]
    pub color_b: f32,
    #[uniform(100)]
    pub _padding1: f32, // Padding for 16-byte alignment
    #[uniform(100)]
    pub _padding2: f32, // Total struct size: 32 bytes (8 * f32)
    #[uniform(100)]
    pub _padding3: f32,
}

impl Material for AuraMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/aura.wgsl".into() // Path relative to assets/ directory
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

// Effect type constants (matching shader)
pub const EFFECT_FIRE: u32 = 0;
pub const EFFECT_LIGHTNING: u32 = 1;
pub const EFFECT_POISON: u32 = 2;
pub const EFFECT_HOLY: u32 = 3;