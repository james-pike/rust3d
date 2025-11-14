// Save as: examples/might/src/hud_system.rs
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

#[derive(Resource)]
pub struct PlayerVitals {
    pub health: f32,
    pub max_health: f32,
    pub energy: f32,
    pub max_energy: f32,
    pub level: u32,
    pub experience: f32,
    pub max_experience: f32,
}

impl Default for PlayerVitals {
    fn default() -> Self {
        Self {
            health: 180.0,
            max_health: 250.0,
            energy: 120.0,
            max_energy: 150.0,
            level: 12,
            experience: 2400.0,
            max_experience: 5000.0,
        }
    }
}

pub fn setup_player_vitals(mut commands: Commands) {
    commands.insert_resource(PlayerVitals::default());
}

pub fn render_diablo_hud(mut contexts: EguiContexts, vitals: Res<PlayerVitals>) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
let screen_rect = ctx.viewport_rect();    
    render_bottom_panel(ctx, &screen_rect);
    render_health_orb(ctx, &screen_rect, &vitals);
    render_energy_orb(ctx, &screen_rect, &vitals);
    render_center_info(ctx, &screen_rect, &vitals);
}

fn render_bottom_panel(ctx: &egui::Context, screen_rect: &egui::Rect) {
    let panel_height = 140.0;
    let panel_rect = egui::Rect::from_min_max(
        egui::pos2(0.0, screen_rect.max.y - panel_height),
        screen_rect.max,
    );
    
    egui::Area::new(egui::Id::new("bottom_panel"))
        .fixed_pos(panel_rect.min)
        .show(ctx, |ui| {
            let painter = ui.allocate_painter(
                egui::vec2(screen_rect.width(), panel_height),
                egui::Sense::hover(),
            ).1;
            
            painter.rect_filled(
                panel_rect,
                0.0,
                egui::Color32::from_rgba_premultiplied(10, 8, 6, 230),
            );
            
            painter.hline(
                panel_rect.min.x..=panel_rect.max.x,
                panel_rect.min.y,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 60, 40)),
            );
            
            let corner_size = 40.0;
            
            painter.rect_stroke(
                egui::Rect::from_min_size(
                    panel_rect.min,
                    egui::vec2(corner_size, panel_height),
                ),
                2.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 75, 50)),
                egui::epaint::StrokeKind::Outside,
            );
            
            painter.rect_stroke(
                egui::Rect::from_min_size(
                    egui::pos2(panel_rect.max.x - corner_size, panel_rect.min.y),
                    egui::vec2(corner_size, panel_height),
                ),
                2.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 75, 50)),
                egui::epaint::StrokeKind::Outside,
            );
        });
}

fn render_health_orb(ctx: &egui::Context, screen_rect: &egui::Rect, vitals: &PlayerVitals) {
    let orb_size = 110.0;
    let orb_offset_x = 50.0;
    let orb_offset_y = 40.0;
    
    let orb_center = egui::pos2(
        orb_offset_x + orb_size / 2.0,
        screen_rect.max.y - orb_offset_y - orb_size / 2.0,
    );
    
    egui::Area::new(egui::Id::new("health_orb"))
        .fixed_pos(egui::pos2(
            orb_offset_x,
            screen_rect.max.y - orb_offset_y - orb_size,
        ))
        .show(ctx, |ui| {
            let painter = ui.painter();
            let radius = orb_size / 2.0;
            
            painter.circle_stroke(
                orb_center,
                radius,
                egui::Stroke::new(3.0, egui::Color32::from_rgb(80, 40, 40)),
            );
            
            painter.circle_filled(
                orb_center,
                radius - 3.0,
                egui::Color32::from_rgb(20, 10, 10),
            );
            
            let health_percent = (vitals.health / vitals.max_health).clamp(0.0, 1.0);
            
            if health_percent > 0.0 {
                let fill_height = (radius * 2.0 - 6.0) * health_percent;
                let gradient_steps = 20;
                let step_height = fill_height / gradient_steps as f32;
                
                for i in 0..gradient_steps {
                    let t = i as f32 / gradient_steps as f32;
                    let color = lerp_color(
                        egui::Color32::from_rgb(200, 30, 30),
                        egui::Color32::from_rgb(120, 20, 20),
                        t,
                    );
                    let step_rect = egui::Rect::from_min_size(
                        egui::pos2(
                            orb_center.x - radius + 3.0,
                            orb_center.y + radius - 3.0 - fill_height + i as f32 * step_height,
                        ),
                        egui::vec2(radius * 2.0 - 6.0, step_height + 1.0),
                    );
                    painter.rect_filled(step_rect, 0.0, color);
                }
            }
            
            painter.circle_stroke(
                orb_center,
                radius - 3.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 20, 20)),
            );
            
            painter.circle_stroke(
                egui::pos2(orb_center.x - radius * 0.3, orb_center.y - radius * 0.3),
                radius * 0.3,
                egui::Stroke::new(2.0, egui::Color32::from_rgba_premultiplied(255, 100, 100, 40)),
            );
            
            painter.text(
                orb_center,
                egui::Align2::CENTER_CENTER,
                format!("{}/{}", vitals.health as i32, vitals.max_health as i32),
                egui::FontId::proportional(14.0),
                egui::Color32::WHITE,
            );
            
            painter.text(
                egui::pos2(orb_center.x, orb_center.y + radius + 15.0),
                egui::Align2::CENTER_TOP,
                "HEALTH",
                egui::FontId::proportional(12.0),
                egui::Color32::from_rgb(200, 150, 120),
            );
        });
}

