use bevy::prelude::*;
use bevy_ggrs::{checksum_hasher, PlayerInputs};
use std::{
    f32::consts::PI,
    hash::{Hash, Hasher},
};

// Import from your crate
use crate::Config;

/// Component to mark game entities that should be despawned when leaving InGame state
#[derive(Component)]
pub struct GameEntity;

#[derive(Component, Clone, Copy)]
#[require(DistanceTraveled)]
pub struct Player {
    pub handle: usize,
}

#[derive(Component, Clone, Copy)]
pub struct BulletReady(pub bool);

#[derive(Component, Clone, Copy)]
pub struct Bullet;

#[derive(Component, Clone, Copy)]
pub struct MoveDir(pub Vec2);

impl MoveDir {
    /// Gets the index of the octant (45 degree sectors), starting from 0 (right) and going counter-clockwise:
    pub fn octant(&self) -> usize {
        // in radians, signed: 0 is right, PI/2 is up, -PI/2 is down
        let angle = self.0.to_angle();

        // divide the angle by 45 degrees (PI/4) to get the octant
        let octant = (angle / (PI / 4.)).round() as i32;

        // convert to an octant index in the range [0, 7]
        let octant = if octant < 0 { octant + 8 } else { octant } as usize;

        octant
    }
}

#[derive(Component, Default, Clone, Copy)]
pub struct DistanceTraveled(pub f32);

#[derive(Component, Clone, Copy)]
pub struct Wall;

pub fn checksum_transform(transform: &Transform) -> u64 {
    let mut hasher = checksum_hasher();

    assert!(
        transform.is_finite(),
        "Hashing is not stable for NaN f32 values."
    );

    transform.translation.x.to_bits().hash(&mut hasher);
    transform.translation.y.to_bits().hash(&mut hasher);
    transform.translation.z.to_bits().hash(&mut hasher);

    transform.rotation.x.to_bits().hash(&mut hasher);
    transform.rotation.y.to_bits().hash(&mut hasher);
    transform.rotation.z.to_bits().hash(&mut hasher);
    transform.rotation.w.to_bits().hash(&mut hasher);

    // skip transform.scale as it's not used for gameplay

    hasher.finish()
}

