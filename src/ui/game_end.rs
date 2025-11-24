// game_end.rs - Game End screen showing match results
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::{
    game::leaderboard::{GameEndData, GameEndTimer},
    core::states::GameState,
};

/// Render the game end screen with results
pub fn render_game_end_ui(
    mut contexts: EguiContexts,
    game_end_data: Option<Res<GameEndData>>,
    timer: Option<Res<GameEndTimer>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let ctx = match contexts.ctx_mut() {
        Ok(c) => c,
        Err(_) => return,
    };

    let screen_rect = ctx.viewport_rect();

    // Get game end data or use defaults
    let data = game_end_data.map(|d| d.clone()).unwrap_or_default();
    let time_remaining = timer.map(|t| t.timer.remaining_secs()).unwrap_or(0.0);

    // Full screen semi-transparent overlay
    egui::Area::new("game_end_overlay".into())
        .fixed_pos([0.0, 0.0])
        .show(ctx, |ui| {
            let full_rect = egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(screen_rect.width(), screen_rect.height()),
            );
            ui.allocate_ui_at_rect(full_rect, |ui| {
                ui.painter().rect_filled(
                    full_rect,
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200),
                );
            });
        });

    // Main game end panel - centered
    egui::Window::new("game_end_main")
        .title_bar(false)
        .resizable(false)
        .fixed_pos([
            screen_rect.center().x - 350.0,
            screen_rect.center().y - 250.0,
        ])
        .fixed_size([700.0, 500.0])
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_rgba_unmultiplied(20, 15, 10, 250))
                .stroke(egui::Stroke::new(4.0, egui::Color32::from_rgb(150, 120, 70)))
                .inner_margin(30.0),
        )
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                // Determine if local player won
                let did_local_player_win = data.local_player_handle == data.winner_handle;

                // Victory/Defeat banner
                if let Some(winner) = data.winner_handle {
                    let (title_text, title_color) = if did_local_player_win {
                        ("⚔ VICTORY ⚔", egui::Color32::from_rgb(255, 215, 0))
                    } else {
                        ("☠ DEFEAT ☠", egui::Color32::from_rgb(200, 50, 50))
                    };

                    ui.label(
                        egui::RichText::new(title_text)
                            .size(48.0)
                            .color(title_color)
                            .strong(),
                    );

                    ui.add_space(10.0);

                    let winner_text = format!("Player {} Wins!", winner + 1);
                    ui.label(
                        egui::RichText::new(winner_text)
                            .size(24.0)
                            .color(egui::Color32::from_rgb(200, 180, 140)),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("GAME ENDED")
                            .size(48.0)
                            .color(egui::Color32::from_rgb(200, 180, 140))
                            .strong(),
                    );
                }

                ui.add_space(30.0);
                ui.separator();
                ui.add_space(20.0);

                // Final Scores
                ui.label(
                    egui::RichText::new("FINAL SCORES")
                        .size(22.0)
                        .color(egui::Color32::from_rgb(200, 180, 140))
                        .strong(),
                );

                ui.add_space(15.0);

                // Score display grid
                egui::Grid::new("score_grid")
                    .spacing([40.0, 15.0])
                    .show(ui, |ui| {
                        // Headers
                        ui.label(
                            egui::RichText::new("PLAYER")
                                .size(16.0)
                                .color(egui::Color32::from_rgb(150, 130, 100))
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new("KILLS")
                                .size(16.0)
                                .color(egui::Color32::from_rgb(150, 130, 100))
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new("STATUS")
                                .size(16.0)
                                .color(egui::Color32::from_rgb(150, 130, 100))
                                .strong(),
                        );
                        ui.end_row();

                        ui.add_space(5.0);
                        ui.add_space(5.0);
                        ui.add_space(5.0);
                        ui.end_row();

                        // Player 1 (handle 0)
                        let p1_is_winner = data.winner_handle == Some(0);
                        let p1_is_local = data.local_player_handle == Some(0);
                        let p1_label = if p1_is_local {
                            "Player 1 (You)"
                        } else {
                            "Player 1"
                        };

                        ui.label(
                            egui::RichText::new(p1_label)
                                .size(18.0)
                                .color(if p1_is_local {
                                    egui::Color32::from_rgb(100, 200, 255)
                                } else {
                                    egui::Color32::WHITE
                                })
                                .strong(),
                        );

                        ui.label(
                            egui::RichText::new(format!("{}", data.player0_score))
                                .size(24.0)
                                .color(if p1_is_winner {
                                    egui::Color32::from_rgb(255, 215, 0)
                                } else {
                                    egui::Color32::WHITE
                                })
                                .strong(),
                        );

                        if p1_is_winner {
                            ui.label(
                                egui::RichText::new("👑 WINNER")
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(255, 215, 0))
                                    .strong(),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new("")
                                    .size(16.0),
                            );
                        }
                        ui.end_row();

                        ui.add_space(5.0);
                        ui.add_space(5.0);
                        ui.add_space(5.0);
                        ui.end_row();

                        // Player 2 (handle 1)
                        let p2_is_winner = data.winner_handle == Some(1);
                        let p2_is_local = data.local_player_handle == Some(1);
                        let p2_label = if p2_is_local {
                            "Player 2 (You)"
                        } else {
                            "Player 2"
                        };

                        ui.label(
                            egui::RichText::new(p2_label)
                                .size(18.0)
                                .color(if p2_is_local {
                                    egui::Color32::from_rgb(100, 200, 255)
                                } else {
                                    egui::Color32::WHITE
                                })
                                .strong(),
                        );

                        ui.label(
                            egui::RichText::new(format!("{}", data.player1_score))
                                .size(24.0)
                                .color(if p2_is_winner {
                                    egui::Color32::from_rgb(255, 215, 0)
                                } else {
                                    egui::Color32::WHITE
                                })
                                .strong(),
                        );

                        if p2_is_winner {
                            ui.label(
                                egui::RichText::new("👑 WINNER")
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(255, 215, 0))
                                    .strong(),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new("")
                                    .size(16.0),
                            );
                        }
                        ui.end_row();
                    });

                ui.add_space(30.0);
                ui.separator();
                ui.add_space(20.0);

                // Return to lobby timer
                ui.label(
                    egui::RichText::new(format!(
                        "Returning to lobby in {:.1}s",
                        time_remaining
                    ))
                    .size(16.0)
                    .color(egui::Color32::from_rgb(150, 130, 100)),
                );

                ui.add_space(10.0);

                // Manual return button
                let button = egui::Button::new(
                    egui::RichText::new("Return to Lobby Now")
                        .size(16.0)
                        .color(egui::Color32::WHITE)
                        .strong(),
                )
                .fill(egui::Color32::from_rgb(60, 100, 60))
                .min_size(egui::vec2(250.0, 40.0));

                if ui.add(button).clicked() {
                    info!("Manual return to lobby clicked");
                    next_state.set(GameState::Lobby);
                }
            });
        });
}
