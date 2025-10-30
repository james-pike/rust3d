#import bevy_pbr::forward_io::{VertexOutput, FragmentOutput};
#import bevy_pbr::mesh_view_bindings::globals
#import bevy_render::view::View

/// Keep up-to-date with the rust definition!
struct AuraMaterial {
    unused: f32,
}

@group(0) @binding(0)   var<uniform> view: View;
@group(3) @binding(100) var<uniform> aura_mat: AuraMaterial;

const PI: f32 = 3.141592653589;

// Lightning colors
const LIGHTNING_BRIGHT: vec3f = vec3f(0.9, 0.95, 1.0);
const LIGHTNING_CORE: vec3f = vec3f(0.6, 0.8, 1.0);
const LIGHTNING_BLUE: vec3f = vec3f(0.3, 0.5, 0.9);
const LIGHTNING_PURPLE: vec3f = vec3f(0.5, 0.3, 0.8);

// Simple hash for randomness
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

// Generate lightning bolt path
fn lightning_bolt(p: vec2f, seed: f32, time: f32) -> f32 {
    var dist = 1000.0;
    var current_pos = vec2f(0.0, 0.0);
    
    // Create jagged lightning path with fewer segments for performance
    for (var i = 0; i < 6; i++) {
        let t = f32(i) / 6.0;
        let next_seed = hash(vec2f(seed, f32(i)));
        
        // Zigzag pattern
        let offset = (hash(vec2f(seed + f32(i), time * 0.1)) - 0.5) * 0.3;
        let next_pos = vec2f(
            cos(seed * 6.28 + t * 0.5) * (0.4 + t * 0.6) + offset,
            sin(seed * 6.28 + t * 0.5) * (0.4 + t * 0.6)
        );
        
        // Distance to line segment
        let pa = p - current_pos;
        let ba = next_pos - current_pos;
        let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
        let segment_dist = length(pa - ba * h);
        
        dist = min(dist, segment_dist);
        current_pos = next_pos;
    }
    
    return dist;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    uv = uv * 2.0 - 1.0;

    let time = globals.time;
    
    // Distance and angle from center
    let dist = length(uv);
    let angle = atan2(uv.y, uv.x);
    
    // Mask center for character feet
    let feet_mask = smoothstep(0.2, 0.28, dist);
    
    // Main circular mask
    let circle_mask = 1.0 - smoothstep(0.25, 1.2, dist);
    
    // Create multiple lightning bolts
    var lightning_intensity = 0.0;
    let num_bolts = 8.0;
    
    for (var i = 0.0; i < num_bolts; i += 1.0) {
        // Each bolt has different timing for variety
        let bolt_time = time * 2.0 + i * 0.5;
        let bolt_phase = fract(bolt_time * 0.3);
        
        // Bolts appear and fade quickly
        let bolt_visibility = smoothstep(0.0, 0.1, bolt_phase) * (1.0 - smoothstep(0.2, 0.4, bolt_phase));
        
        if (bolt_visibility > 0.01) {
            let bolt_seed = i / num_bolts + floor(bolt_time * 0.3) * 0.123;
            let bolt_dist = lightning_bolt(uv, bolt_seed, time);
            
            // Thicker core with bright glow
            let bolt_core = smoothstep(0.02, 0.0, bolt_dist);
            let bolt_glow = smoothstep(0.15, 0.0, bolt_dist);
            
            lightning_intensity += (bolt_core + bolt_glow * 0.5) * bolt_visibility;
        }
    }
    
    // Electric arcs between bolts
    let arc_pattern = sin(angle * 20.0 + time * 10.0) * sin(dist * 30.0 - time * 15.0);
    let arcs = smoothstep(0.7, 0.9, arc_pattern) * 0.3;
    lightning_intensity += arcs;
    
    // Ambient electric field
    let field_noise = hash(uv * 10.0 + time * 0.5);
    let electric_field = field_noise * 0.15 * (1.0 - dist * 0.5);
    lightning_intensity += electric_field;
    
    // Pulsing ground ring
    let ground_pulse = sin(time * 8.0) * 0.5 + 0.5;
    let ground_ring = smoothstep(0.32, 0.28, dist) * smoothstep(0.22, 0.26, dist);
    lightning_intensity += ground_ring * ground_pulse * 0.8;
    
    // Color based on intensity
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
    
    // Add screen-space flicker
    let screen_flash = smoothstep(0.5, 1.0, lightning_intensity) * (sin(time * 30.0) * 0.5 + 0.5) * 0.2;
    lightning_color += LIGHTNING_BRIGHT * screen_flash;
    
    // Apply masks
    let final_color = lightning_color * circle_mask * feet_mask;
    let alpha = lightning_intensity * circle_mask * feet_mask;
    
    return vec4f(final_color, alpha);
}

fn sdCircle(p: vec2f, r: f32) -> f32 {
    return length(p) - r;
}