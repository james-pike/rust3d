// leaderboard.rs - Leaderboard UI display in lobby
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use serde::{Deserialize, Serialize};
use crate::core::states::GameState;

/// Resource to store leaderboard data
#[derive(Resource, Default, Clone)]
pub struct LeaderboardData {
    pub entries: Vec<LeaderboardEntry>,
    pub is_loading: bool,
    pub last_fetch: f64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub kaspa_address: String,
    pub display_name: Option<String>,
    pub total_kills: u32,
    pub total_deaths: u32,
    pub wins: u32,
    pub losses: u32,
    pub games_played: u32,
    pub kd_ratio: f32,
}

impl LeaderboardData {
    pub fn set_loading(&mut self) {
        self.is_loading = true;
        self.error = None;
    }

    pub fn set_entries(&mut self, entries: Vec<LeaderboardEntry>, timestamp: f64) {
        self.entries = entries;
        self.is_loading = false;
        self.last_fetch = timestamp;
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.is_loading = false;
        self.error = Some(error);
    }

    /// Force a fresh fetch on next update
    pub fn force_refresh(&mut self) {
        self.last_fetch = 0.0;
        self.is_loading = false;
    }
}

/// Fetch leaderboard data from API
#[cfg(target_arch = "wasm32")]
pub fn fetch_leaderboard(
    mut leaderboard: ResMut<LeaderboardData>,
    time: Res<Time>,
) {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::spawn_local;

    let current_time = time.elapsed_secs_f64();

    // Fetch every 30 seconds
    if !leaderboard.is_loading && (leaderboard.last_fetch == 0.0 || current_time - leaderboard.last_fetch > 30.0) {
        leaderboard.set_loading();

        spawn_local(async move {
            // Note: This will run async but we can't update the resource from here
            // The actual update happens in a separate system
            if let Err(e) = fetch_leaderboard_async().await {
                error!("Failed to fetch leaderboard: {}", e);
            }
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_leaderboard(
    mut leaderboard: ResMut<LeaderboardData>,
    time: Res<Time>,
) {
    // Desktop mode: create mock data for testing
    if leaderboard.entries.is_empty() {
        leaderboard.entries = vec![
            LeaderboardEntry {
                kaspa_address: "kaspa:test123...".to_string(),
                display_name: Some("TestPlayer1".to_string()),
                total_kills: 100,
                total_deaths: 45,
                wins: 15,
                losses: 5,
                games_played: 20,
                kd_ratio: 2.22,
            },
            LeaderboardEntry {
                kaspa_address: "kaspa:test456...".to_string(),
                display_name: Some("TestPlayer2".to_string()),
                total_kills: 85,
                total_deaths: 50,
                wins: 12,
                losses: 8,
                games_played: 20,
                kd_ratio: 1.70,
            },
            LeaderboardEntry {
                kaspa_address: "kaspa:test789...".to_string(),
                display_name: Some("TestPlayer3".to_string()),
                total_kills: 75,
                total_deaths: 55,
                wins: 10,
                losses: 10,
                games_played: 20,
                kd_ratio: 1.36,
            },
        ];
        leaderboard.last_fetch = time.elapsed_secs_f64();
    }
}

#[cfg(target_arch = "wasm32")]
async fn fetch_leaderboard_async() -> Result<Vec<LeaderboardEntry>, String> {
    use wasm_bindgen::{JsValue, JsCast};
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, RequestMode, Response};

    // TODO: Replace with your deployed Cloudflare Worker URL
    let api_url = "https://dk-leaderboard-api.your-subdomain.workers.dev/api/leaderboard?sort=kd&limit=10";

    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(api_url, &opts)
        .map_err(|e| format!("Request creation error: {:?}", e))?;

    let window = web_sys::window().ok_or("No window object")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;

    let resp: Response = resp_value.dyn_into()
        .map_err(|_| "Response cast error")?;

    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()));
    }

    let json = JsFuture::from(resp.json().map_err(|e| format!("JSON parse error: {:?}", e))?)
        .await
        .map_err(|e| format!("JSON await error: {:?}", e))?;

    let json_str = js_sys::JSON::stringify(&json)
        .map_err(|_| "Stringify error")?
        .as_string()
        .ok_or("String conversion error")?;

    #[derive(Deserialize)]
    struct ApiResponse {
        leaderboard: Vec<LeaderboardEntry>,
    }

    let response: ApiResponse = serde_json::from_str(&json_str)
        .map_err(|e| format!("Deserialization error: {}", e))?;

    Ok(response.leaderboard)
}

