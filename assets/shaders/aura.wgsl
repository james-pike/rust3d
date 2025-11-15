#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::globals
#import bevy_render::view::View

/// Keep up-to-date with the rust definition!
struct AuraMaterial {
    effect_type: u32,
    intensity: f32,
    color_r: f32,
    color_g: f32,
    color_b: f32,
    _padding: f32,
}

@group(0) @binding(0) var<uniform> view: View;
@group(3) @binding(100) var<uniform> material: AuraMaterial;

// Effect type constants
const EFFECT_FIRE: u32 = 0u;
const EFFECT_LIGHTNING: u32 = 1u;
const EFFECT_POISON: u32 = 2u;
const EFFECT_HOLY: u32 = 3u;

const PI: f32 = 3.141592653589;

// ============================================================
// SHARED UTILITY FUNCTIONS
// ============================================================

fn hash(p: vec2f) -> f32 {
    let p3 = fract(vec3f(p.x, p.y, p.x) * 0.13);
    let dot_product = dot(p3, vec3f(p3.y, p3.z, p3.x) + 3.333);
    return fract((p3.x + p3.y) * dot_product);
}

fn hash2(p: vec2f) -> vec2f {
    let p3 = fract(vec3f(p.x, p.y, p.x) * vec3f(0.1031, 0.1030, 0.0973));
    let p3_shifted = p3 + vec3f(dot(p3, vec3f(p3.y, p3.z, p3.x) + 33.33));
    return fract(vec2f(p3_shifted.x + p3_shifted.y, p3_shifted.y + p3_shifted.z) * p3_shifted.x);
}

fn noise(p: vec2f) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(
        mix(hash(i + vec2f(0.0, 0.0)), hash(i + vec2f(1.0, 0.0)), u.x),
        mix(hash(i + vec2f(0.0, 1.0)), hash(i + vec2f(1.0, 1.0)), u.x),
        u.y
    );
}

