use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

/// Resource to track inventory drawer state
#[derive(Resource)]
pub struct InventoryDrawerState {
    pub is_open: bool,
    pub animation_progress: f32, // 0.0 = closed, 1.0 = fully open
}

impl Default for InventoryDrawerState {
    fn default() -> Self {
        Self {
            is_open: false,
            animation_progress: 0.0,
        }
    }
}

/// Represents an equippable item slot on the character
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipSlot {
    Head,
    Amulet,
    RightHand,
    LeftHand,
    Torso,
    Belt,
    Gloves,
    Boots,
    RingLeft,
    RingRight,
}

/// Configuration for a single inventory item
#[derive(Clone)]
pub struct InventoryItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub asset_path: String,
    pub icon_path: String, // Path to icon image
    pub slot: EquipSlot,
    pub attachment_point: String,
}

/// Resource managing available items and current equipment
#[derive(Resource)]
pub struct InventorySystem {
    pub items: Vec<InventoryItem>,
    pub equipped: std::collections::HashMap<EquipSlot, Option<String>>,
    pub handles: std::collections::HashMap<String, Handle<Scene>>,
    pub icon_handles: std::collections::HashMap<String, Handle<Image>>,
}

impl Default for InventorySystem {
    fn default() -> Self {
        Self {
            items: vec![
                // Head
                InventoryItem {
                    id: "helm".to_string(),
                    name: "War Helm".to_string(),
                    description: "Heavy protection for the head".to_string(),
                    asset_path: "scenes/items/armor/helm.glb#Scene0".to_string(),
                    icon_path: "icons/mask.png".to_string(),
                    slot: EquipSlot::Head,
                    attachment_point: "head".to_string(),
                },
                
                // Amulet
                InventoryItem {
                    id: "amulet".to_string(),
                    name: "Amulet of Power".to_string(),
                    description: "Mystical amulet".to_string(),
                    asset_path: "scenes/items/accessories/amulet.glb#Scene0".to_string(),
                    icon_path: "icons/amulet.png".to_string(),
                    slot: EquipSlot::Amulet,
                    attachment_point: "neck".to_string(),
                },
                
                // Weapons - Right Hand
                InventoryItem {
                    id: "longsword".to_string(),
                    name: "Longsword".to_string(),
                    description: "A balanced blade for versatile combat".to_string(),
                    asset_path: "scenes/items/weapons/longsword.glb#Scene0".to_string(),
                    icon_path: "icons/longsword.png".to_string(),
                    slot: EquipSlot::RightHand,
                    attachment_point: "SKM_Robot3_HandRt".to_string(),
                },
                InventoryItem {
                    id: "greatsword".to_string(),
                    name: "Greatsword".to_string(),
                    description: "A massive two-handed weapon".to_string(),
                    asset_path: "scenes/items/weapons/greatsword.glb#Scene0".to_string(),
                    icon_path: "icons/greatsword.png".to_string(),
                    slot: EquipSlot::RightHand,
                    attachment_point: "SKM_Robot3_HandRt".to_string(),
                },
                InventoryItem {
                    id: "battleaxe".to_string(),
                    name: "Battle Axe".to_string(),
                    description: "Heavy axe for crushing blows".to_string(),
                    asset_path: "scenes/items/weapons/axe.glb#Scene0".to_string(),
                    icon_path: "icons/axe.png".to_string(),
                    slot: EquipSlot::RightHand,
                    attachment_point: "SKM_Robot3_HandRt".to_string(),
                },
                InventoryItem {
                    id: "mace".to_string(),
                    name: "Mace".to_string(),
                    description: "Blunt weapon effective against armor".to_string(),
                    asset_path: "scenes/items/weapons/mace.glb#Scene0".to_string(),
                    icon_path: "icons/mace.png".to_string(),
                    slot: EquipSlot::RightHand,
                    attachment_point: "SKM_Robot3_HandRt".to_string(),
                },
                
                // Shields - Left Hand
                InventoryItem {
                    id: "round_shield".to_string(),
                    name: "Round Shield".to_string(),
                    description: "Quick defensive option".to_string(),
                    asset_path: "scenes/items/shields/round_shield.glb#Scene0".to_string(),
                    icon_path: "icons/round_shield.png".to_string(),
                    slot: EquipSlot::LeftHand,
                    attachment_point: "SKM_Robot3_HandLt".to_string(),
                },
                InventoryItem {
                    id: "kite_shield".to_string(),
                    name: "Kite Shield".to_string(),
                    description: "Large shield for maximum protection".to_string(),
                    asset_path: "scenes/items/shields/kite_shield.glb#Scene0".to_string(),
                    icon_path: "icons/kite_shield.png".to_string(),
                    slot: EquipSlot::LeftHand,
                    attachment_point: "SKM_Robot3_HandLt".to_string(),
                },
                InventoryItem {
                    id: "torch".to_string(),
                    name: "Torch".to_string(),
                    description: "Lights the way in darkness".to_string(),
                    asset_path: "scenes/items/accessories/torch.glb#Scene0".to_string(),
                    icon_path: "icons/torch.png".to_string(),
                    slot: EquipSlot::LeftHand,
                    attachment_point: "SKM_Robot3_HandLt".to_string(),
                },
                
                // Armor
                InventoryItem {
                    id: "plate_armor".to_string(),
                    name: "Plate Armor".to_string(),
                    description: "Heavy full body protection".to_string(),
                    asset_path: "scenes/items/armor/plate.glb#Scene0".to_string(),
                    icon_path: "icons/plate.png".to_string(),
                    slot: EquipSlot::Torso,
                    attachment_point: "spine_02".to_string(),
                },
                
                // Belt
                InventoryItem {
                    id: "leather_belt".to_string(),
                    name: "Leather Belt".to_string(),
                    description: "Sturdy belt with pouches".to_string(),
                    asset_path: "scenes/items/armor/belt.glb#Scene0".to_string(),
                    icon_path: "icons/belt.png".to_string(),
                    slot: EquipSlot::Belt,
                    attachment_point: "spine_01".to_string(),
                },
                
                // Gloves
                InventoryItem {
                    id: "gauntlets".to_string(),
                    name: "Iron Gauntlets".to_string(),
                    description: "Reinforced hand protection".to_string(),
                    asset_path: "scenes/items/armor/gauntlets.glb#Scene0".to_string(),
                    icon_path: "icons/gauntlets.png".to_string(),
                    slot: EquipSlot::Gloves,
                    attachment_point: "SKM_Robot3_HandRt".to_string(),
                },
                
                // Boots
                InventoryItem {
                    id: "boots".to_string(),
                    name: "War Boots".to_string(),
                    description: "Heavy combat boots".to_string(),
                    asset_path: "scenes/items/armor/boots.glb#Scene0".to_string(),
                    icon_path: "icons/boots.png".to_string(),
                    slot: EquipSlot::Boots,
                    attachment_point: "foot_r".to_string(),
                },
                
                // Rings
                InventoryItem {
                    id: "ring1".to_string(),
                    name: "Ring of Strength".to_string(),
                    description: "Increases physical power".to_string(),
                    asset_path: "scenes/items/accessories/ring.glb#Scene0".to_string(),
                    icon_path: "icons/ring.png".to_string(),
                    slot: EquipSlot::RingLeft,
                    attachment_point: "SKM_Robot3_HandLt".to_string(),
                },
                InventoryItem {
                    id: "ring2".to_string(),
                    name: "Ring of Vitality".to_string(),
                    description: "Increases health".to_string(),
                    asset_path: "scenes/items/accessories/ring.glb#Scene0".to_string(),
                    icon_path: "icons/ring.png".to_string(),
                    slot: EquipSlot::RingRight,
                    attachment_point: "SKM_Robot3_HandRt".to_string(),
                },
            ],
            equipped: std::collections::HashMap::from([
                (EquipSlot::Head, None),
                (EquipSlot::Amulet, None),
                (EquipSlot::RightHand, Some("longsword".to_string())),
                (EquipSlot::LeftHand, None),
                (EquipSlot::Torso, None),
                (EquipSlot::Belt, None),
                (EquipSlot::Gloves, None),
                (EquipSlot::Boots, None),
                (EquipSlot::RingLeft, None),
                (EquipSlot::RingRight, None),
            ]),
            handles: std::collections::HashMap::new(),
            icon_handles: std::collections::HashMap::new(),
        }
    }
}

