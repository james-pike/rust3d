// ui/chat/mod.rs - Chat system

pub mod network;
pub mod ui;

// Re-export plugins and main types
pub use network::{ChatPlugin, ChatMessages, ChatInput, setup_chat_socket};
pub use ui::ChatUIPlugin;
