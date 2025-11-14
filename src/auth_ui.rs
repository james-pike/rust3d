use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::states::GameState;

#[derive(Resource, Default)]
pub struct LoginUI {
    pub show_login: bool,
    pub error_message: Option<String>,
    pub connecting: bool,
    pub wallet_address: Option<String>,
}

#[derive(Message)]
pub struct WalletConnectEvent;

pub struct AuthUIPlugin;

impl Plugin for AuthUIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoginUI>()
            .add_message::<WalletConnectEvent>()
            .add_systems(OnEnter(GameState::WalletAuth), setup_wallet_display)
            .add_systems(
                PostUpdate,
                (
                    draw_login_ui
                        .run_if(|login_ui: Option<Res<LoginUI>>| login_ui.is_some()),
                    handle_wallet_connect_message,
                    update_wallet_display,
                ),
            );
    }
}

pub fn setup_wallet_display(mut login_ui: ResMut<LoginUI>) {
    login_ui.show_login = true;
    login_ui.error_message = None;
    login_ui.connecting = false;
    login_ui.wallet_address = None;
}

fn draw_login_ui(
    mut contexts: EguiContexts,
    mut login_ui: ResMut<LoginUI>,
    mut connect_message: MessageWriter<WalletConnectEvent>,
) {
    // Fixed: Safe unwrap—skip if context not ready (happens on first frame)
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::Window::new("Connect Kasware Wallet")
            .title_bar(false)
            .resizable(false)
            .fixed_size(egui::Vec2::new(400.0, 300.0))
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Connect Kasware Wallet");
                    ui.label("Please connect your Kasware wallet to continue");
                    ui.add_space(20.0);
                });

                let button_text = if login_ui.connecting {
                    "Connecting..."
                } else {
                    "Connect Wallet"
                };

                let response = ui.add_enabled(
                    !login_ui.connecting,
                    egui::Button::new(button_text).min_size(egui::Vec2::new(200.0, 50.0)),
                );

                if response.clicked() {
                    connect_message.write(WalletConnectEvent);
                    login_ui.connecting = true;
                    login_ui.error_message = None;
                }

                if let Some(error) = &login_ui.error_message {
                    ui.add_space(10.0);
                    ui.colored_label(egui::Color32::RED, error);
                }

                ui.add_space(20.0);
                ui.label("Don't have Kasware? Visit kasware.xyz");
            });
    }
}

fn handle_wallet_connect_message(
    mut messages: MessageReader<WalletConnectEvent>,
    mut login_ui: ResMut<LoginUI>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for _ in messages.read() {
        // TODO: Integrate with auth.rs (e.g., call a wasm_bindgen function for Kasware.connect())
        // Stub success for testing:
        login_ui.wallet_address = Some("0x1234...abcd".to_string());
        login_ui.connecting = false;
        login_ui.show_login = false;
        next_state.set(GameState::AssetLoading);
    }
}

pub fn update_wallet_display(
    mut contexts: EguiContexts,
    login_ui: Res<LoginUI>,
) {
    if let Some(address) = &login_ui.wallet_address {
        // Fixed: Safe unwrap—skip if context not ready
        if let Ok(ctx) = contexts.ctx_mut() {
            egui::Window::new("Wallet Connected")
                .title_bar(false)
                .resizable(false)
                .fixed_pos(egui::Pos2::new(10.0, 10.0))
                .show(ctx, |ui| {
                    let display_address = if address.len() > 20 {
                        format!("{}...{}", &address[..10], &address[address.len() - 6..])
                    } else {
                        address.clone()
                    };

                    ui.label(format!("Wallet: {}", display_address));
                });
        }
    }
}