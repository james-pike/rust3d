// systems/aura_effects.rs (fixed borrow errors)
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::materials::aura::{AuraMaterial, EFFECT_FIRE, EFFECT_LIGHTNING, EFFECT_POISON, EFFECT_HOLY};

/// Resource to track available aura effects
#[derive(Resource)]
pub struct AuraEffects {
    pub effects: Vec<AuraEffect>,
    pub current_index: usize,
}

/// Represents a shader effect configuration
#[derive(Clone)]
pub struct AuraEffect {
    pub name: String,
    pub effect_type: u32,
    pub description: String,
    pub color_tint: Color,
    pub intensity: f32,
}

impl Default for AuraEffects {
    fn default() -> Self {
        Self {
            effects: vec![
                AuraEffect {
                    name: "Fire Aura".to_string(),
                    effect_type: EFFECT_FIRE,
                    description: "Classic fiery aura effect".to_string(),
                    color_tint: Color::srgb(1.0, 0.5, 0.0),
                    intensity: 1.0,
                },
                AuraEffect {
                    name: "Lightning Aura".to_string(),
                    effect_type: EFFECT_LIGHTNING,
                    description: "Electric crackling energy".to_string(),
                    color_tint: Color::srgb(0.8, 0.8, 1.0),
                    intensity: 1.2,
                },
                AuraEffect {
                    name: "Poison Aura".to_string(),
                    effect_type: EFFECT_POISON,
                    description: "Toxic green miasma".to_string(),
                    color_tint: Color::srgb(0.2, 0.8, 0.2),
                    intensity: 0.9,
                },
                AuraEffect {
                    name: "Holy Aura".to_string(),
                    effect_type: EFFECT_HOLY,
                    description: "Divine golden radiance".to_string(),
                    color_tint: Color::srgb(1.0, 0.9, 0.5),
                    intensity: 1.0,
                },
            ],
            current_index: 0,
        }
    }
}

/// Component to mark the aura disc entity
#[derive(Component, Clone)]
pub struct AuraDisc;

/// Setup the aura effects system
pub fn setup_aura_effects(mut commands: Commands) {
    commands.insert_resource(AuraEffects::default());
}

/// UI system to display and switch aura effects
pub fn aura_effects_ui(
    mut contexts: EguiContexts,
    mut aura_effects: ResMut<AuraEffects>,
    mut materials: Option<ResMut<Assets<AuraMaterial>>>, // Fixed: Add mut for mutable borrow
    aura_query: Query<&MeshMaterial3d<AuraMaterial>, With<AuraDisc>>,
) {
    egui::Window::new("🔥 Aura Effects")
        .default_width(300.0)
        .show(contexts.ctx_mut().expect("No Egui context found"), |ui| {
            ui.heading("Select Aura Effect");
            ui.separator();

            // Display current effect
            let current = &aura_effects.effects[aura_effects.current_index];
            ui.label(format!("Current: {}", current.name));
            ui.label(egui::RichText::new(&current.description).italics().small());
            ui.add_space(10.0);

            // Effect selection buttons
            let current_index = aura_effects.current_index;
            let mut new_index = None;
            
            ui.vertical(|ui| {
                for (index, effect) in aura_effects.effects.iter().enumerate() {
                    let is_current = index == current_index;
                    
                    let button = egui::Button::new(&effect.name)
                        .fill(if is_current {
                            egui::Color32::from_rgb(60, 120, 60)
                        } else {
                            egui::Color32::from_rgb(40, 40, 40)
                        });

                    if ui.add_sized([280.0, 30.0], button).clicked() && !is_current {
                        new_index = Some(index);
                        info!("Switching to {} effect", effect.name);
                    }
                }
            });
            
            // Apply the effect change (only if materials exist)
            if let Some(index) = new_index {
                aura_effects.current_index = index;
                let effect = &aura_effects.effects[index];
                let [r, g, b, _] = effect.color_tint.to_srgba().to_f32_array();
                
                if let Some(materials) = materials.as_deref_mut() {
                    for aura_material in aura_query.iter() {
                        if let Some(material) = materials.get_mut(&aura_material.0) {
                            material.effect_type = effect.effect_type;
                            material.intensity = effect.intensity;
                            material.color_r = r;
                            material.color_g = g;
                            material.color_b = b;
                        }
                    }
                }
            }

            ui.add_space(10.0);
            ui.separator();
            
            // Effect parameters
            ui.heading("Effect Parameters");
            let current_idx = aura_effects.current_index;
            let current_effect = &mut aura_effects.effects[current_idx];
            
            // Intensity slider
            ui.horizontal(|ui| {
                ui.label("Intensity:");
                if ui.add(egui::Slider::new(&mut current_effect.intensity, 0.0..=2.0)).changed() {
                    let intensity = current_effect.intensity;
                    if let Some(materials) = materials.as_deref_mut() {
                        for aura_material in aura_query.iter() {
                            if let Some(material) = materials.get_mut(&aura_material.0) {
                                material.intensity = intensity;
                            }
                        }
                    }
                }
            });

            // Color tint adjustment
            ui.horizontal(|ui| {
                ui.label("Color Tint:");
                let mut color = [
                    current_effect.color_tint.to_srgba().red,
                    current_effect.color_tint.to_srgba().green,
                    current_effect.color_tint.to_srgba().blue,
                ];
                
                if ui.color_edit_button_rgb(&mut color).changed() {
                    current_effect.color_tint = Color::srgb(color[0], color[1], color[2]);
                    
                    if let Some(materials) = materials.as_deref_mut() {
                        for aura_material in aura_query.iter() {
                            if let Some(material) = materials.get_mut(&aura_material.0) {
                                material.color_r = color[0];
                                material.color_g = color[1];
                                material.color_b = color[2];
                            }
                        }
                    }
                }
            });

            ui.add_space(5.0);
            ui.label(egui::RichText::new("💡 Tip: Edit shaders/aura.wgsl to modify effects")
                .small()
                .color(egui::Color32::GRAY));
            ui.label(egui::RichText::new("Changes auto-reload on save!")
                .small()
                .color(egui::Color32::GRAY));
        });
}

/// System to handle shader hot-reloading (fixed for Bevy 0.14)
pub fn handle_shader_reload(
    mut events: MessageReader<AssetEvent<Shader>>, // Use MessageReader (standard for Events)
    _shaders: Res<Assets<Shader>>, // Prefix with _ to suppress unused warning
) {
    for event in events.read() { // Use .read() for iteration
        if let AssetEvent::Modified { id } = event { // Field is id, not handle
            info!("Shader reloaded successfully! ✨ id: {:?}", id);
        }
    }
}