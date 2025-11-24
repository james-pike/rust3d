// ui/auth/mod.rs - Authentication system

pub mod system;
pub mod ui;

// Re-export main types
pub use system::*;
pub use ui::AuthUIPlugin;
