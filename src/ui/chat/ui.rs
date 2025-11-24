// src/chat_ui.rs - FIXED (better visibility and debugging)
use bevy::prelude::*;
use crate::core::states::GameState;  // <-- Added: Import for state guards
use crate::ui::chat::{ChatMessages, ChatInput};

pub struct ChatUIPlugin;

impl Plugin for ChatUIPlugin {
    fn build(&self, app: &mut App) {
        app
            // Setup UI on Lobby enter (earlier than Matchmaking)
            .add_systems(
                OnEnter(GameState::Lobby),
                setup_chat_ui,
            )
            // Update and debug in PostUpdate, but ONLY in game states
            .add_systems(
                PostUpdate,
                (
                    handle_chat_ui_interaction,
                    update_chat_ui
                        .run_if(
                            in_state(GameState::Lobby)
                                .or(in_state(GameState::Matchmaking))
                                .or(in_state(GameState::InGame))
                                .or(in_state(GameState::GameEnd))
                        ),  // <-- Guard: Chat in Lobby, Matchmaking, InGame, and GameEnd
                    debug_chat_state
                        .run_if(
                            in_state(GameState::Lobby)
                                .or(in_state(GameState::Matchmaking))
                                .or(in_state(GameState::InGame))
                                .or(in_state(GameState::GameEnd))
                        ),  // <-- Guard: No logs in early states
                ),
            );
    }
}

#[derive(Component)]
struct ChatMessagesDisplay;

#[derive(Component)]
struct ChatInputDisplay;

#[derive(Component)]
struct ChatInputContainer;

#[derive(Component)]
struct ChatMessageText;

#[derive(Component)]
struct ChatStatusIndicator;

fn setup_chat_ui(
    mut commands: Commands,
    existing_chat: Query<Entity, With<ChatInputContainer>>,
) {
    // Only create chat UI if it doesn't already exist
    if !existing_chat.is_empty() {
        info!("Chat UI already exists, skipping setup");
        return;
    }

    info!("Setting up chat UI");  // <-- Debug: Confirm enter
    // Root UI node
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::End,
            padding: UiRect {
                left: Val::Px(10.0),
                right: Val::Px(10.0),
                top: Val::Px(10.0),
                bottom: Val::Px(150.0), // Push chat above the HUD (140px + 10px margin)
            },
            ..default()
        },
        BackgroundColor(Color::NONE),
    ))
    .with_children(|parent| {
        // Status indicator (top-left corner)
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.5, 0.0, 0.8)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Chat: Click to type"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                ChatStatusIndicator,
            ));
        });

        // Chat messages container
        parent.spawn((
            Node {
                width: Val::Px(400.0),
                height: Val::Px(200.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
            ChatMessagesDisplay,
        ));

        // Chat input box
        parent.spawn((
            Node {
                width: Val::Px(400.0),
                height: Val::Px(40.0),
                padding: UiRect::all(Val::Px(8.0)),
                margin: UiRect::top(Val::Px(5.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
            Interaction::default(),
            ChatInputContainer,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Click to chat..."),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ChatInputDisplay,
            ));
        });
    });
}

fn debug_chat_state(chat_input: Res<ChatInput>) {
    if chat_input.is_changed() {
        info!(
            "Chat state - Focused: {}, Text: '{}'",
            chat_input.is_focused,
            chat_input.current_text
        );
    }
}

fn update_chat_ui(
    chat_messages: Res<ChatMessages>,
    chat_input: Res<ChatInput>,
    mut input_query: Query<(&mut Text, &mut TextColor), With<ChatInputDisplay>>,
    mut status_query: Query<&mut Text, (With<ChatStatusIndicator>, Without<ChatInputDisplay>)>,
    mut commands: Commands,
    messages_display_query: Query<Entity, With<ChatMessagesDisplay>>,
    existing_messages: Query<Entity, With<ChatMessageText>>,
) {
    // Update status indicator
    if let Ok(mut status_text) = status_query.single_mut() {
        **status_text = if chat_input.is_focused {
            "Chat: ACTIVE (ESC or click away to close)".to_string()
        } else {
            "Chat: Click to type".to_string()
        };
    }

    // Update chat input display
    if let Ok((mut text, mut color)) = input_query.single_mut() {
        if chat_input.is_focused {
            **text = format!("> {}", chat_input.current_text);
            *color = TextColor(Color::WHITE);
        } else {
            **text = "Click to chat...".to_string();
            *color = TextColor(Color::srgb(0.7, 0.7, 0.7));
        }
    }

    // Update chat messages
    if chat_messages.is_changed() {
        if let Ok(entity) = messages_display_query.single() {
            // Despawn all existing message entities
            for msg_entity in &existing_messages {
                commands.entity(msg_entity).despawn();
            }
            
            commands.entity(entity).with_children(|parent| {
                // Show last 8 messages
                let start = chat_messages.messages.len().saturating_sub(8);
                for msg in &chat_messages.messages[start..] {
                    parent.spawn((
                        Text::new(format!("{}: {}", msg.author, msg.text)),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 1.0)),
                        Node {
                            margin: UiRect::bottom(Val::Px(4.0)),
                            ..default()
                        },
                        ChatMessageText,
                    ));
                }
            });
        }
    }
}

fn handle_chat_ui_interaction(
    mut chat_input: ResMut<ChatInput>,
    chat_input_query: Query<&Interaction, (With<ChatInputContainer>, Changed<Interaction>)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    // Check if chat input box was clicked
    if let Ok(interaction) = chat_input_query.single() {
        if *interaction == Interaction::Pressed {
            chat_input.is_focused = true;
            info!("Chat focused by click");
        }
    }

    // Check for clicks outside the chat box to unfocus
    // We'll only unfocus if clicking and not clicking on the chat input
    if mouse_button.just_pressed(MouseButton::Left) {
        // Check if we clicked on the chat input container
        let clicked_chat = chat_input_query
            .iter()
            .any(|interaction| matches!(interaction, Interaction::Pressed));

        // If we didn't click on chat and chat is focused, unfocus it
        if !clicked_chat && chat_input.is_focused {
            chat_input.is_focused = false;
            chat_input.current_text.clear();
            info!("Chat unfocused by clicking outside");
        }
    }
}