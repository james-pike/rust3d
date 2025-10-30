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

// Fire colors
const FIRE_YELLOW: vec3f = vec3f(1.0, 0.9, 0.3);
const FIRE_ORANGE: vec3f = vec3f(1.0, 0.5, 0.1);
const FIRE_RED: vec3f = vec3f(0.9, 0.2, 0.05);
const FIRE_DARK: vec3f = vec3f(0.3, 0.05, 0.0);

// Simple noise function
fn hash(p: vec2f) -> f32 {
    let p3 = fract(vec3f(p.x, p.y, p.x) * 0.13);
    let dot_product = dot(p3, vec3f(p3.y, p3.z, p3.x) + 3.333);
    return fract((p3.x + p3.y) * dot_product);
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

// Fractal brownian motion for fire turbulence
fn fbm(p: vec2f) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var p_var = p;
    
    for (var i = 0; i < 5; i++) {
        value += amplitude * noise(p_var * frequency);
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    
    return value;
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
    
    // Main circular mask with soft edges
    let circle_mask = 1.0 - smoothstep(0.25, 1.1, dist);
    
    // Create rising flame effect using noise
    let flame_coord = vec2f(angle * 3.0, dist * 4.0 - time * 1.5);
    let turbulence1 = fbm(flame_coord * 2.0 + time * 0.5);
    let turbulence2 = fbm(flame_coord * 3.0 - time * 0.8);
    
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
    let hot_spots = fbm(vec2f(angle * 5.0, dist * 8.0) + time * 2.0);
    fire_color += FIRE_YELLOW * hot_spots * 0.3 * (1.0 - dist);
    
    // Inner glow ring
    let inner_glow = smoothstep(0.35, 0.25, dist) * 0.5;
    fire_color += FIRE_ORANGE * inner_glow;
    
    // Apply masks
    let final_color = fire_color * circle_mask * feet_mask;
    let alpha = flame * circle_mask * feet_mask;
    
    return vec4f(final_color, alpha);
}

fn sdCircle(p: vec2f, r: f32) -> f32 {
    return length(p) - r;
}