/// Component to mark equipped item entities
#[derive(Component)]
pub struct EquippedItem {
    pub slot: EquipSlot,
    pub item_id: String,
}

/// Component to mark the knight character root
#[derive(Component)]
pub struct KnightCharacter;

/// Resource to cache egui texture IDs
#[derive(Resource, Default)]
pub struct InventoryTextureCache {
    pub textures: std::collections::HashMap<String, egui::TextureId>,
}

/// System to update texture cache - now with better loading checks
pub fn update_texture_cache(
    mut contexts: EguiContexts,
    inventory: Res<InventorySystem>,
    mut texture_cache: ResMut<InventoryTextureCache>,
    images: Res<Assets<Image>>,
) {
    use bevy_egui::EguiTextureHandle;
    
    for (item_id, icon_handle) in &inventory.icon_handles {
        // Only add to cache if not already cached
        if !texture_cache.textures.contains_key(item_id) {
            // Check if the image asset is actually loaded
            if let Some(image) = images.get(icon_handle) {
                let texture_id = contexts.add_image(EguiTextureHandle::Strong(icon_handle.clone()));
                texture_cache.textures.insert(item_id.clone(), texture_id);
                info!("✅ Cached texture for item: {} (size: {}x{})", item_id, image.width(), image.height());
            }
        }
    }
}