// ============= COMBAT COMPONENTS =============

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct CombatState {
    pub current_action: CombatAction,
    pub can_cancel: bool,
    pub can_combo: bool,
    pub combo_window_active: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CombatAction {
    #[default]
    Idle,
    Attack1,
    Attack2,
    Attack3,
    Dodge,
    Block,
    Hit,
    Dead,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
    pub regen_rate: f32,
    pub regen_delay: f32,
    pub last_use_time: f32,
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            regen_rate: 20.0,
            regen_delay: 1.0,
            last_use_time: 0.0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct AttackBox {
    pub active: bool,
    pub damage: f32,
    pub range: f32,
    pub knockback: f32,
    pub owner: Entity,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Invincible {
    pub active: bool,
    pub duration: f32,
    pub elapsed: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct AnimationState {
    pub current_animation: AnimationType,
    pub animation_time: f32,
    pub transition_speed: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationType {
    Idle,
    IdleAlert,
    WalkForward,
    WalkBackward,
    WalkLeft,
    WalkRight,
    RunForward,
    Attack1,
    Attack2,
    Attack3,
    CrouchIdleAlert,
    CrouchWalkForward,
    Dodge,
    HitReact,
    Death,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            current_animation: AnimationType::Idle,
            animation_time: 0.0,
            transition_speed: 0.2,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ComboState {
    pub combo_count: u32,
    pub max_combo: u32,
    pub combo_timer: f32,
    pub combo_window: f32,
}

// ============= COMBAT COSTS =============

pub const ATTACK_STAMINA_COST: f32 = 20.0;
pub const DODGE_STAMINA_COST: f32 = 25.0;
pub const BLOCK_STAMINA_DRAIN: f32 = 10.0;
pub const SPRINT_STAMINA_DRAIN: f32 = 15.0;

// ============= ATTACK TIMINGS =============

pub struct AttackTiming {
    pub startup: f32,      // Frames before hitbox activates
    pub active: f32,       // Frames hitbox is active
    pub recovery: f32,     // Frames after hitbox deactivates
    pub combo_window: f32, // When you can chain next attack
}

pub const ATTACK1_TIMING: AttackTiming = AttackTiming {
    startup: 0.2,
    active: 0.15,
    recovery: 0.35,
    combo_window: 0.5,
};

pub const ATTACK2_TIMING: AttackTiming = AttackTiming {
    startup: 0.25,
    active: 0.2,
    recovery: 0.4,
    combo_window: 0.6,
};

pub const ATTACK3_TIMING: AttackTiming = AttackTiming {
    startup: 0.3,
    active: 0.25,
    recovery: 0.6,
    combo_window: 0.0, // No combo after final attack
};

// ============= ANIMATION MAPPINGS =============

pub struct AnimationClips {
    pub idle: Handle<AnimationClip>,
    pub idle_alert: Handle<AnimationClip>,
    pub walk_forward: Handle<AnimationClip>,
    pub walk_backward: Handle<AnimationClip>,
    pub walk_left: Handle<AnimationClip>,
    pub walk_right: Handle<AnimationClip>,
    pub run_forward: Handle<AnimationClip>,
    pub attack_01: Handle<AnimationClip>,
    pub attack_02: Handle<AnimationClip>,
    pub attack_03: Handle<AnimationClip>,
    pub crouch_idle: Handle<AnimationClip>,
    pub crouch_forward: Handle<AnimationClip>,
    pub dodge: Handle<AnimationClip>,
    pub hit_react: Handle<AnimationClip>,
    pub death: Handle<AnimationClip>,
}

// ============= COMBAT SYSTEMS =============

pub fn combat_input_system(
    mut players: Query<(
        &mut CombatState,
        &mut Stamina,
        &mut AnimationState,
        &mut ComboState,
        &Player,
    )>,
    time: Res<Time>,
    inputs: Res<PlayerInputs<Config>>,
) {
    use crate::game::input::{attack, dodge, block};
    
    for (mut combat, mut stamina, mut anim, mut combo, player) in &mut players {
        let (input, _) = inputs[player.handle];
        
        // Skip if in uninterruptible action
        if matches!(combat.current_action, CombatAction::Hit | CombatAction::Dead) {
            continue;
        }
        
        // Update combo timer
        if combo.combo_timer > 0.0 {
            combo.combo_timer -= time.delta_secs();
        } else if combo.combo_count > 0 {
            combo.combo_count = 0; // Reset combo
        }
        
        // Attack input
        if attack(input) && stamina.current >= ATTACK_STAMINA_COST {
            if combat.current_action == CombatAction::Idle || combat.can_combo {
                // Determine which attack in combo
                let next_action = match combo.combo_count {
                    0 => {
                        anim.current_animation = AnimationType::Attack1;
                        CombatAction::Attack1
                    }
                    1 => {
                        anim.current_animation = AnimationType::Attack2;
                        CombatAction::Attack2
                    }
                    2 => {
                        anim.current_animation = AnimationType::Attack3;
                        CombatAction::Attack3
                    }
                    _ => continue,
                };
                
                combat.current_action = next_action;
                combat.can_combo = false;
                anim.animation_time = 0.0;
                
                // Consume stamina
                stamina.current -= ATTACK_STAMINA_COST;
                stamina.last_use_time = time.elapsed_secs();
                
                // Advance combo
                combo.combo_count = (combo.combo_count + 1).min(combo.max_combo);
                combo.combo_timer = combo.combo_window;
            }
        }
        
        // Dodge input
        if dodge(input) && stamina.current >= DODGE_STAMINA_COST {
            if combat.can_cancel || combat.current_action == CombatAction::Idle {
                combat.current_action = CombatAction::Dodge;
                anim.current_animation = AnimationType::Dodge;
                anim.animation_time = 0.0;
                
                stamina.current -= DODGE_STAMINA_COST;
                stamina.last_use_time = time.elapsed_secs();
                
                // Reset combo
                combo.combo_count = 0;
                combo.combo_timer = 0.0;
            }
        }
        
        // Block input (hold)
        if block(input) {
            if combat.current_action == CombatAction::Idle {
                combat.current_action = CombatAction::Block;
            }
        } else if combat.current_action == CombatAction::Block {
            combat.current_action = CombatAction::Idle;
        }
    }
}

pub fn stamina_system(
    mut players: Query<(&mut Stamina, &CombatState)>,
    time: Res<Time>,
) {
    for (mut stamina, combat) in &mut players {
        let current_time = time.elapsed_secs();
        
        // Drain stamina while blocking
        if combat.current_action == CombatAction::Block {
            stamina.current -= BLOCK_STAMINA_DRAIN * time.delta_secs();
            stamina.current = stamina.current.max(0.0);
            stamina.last_use_time = current_time;
        }
        
        // Regenerate stamina after delay
        if current_time - stamina.last_use_time >= stamina.regen_delay {
            stamina.current += stamina.regen_rate * time.delta_secs();
            stamina.current = stamina.current.min(stamina.max);
        }
    }
}

pub fn animation_state_system(
    mut players: Query<(
        &mut AnimationState,
        &mut CombatState,
        &MoveDir,
        &DistanceTraveled,
    )>,
    time: Res<Time>,
) {
    for (mut anim_state, mut combat, move_dir, _distance) in &mut players {
        // Update animation time
        anim_state.animation_time += time.delta_secs();
        
        // Handle attack animations with timing
        match combat.current_action {
            CombatAction::Attack1 => {
                if anim_state.animation_time > ATTACK1_TIMING.startup + ATTACK1_TIMING.active {
                    if anim_state.animation_time < ATTACK1_TIMING.startup + ATTACK1_TIMING.active + ATTACK1_TIMING.recovery {
                        combat.combo_window_active = true;
                        combat.can_combo = true;
                    } else {
                        combat.current_action = CombatAction::Idle;
                        combat.can_combo = false;
                        combat.combo_window_active = false;
                    }
                }
            }
            CombatAction::Attack2 => {
                if anim_state.animation_time > ATTACK2_TIMING.startup + ATTACK2_TIMING.active {
                    if anim_state.animation_time < ATTACK2_TIMING.startup + ATTACK2_TIMING.active + ATTACK2_TIMING.recovery {
                        combat.combo_window_active = true;
                        combat.can_combo = true;
                    } else {
                        combat.current_action = CombatAction::Idle;
                        combat.can_combo = false;
                        combat.combo_window_active = false;
                    }
                }
            }
            CombatAction::Attack3 => {
                if anim_state.animation_time > ATTACK3_TIMING.startup + ATTACK3_TIMING.active + ATTACK3_TIMING.recovery {
                    combat.current_action = CombatAction::Idle;
                    combat.can_combo = false;
                    combat.combo_window_active = false;
                }
            }
            CombatAction::Dodge => {
                if anim_state.animation_time > 0.6 {
                    combat.current_action = CombatAction::Idle;
                }
            }
            CombatAction::Idle => {
                // Choose appropriate idle/movement animation
                if move_dir.0.length() > 0.01 {
                    anim_state.current_animation = AnimationType::WalkForward;
                } else {
                    anim_state.current_animation = AnimationType::Idle;
                }
            }
            _ => {}
        }
    }
}

pub fn attack_hitbox_system(
    mut commands: Commands,
    players: Query<(
        Entity,
        &Transform,
        &CombatState,
        &AnimationState,
        &MoveDir,
    )>,
) {
    for (entity, transform, combat, anim, move_dir) in &players {
        let (should_spawn, damage, range) = match combat.current_action {
            CombatAction::Attack1 => {
                if anim.animation_time >= ATTACK1_TIMING.startup 
                    && anim.animation_time <= ATTACK1_TIMING.startup + ATTACK1_TIMING.active {
                    (true, 25.0, 2.0)
                } else {
                    (false, 0.0, 0.0)
                }
            }
            CombatAction::Attack2 => {
                if anim.animation_time >= ATTACK2_TIMING.startup 
                    && anim.animation_time <= ATTACK2_TIMING.startup + ATTACK2_TIMING.active {
                    (true, 30.0, 2.2)
                } else {
                    (false, 0.0, 0.0)
                }
            }
            CombatAction::Attack3 => {
                if anim.animation_time >= ATTACK3_TIMING.startup 
                    && anim.animation_time <= ATTACK3_TIMING.startup + ATTACK3_TIMING.active {
                    (true, 40.0, 2.5)
                } else {
                    (false, 0.0, 0.0)
                }
            }
            _ => (false, 0.0, 0.0),
        };
        
        if should_spawn {
            // Spawn hitbox in front of player
            let forward = Vec3::new(move_dir.0.x, 0.0, move_dir.0.y).normalize_or_zero();
            let hitbox_pos = transform.translation + forward * range / 2.0;
            
            commands.spawn((
                AttackBox {
                    active: true,
                    damage,
                    range,
                    knockback: 1.5,
                    owner: entity,
                },
                Transform::from_translation(hitbox_pos),
                GlobalTransform::default(),
            ));
        }
    }
}

pub fn damage_system(
    mut commands: Commands,
    hitboxes: Query<(Entity, &AttackBox, &Transform)>,
    mut targets: Query<(
        Entity,
        &Transform,
        &mut Health,
        &mut CombatState,
        &Invincible,
    ), Without<AttackBox>>,
) {
    for (hitbox_entity, attack, hitbox_transform) in &hitboxes {
        for (target_entity, target_transform, mut health, mut combat, invincible) in &mut targets {
            // Don't hit yourself
            if target_entity == attack.owner {
                continue;
            }
            
            // Skip if invincible
            if invincible.active {
                continue;
            }
            
            let distance = hitbox_transform.translation.distance(target_transform.translation);
            
            if distance < attack.range {
                // Apply damage
                health.current -= attack.damage;
                
                // Set hit state
                if health.current <= 0.0 {
                    combat.current_action = CombatAction::Dead;
                } else {
                    combat.current_action = CombatAction::Hit;
                }
                
                // Remove hitbox after hit
                commands.entity(hitbox_entity).despawn();
                break;
            }
        }
    }
}

pub fn invincibility_system(
    mut players: Query<(&mut Invincible, &CombatState)>,
    time: Res<Time>,
) {
    for (mut invincible, combat) in &mut players {
        // Activate iframes during dodge
        if combat.current_action == CombatAction::Dodge {
            invincible.active = true;
            invincible.duration = 0.4; // 400ms of iframes
            invincible.elapsed = 0.0;
        }
        
        // Update iframe timer
        if invincible.active {
            invincible.elapsed += time.delta_secs();
            if invincible.elapsed >= invincible.duration {
                invincible.active = false;
            }
        }
    }
}