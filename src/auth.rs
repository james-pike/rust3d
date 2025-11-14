use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Resource, Default)]
pub struct WalletInfo {
    pub address: String,
    pub connected: bool,
}

#[derive(Debug, Clone, Resource)]
pub struct AuthState {
    pub wallet: Option<WalletInfo>,
    pub is_checking: bool,
    pub error_message: Option<String>,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            wallet: None,
            is_checking: false,
            error_message: None,
        }
    }
}

impl AuthState {
    pub fn is_authenticated(&self) -> bool {
        self.wallet.as_ref().map_or(false, |w| w.connected)
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "kasware"], js_name = requestAccounts)]
    async fn kasware_request_accounts() -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "kasware"], js_name = getAccounts)]
    async fn kasware_get_accounts() -> JsValue;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = ["window"], js_name = kasware)]
    static KASWARE: JsValue;
}

#[cfg(target_arch = "wasm32")]
pub fn is_kasware_installed() -> bool {
    use wasm_bindgen::JsCast;
    !KASWARE.is_undefined() && !KASWARE.is_null()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn is_kasware_installed() -> bool {
    false // Desktop mode doesn't support Kasware
}

#[cfg(target_arch = "wasm32")]
pub async fn connect_kasware() -> Result<String, String> {
    use wasm_bindgen::JsCast;
    
    let result = kasware_request_accounts().await;
    
    if result.is_array() {
        let array = js_sys::Array::from(&result);
        if array.length() > 0 {
            if let Some(address) = array.get(0).as_string() {
                return Ok(address);
            }
        }
        Err("No accounts found".to_string())
    } else {
        Err("Failed to connect to Kasware wallet".to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn connect_kasware() -> Result<String, String> {
    Err("Kasware only available in browser".to_string())
}

#[derive(Event)]
pub struct ConnectWalletEvent;

#[derive(Event)]
pub struct WalletConnectedEvent {
    pub address: String,
}

#[derive(Event)]
pub struct WalletConnectionFailedEvent {
    pub error: String,
}