/// Helper function to get item color
fn get_item_color(slot: EquipSlot) -> egui::Color32 {
    match slot {
        EquipSlot::RightHand | EquipSlot::LeftHand => egui::Color32::from_rgb(180, 180, 200),
        EquipSlot::Head | EquipSlot::Torso | EquipSlot::Gloves | EquipSlot::Boots => egui::Color32::from_rgb(160, 140, 120),
        _ => egui::Color32::from_rgb(200, 180, 100),
    }
}

/// Setup system
pub fn setup_inventory_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut inventory = InventorySystem::default();
    
    // Load 3D model handles
    inventory.handles = inventory.items.iter().map(|item| {
        (item.id.clone(), asset_server.load::<Scene>(item.asset_path.clone()))
    }).collect();
    
    // Load icon handles
    inventory.icon_handles = inventory.items.iter().map(|item| {
        (item.id.clone(), asset_server.load::<Image>(item.icon_path.clone()))
    }).collect();
    
    commands.insert_resource(inventory);
    commands.insert_resource(InventoryDrawerState::default());
    commands.insert_resource(InventoryTextureCache::default());
}

/// System to handle keyboard input for inventory toggle
pub fn handle_inventory_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut drawer_state: ResMut<InventoryDrawerState>,
    chat_input: Option<Res<crate::chat::ChatInput>>,
) {
    // Don't process inventory hotkey if chat is focused
    if let Some(chat) = chat_input {
        if chat.is_focused {
            return;
        }
    }

    if keyboard.just_pressed(KeyCode::KeyI) {
        drawer_state.is_open = !drawer_state.is_open;
    }
}

/// System to animate the drawer
pub fn animate_inventory_drawer(
    mut drawer_state: ResMut<InventoryDrawerState>,
    time: Res<Time>,
) {
    let target = if drawer_state.is_open { 1.0 } else { 0.0 };
    let speed = 8.0; // Animation speed
    
    drawer_state.animation_progress = drawer_state.animation_progress
        + (target - drawer_state.animation_progress) * speed * time.delta_secs();
    
    // Clamp to avoid overshooting
    drawer_state.animation_progress = drawer_state.animation_progress.clamp(0.0, 1.0);
}

