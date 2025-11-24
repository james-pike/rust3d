// combat.rs
use bevy::prelude::*;
use bevy_roll_safe::prelude::*;
use bevy_ggrs::{PlayerInputs, Rollback, AddRollbackCommandExtension};
use crate::entities::components::*;
use crate::core::constants::*;
use crate::Config;
use crate::game::input::{INPUT_FIRE, INPUT_DOWN};

// Combat Components
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct CombatState {
    pub current_action: CombatAction,
    pub combo_count: u8,
    pub last_attack_time: f32,
    pub attack_cooldown: f32,
    pub is_attacking: bool,
    pub hit_landed: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CombatAction {
    Idle,
    Walking,
    Running,
    Crouching,
    Attack(u8), // Attack combo number (1-3)
    Hit,
    Death,
}

#[derive(Component, Copy, Clone, Debug)]
pub struct AttackHitbox {
    pub damage: f32,
    pub knockback: f32,
    pub active: bool,
    pub lifetime: f32,
}

#[derive(Component, Copy, Clone, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub invulnerable_timer: f32,
}

// AnimationState is defined in entities::components, imported above

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
    Left,
    Right,
}

impl Default for CombatState {
    fn default() -> Self {
        Self {
            current_action: CombatAction::Idle,
            combo_count: 0,
            last_attack_time: 0.0,
            attack_cooldown: 0.5,
            is_attacking: false,
            hit_landed: false,
        }
    }
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            invulnerable_timer: 0.0,
        }
    }
}

// Constants
const COMBO_WINDOW: f32 = 1.0; // Time window to continue combo
const ATTACK_DURATION: f32 = 0.4;
const HIT_STUN_DURATION: f32 = 0.3;
const INVULNERABILITY_DURATION: f32 = 0.5;
const HITBOX_RANGE: f32 = 2.0;

// Input handling for combat
pub fn process_combat_input(
    mut query: Query<(&mut CombatState, &Player, &Transform, &MoveDir, &Health)>,
    inputs: Res<PlayerInputs<Config>>,
    time: Res<Time>,
) {
    for (mut combat, player, transform, move_dir, health) in query.iter_mut() {
        if health.current <= 0.0 {
            combat.current_action = CombatAction::Death;
            continue;
        }

        let (input, _) = inputs[player.handle];
        let current_time = time.elapsed_secs();

        // Update attack cooldown
        if combat.is_attacking && current_time - combat.last_attack_time > ATTACK_DURATION {
            combat.is_attacking = false;
        }

        // Reset combo if window expired
        if current_time - combat.last_attack_time > COMBO_WINDOW {
            combat.combo_count = 0;
        }

        // Handle attack input
        if input & INPUT_FIRE != 0 && !combat.is_attacking {
            combat.combo_count = (combat.combo_count % 3) + 1;
            combat.current_action = CombatAction::Attack(combat.combo_count);
            combat.last_attack_time = current_time;
            combat.is_attacking = true;
            combat.hit_landed = false;
        }
        // Handle crouch
        else if input & INPUT_DOWN != 0 && !combat.is_attacking {
            combat.current_action = CombatAction::Crouching;
        }
        // Handle movement states
        else if !combat.is_attacking {
            let moving = move_dir.0.x.abs() > 0.01 || move_dir.0.y.abs() > 0.01;
            
            if moving {
                // Determine if running (shift key) or walking
                let is_running = input & INPUT_FIRE != 0; // Reuse fire as sprint for demo
                combat.current_action = if is_running {
                    CombatAction::Running
                } else {
                    CombatAction::Walking
                };
            } else {
                combat.current_action = CombatAction::Idle;
            }
        }
    }
}

// Spawn attack hitboxes
pub fn spawn_attack_hitboxes(
    mut commands: Commands,
    query: Query<(Entity, &CombatState, &Transform, &Player), Changed<CombatState>>,
    time: Res<Time>,
) {
    for (entity, combat, transform, player) in query.iter() {
        if let CombatAction::Attack(combo) = combat.current_action {
            if combat.is_attacking && !combat.hit_landed {
                // Calculate hitbox position based on player facing direction
                let forward = transform.rotation * Vec3::Z;
                let hitbox_pos = transform.translation + forward * HITBOX_RANGE;

                // Different damage and knockback for each combo hit
                let (damage, knockback) = match combo {
                    1 => (15.0, 5.0),
                    2 => (20.0, 8.0),
                    3 => (30.0, 12.0), // Finisher
                    _ => (15.0, 5.0),
                };

                commands.spawn((
                    AttackHitbox {
                        damage,
                        knockback,
                        active: true,
                        lifetime: ATTACK_DURATION,
                    },
                    Transform::from_translation(hitbox_pos),
                    GlobalTransform::default(),
                    *player,
                )).add_rollback();
            }
        }
    }
}

