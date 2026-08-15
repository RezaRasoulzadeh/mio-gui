// image.wgsl

struct ImageUniform {
    position: vec2<f32>,
    size: vec2<f32>,
    clip_position: vec2<f32>,
    clip_size: vec2<f32>,
    viewport: vec2<f32>,
    tint: vec4<f32>,
    mirror_horizontal: u32,
    has_tint: u32,
    padding: vec2<u32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) pixel_position: vec2<f32>,
};

@group(0) @binding(0)
var image_texture: texture_2d<f32>;

@group(0) @binding(1)
var image_sampler: sampler;

@group(0) @binding(2)
var<storage, read> image: ImageUniform;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    let corner = corners[index];
    let pixel_position = image.position + corner * image.size;
    var out: VertexOutput;
    out.clip_position = vec4<f32>(
        pixel_position.x / image.viewport.x * 2.0 - 1.0,
        1.0 - pixel_position.y / image.viewport.y * 2.0,
        0.0,
        1.0,
    );
    out.uv = vec2<f32>(select(corner.x, 1.0 - corner.x, image.mirror_horizontal != 0u), corner.y);
    out.pixel_position = pixel_position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let clip_end = image.clip_position + image.clip_size;
    if any(in.pixel_position < image.clip_position) || any(in.pixel_position >= clip_end) {
        discard;
    }
    let sample = textureSample(image_texture, image_sampler, in.uv);
    if image.has_tint != 0u {
        return vec4<f32>(image.tint.rgb, sample.a * image.tint.a);
    }
    return sample;
}
