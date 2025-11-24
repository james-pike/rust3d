// ui/skills.rs - D2R-style skill hotbar
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

/// Resource to manage the skill hotbar
#[derive(Resource)]
pub struct SkillBar {
    pub slots: [Option<Skill>; 8],
}

impl Default for SkillBar {
    fn default() -> Self {
        Self {
            slots: [
                Some(Skill::new("Fireball", "🔥", 5.0, 15.0)),
                Some(Skill::new("Ice Bolt", "❄️", 3.0, 10.0)),
                Some(Skill::new("Lightning", "⚡", 8.0, 25.0)),
                Some(Skill::new("Heal", "💚", 6.0, 20.0)),
                None,
                None,
                None,
                None,
            ],
        }
    }
}

/// Represents a skill that can be placed in the hotbar
#[derive(Clone)]
pub struct Skill {
    pub name: String,
    pub icon: String,           // Emoji or icon path
    pub cooldown_remaining: f32,
    pub cooldown_max: f32,
    pub mana_cost: f32,
}

impl Skill {
    pub fn new(name: &str, icon: &str, cooldown: f32, mana_cost: f32) -> Self {
        Self {
            name: name.to_string(),
            icon: icon.to_string(),
            cooldown_remaining: 0.0,
            cooldown_max: cooldown,
            mana_cost,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.cooldown_remaining <= 0.0
    }

    pub fn use_skill(&mut self) {
        if self.is_ready() {
            self.cooldown_remaining = self.cooldown_max;
        }
    }
}

/// Update skill cooldowns
pub fn update_skill_cooldowns(
    mut skill_bar: ResMut<SkillBar>,
    time: Res<Time>,
) {
    for slot in &mut skill_bar.slots {
        if let Some(skill) = slot {
            if skill.cooldown_remaining > 0.0 {
                skill.cooldown_remaining -= time.delta_secs();
                if skill.cooldown_remaining < 0.0 {
                    skill.cooldown_remaining = 0.0;
                }
            }
        }
    }
}

/// Handle keyboard input for skills (1-8 keys)
pub fn handle_skill_hotkeys(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut skill_bar: ResMut<SkillBar>,
    mut vitals: ResMut<crate::ui::hud::PlayerVitals>,
    chat_input: Option<Res<crate::ui::chat::ChatInput>>,
) {
    // Don't use skills if chat is focused
    if let Some(chat) = chat_input {
        if chat.is_focused {
            return;
        }
    }

    let keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
    ];

    for (index, key) in keys.iter().enumerate() {
        if keyboard.just_pressed(*key) {
            if let Some(Some(skill)) = skill_bar.slots.get_mut(index) {
                // Check if skill is ready and player has enough mana
                if skill.is_ready() && vitals.energy >= skill.mana_cost {
                    info!("🔮 Using skill: {} (Slot {})", skill.name, index + 1);

                    // Use mana
                    vitals.energy -= skill.mana_cost;

                    // Start cooldown
                    skill.use_skill();

                    // TODO: Actually activate the skill effect
                    // This is where you'd spawn projectiles, apply buffs, etc.
                } else if !skill.is_ready() {
                    warn!("⏳ Skill on cooldown: {:.1}s remaining", skill.cooldown_remaining);
                } else {
                    warn!("💙 Not enough mana! Need: {}, Have: {}", skill.mana_cost, vitals.energy);
                }
            }
        }
    }
}

/// Render the skill hotbar UI
pub fn render_skill_bar(
    mut contexts: EguiContexts,
    skill_bar: Res<SkillBar>,
    vitals: Res<crate::ui::hud::PlayerVitals>,
) {
    let ctx = contexts.ctx_mut().expect("No Egui context found");

    let screen_rect = ctx.viewport_rect();

    // Position below center, above the HUD
    let bar_width = 440.0;
    let bar_x = screen_rect.center().x - bar_width / 2.0;
    let bar_y = screen_rect.max.y - 120.0;

    egui::Window::new("skill_hotbar")
        .title_bar(false)
        .resizable(false)
        .fixed_pos([bar_x, bar_y])
        .fixed_size([bar_width, 70.0])
        .frame(egui::Frame::default()
            .fill(egui::Color32::from_rgba_unmultiplied(20, 15, 10, 200))
            .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 80, 50)))
            .inner_margin(8.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, skill) in skill_bar.slots.iter().enumerate() {
                    render_skill_slot(ui, i, skill, &vitals);
                    if i < 7 {
                        ui.add_space(2.0);
                    }
                }
            });
        });
}