// Update hitbox lifetimes
pub fn update_hitboxes(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AttackHitbox)>,
    time: Res<Time>,
) {
    for (entity, mut hitbox) in query.iter_mut() {
        hitbox.lifetime -= time.delta_secs();
        
        if hitbox.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// Detect hits and apply damage
pub fn detect_hits(
    mut victim_query: Query<(Entity, &Transform, &mut Health, &mut CombatState, &Player)>,
    hitbox_query: Query<(&Transform, &AttackHitbox, &Player)>,
    mut attacker_query: Query<&mut CombatState, With<Player>>,
) {
    for (victim_entity, victim_transform, mut health, mut victim_combat, victim_player) in victim_query.iter_mut() {
        // Skip if invulnerable
        if health.invulnerable_timer > 0.0 {
            continue;
        }

        for (hitbox_transform, hitbox, attacker_player) in hitbox_query.iter() {
            // Don't hit yourself
            if victim_player.handle == attacker_player.handle {
                continue;
            }

            // Check distance
            let distance = victim_transform.translation.distance(hitbox_transform.translation);

            if distance < HITBOX_RANGE && hitbox.active {
                // Apply damage
                health.current = (health.current - hitbox.damage).max(0.0);
                health.invulnerable_timer = INVULNERABILITY_DURATION;

                // Set hit state
                if health.current > 0.0 {
                    victim_combat.current_action = CombatAction::Hit;
                    victim_combat.is_attacking = false;
                } else {
                    victim_combat.current_action = CombatAction::Death;
                }

                // Note: Can't easily get attacker entity from hitbox Player component
                // This would need entity tracking in AttackHitbox component
            }
        }
    }
}

// Update health timers
pub fn update_health_timers(
    mut query: Query<&mut Health>,
    time: Res<Time>,
) {
    for mut health in query.iter_mut() {
        if health.invulnerable_timer > 0.0 {
            health.invulnerable_timer -= time.delta_secs();
        }
    }
}

// Recover from hit stun
pub fn recover_from_hit(
    mut query: Query<(&mut CombatState, &Health)>,
    time: Res<Time>,
) {
    for (mut combat, health) in query.iter_mut() {
        if combat.current_action == CombatAction::Hit {
            if health.invulnerable_timer <= INVULNERABILITY_DURATION - HIT_STUN_DURATION {
                combat.current_action = CombatAction::Idle;
            }
        }
    }
}

// Animation selection system
pub fn select_animation(
    query: Query<(&CombatState, &MoveDir, &Transform), Changed<CombatState>>,
) {
    for (combat, move_dir, transform) in query.iter() {
        let direction = determine_direction(move_dir, transform);
        let alert = combat.is_attacking || matches!(combat.current_action, CombatAction::Hit);
        
        let animation_name = match combat.current_action {
            CombatAction::Idle => {
                if alert {
                    format!("Anim_WKM_Idle_Alert")
                } else {
                    format!("Anim_WKM_Idle")
                }
            }
            CombatAction::Walking => {
                let dir_suffix = get_direction_suffix(direction);
                if alert {
                    format!("Anim_WKM_Walk_Alert_{}", dir_suffix)
                } else {
                    format!("Anim_WKM_Walk_{}", dir_suffix)
                }
            }
            CombatAction::Running => {
                let dir_suffix = get_direction_suffix(direction);
                if alert {
                    format!("Anim_WKM_Run_Alert_{}", dir_suffix)
                } else {
                    format!("Anim_WKM_Run_{}", dir_suffix)
                }
            }
            CombatAction::Crouching => {
                let dir_suffix = get_direction_suffix(direction);
                if alert {
                    format!("Anim_WKM_Crouch_Alert_{}", dir_suffix)
                } else {
                    format!("Anim_WKM_Crouch_{}", dir_suffix)
                }
            }
            CombatAction::Attack(combo) => {
                format!("Anim_WKM_Attack_{:02}", combo)
            }
            CombatAction::Hit => {
                let dir_suffix = get_direction_suffix(direction);
                format!("Anim_WKM_Hit_Alert_{}", dir_suffix)
            }
            CombatAction::Death => {
                "Anim_WKM_Death".to_string()
            }
        };
        
        // Here you would apply the animation to your animation player
        // This depends on your animation system setup
    }
}

fn determine_direction(move_dir: &MoveDir, transform: &Transform) -> Direction {
    if move_dir.0.x.abs() < 0.01 && move_dir.0.y.abs() < 0.01 {
        return Direction::Forward;
    }
    
    let forward = transform.rotation * Vec3::Z;
    let move_vec = Vec3::new(move_dir.0.x, 0.0, move_dir.0.y).normalize();
    let dot = forward.dot(move_vec);
    let cross = forward.cross(move_vec).y;
    
    if dot > 0.5 {
        Direction::Forward
    } else if dot < -0.5 {
        Direction::Backward
    } else if cross > 0.0 {
        Direction::Right
    } else {
        Direction::Left
    }
}

fn get_direction_suffix(direction: Direction) -> &'static str {
    match direction {
        Direction::Forward => "Fwd",
        Direction::Backward => "Bwd",
        Direction::Left => "Left",
        Direction::Right => "Right",
    }
}

// Plugin for easy integration
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            RollbackUpdate,
            (
                process_combat_input,
                spawn_attack_hitboxes.after(process_combat_input),
                update_hitboxes,
                detect_hits.after(spawn_attack_hitboxes),
                update_health_timers,
                recover_from_hit.after(detect_hits),
                select_animation.after(process_combat_input),
            ),
        );
    }
}