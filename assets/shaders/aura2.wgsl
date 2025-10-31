#import bevy_pbr::forward_io::{VertexOutput, FragmentOutput};
#import bevy_pbr::mesh_view_bindings::globals
#import bevy_render::view::View

/// Keep up-to-date with the rust definition!
struct AuraMaterial {
    unused: f32,
}

@group(0) @binding(0)   var<uniform> view: View;
@group(3) @binding(100) var<uniform> aura_mat: AuraMaterial;

// Colour picker tells us the values of the original..
// Darkish
// #CEAA4F
const GOLD: vec3f = vec3f(0.807843, 0.666667, 0.309804);
const GOLD_BRIGHT: vec3f = vec3f(1.0, 0.9, 0.6);
const GOLD_DIM: vec3f = vec3f(0.6, 0.5, 0.2);
const SPIKE_NUM: f32 = 9.0;
const SPIKE_LEN: f32 = 1.95;
const SPIKE_SPEED: f32 = 28.8;
const PI: f32 = 3.141592653589;

// Simple hash for randomness
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

// Fractal brownian motion for flowing effect
fn fbm(p: vec2f) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var p_var = p;
    
    for (var i = 0; i < 4; i++) {
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
    
    // Mask center for character feet (smaller diameter like lightning)
    let feet_mask = smoothstep(0.2, 0.28, dist);
    
    // Main circular mask - only for the small circle, no extended range
    let circle_mask = 1.0;
    
    // Mask to push spikes outward from the circle
    let spike_start_mask = smoothstep(0.28, 0.35, dist);
    
    // Extended circular mask for golden aura underneath
    let aura_mask = 1.0 - smoothstep(0.25, 1.1, dist);

    // === GOLDEN FLOWING AURA LAYER (underneath) ===
    // Create flowing golden energy from center outward
    let flow_coord = vec2f(angle * 3.0, dist * 4.0 - time * 1.2);
    let turbulence1 = fbm(flow_coord * 2.0 + time * 0.4);
    let turbulence2 = fbm(flow_coord * 2.5 - time * 0.6);
    
    // Combine turbulence for flowing shape
    var golden_flow = turbulence1 * 0.6 + turbulence2 * 0.4;
    
    // Make flow fade with distance and pulse gently
    golden_flow *= (1.0 - dist * 0.7);
    golden_flow = pow(golden_flow, 1.8);
    
    // Gentle pulsing
    let pulse = sin(time * 3.0 + angle * 2.0) * 0.1 + 0.9;
    golden_flow *= pulse;
    
    // Add flowing tendrils
   // let tendril_angle = angle * 6.0 + time * 2.0;
   // let tendrils = sin(tendril_angle) * 0.5 + 0.5;
   // let tendril_intensity = smoothstep(0.5, 0.9, dist) * tendrils * 0.25;
   // golden_flow += tendril_intensity;
    
    // Golden color gradient - darker at edges, brighter at center
    var aura_color: vec3f;
    if (golden_flow > 0.6) {
        aura_color = mix(GOLD, GOLD_BRIGHT, (golden_flow - 0.6) / 0.4);
    } else if (golden_flow > 0.3) {
        aura_color = mix(GOLD_DIM, GOLD, (golden_flow - 0.3) / 0.3);
    } else {
        aura_color = GOLD_DIM * (golden_flow / 0.3);
    }
    
    // Add shimmering highlights
    let shimmer = fbm(vec2f(angle * 4.0, dist * 6.0) + time * 1.5);
    aura_color += GOLD_BRIGHT * shimmer * 0.2 * (1.0 - dist * 0.8);
    
    let aura_alpha = golden_flow * aura_mask * feet_mask * 0.7;

    // === SPIKES LAYER (on top) ===

    // Move into polar coordinates for spikes
    var pc = vec2f(atan2(uv.x, uv.y), length(uv));
    let x = (pc.x / PI) * SPIKE_NUM;

    // Make the spikes
    let f_x = fract(x);
    let f2_x = fract(1.0 - x);
    var m = min(f_x, f2_x);
    m = m * SPIKE_LEN - pc.y;
    
    // Draw the spikes
    var spike_intensity = smoothstep(0.03, 0.9, m);

    let rate: f32 = time * SPIKE_SPEED;
    let idx: f32 = rate % (SPIKE_NUM * 2.0) - (SPIKE_NUM - 1.0);
    var x_clamp = -floor(x);
    let is_focused_spike = step(0.5, abs(idx - x_clamp));
    
    // Spike color with focus effect
    var spike_col = mix(GOLD / 0.15, GOLD * 0.54, is_focused_spike);
    
    // Apply spike start mask to create space between circle and spikes
    spike_intensity *= spike_start_mask;
    
    // Ground ring only - thinner to match spike design
    var background_intensity = 0.0;
    
    // Calculate angular distance from focused spike for directional glow
    let focused_angle = (idx / SPIKE_NUM) * PI;
    let angle_diff = abs(atan2(sin(angle - focused_angle), cos(angle - focused_angle)));
    let angular_glow = 1.0 - smoothstep(0.0, PI * 0.8, angle_diff); // Longer tail
    
    let ground_ring = smoothstep(0.285, 0.28, dist) * smoothstep(0.26, 0.265, dist);
    
    // Circle barely visible except where bright spot is, with gradual falloff
    let base_brightness = 0.09; // Very dim when not selected
    let glow_brightness = angular_glow * 0.81; // Strong rotating glow with long tail (10% reduction)
    background_intensity = ground_ring * (base_brightness + glow_brightness);
    
    // Background is just the ground ring in darker gold
    var background_col = (GOLD * 0.54) * background_intensity; // Match spike darkening
    
    // Apply edge fade mask to circle (matching spikes behavior)
    background_intensity *= feet_mask;
    
    // Combine spikes with minimal background
    var final_col = mix(background_col, spike_col, spike_intensity);
    
    // Calculate alpha - spikes and ground ring only
    let spike_alpha = m * spike_intensity;
    let background_alpha = background_intensity;
    let spike_layer_alpha = max(spike_alpha, background_alpha);
    
    // Blend aura underneath with spikes on top
    final_col = mix(aura_color, final_col, spike_layer_alpha);
    let final_alpha = max(aura_alpha, spike_layer_alpha);
    
    // Apply masks
    var out = vec4f(final_col, final_alpha);
    out *= feet_mask;

    return out;
}

fn sdCircle(p: vec2f, r: f32) -> f32 {
    return length(p) - r;
}