fn render_skill_slot(
    ui: &mut egui::Ui,
    index: usize,
    skill: &Option<Skill>,
    vitals: &crate::ui::hud::PlayerVitals,
) {
    let slot_size = egui::vec2(52.0, 52.0);
    let (rect, response) = ui.allocate_exact_size(slot_size, egui::Sense::click());

    // Background
    ui.painter().rect_filled(
        rect,
        4.0,
        egui::Color32::from_rgb(30, 25, 20),
    );

    ui.painter().rect_stroke(
        rect,
        4.0,
        egui::Stroke::new(1.5, egui::Color32::from_rgb(80, 70, 50)),
        egui::epaint::StrokeKind::Outside,
    );

    if let Some(skill) = skill {
        // Skill icon (center)
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &skill.icon,
            egui::FontId::proportional(28.0),
            egui::Color32::WHITE,
        );

        // Cooldown overlay (dark overlay that shrinks as cooldown decreases)
        if skill.cooldown_remaining > 0.0 {
            let cd_percent = skill.cooldown_remaining / skill.cooldown_max;
            let cd_height = rect.height() * cd_percent;

            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(rect.min.x, rect.max.y - cd_height),
                    egui::vec2(rect.width(), cd_height),
                ),
                0.0,
                egui::Color32::from_rgba_premultiplied(0, 0, 0, 180),
            );

            // Cooldown text
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{:.1}", skill.cooldown_remaining),
                egui::FontId::proportional(14.0),
                egui::Color32::WHITE,
            );
        }

        // Not enough mana indicator (red border)
        if vitals.energy < skill.mana_cost {
            ui.painter().rect_stroke(
                rect,
                4.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(200, 50, 50)),
                egui::epaint::StrokeKind::Outside,
            );
        }

        // Hotkey number (top-left corner)
        ui.painter().text(
            egui::pos2(rect.min.x + 4.0, rect.min.y + 4.0),
            egui::Align2::LEFT_TOP,
            (index + 1).to_string(),
            egui::FontId::proportional(12.0),
            egui::Color32::from_rgb(200, 180, 140),
        );

        // Tooltip on hover and click handling
        let response = response.on_hover_ui(|ui| {
            ui.label(egui::RichText::new(&skill.name)
                .size(14.0)
                .color(egui::Color32::from_rgb(200, 180, 140))
                .strong());

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Mana:")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(150, 130, 100)));
                ui.label(egui::RichText::new(format!("{:.0}", skill.mana_cost))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(100, 200, 255)));
            });

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Cooldown:")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(150, 130, 100)));
                ui.label(egui::RichText::new(format!("{:.1}s", skill.cooldown_max))
                    .size(12.0)
                    .color(egui::Color32::WHITE));
            });

            if skill.cooldown_remaining > 0.0 {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(format!("Ready in: {:.1}s", skill.cooldown_remaining))
                    .size(11.0)
                    .color(egui::Color32::from_rgb(255, 200, 100))
                    .italics());
            }
        });

        // Click to use
        if response.clicked() && skill.is_ready() && vitals.energy >= skill.mana_cost {
            info!("🔮 Skill clicked: {}", skill.name);
            // Would trigger skill use here
        }
    } else {
        // Empty slot
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "◇",
            egui::FontId::proportional(24.0),
            egui::Color32::from_rgb(60, 50, 40),
        );

        // Hotkey number
        ui.painter().text(
            egui::pos2(rect.min.x + 4.0, rect.min.y + 4.0),
            egui::Align2::LEFT_TOP,
            (index + 1).to_string(),
            egui::FontId::proportional(12.0),
            egui::Color32::from_rgb(100, 80, 60),
        );
    }
}

/// Plugin to add skill bar systems
pub struct SkillBarPlugin;

impl Plugin for SkillBarPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SkillBar>()
            .add_systems(
                Update,
                (
                    update_skill_cooldowns,
                    handle_skill_hotkeys,
                    render_skill_bar,
                ),
            );
    }
}

// Usage in lib.rs:
// .add_plugins(SkillBarPlugin)
