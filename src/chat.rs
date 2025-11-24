// src/chat.rs - FIXED (better input handling)
use bevy::prelude::*;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy_matchbox::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct ChatSocket {
    socket: MatchboxSocket,
}

#[derive(Resource, Default)]
pub struct ChatMessages {
    pub messages: Vec<ChatMessage>,
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub author: String,
    pub text: String,
    pub timestamp: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct NetworkChatMessage {
    author: String,
    text: String,
}

#[derive(Resource)]
pub struct ChatInput {
    pub current_text: String,
    pub is_focused: bool,
}

impl Default for ChatInput {
    fn default() -> Self {
        Self {
            current_text: String::new(),
            is_focused: false,
        }
    }
}

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChatMessages>()
            .init_resource::<ChatInput>()
            .add_systems(Update, (
                handle_chat_input,
                receive_chat_messages,
                send_chat_messages,
            ));
    }
}

pub fn setup_chat_socket(mut commands: Commands) {
    // Use a different room for chat to avoid conflicts with GGRS
    let chat_room_url = "ws://127.0.0.1:3536/extreme_bevy_chat?next=2";
    info!("Connecting to chat server: {chat_room_url}");
    let socket = MatchboxSocket::new_reliable(chat_room_url);
    commands.insert_resource(ChatSocket { socket });
}

fn handle_chat_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut chat_input: ResMut<ChatInput>,
    mut char_events: MessageReader<KeyboardInput>,
) {
    // Debug: Log all keyboard events
    let event_count = char_events.len();
    if event_count > 0 {
        info!("Received {} keyboard events", event_count);
    }

    // Check for Enter key to toggle focus or send
    if keys.just_pressed(KeyCode::Enter) {
        info!("Enter pressed! Focused: {}, Text: '{}'", chat_input.is_focused, chat_input.current_text);
        if chat_input.is_focused && !chat_input.current_text.is_empty() {
            // Message will be sent by send_chat_messages system
            // Don't toggle focus here, let send system handle it
        } else {
            // Toggle focus when Enter is pressed with empty text
            chat_input.is_focused = !chat_input.is_focused;
            info!("Chat focus toggled: {}", chat_input.is_focused);
        }
        return;
    }

    // Only process input when focused
    if !chat_input.is_focused {
        // Clear unread events when not focused
        char_events.clear();
        return;
    }

    // Handle backspace
    if keys.just_pressed(KeyCode::Backspace) {
        chat_input.current_text.pop();
        info!("Backspace - Chat text: '{}'", chat_input.current_text);
    }

    // Handle escape to close chat
    if keys.just_pressed(KeyCode::Escape) {
        chat_input.is_focused = false;
        chat_input.current_text.clear();
        info!("Chat closed with Escape");
        return;
    }

    // Add typed characters
    for event in char_events.read() {
        info!("Processing event: {:?}", event);
        
        if event.state == ButtonState::Pressed {
            match &event.logical_key {
                Key::Character(s) => {
                    // Filter out control characters
                    let filtered: String = s.chars()
                        .filter(|c| !c.is_control() && *c != '\n' && *c != '\r')
                        .collect();
                    
                    if !filtered.is_empty() {
                        chat_input.current_text.push_str(&filtered);
                        info!("Added '{}' - Chat text now: '{}'", filtered, chat_input.current_text);
                    }
                }
                Key::Space => {
                    chat_input.current_text.push(' ');
                    info!("Added space - Chat text now: '{}'", chat_input.current_text);
                }
                _ => {
                    info!("Unhandled key: {:?}", event.logical_key);
                }
            }
        }
    }
}

fn send_chat_messages(
    chat_socket: Option<ResMut<ChatSocket>>,
    mut chat_input: ResMut<ChatInput>,
    mut chat_messages: ResMut<ChatMessages>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    profile: Option<Res<crate::lobby::PlayerProfile>>,
) {
    let Some(mut chat_socket) = chat_socket else {
        return;
    };

    // Send message on Enter when focused and has text
    if !chat_input.is_focused {
        return;
    }

    if !keys.just_pressed(KeyCode::Enter) {
        return;
    }

    let text = chat_input.current_text.trim();
    if text.is_empty() {
        return;
    }

    info!("Sending chat message: '{}'", text);

    // Use display name from profile, fallback to player ID
    let author = if let Some(profile) = profile {
        profile.display_name.clone()
    } else if let Some(id) = chat_socket.socket.id() {
        format!("Player {}", id.0.as_u64_pair().0 % 1000)
    } else {
        "Unknown".to_string()
    };

    // Add to local messages
    chat_messages.messages.push(ChatMessage {
        author: author.clone(),
        text: text.to_string(),
        timestamp: time.elapsed_secs_f64(),
    });

    // Send to peers
    let msg = NetworkChatMessage {
        author,
        text: text.to_string(),
    };

    if let Ok(serialized) = serde_json::to_vec(&msg) {
        let packet = serialized.into_boxed_slice();
        
        // Get peers list first to avoid borrow conflicts
        let peers: Vec<_> = chat_socket.socket.connected_peers().collect();
        
        if let Ok(channel) = chat_socket.socket.get_channel_mut(0) {
            for peer in peers {
                channel.send(packet.clone(), peer);
            }
        }
    }

    // Clear input and keep focus
    chat_input.current_text.clear();
    chat_input.is_focused = false; // Close chat after sending
}

fn receive_chat_messages(
    chat_socket: Option<ResMut<ChatSocket>>,
    mut chat_messages: ResMut<ChatMessages>,
    time: Res<Time>,
) {
    let Some(mut chat_socket) = chat_socket else {
        return;
    };

    chat_socket.socket.update_peers();

    let Ok(channel) = chat_socket.socket.get_channel_mut(0) else {
        return;
    };

    for (peer, packet) in channel.receive() {
        match serde_json::from_slice::<NetworkChatMessage>(&packet) {
            Ok(msg) => {
                info!("Received chat from {peer:?}: {} - {}", msg.author, msg.text);
                chat_messages.messages.push(ChatMessage {
                    author: msg.author,
                    text: msg.text,
                    timestamp: time.elapsed_secs_f64(),
                });
            }
            Err(e) => {
                warn!("Failed to deserialize chat message: {e}");
            }
        }
    }
}