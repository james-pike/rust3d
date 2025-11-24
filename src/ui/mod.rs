// ui/mod.rs - User interface systems

pub mod hud;
pub mod inventory;
pub mod lobby;
pub mod score;
pub mod chat;
pub mod auth;
pub mod skills;
pub mod leaderboard;
pub mod game_end;

// Re-export commonly used items
pub use hud::*;
pub use inventory::*;
pub use lobby::*;
pub use chat::{ChatPlugin, ChatUIPlugin, ChatInput, ChatMessages, setup_chat_socket};
pub use auth::AuthUIPlugin;
pub use skills::*;
pub use leaderboard::*;
pub use game_end::*;
