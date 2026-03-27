// Display pass (equivalent to Image tab in Shadertoy).
// Colorizes the GOL state texture using alive/dead colors from the material uniform.

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var display_texture: texture_2d<f32>;
@group(2) @binding(1) var display_sampler: sampler;
@group(2) @binding(2) var<uniform> alive_color: vec4<f32>;
@group(2) @binding(3) var<uniform> dead_color: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let cell = textureSampleLevel(display_texture, display_sampler, in.uv, 0.0);
    // The simulation encodes alive as green=1 (see gol_simulation.wgsl ON_COL)
    if cell.g > 0.5 {
        return alive_color;
    }
    return dead_color;
}