/// UI system for D2R-style inventory
pub fn inventory_ui(
    mut contexts: EguiContexts,
    mut inventory: ResMut<InventorySystem>,
    mut commands: Commands,
    equipped_query: Query<(Entity, &EquippedItem)>,
    knight_query: Query<Entity, With<KnightCharacter>>,
    children_query: Query<&Children>,
    name_query: Query<(&Name, &GlobalTransform)>,
    mut drawer_state: ResMut<InventoryDrawerState>,
    texture_cache: Res<InventoryTextureCache>,
) {
    let ctx = contexts.ctx_mut().expect("No Egui context found");
let screen_rect = ctx.viewport_rect();
    let drawer_width = screen_rect.width() / 3.0;
    
    // Calculate drawer position based on animation progress
    let offset_x = drawer_width * (1.0 - drawer_state.animation_progress);
    
    // Draw the tab when drawer is closed or closing
    if drawer_state.animation_progress < 0.95 {
        let tab_width = 80.0;
        let tab_height = 120.0;
        let tab_x = screen_rect.right() - tab_width * (1.0 - drawer_state.animation_progress);
        let tab_y = screen_rect.height() / 2.0 - tab_height / 2.0;
        
        egui::Window::new("inventory_tab")
            .title_bar(false)
            .resizable(false)
            .fixed_pos([tab_x, tab_y])
            .fixed_size([tab_width, tab_height])
            .frame(egui::Frame::default()
                .fill(egui::Color32::from_rgba_unmultiplied(20, 15, 10, 240))
                .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 80, 50)))
                .inner_margin(8.0))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    
                    // Rotate text vertically or just show icon
                    ui.label(egui::RichText::new("⚔️")
                        .size(24.0)
                        .color(egui::Color32::from_rgb(200, 180, 140)));
                    
                    ui.add_space(5.0);
                    
                    // Vertical text
                    ui.vertical(|ui| {
                        for c in "INVENTORY".chars() {
                            ui.label(egui::RichText::new(c.to_string())
                                .size(12.0)
                                .color(egui::Color32::from_rgb(180, 160, 120)));
                        }
                    });
                    
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("(I)")
                        .size(10.0)
                        .color(egui::Color32::from_rgb(120, 100, 80)));
                });
                
                // Click tab to open
                if ui.interact(ui.max_rect(), egui::Id::new("tab_click"), egui::Sense::click()).clicked() {
                    drawer_state.is_open = true;
                }
            });
    }
    
    // Only show main inventory panel if animating or open
    if drawer_state.animation_progress > 0.01 {
        let panel_x = screen_rect.right() - drawer_width + offset_x;
        
        egui::Window::new("INVENTORY")
            .title_bar(false)
            .resizable(false)
            .fixed_pos([panel_x, 0.0])
            .fixed_size([drawer_width, screen_rect.height()])
            .frame(egui::Frame::default()
                .fill(egui::Color32::from_rgba_unmultiplied(20, 15, 10, 250))
                .stroke(egui::Stroke::new(3.0, egui::Color32::from_rgb(100, 80, 50)))
                .inner_margin(20.0))
            .show(ctx, |ui| {
                // Close button
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if ui.button(egui::RichText::new("✕")
                            .size(18.0)
                            .color(egui::Color32::from_rgb(200, 150, 100)))
                            .clicked() {
                            drawer_state.is_open = false;
                        }
                    });
                });
                
                ui.add_space(10.0);
                
                // Title
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("INVENTORY")
                        .size(24.0)
                        .color(egui::Color32::from_rgb(200, 180, 140))
                        .strong());
                    
                    ui.label(egui::RichText::new("Press 'I' to close")
                        .size(11.0)
                        .color(egui::Color32::from_rgb(120, 100, 80)));
                });
                
                ui.add_space(15.0);
                
                egui::ScrollArea::vertical()
                    .show(ui, |ui| {
                        render_inventory_content(ui, &mut inventory, &mut commands, 
                                               &equipped_query, &knight_query, 
                                               &children_query, &name_query, &texture_cache);
                    });
            });
    }
}

