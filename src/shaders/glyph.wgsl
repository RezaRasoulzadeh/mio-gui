// glyph.wgsl

struct GlyphInstance {
    position: vec2<f32>,
    size: vec2<f32>,
    uv_min: vec2<f32>,
    uv_max: vec2<f32>,
    viewport: vec2<f32>,
    padding: vec2<f32>,
    color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) instance_index: u32,
};

@group(0) @binding(0)
var glyph_texture: texture_2d<f32>;

@group(0) @binding(1)
var glyph_sampler: sampler;

@group(0) @binding(2)
var<storage, read> glyphs: array<GlyphInstance>;

@vertex
fn vs_main(
    @builtin(vertex_index) index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    let glyph = glyphs[instance_index];
    let corner = corners[index];
    let pixel_position = glyph.position + corner * glyph.size;
    let clip_position = vec2<f32>(
        pixel_position.x / glyph.viewport.x * 2.0 - 1.0,
        1.0 - pixel_position.y / glyph.viewport.y * 2.0,
    );

    var out: VertexOutput;
    out.clip_position = vec4<f32>(clip_position, 0.0, 1.0);
    out.uv = mix(glyph.uv_min, glyph.uv_max, corner);
    out.instance_index = instance_index;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let glyph = glyphs[in.instance_index];
    let sample = textureSample(glyph_texture, glyph_sampler, in.uv);
    return vec4<f32>(sample.rgb * glyph.color.rgb, sample.a * glyph.color.a);
}
