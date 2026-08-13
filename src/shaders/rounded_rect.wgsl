// rounded_rect.wgsl

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_position: vec2<f32>,
};

struct RectUniform {
    position: vec2<f32>,
    size: vec2<f32>,
    viewport: vec2<f32>,
    radius: f32,
    antialias_padding: f32,
    color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> rect: RectUniform;

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

    var out: VertexOutput;
    let corner = corners[index];
    let padding = vec2<f32>(rect.antialias_padding, rect.antialias_padding);
    let expanded_position = rect.position - padding;
    let expanded_size = rect.size + padding * 2.0;
    let pixel_position = expanded_position + corner * expanded_size;
    let clip_position = vec2<f32>(
        pixel_position.x / rect.viewport.x * 2.0 - 1.0,
        1.0 - pixel_position.y / rect.viewport.y * 2.0,
    );
    out.clip_position = vec4<f32>(clip_position, 0.0, 1.0);
    out.local_position = (corner - vec2<f32>(0.5, 0.5)) * expanded_size;
    return out;
}

fn sdf_rounded_box(p: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(p) - half_size + vec2<f32>(radius, radius);
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let half_size = rect.size * 0.5;
    let dist = sdf_rounded_box(in.local_position, half_size, rect.radius);

    let antialias_width = fwidth(dist);
    let alpha = 1.0 - smoothstep(-0.5 * antialias_width, 0.5 * antialias_width, dist);

    return vec4<f32>(rect.color.rgb, rect.color.a * alpha);
}