/// Render leaderboard UI in lobby
pub fn render_leaderboard_ui(
    mut contexts: EguiContexts,
    leaderboard: Res<LeaderboardData>,
) {
    let ctx = match contexts.ctx_mut() {
        Ok(c) => c,
        Err(_) => return,
    };

    let screen_rect = ctx.viewport_rect();

    // Leaderboard panel - right side, below player info
    egui::Window::new("leaderboard")
        .title_bar(false)
        .resizable(false)
        .fixed_pos([screen_rect.width() - 370.0, 180.0])
        .fixed_size([350.0, 450.0])
        .frame(egui::Frame::default()
            .fill(egui::Color32::from_rgba_unmultiplied(20, 15, 10, 230))
            .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 60, 40)))
            .inner_margin(15.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("LEADERBOARD")
                    .size(18.0)
                    .color(egui::Color32::from_rgb(200, 180, 140))
                    .strong());

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(8.0);
            });

            if leaderboard.is_loading {
                ui.vertical_centered(|ui| {
                    ui.add_space(150.0);
                    ui.label(egui::RichText::new("Loading...")
                        .size(14.0)
                        .color(egui::Color32::from_rgb(150, 130, 100)));
                });
                return;
            }

            if let Some(error) = &leaderboard.error {
                ui.vertical_centered(|ui| {
                    ui.add_space(150.0);
                    ui.label(egui::RichText::new(format!("Error: {}", error))
                        .size(12.0)
                        .color(egui::Color32::from_rgb(255, 100, 100)));
                });
                return;
            }

            if leaderboard.entries.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(150.0);
                    ui.label(egui::RichText::new("No data available")
                        .size(14.0)
                        .color(egui::Color32::from_rgb(150, 130, 100)));
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Play some games to appear on the leaderboard!")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(120, 100, 80)));
                });
                return;
            }

            egui::ScrollArea::vertical()
                .max_height(380.0)
                .show(ui, |ui| {
                    for (rank, entry) in leaderboard.entries.iter().enumerate() {
                        let rank_num = rank + 1;

                        // Rank badge with color based on position
                        let rank_color = match rank_num {
                            1 => egui::Color32::from_rgb(255, 215, 0),   // Gold
                            2 => egui::Color32::from_rgb(192, 192, 192), // Silver
                            3 => egui::Color32::from_rgb(205, 127, 50),  // Bronze
                            _ => egui::Color32::from_rgb(150, 130, 100), // Default
                        };

                        ui.group(|ui| {
                            ui.set_min_width(310.0);

                            // Rank and player name
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("#{}", rank_num))
                                    .size(16.0)
                                    .color(rank_color)
                                    .strong());

                                ui.add_space(8.0);

                                let display_name = entry.display_name.as_ref()
                                    .map(|n| n.as_str())
                                    .unwrap_or("Unknown");
                                ui.label(egui::RichText::new(display_name)
                                    .size(14.0)
                                    .color(egui::Color32::WHITE)
                                    .strong());
                            });

                            ui.add_space(5.0);

                            // Stats grid
                            egui::Grid::new(format!("stats_grid_{}", rank))
                                .spacing([15.0, 5.0])
                                .show(ui, |ui| {
                                    // K/D Ratio - most prominent
                                    ui.label(egui::RichText::new("K/D:")
                                        .size(12.0)
                                        .color(egui::Color32::from_rgb(150, 130, 100)));
                                    ui.label(egui::RichText::new(format!("{:.2}", entry.kd_ratio))
                                        .size(13.0)
                                        .color(egui::Color32::from_rgb(100, 255, 100))
                                        .strong());

                                    ui.label(egui::RichText::new("Wins:")
                                        .size(12.0)
                                        .color(egui::Color32::from_rgb(150, 130, 100)));
                                    ui.label(egui::RichText::new(format!("{}", entry.wins))
                                        .size(12.0)
                                        .color(egui::Color32::WHITE));
                                    ui.end_row();

                                    // Kills and Deaths
                                    ui.label(egui::RichText::new("Kills:")
                                        .size(12.0)
                                        .color(egui::Color32::from_rgb(150, 130, 100)));
                                    ui.label(egui::RichText::new(format!("{}", entry.total_kills))
                                        .size(12.0)
                                        .color(egui::Color32::WHITE));

                                    ui.label(egui::RichText::new("Losses:")
                                        .size(12.0)
                                        .color(egui::Color32::from_rgb(150, 130, 100)));
                                    ui.label(egui::RichText::new(format!("{}", entry.losses))
                                        .size(12.0)
                                        .color(egui::Color32::WHITE));
                                    ui.end_row();

                                    // Deaths and Games
                                    ui.label(egui::RichText::new("Deaths:")
                                        .size(12.0)
                                        .color(egui::Color32::from_rgb(150, 130, 100)));
                                    ui.label(egui::RichText::new(format!("{}", entry.total_deaths))
                                        .size(12.0)
                                        .color(egui::Color32::WHITE));

                                    ui.label(egui::RichText::new("Games:")
                                        .size(12.0)
                                        .color(egui::Color32::from_rgb(150, 130, 100)));
                                    ui.label(egui::RichText::new(format!("{}", entry.games_played))
                                        .size(12.0)
                                        .color(egui::Color32::WHITE));
                                    ui.end_row();
                                });
                        });

                        ui.add_space(5.0);
                    }
                });
        });
}
