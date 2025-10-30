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

// Poison colors
const POISON_BRIGHT: vec3f = vec3f(0.6, 1.0, 0.3);
const POISON_GREEN: vec3f = vec3f(0.3, 0.8, 0.2);
const POISON_DARK: vec3f = vec3f(0.1, 0.4, 0.1);
const POISON_YELLOW: vec3f = vec3f(0.7, 0.9, 0.2);

// Hash functions for noise
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

// Fractal noise for poison clouds
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
    let circle_mask = 1.0 - smoothstep(0.25, 1.15, dist);
    
    // Slow swirling poison clouds
    let cloud_coord1 = uv * 3.0 + vec2f(time * 0.1, time * 0.15);
    let cloud_coord2 = uv * 2.5 - vec2f(time * 0.08, -time * 0.12);
    
    let cloud1 = fbm(cloud_coord1, 4);
    let cloud2 = fbm(cloud_coord2, 4);
    
    // Combine clouds with rotation
    var poison_cloud = (cloud1 * 0.6 + cloud2 * 0.4);
    poison_cloud = pow(poison_cloud, 1.5);
    
    // Poison bubbles rising from ground - more organic
    let bubble_coord = vec2f(angle * 4.0, dist * 8.0 - time * 0.5);
    let bubbles = fbm(bubble_coord, 3);
    // Softer, rounder bubbles
    let bubble_mask = smoothstep(0.55, 0.75, bubbles) * smoothstep(0.85, 0.7, bubbles) * (1.0 - dist * 0.5);
    
    // Add bubbles to cloud
    poison_cloud += bubble_mask * 0.4;
    
    // Organic tendril shapes instead of drips
    let tendril_angle = angle * 8.0 + time * 0.3;
    let tendril_base = sin(tendril_angle) * 0.5 + 0.5;
    let tendril_coord = vec2f(tendril_base * 3.0 + time * 0.2, dist * 6.0 - time * 0.6);
    let tendril_noise = fbm(tendril_coord, 3);
    // Create blob-like tendrils with smooth falloff
    let tendril_shape = pow(tendril_noise, 2.0);
    let tendril_mask = smoothstep(0.3, 0.6, tendril_shape) * smoothstep(0.4, 0.8, dist);
    poison_cloud += tendril_mask * 0.3;
    
    // Pulsing toxic ground pool
    let ground_pulse = sin(time * 2.0) * 0.3 + 0.7;
    let ground_pool = smoothstep(0.35, 0.25, dist) * ground_pulse;
    poison_cloud += ground_pool * 0.5;
    
    // Miasma waves expanding from center
    let wave1 = sin(dist * 15.0 - time * 3.0) * 0.5 + 0.5;
    let wave2 = sin(dist * 20.0 - time * 4.0 + angle * 3.0) * 0.5 + 0.5;
    let miasma = (wave1 * 0.3 + wave2 * 0.2) * smoothstep(0.3, 0.7, dist);
    poison_cloud += miasma * 0.25;
    
    // Organic virus-like particles floating around
    let particle_coord = uv * 12.0 + time * vec2f(0.15, 0.25);
    let particle_noise = fbm(particle_coord, 2);
    // Create blob-like organic particles instead of square grid
    let particle_blob = pow(particle_noise, 3.0);
    let particle_size = hash(floor(particle_coord)) * 0.3 + 0.2;
    let particle_dist = length(fract(particle_coord) - 0.5);
    let particle_glow = smoothstep(particle_size, particle_size * 0.5, particle_dist) * particle_blob * 0.5;
    poison_cloud += particle_glow;
    
    // Color gradient based on intensity
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
    
    // Add sickly glow variation
    let glow_variation = sin(time * 1.5 + dist * 5.0) * 0.15 + 0.85;
    poison_color *= glow_variation;
    
    // Darker edges for depth
    let edge_darken = smoothstep(1.0, 0.4, dist);
    poison_color *= 0.5 + edge_darken * 0.5;
    
    // Apply masks
    let final_color = poison_color * circle_mask * feet_mask;
    let alpha = poison_cloud * circle_mask * feet_mask;
    
    return vec4f(final_color, alpha);
}

fn sdCircle(p: vec2f, r: f32) -> f32 {
    return length(p) - r;
}