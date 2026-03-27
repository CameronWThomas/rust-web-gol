// GOL simulation pass (equivalent to BufferA in Shadertoy).
// Reads from `input_texture` (last frame), writes the next generation.
// On frame 0 the CPU will have initialised the texture with random data,
// so we only need the update logic here.

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> params: SimParams;

struct SimParams {
    resolution: vec2<f32>,
    frame: u32,
    _pad: u32,
}

const ON_COL:  vec4<f32> = vec4<f32>(0.0, 1.0, 0.0, 1.0);
const OFF_COL: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 1.0);

// Sample a cell at integer pixel coordinate, wrapping at edges.
fn sample_cell(coord: vec2<f32>) -> vec4<f32> {
    var c = coord;
    // wrap
    c = ((c % params.resolution) + params.resolution) % params.resolution;
    let uv = (c + 0.5) / params.resolution;
    return textureSampleLevel(input_texture, input_sampler, uv, 0.0);
}

fn is_alive(coord: vec2<f32>) -> bool {
    let cur = sample_cell(coord);
    let cur_alive = all(cur == ON_COL);

    var alive_count: i32 = 0;
    for (var dx: i32 = -1; dx <= 1; dx++) {
        for (var dy: i32 = -1; dy <= 1; dy++) {
            if dx == 0 && dy == 0 { continue; }
            let neighbour = sample_cell(coord + vec2<f32>(f32(dx), f32(dy)));
            if all(neighbour == ON_COL) {
                alive_count += 1;
            }
        }
    }

    if !cur_alive {
        return alive_count == 3;
    }
    if alive_count < 2 { return false; }
    if alive_count <= 3 { return true; }
    return false;
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    // Fullscreen triangle trick
    var out: VertexOutput;
    let x = f32((vi & 1u) * 2u) - 1.0;
    let y = f32((vi & 2u)) - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    return out;
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let coord = floor(frag_coord.xy);
    if is_alive(coord) {
        return ON_COL;
    }
    return OFF_COL;
}
