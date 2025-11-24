use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use crate::core::states::GameState;
use crate::UiReady;
use crate::ui::auth::{connect_kasware, is_kasware_installed, WalletInfo};

#[derive(Resource, Default)]
pub struct LoginUI {
    pub show_login: bool,
    pub error_message: Option<String>,
    pub connecting: bool,
    pub wallet_address: Option<String>,
}

// NEW: Cache Kasware detection (avoids per-frame JS call)
#[derive(Resource, Default)]
pub struct KaswareDetected(pub bool);

#[derive(Message)]
pub struct WalletConnectEvent;

pub struct AuthUIPlugin;

impl Plugin for AuthUIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoginUI>()
            .init_resource::<KaswareDetected>()  // NEW: Init cache
            .add_message::<WalletConnectEvent>()
            .add_systems(
                OnEnter(GameState::WalletAuth),
                (
                    setup_wallet_display,
                    setup_camera,
                    log_wallet_auth_entry,
                    cache_kasware_detection,  // NEW: Cache on entry
                ),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (
                    draw_login_ui,
                    handle_wallet_connect_event,
                    update_wallet_display,
                )
                .chain()
                .run_if(in_state(GameState::WalletAuth)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                update_wallet_display
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

// NEW: Cache Kasware detection once on state entry
fn cache_kasware_detection(mut kasware: ResMut<KaswareDetected>) {
    let detected = is_kasware_installed();
    kasware.0 = detected;
    if detected {
        info!("✅ Kasware wallet detected (cached)");  // Log once
    } else {
        warn!("⚠️ Kasware wallet not detected");
    }
}

pub fn setup_camera(mut commands: Commands) {
    info!("📷 Setting up camera");
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 15.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));
}

pub fn setup_wallet_display(mut login_ui: ResMut<LoginUI>) {
    info!("🔐 Setting up wallet display");
    login_ui.show_login = true;
    login_ui.error_message = None;
    login_ui.connecting = false;
    login_ui.wallet_address = None;
    info!("✅ setup_wallet_display: show_login set to true");
}

// Logging function for WalletAuth entry
pub fn log_wallet_auth_entry() {
    info!("🔐 Entering Wallet Auth state");
}

fn draw_login_ui(
    mut contexts: EguiContexts,
    mut login_ui: ResMut<LoginUI>,
    ui_ready: Res<UiReady>,
    kasware: Res<KaswareDetected>,  // NEW: Use cache
    mut connect_event: MessageWriter<WalletConnectEvent>,
) {
    if !ui_ready.0 {
        info!("⏳ UI not ready yet, skipping draw");
        return;
    }
    // CHANGED: Demote to debug! (less spam)
    debug!("✅ draw_login_ui: UiReady true, proceeding to draw");

    if !login_ui.show_login {
        return;
    }

    let ctx = contexts.ctx_mut().expect("Egui context not found");

    debug!("🎨 Drawing login UI");  // CHANGED: debug!
    
    egui::Window::new("wallet_connect")
        .title_bar(false)
        .resizable(false)
        .fixed_size(egui::Vec2::new(400.0, 300.0))
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.heading(
                    egui::RichText::new("🔐 Connect Kasware Wallet")
                        .size(24.0)
                        .color(egui::Color32::from_rgb(200, 180, 140))
                );
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("Please connect your Kasware wallet to continue")
                        .size(14.0)
                        .color(egui::Color32::from_rgb(180, 160, 120))
                );
                ui.add_space(30.0);

                if !kasware.0 {  // NEW: Use cache
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 100, 100),
                        "⚠️ Kasware wallet not detected"
                    );
                    ui.add_space(10.0);
                    ui.label("Please install Kasware extension");
                    ui.add_space(10.0);
                    if ui.button("Visit kasware.xyz").clicked() {
                        #[cfg(target_arch = "wasm32")]
                        {
                            if let Some(window) = web_sys::window() {
                                let _ = window.open_with_url("https://kasware.xyz");
                            }
                        }
                    }
                } else {
                    let button_text = if login_ui.connecting {
                        "🔄 Connecting..."
                    } else {
                        "🔗 Connect Wallet"
                    };

                    let button = egui::Button::new(
                        egui::RichText::new(button_text)
                            .size(18.0)
                            .color(egui::Color32::WHITE)
                    )
                    .min_size(egui::Vec2::new(200.0, 50.0))
                    .fill(egui::Color32::from_rgb(60, 80, 120));

                    let response = ui.add_enabled(!login_ui.connecting, button);

                    if response.clicked() {
                        info!("🔘 Connect button clicked!");
                        connect_event.write(WalletConnectEvent);
                        login_ui.connecting = true;
                        login_ui.error_message = None;
                    }
                }

                if let Some(error) = &login_ui.error_message {
                    ui.add_space(20.0);
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 80, 80),
                        format!("❌ {}", error)
                    );
                }
            });
        });
}

pub fn update_wallet_display(
    mut contexts: EguiContexts,
    login_ui: Res<LoginUI>,
    ui_ready: Res<UiReady>,
) {
    if !ui_ready.0 {
        return;
    }
    
    if let Some(address) = &login_ui.wallet_address {
        let ctx = contexts.ctx_mut().expect("Egui context not found");
        
        egui::Window::new("wallet_display")
            .title_bar(false)
            .resizable(false)
            .fixed_pos(egui::Pos2::new(10.0, 10.0))
            .show(ctx, |ui| {
                let display_address = if address.len() > 20 {
                    format!("{}...{}", &address[..10], &address[address.len() - 6..])
                } else {
                    address.clone()
                };

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("💼")
                            .size(16.0)
                    );
                    ui.label(
                        egui::RichText::new(format!("Wallet: {}", display_address))
                            .size(14.0)
                            .color(egui::Color32::from_rgb(200, 180, 140))
                    );
                });
            });
    }
}

fn handle_wallet_connect_event(
    mut events: MessageReader<WalletConnectEvent>,
    mut login_ui: ResMut<LoginUI>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    for _ in events.read() {
        info!("📨 Processing wallet connect event");
        
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen_futures::spawn_local;
            
            // Spawn async task to connect wallet
            spawn_local(async move {
                info!("🔌 Attempting to connect to Kasware...");
                match connect_kasware().await {
                    Ok(address) => {
                        info!("✅ Wallet connected: {}", address);
                    }
                    Err(e) => {
                        error!("❌ Failed to connect wallet: {}", e);
                    }
                }
            });
            
            // For testing, simulate success
            login_ui.wallet_address = Some("kaspa:test1234567890abcdef".to_string());
            login_ui.connecting = false;
            login_ui.show_login = false;
            
            commands.insert_resource(WalletInfo {
                address: "kaspa:test1234567890abcdef".to_string(),
                connected: true,
            });
            
            info!("🎮 Transitioning to AssetLoading");
            next_state.set(GameState::AssetLoading);
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Desktop testing mode
            info!("🖥️ Desktop mode: simulating wallet connection");
            login_ui.wallet_address = Some("kaspa:desktop_test".to_string());
            login_ui.connecting = false;
            login_ui.show_login = false;
            
            commands.insert_resource(WalletInfo {
                address: "kaspa:desktop_test".to_string(),
                connected: true,
            });
            
            next_state.set(GameState::AssetLoading);
        }
    }
}