fn fbm(p: vec2f, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var p_var = p;
    
    for (var i = 0; i < octaves; i++) {
        value += amplitude * noise(p_var * frequency);
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    
    return value;
}

fn sdCircle(p: vec2f, r: f32) -> f32 {
    return length(p) - r;
}

// ============================================================
// FIRE EFFECT
// ============================================================

const FIRE_YELLOW: vec3f = vec3f(1.0, 0.9, 0.3);
const FIRE_ORANGE: vec3f = vec3f(1.0, 0.5, 0.1);
const FIRE_RED: vec3f = vec3f(0.9, 0.2, 0.05);
const FIRE_DARK: vec3f = vec3f(0.3, 0.05, 0.0);

fn fire_effect(uv: vec2<f32>, time: f32, dist: f32, angle: f32, feet_mask: f32) -> vec4<f32> {
    // Main circular mask with soft edges
    let circle_mask = 1.0 - smoothstep(0.25, 1.1, dist);
    
    // Create rising flame effect using noise
    let flame_coord = vec2f(angle * 3.0, dist * 4.0 - time * 1.5);
    let turbulence1 = fbm(flame_coord * 2.0 + time * 0.5, 5);
    let turbulence2 = fbm(flame_coord * 3.0 - time * 0.8, 5);
    
    // Combine turbulence for flame shape
    var flame = turbulence1 * 0.6 + turbulence2 * 0.4;
    
    // Make flames rise and fade with distance
    flame *= (1.0 - dist * 0.8);
    flame = pow(flame, 2.0);
    
    // Flickering intensity
    let flicker = sin(time * 8.0 + angle * 4.0) * 0.15 + 0.85;
    flame *= flicker;
    
    // Add dancing flame tendrils
    let tendril_angle = angle * 8.0 + time * 3.0;
    let tendrils = sin(tendril_angle) * 0.5 + 0.5;
    let tendril_intensity = smoothstep(0.6, 1.0, dist) * tendrils * 0.3;
    flame += tendril_intensity;
    
    // Color gradient from hot center to cooler edges
    var fire_color: vec3f;
    if (flame > 0.7) {
        fire_color = mix(FIRE_ORANGE, FIRE_YELLOW, (flame - 0.7) / 0.3);
    } else if (flame > 0.3) {
        fire_color = mix(FIRE_RED, FIRE_ORANGE, (flame - 0.3) / 0.4);
    } else {
        fire_color = mix(FIRE_DARK, FIRE_RED, flame / 0.3);
    }
    
    // Add hot spots
    let hot_spots = fbm(vec2f(angle * 5.0, dist * 8.0) + time * 2.0, 5);
    fire_color += FIRE_YELLOW * hot_spots * 0.3 * (1.0 - dist);
    
    // Inner glow ring
    let inner_glow = smoothstep(0.35, 0.25, dist) * 0.5;
    fire_color += FIRE_ORANGE * inner_glow;
    
    // Apply masks
    let final_color = fire_color * circle_mask * feet_mask;
    let alpha = flame * circle_mask * feet_mask;
    
    return vec4f(final_color, alpha);
}

// ============================================================
// LIGHTNING EFFECT
// ============================================================

const LIGHTNING_BRIGHT: vec3f = vec3f(0.9, 0.95, 1.0);
const LIGHTNING_CORE: vec3f = vec3f(0.6, 0.8, 1.0);
const LIGHTNING_BLUE: vec3f = vec3f(0.3, 0.5, 0.9);
const LIGHTNING_PURPLE: vec3f = vec3f(0.5, 0.3, 0.8);

fn lightning_bolt(p: vec2f, seed: f32, time: f32) -> f32 {
    var dist = 1000.0;
    var current_pos = vec2f(0.0, 0.0);
    
    for (var i = 0; i < 6; i++) {
        let t = f32(i) / 6.0;
        let offset = (hash(vec2f(seed + f32(i), time * 0.1)) - 0.5) * 0.3;
        let next_pos = vec2f(
            cos(seed * 6.28 + t * 0.5) * (0.4 + t * 0.6) + offset,
            sin(seed * 6.28 + t * 0.5) * (0.4 + t * 0.6)
        );
        
        let pa = p - current_pos;
        let ba = next_pos - current_pos;
        let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
        let segment_dist = length(pa - ba * h);
        
        dist = min(dist, segment_dist);
        current_pos = next_pos;
    }
    
    return dist;
}

fn random_lightning_strike(p: vec2f, seed: f32, time: f32) -> f32 {
    var dist = 1000.0;
    
    let random_angle = hash(vec2f(seed, 0.0)) * 6.28;
    let start_offset = hash(vec2f(seed, 1.0)) * 0.4 + 0.2;
    
    var current_pos = vec2f(
        cos(random_angle) * start_offset,
        sin(random_angle) * start_offset
    );
    
    for (var i = 0; i < 4; i++) {
        let t = f32(i) / 4.0;
        let deviation_x = (hash(vec2f(seed * 2.0, f32(i) * 2.0)) - 0.5) * 0.3;
        let deviation_y = (hash(vec2f(seed * 3.0, f32(i) * 3.0)) - 0.5) * 0.3;
        
        let next_pos = vec2f(
            cos(random_angle) * (start_offset + t * 0.6) + deviation_x,
            sin(random_angle) * (start_offset + t * 0.6) + deviation_y
        );
        
        let pa = p - current_pos;
        let ba = next_pos - current_pos;
        let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
        let segment_dist = length(pa - ba * h);
        
        dist = min(dist, segment_dist);
        current_pos = next_pos;
    }
    
    return dist;
}

fn lightning_effect(uv: vec2<f32>, time: f32, dist: f32, angle: f32, feet_mask: f32) -> vec4<f32> {
    let circle_mask = 1.0 - smoothstep(0.25, 1.2, dist);
    
    var lightning_intensity = 0.0;
    let num_bolts = 8.0;
    
    for (var i = 0.0; i < num_bolts; i += 1.0) {
        let bolt_time = time * 2.0 + i * 0.5;
        let bolt_phase = fract(bolt_time * 0.3);
        let bolt_visibility = smoothstep(0.0, 0.1, bolt_phase) * (1.0 - smoothstep(0.2, 0.4, bolt_phase));
        
        if (bolt_visibility > 0.01) {
            let bolt_seed = i / num_bolts + floor(bolt_time * 0.3) * 0.123;
            let bolt_dist = lightning_bolt(uv, bolt_seed, time);
            let bolt_core = smoothstep(0.02, 0.0, bolt_dist);
            let bolt_glow = smoothstep(0.15, 0.0, bolt_dist);
            lightning_intensity += (bolt_core + bolt_glow * 0.5) * bolt_visibility;
        }
    }
    
    let num_random_strikes = 12.0;
    for (var j = 0.0; j < num_random_strikes; j += 1.0) {
        let strike_time = time * 3.5 + j * 0.7;
        let strike_phase = fract(strike_time * 0.4);
        let strike_visibility = smoothstep(0.0, 0.05, strike_phase) * (1.0 - smoothstep(0.1, 0.25, strike_phase));
        
        if (strike_visibility > 0.01) {
            let strike_seed = j / num_random_strikes + floor(strike_time * 0.4) * 0.456;
            let strike_dist = random_lightning_strike(uv, strike_seed, time);
            let strike_core = smoothstep(0.025, 0.0, strike_dist);
            let strike_glow = smoothstep(0.15, 0.0, strike_dist);
            lightning_intensity += (strike_core * 1.5 + strike_glow * 0.5) * strike_visibility;
        }
    }
    
    let arc_pattern = sin(angle * 20.0 + time * 10.0) * sin(dist * 30.0 - time * 15.0);
    let arcs = smoothstep(0.7, 0.9, arc_pattern) * 0.3;
    lightning_intensity += arcs;
    
    let field_noise = hash(uv * 10.0 + time * 0.5);
    let electric_field = field_noise * 0.15 * (1.0 - dist * 0.5);
    lightning_intensity += electric_field;
    
    let ground_pulse = sin(time * 8.0) * 0.5 + 0.5;
    let ground_ring = smoothstep(0.32, 0.28, dist) * smoothstep(0.22, 0.26, dist);
    lightning_intensity += ground_ring * ground_pulse * 0.8;
    
    var lightning_color: vec3f;
    if (lightning_intensity > 0.8) {
        lightning_color = LIGHTNING_BRIGHT;
    } else if (lightning_intensity > 0.4) {
        lightning_color = mix(LIGHTNING_CORE, LIGHTNING_BRIGHT, (lightning_intensity - 0.4) / 0.4);
    } else if (lightning_intensity > 0.1) {
        lightning_color = mix(LIGHTNING_BLUE, LIGHTNING_CORE, (lightning_intensity - 0.1) / 0.3);
    } else {
        lightning_color = mix(LIGHTNING_PURPLE, LIGHTNING_BLUE, lightning_intensity / 0.1);
    }
    
    let screen_flash = smoothstep(0.5, 1.0, lightning_intensity) * (sin(time * 30.0) * 0.5 + 0.5) * 0.2;
    lightning_color += LIGHTNING_BRIGHT * screen_flash;
    
    let final_color = lightning_color * circle_mask * feet_mask;
    let alpha = lightning_intensity * circle_mask * feet_mask;
    
    return vec4f(final_color, alpha);
}

// ============================================================
// POISON EFFECT
// ============================================================

const POISON_BRIGHT: vec3f = vec3f(0.6, 1.0, 0.3);
const POISON_GREEN: vec3f = vec3f(0.3, 0.8, 0.2);
const POISON_DARK: vec3f = vec3f(0.1, 0.4, 0.1);
const POISON_YELLOW: vec3f = vec3f(0.7, 0.9, 0.2);

fn poison_effect(uv: vec2<f32>, time: f32, dist: f32, angle: f32, feet_mask: f32) -> vec4<f32> {
    let circle_mask = 1.0 - smoothstep(0.25, 1.15, dist);
    
    let cloud_coord1 = uv * 3.0 + vec2f(time * 0.1, time * 0.15);
    let cloud_coord2 = uv * 2.5 - vec2f(time * 0.08, -time * 0.12);
    
    let cloud1 = fbm(cloud_coord1, 4);
    let cloud2 = fbm(cloud_coord2, 4);
    
    var poison_cloud = (cloud1 * 0.6 + cloud2 * 0.4);
    poison_cloud = pow(poison_cloud, 1.5);
    
    let bubble_coord = vec2f(angle * 4.0, dist * 8.0 - time * 0.5);
    let bubbles = fbm(bubble_coord, 3);
    let bubble_mask = smoothstep(0.55, 0.75, bubbles) * smoothstep(0.85, 0.7, bubbles) * (1.0 - dist * 0.5);
    poison_cloud += bubble_mask * 0.4;
    
    let tendril_angle = angle * 8.0 + time * 0.3;
    let tendril_base = sin(tendril_angle) * 0.5 + 0.5;
    let tendril_coord = vec2f(tendril_base * 3.0 + time * 0.2, dist * 6.0 - time * 0.6);
    let tendril_noise = fbm(tendril_coord, 3);
    let tendril_shape = pow(tendril_noise, 2.0);
    let tendril_mask = smoothstep(0.3, 0.6, tendril_shape) * smoothstep(0.4, 0.8, dist);
    poison_cloud += tendril_mask * 0.3;
    
    let ground_pulse = sin(time * 2.0) * 0.3 + 0.7;
    let ground_pool = smoothstep(0.35, 0.25, dist) * ground_pulse;
    poison_cloud += ground_pool * 0.5;
    
    let wave1 = sin(dist * 15.0 - time * 3.0) * 0.5 + 0.5;
    let wave2 = sin(dist * 20.0 - time * 4.0 + angle * 3.0) * 0.5 + 0.5;
    let miasma = (wave1 * 0.3 + wave2 * 0.2) * smoothstep(0.3, 0.7, dist);
    poison_cloud += miasma * 0.25;
    
    let particle_coord = uv * 12.0 + time * vec2f(0.15, 0.25);
    let particle_noise = fbm(particle_coord, 2);
    let particle_blob = pow(particle_noise, 3.0);
    let particle_size = hash(floor(particle_coord)) * 0.3 + 0.2;
    let particle_dist = length(fract(particle_coord) - 0.5);
    let particle_glow = smoothstep(particle_size, particle_size * 0.5, particle_dist) * particle_blob * 0.5;
    poison_cloud += particle_glow;
    
    var poison_color: vec3f;
    if (poison_cloud > 0.7) {
        poison_color = mix(POISON_BRIGHT, POISON_YELLOW, (poison_cloud - 0.7) / 0.3);
    } else if (poison_cloud > 0.4) {
        poison_color = mix(POISON_GREEN, POISON_BRIGHT, (poison_cloud - 0.4) / 0.3);
    } else if (poison_cloud > 0.2) {
        poison_color = mix(POISON_DARK, POISON_GREEN, (poison_cloud - 0.2) / 0.2);
    } else {
        poison_color = POISON_DARK * (poison_cloud / 0.2);
    }
    
    let glow_variation = sin(time * 1.5 + dist * 5.0) * 0.15 + 0.85;
    poison_color *= glow_variation;
    
    let edge_darken = smoothstep(1.0, 0.4, dist);
    poison_color *= 0.5 + edge_darken * 0.5;
    
    let final_color = poison_color * circle_mask * feet_mask;
    let alpha = poison_cloud * circle_mask * feet_mask;
    
    return vec4f(final_color, alpha);
}

// ============================================================
// HOLY EFFECT
// ============================================================

const GOLD: vec3f = vec3f(0.807843, 0.666667, 0.309804);
const GOLD_BRIGHT: vec3f = vec3f(1.0, 0.9, 0.6);
const GOLD_DIM: vec3f = vec3f(0.6, 0.5, 0.2);
const SPIKE_NUM: f32 = 9.0;
const SPIKE_LEN: f32 = 1.95;
const SPIKE_SPEED: f32 = 28.8;

fn holy_effect(uv: vec2<f32>, time: f32, dist: f32, angle: f32, feet_mask: f32) -> vec4<f32> {
    let aura_mask = 1.0 - smoothstep(0.25, 1.1, dist);
    let spike_start_mask = smoothstep(0.28, 0.35, dist);
    
    // Golden flowing aura layer
    let flow_coord = vec2f(angle * 3.0, dist * 4.0 - time * 1.2);
    let turbulence1 = fbm(flow_coord * 2.0 + time * 0.4, 4);
    let turbulence2 = fbm(flow_coord * 2.5 - time * 0.6, 4);
    
    var golden_flow = turbulence1 * 0.6 + turbulence2 * 0.4;
    golden_flow *= (1.0 - dist * 0.7);
    golden_flow = pow(golden_flow, 1.8);
    
    let pulse = sin(time * 3.0 + angle * 2.0) * 0.1 + 0.9;
    golden_flow *= pulse;
    
    var aura_color: vec3f;
    if (golden_flow > 0.6) {
        aura_color = mix(GOLD, GOLD_BRIGHT, (golden_flow - 0.6) / 0.4);
    } else if (golden_flow > 0.3) {
        aura_color = mix(GOLD_DIM, GOLD, (golden_flow - 0.3) / 0.3);
    } else {
        aura_color = GOLD_DIM * (golden_flow / 0.3);
    }
    
    let shimmer = fbm(vec2f(angle * 4.0, dist * 6.0) + time * 1.5, 4);
    aura_color += GOLD_BRIGHT * shimmer * 0.2 * (1.0 - dist * 0.8);
    
    let aura_alpha = golden_flow * aura_mask * feet_mask * 0.7;
    
    // Spikes layer
    var pc = vec2f(atan2(uv.x, uv.y), length(uv));
    let x = (pc.x / PI) * SPIKE_NUM;
    let f_x = fract(x);
    let f2_x = fract(1.0 - x);
    var m = min(f_x, f2_x);
    m = m * SPIKE_LEN - pc.y;
    
    var spike_intensity = smoothstep(0.03, 0.9, m);
    let rate: f32 = time * SPIKE_SPEED;
    let idx: f32 = rate % (SPIKE_NUM * 2.0) - (SPIKE_NUM - 1.0);
    var x_clamp = -floor(x);
    let is_focused_spike = step(0.5, abs(idx - x_clamp));
    
    var spike_col = mix(GOLD / 0.15, GOLD * 0.54, is_focused_spike);
    spike_intensity *= spike_start_mask;
    
    // Ground ring with rotating glow
    let focused_angle = (idx / SPIKE_NUM) * PI;
    let angle_diff = abs(atan2(sin(angle - focused_angle), cos(angle - focused_angle)));
    let angular_glow = 1.0 - smoothstep(0.0, PI * 0.8, angle_diff);
    let ground_ring = smoothstep(0.285, 0.28, dist) * smoothstep(0.26, 0.265, dist);
    let base_brightness = 0.09;
    let glow_brightness = angular_glow * 0.81;
    let background_intensity = ground_ring * (base_brightness + glow_brightness) * feet_mask;
    var background_col = (GOLD * 0.54) * background_intensity;
    
    var final_col = mix(background_col, spike_col, spike_intensity);
    let spike_alpha = m * spike_intensity;
    let spike_layer_alpha = max(spike_alpha, background_intensity);
    
    final_col = mix(aura_color, final_col, spike_layer_alpha);
    let final_alpha = max(aura_alpha, spike_layer_alpha);
    
    var out = vec4f(final_col, final_alpha);
    out *= feet_mask;
    
    return out;
}

// ============================================================
// MAIN FRAGMENT SHADER
// ============================================================

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    uv = uv * 2.0 - 1.0;

    let time = globals.time;
    let dist = length(uv);
    let angle = atan2(uv.y, uv.x);
    let feet_mask = smoothstep(0.2, 0.28, dist);
    
    var color: vec4<f32>;
    
    // Switch between effects
    switch material.effect_type {
        case EFFECT_FIRE: {
            color = fire_effect(uv, time, dist, angle, feet_mask);
        }
        case EFFECT_LIGHTNING: {
            color = lightning_effect(uv, time, dist, angle, feet_mask);
        }
        case EFFECT_POISON: {
            color = poison_effect(uv, time, dist, angle, feet_mask);
        }
        case EFFECT_HOLY: {
            color = holy_effect(uv, time, dist, angle, feet_mask);
        }
        default: {
            color = vec4<f32>(1.0, 1.0, 1.0, 0.5);
        }
    }
    
    // Apply intensity
    color = color * material.intensity;
    
    // Apply color tint
    let tint = vec3<f32>(material.color_r, material.color_g, material.color_b);
    color = vec4<f32>(color.rgb * tint, color.a);
    
    return color;
}