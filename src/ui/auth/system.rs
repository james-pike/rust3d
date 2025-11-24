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

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(s: &str);
    
    #[wasm_bindgen(js_namespace = ["window"], catch)]
    fn eval(s: &str) -> Result<JsValue, JsValue>;
}

#[cfg(target_arch = "wasm32")]
pub fn is_kasware_installed() -> bool {
    // Check if window.kasware exists
    match eval("typeof window.kasware !== 'undefined'") {
        Ok(result) => {
            if let Some(is_defined) = result.as_bool() {
                if is_defined {
                    console_log("✅ Kasware wallet detected");
                    return true;
                }
            }
            console_log("❌ Kasware wallet not found");
            false
        }
        Err(_) => {
            console_log("❌ Error checking for Kasware");
            false
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn is_kasware_installed() -> bool {
    false // Desktop mode doesn't support Kasware
}

#[cfg(target_arch = "wasm32")]
pub async fn connect_kasware() -> Result<String, String> {
    console_log("🔌 Attempting to connect to Kasware...");
    
    let result = kasware_request_accounts().await;
    
    console_log(&format!("📦 Received result from Kasware: {:?}", result));
    
    if result.is_array() {
        let array = js_sys::Array::from(&result);
        console_log(&format!("📊 Array length: {}", array.length()));
        
        if array.length() > 0 {
            if let Some(address) = array.get(0).as_string() {
                console_log(&format!("✅ Connected to address: {}", address));
                return Ok(address);
            }
        }
        console_log("❌ No accounts found in array");
        Err("No accounts found".to_string())
    } else if let Some(error_msg) = result.as_string() {
        console_log(&format!("❌ Error from Kasware: {}", error_msg));
        Err(error_msg)
    } else {
        console_log("❌ Unexpected response from Kasware");
        Err("Failed to connect to Kasware wallet".to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn connect_kasware() -> Result<String, String> {
    Err("Kasware only available in browser".to_string())
}

#[derive(Message)]
pub struct ConnectWalletEvent;

#[derive(Message)]
pub struct WalletConnectedEvent {
    pub address: String,
}

#[derive(Message)]
pub struct WalletConnectionFailedEvent {
    pub error: String,
}