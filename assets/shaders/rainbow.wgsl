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

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    uv = uv * 2.0 - 1.0;

    let time = globals.time;
    
    // Simple rainbow test - entire screen cycles through colors
    let r = sin(time * 2.0) * 0.5 + 0.5;
    let g = sin(time * 2.0 + 2.0) * 0.5 + 0.5;
    let b = sin(time * 2.0 + 4.0) * 0.5 + 0.5;
    
    // Distance from center
    let dist = length(uv);
    
    // Simple circular gradient
    let circle = 1.0 - smoothstep(0.3, 1.0, dist);
    
    // Rotating stripe pattern
    let angle = atan2(uv.y, uv.x);
    let stripes = sin(angle * 10.0 + time * 5.0) * 0.5 + 0.5;
    
    var col = vec3f(r, g, b) * circle * stripes;
    
    // Mask center
    let feet_mask = smoothstep(0.25, 0.3, dist);
    
    return vec4f(col * feet_mask, circle * feet_mask);
}

fn sdCircle(p: vec2f, r: f32) -> f32 {
    return length(p) - r;
}