fn render_energy_orb(ctx: &egui::Context, screen_rect: &egui::Rect, vitals: &PlayerVitals) {
    let orb_size = 110.0;
    let orb_offset_x = 50.0;
    let orb_offset_y = 40.0;
    
    let orb_center = egui::pos2(
        screen_rect.max.x - orb_offset_x - orb_size / 2.0,
        screen_rect.max.y - orb_offset_y - orb_size / 2.0,
    );
    
    egui::Area::new(egui::Id::new("energy_orb"))
        .fixed_pos(egui::pos2(
            screen_rect.max.x - orb_offset_x - orb_size,
            screen_rect.max.y - orb_offset_y - orb_size,
        ))
        .show(ctx, |ui| {
            let painter = ui.painter();
            let radius = orb_size / 2.0;
            
            let kaspa_green = egui::Color32::from_rgb(70, 255, 150);
            let dark_kaspa = egui::Color32::from_rgb(20, 80, 50);
            
            painter.circle_stroke(
                orb_center,
                radius,
                egui::Stroke::new(3.0, egui::Color32::from_rgb(40, 100, 60)),
            );
            
            painter.circle_filled(
                orb_center,
                radius - 3.0,
                egui::Color32::from_rgb(10, 20, 15),
            );
            
            let energy_percent = (vitals.energy / vitals.max_energy).clamp(0.0, 1.0);
            
            if energy_percent > 0.0 {
                let fill_height = (radius * 2.0 - 6.0) * energy_percent;
                let gradient_steps = 20;
                let step_height = fill_height / gradient_steps as f32;
                
                for i in 0..gradient_steps {
                    let t = i as f32 / gradient_steps as f32;
                    let color = lerp_color(kaspa_green, dark_kaspa, t);
                    let step_rect = egui::Rect::from_min_size(
                        egui::pos2(
                            orb_center.x - radius + 3.0,
                            orb_center.y + radius - 3.0 - fill_height + i as f32 * step_height,
                        ),
                        egui::vec2(radius * 2.0 - 6.0, step_height + 1.0),
                    );
                    painter.rect_filled(step_rect, 0.0, color);
                }
            }
            
            painter.circle_stroke(
                orb_center,
                radius - 3.0,
                egui::Stroke::new(1.0, dark_kaspa),
            );
            
            painter.circle_stroke(
                egui::pos2(orb_center.x - radius * 0.3, orb_center.y - radius * 0.3),
                radius * 0.3,
                egui::Stroke::new(2.0, egui::Color32::from_rgba_premultiplied(70, 255, 150, 40)),
            );
            
            painter.text(
                orb_center,
                egui::Align2::CENTER_CENTER,
                format!("{}/{}", vitals.energy as i32, vitals.max_energy as i32),
                egui::FontId::proportional(14.0),
                egui::Color32::WHITE,
            );
            
            painter.text(
                egui::pos2(orb_center.x, orb_center.y + radius + 15.0),
                egui::Align2::CENTER_TOP,
                "ENERGY",
                egui::FontId::proportional(12.0),
                egui::Color32::from_rgb(200, 150, 120),
            );
        });
}

