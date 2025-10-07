#import bevy_sprite::mesh2d_view_bindings::globals
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import shadplay::shader_utils::common::{NEG_HALF_PI, shader_toy_default, rotate2D}
#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;

const SPEED: f32 = 1.0;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    uv = (uv * 2.0) - 1.0;
    let resolution = view.viewport.zw;
    let t = globals.time * SPEED;
    uv.x *= resolution.x / resolution.y;
    uv *= rotate2D(NEG_HALF_PI);

    return vec4f(shader_toy_default(t, uv), 0.3);
}