fn render_inventory_content(
    ui: &mut egui::Ui,
    inventory: &mut InventorySystem,
    commands: &mut Commands,
    equipped_query: &Query<(Entity, &EquippedItem)>,
    knight_query: &Query<Entity, With<KnightCharacter>>,
    children_query: &Query<&Children>,
    name_query: &Query<(&Name, &GlobalTransform)>,
    texture_cache: &InventoryTextureCache,
) {
    ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 8.0);
    
    // Main layout: Left equipment, Center character, Right equipment
    ui.horizontal(|ui| {
        // LEFT COLUMN
        ui.vertical(|ui| {
            render_equipment_slot(ui, inventory, commands, equipped_query, 
                                knight_query, children_query, name_query, EquipSlot::Head, "HEAD", texture_cache);
            ui.add_space(5.0);
            render_equipment_slot(ui, inventory, commands, equipped_query, 
                                knight_query, children_query, name_query, EquipSlot::Amulet, "AMULET", texture_cache);
            ui.add_space(5.0);
            render_equipment_slot(ui, inventory, commands, equipped_query, 
                                knight_query, children_query, name_query, EquipSlot::RightHand, "RIGHT HAND", texture_cache);
            ui.add_space(5.0);
            render_equipment_slot(ui, inventory, commands, equipped_query, 
                                knight_query, children_query, name_query, EquipSlot::RingLeft, "RING", texture_cache);
        });
        
        ui.add_space(15.0);
        
        // CENTER - Character Paper Doll (placeholder)
        ui.vertical(|ui| {
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(140.0, 320.0),
                egui::Sense::hover()
            );
            ui.painter().rect_filled(
                rect,
                3.0,
                egui::Color32::from_rgba_unmultiplied(40, 35, 30, 180)
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "CHARACTER",
                egui::FontId::proportional(14.0),
                egui::Color32::from_rgb(120, 100, 80)
            );
        });
        
        ui.add_space(15.0);
        
        // RIGHT COLUMN
        ui.vertical(|ui| {
            render_equipment_slot(ui, inventory, commands, equipped_query, 
                                knight_query, children_query, name_query, EquipSlot::Torso, "ARMOR", texture_cache);
            ui.add_space(5.0);
            render_equipment_slot(ui, inventory, commands, equipped_query, 
                                knight_query, children_query, name_query, EquipSlot::Belt, "BELT", texture_cache);
            ui.add_space(5.0);
            render_equipment_slot(ui, inventory, commands, equipped_query, 
                                knight_query, children_query, name_query, EquipSlot::LeftHand, "LEFT HAND", texture_cache);
            ui.add_space(5.0);
            render_equipment_slot(ui, inventory, commands, equipped_query, 
                                knight_query, children_query, name_query, EquipSlot::RingRight, "RING", texture_cache);
        });
    });
    
    ui.add_space(10.0);
    
    // BOTTOM ROW
    ui.horizontal(|ui| {
        render_equipment_slot(ui, inventory, commands, equipped_query, 
                            knight_query, children_query, name_query, EquipSlot::Gloves, "GLOVES", texture_cache);
        ui.add_space(10.0);
        render_equipment_slot(ui, inventory, commands, equipped_query, 
                            knight_query, children_query, name_query, EquipSlot::Boots, "BOOTS", texture_cache);
    });
    
    ui.add_space(15.0);
    ui.separator();
    
    // Available items section
    ui.label(egui::RichText::new("AVAILABLE ITEMS")
        .color(egui::Color32::from_rgb(180, 160, 120))
        .strong());
    
    ui.add_space(5.0);
    
    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            render_available_items(ui, inventory, commands, equipped_query, 
                                 knight_query, children_query, name_query, texture_cache);
        });
}