fn render_center_info(ctx: &egui::Context, screen_rect: &egui::Rect, vitals: &PlayerVitals) {
    let center_x = screen_rect.center().x;
    let bottom_y = screen_rect.max.y;
    
    egui::Area::new(egui::Id::new("center_info"))
        .fixed_pos(egui::pos2(center_x - 150.0, bottom_y - 120.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(format!("LEVEL {}", vitals.level))
                        .color(egui::Color32::from_rgb(220, 200, 150))
                        .size(18.0)
                        .strong()
                );
                
                ui.add_space(8.0);
                
                let bar_width = 300.0;
                let bar_height = 20.0;
                let exp_percent = (vitals.experience / vitals.max_experience).clamp(0.0, 1.0);
                
                let bar_rect = egui::Rect::from_min_size(
                    ui.cursor().min,
                    egui::vec2(bar_width, bar_height),
                );
                
                ui.allocate_space(egui::vec2(bar_width, bar_height));
                
                let painter = ui.painter();
                
                painter.rect_filled(
                    bar_rect,
                    2.0,
                    egui::Color32::from_rgb(20, 15, 10),
                );
                
                let fill_width = bar_width * exp_percent;
                if fill_width > 0.0 {
                    let fill_rect = egui::Rect::from_min_size(
                        bar_rect.min,
                        egui::vec2(fill_width, bar_height),
                    );
                    painter.rect_filled(
                        fill_rect,
                        2.0,
                        egui::Color32::from_rgb(180, 140, 60),
                    );
                }
                
                painter.rect_stroke(
                    bar_rect,
                    2.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 75, 50)),
                    egui::epaint::StrokeKind::Outside,
                );
                
                painter.text(
                    bar_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{:.0}/{:.0}", vitals.experience, vitals.max_experience),
                    egui::FontId::proportional(12.0),
                    egui::Color32::WHITE,
                );
                
                ui.add_space(15.0);
                
                ui.horizontal(|ui| {
                    for i in 1..=5 {
                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(40.0, 40.0),
                            egui::Sense::click(),
                        );
                        
                        let painter = ui.painter();
                        
                        painter.rect_filled(
                            rect,
                            3.0,
                            egui::Color32::from_rgb(25, 20, 15),
                        );
                        
                        painter.rect_stroke(
                            rect,
                            3.0,
                            egui::Stroke::new(1.5, egui::Color32::from_rgb(80, 60, 40)),
                            egui::epaint::StrokeKind::Outside,
                        );
                        
                        painter.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            i.to_string(),
                            egui::FontId::proportional(14.0),
                            egui::Color32::from_rgb(120, 100, 80),
                        );
                        
                        if response.hovered() {
                            painter.rect_stroke(
                                rect,
                                3.0,
                                egui::Stroke::new(2.0, egui::Color32::from_rgb(150, 120, 80)),
                                egui::epaint::StrokeKind::Outside,
                            );
                        }
                        
                        ui.add_space(5.0);
                    }
                });
            });
        });
}

fn lerp_color(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    egui::Color32::from_rgb(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.g() as f32) * t) as u8,
    )
}

pub fn simulate_vitals_changes(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut vitals: ResMut<PlayerVitals>,
    time: Res<Time>,
) {
    if keyboard.just_pressed(KeyCode::KeyH) {
        vitals.health = (vitals.health - 25.0).max(0.0);
    }
    
    if keyboard.just_pressed(KeyCode::KeyJ) {
        vitals.health = (vitals.health + 25.0).min(vitals.max_health);
    }
    
    if keyboard.just_pressed(KeyCode::KeyE) {
        vitals.energy = (vitals.energy - 20.0).max(0.0);
    }
    
    if keyboard.just_pressed(KeyCode::KeyR) {
        vitals.energy = (vitals.energy + 20.0).min(vitals.max_energy);
    }
    
    vitals.energy = (vitals.energy + 5.0 * time.delta_secs()).min(vitals.max_energy);
    
    vitals.experience += 10.0 * time.delta_secs();
    if vitals.experience >= vitals.max_experience {
        vitals.experience = 0.0;
        vitals.level += 1;
        vitals.max_experience *= 1.2;
    }
}