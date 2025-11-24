// game/damage_numbers.rs - Floating combat text (D2R style)
use bevy::prelude::*;

/// Component for damage numbers that float upward and fade
#[derive(Component)]
pub struct DamageNumber {
    pub lifetime: f32,
    pub velocity: Vec2,
    pub is_crit: bool,
}

/// Spawn a damage number at a position
pub fn spawn_damage_number(
    commands: &mut Commands,
    position: Vec3,
    damage: f32,
    is_crit: bool,
    damage_type: DamageType,
) {
    let color = match damage_type {
        DamageType::Physical => {
            if is_crit {
                Color::srgb(1.0, 1.0, 0.0) // Yellow for crits
            } else {
                Color::WHITE
            }
        }
        DamageType::Fire => Color::srgb(1.0, 0.4, 0.0),      // Orange
        DamageType::Cold => Color::srgb(0.4, 0.8, 1.0),      // Light blue
        DamageType::Lightning => Color::srgb(0.8, 0.8, 1.0),  // Electric blue
        DamageType::Poison => Color::srgb(0.4, 1.0, 0.2),    // Green
    };

    let font_size = if is_crit { 32.0 } else { 24.0 };
    let text = if is_crit {
        format!("{:.0}!", damage) // Add ! for crits
    } else {
        format!("{:.0}", damage)
    };

    commands.spawn((
        Text2d::new(text),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(color),
        Transform::from_translation(position + Vec3::new(0.0, 1.5, 0.0)),
        DamageNumber {
            lifetime: 1.5,
            velocity: Vec2::new(
                rand::random::<f32>() * 20.0 - 10.0, // Small random X movement
                60.0, // Float upward
            ),
            is_crit,
        },
    ));
}

/// Update damage numbers - move upward and fade out
pub fn update_damage_numbers(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut DamageNumber, &mut TextColor)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut number, mut color) in &mut query {
        number.lifetime -= time.delta_secs();

        if number.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            // Float upward with slight horizontal drift
            transform.translation.x += number.velocity.x * time.delta_secs();
            transform.translation.y += number.velocity.y * time.delta_secs();

            // Fade out based on lifetime
            let alpha = (number.lifetime / 1.5).clamp(0.0, 1.0);
            color.0.set_alpha(alpha);

            // Slow down velocity over time
            number.velocity *= 0.98;
        }
    }
}

#[derive(Clone, Copy)]
pub enum DamageType {
    Physical,
    Fire,
    Cold,
    Lightning,
    Poison,
}

/// Plugin to add damage number systems
pub struct DamageNumberPlugin;

impl Plugin for DamageNumberPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_damage_numbers);
    }
}

// Usage example:
// spawn_damage_number(&mut commands, enemy_pos, 125.0, false, DamageType::Physical);
// spawn_damage_number(&mut commands, enemy_pos, 450.0, true, DamageType::Fire);