fn render_equipment_slot(
    ui: &mut egui::Ui,
    inventory: &mut InventorySystem,
    commands: &mut Commands,
    equipped_query: &Query<(Entity, &EquippedItem)>,
    _knight_query: &Query<Entity, With<KnightCharacter>>,
    _children_query: &Query<&Children>,
    _name_query: &Query<(&Name, &GlobalTransform)>,
    slot: EquipSlot,
    label: &str,
    texture_cache: &InventoryTextureCache,
) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new(label)
            .size(11.0)
            .color(egui::Color32::from_rgb(150, 130, 100)));
        
        let slot_size = egui::vec2(60.0, 60.0);
        let (rect, response) = ui.allocate_exact_size(slot_size, egui::Sense::click());
        
        // Draw slot background
        ui.painter().rect_filled(
            rect,
            3.0,
            egui::Color32::from_rgba_unmultiplied(30, 25, 20, 200)
        );
        ui.painter().rect(
            rect,
            3.0,
            egui::Color32::TRANSPARENT,
            egui::Stroke::new(1.5, egui::Color32::from_rgb(80, 70, 50)),
            egui::epaint::StrokeKind::Outside
        );
        
        // Show equipped item or empty slot
        if let Some(Some(item_id)) = inventory.equipped.get(&slot) {
            let item_id_clone = item_id.clone();
            if let Some(item) = inventory.items.iter().find(|i| i.id == item_id_clone) {
                // Try to display the icon from cache
                if let Some(texture_id) = texture_cache.textures.get(&item.id) {
                    // Display the actual icon image
                    ui.painter().image(
                        *texture_id,
                        rect.shrink(5.0),
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                } else {
                    // Fallback: colored placeholder if icon not loaded yet
                    let item_color = get_item_color(slot);
                    ui.painter().rect_filled(
                        rect.shrink(5.0),
                        2.0,
                        item_color.linear_multiply(0.6)
                    );
                }
                
                let item_name = item.name.clone();
                
                // Right-click to unequip
                if response.clicked_by(egui::PointerButton::Secondary) {
                    unequip_item(commands, equipped_query, slot, inventory);
                }
                
                response.on_hover_text(&item_name);
            }
        } else {
            // Empty slot indicator
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "◇",
                egui::FontId::proportional(20.0),
                egui::Color32::from_rgb(60, 50, 40)
            );
        }
    });
}

fn render_available_items(
    ui: &mut egui::Ui,
    inventory: &mut InventorySystem,
    commands: &mut Commands,
    equipped_query: &Query<(Entity, &EquippedItem)>,
    knight_query: &Query<Entity, With<KnightCharacter>>,
    children_query: &Query<&Children>,
    name_query: &Query<(&Name, &GlobalTransform)>,
    texture_cache: &InventoryTextureCache,
) {
    ui.horizontal_wrapped(|ui| {
        for item in inventory.items.clone().iter() {
            let is_equipped = inventory.equipped.get(&item.slot)
                .and_then(|x| x.as_ref())
                .map(|id| id == &item.id)
                .unwrap_or(false);
            
            let item_size = egui::vec2(50.0, 50.0);
            let (rect, response) = ui.allocate_exact_size(item_size, egui::Sense::click());
            
            // Background
            let bg_color = if is_equipped {
                egui::Color32::from_rgb(60, 80, 60)
            } else {
                egui::Color32::from_rgba_unmultiplied(30, 25, 20, 200)
            };
            
            ui.painter().rect_filled(rect, 2.0, bg_color);
            ui.painter().rect(
                rect,
                2.0,
                egui::Color32::TRANSPARENT,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 70, 50)),
                egui::epaint::StrokeKind::Outside
            );
            
            // Try to display the icon from cache
            if let Some(texture_id) = texture_cache.textures.get(&item.id) {
                // Display the actual icon image
                ui.painter().image(
                    *texture_id,
                    rect.shrink(5.0),
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            } else {
                // Fallback: colored placeholder if icon not loaded yet
                let item_color = get_item_color(item.slot);
                ui.painter().rect_filled(rect.shrink(5.0), 2.0, item_color.linear_multiply(0.5));
            }
            
            // Equipped checkmark
            if is_equipped {
                ui.painter().text(
                    rect.right_top() + egui::vec2(-8.0, 8.0),
                    egui::Align2::CENTER_CENTER,
                    "✓",
                    egui::FontId::proportional(16.0),
                    egui::Color32::from_rgb(100, 255, 100)
                );
            }
            
            // Click to equip
            if response.clicked() && !is_equipped {
                equip_item(
                    commands,
                    equipped_query,
                    knight_query,
                    children_query,
                    name_query,
                    item.slot,
                    item.clone(),
                    inventory,
                );
            }
            
            response.on_hover_text(format!("{}\n{}", item.name, item.description));
        }
    });
}

/// Equip an item to a slot
fn equip_item(
    commands: &mut Commands,
    equipped_query: &Query<(Entity, &EquippedItem)>,
    knight_query: &Query<Entity, With<KnightCharacter>>,
    children_query: &Query<&Children>,
    name_query: &Query<(&Name, &GlobalTransform)>,
    slot: EquipSlot,
    item: InventoryItem,
    inventory: &mut InventorySystem,
) {
    unequip_item(commands, equipped_query, slot, inventory);

    let knight_entity = knight_query.single().expect("Expected exactly one knight");
    info!("🔧 Equipping {} to {:?}", item.name, slot);
    
    if let Ok(knight_children) = children_query.get(knight_entity) {
        if let Some((bone_entity, _)) = find_bone_recursive(knight_children, &item.attachment_point, name_query, children_query) {
            let scene_handle = inventory.handles.get(&item.id).expect("Handle should exist").clone();
            
            let item_entity = commands
                .spawn((
                    SceneRoot(scene_handle),
                    Transform::from_scale(Vec3::splat(1.0)),
                    Visibility::Inherited,
                ))
                .insert(EquippedItem {
                    slot,
                    item_id: item.id.clone(),
                })
                .id();

            commands.entity(bone_entity).add_child(item_entity);
            inventory.equipped.insert(slot, Some(item.id));
        } else {
            warn!("❌ Attachment point not found: {}", item.attachment_point);
        }
    }
}

/// Unequip item from a slot
fn unequip_item(
    commands: &mut Commands,
    equipped_query: &Query<(Entity, &EquippedItem)>,
    slot: EquipSlot,
    inventory: &mut InventorySystem,
) {
    for (entity, equipped) in equipped_query.iter() {
        if equipped.slot == slot {
            commands.entity(entity).despawn();
        }
    }
    inventory.equipped.insert(slot, None);
}

/// System to handle item asset hot-reloading
pub fn handle_item_reload(
    mut events: MessageReader<AssetEvent<Scene>>,
    inventory: Res<InventorySystem>,
) {
    for event in events.read() {
        if let AssetEvent::Modified { id } = event {
            for (item_id, handle) in &inventory.handles {
                if handle.id() == *id {
                    if let Some(item) = inventory.items.iter().find(|i| &i.id == item_id) {
                        info!("Item asset reloaded: {} ✨", item.name);
                    }
                }
            }
        }
    }
}

/// Helper system to attach items to specific bones/nodes
pub fn attach_to_bones(
    mut equipped_query: Query<(&mut Transform, &EquippedItem), Added<EquippedItem>>,
    knight_query: Query<&Children, With<KnightCharacter>>,
    name_query: Query<(&Name, &GlobalTransform)>,
    children_query: Query<&Children>,
    inventory: Res<InventorySystem>,
) {
    for (mut transform, equipped) in &mut equipped_query {
        if let Some(item) = inventory.items.iter().find(|i| i.id == equipped.item_id) {
            let children = knight_query.single().expect("Expected exactly one knight");
            if let Some(_) = find_bone_recursive(children, &item.attachment_point, &name_query, &children_query) {
                *transform = Transform {
                    translation: Vec3::ZERO,
                    rotation: Quat::IDENTITY,
                    scale: Vec3::splat(1.0),
                };
            }
        }
    }
}

/// Recursive helper to find a bone by name
fn find_bone_recursive(
    children: &Children,
    bone_name: &str,
    name_query: &Query<(&Name, &GlobalTransform)>,
    children_query: &Query<&Children>,
) -> Option<(Entity, GlobalTransform)> {
    for child in children.iter() {
        if let Ok((name, transform)) = name_query.get(child) {
            if name.as_str() == bone_name {
                return Some((child, *transform));
            }
        }
        if let Ok(grandchildren) = children_query.get(child) {
            if let Some(result) = find_bone_recursive(grandchildren, bone_name, name_query, children_query) {
                return Some(result);
            }
        }
    }
    None
}