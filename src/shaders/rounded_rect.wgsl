// rounded_rect.wgsl

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_position: vec2<f32>,
    @location(1) @interpolate(flat) instance_index: u32,
};

struct RectUniform {
    position: vec2<f32>,
    size: vec2<f32>,
    viewport: vec2<f32>,
    antialias_padding: f32,
    padding: f32,
    radii: vec4<f32>,
    color: vec4<f32>,
    border_color: vec4<f32>,
    border: vec4<f32>,
};

@group(0) @binding(0)
var<storage, read> rectangles: array<RectUniform>;

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

    var out: VertexOutput;
    let rect = rectangles[instance_index];
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
    out.instance_index = instance_index;
    return out;
}

fn corner_radius(p: vec2<f32>, radii: vec4<f32>) -> f32 {
    if p.y <= 0.0 {
        return select(radii.x, radii.y, p.x > 0.0);
    }
    return select(radii.w, radii.z, p.x > 0.0);
}

fn sdf_rounded_box(p: vec2<f32>, half_size: vec2<f32>, radii: vec4<f32>) -> f32 {
    let radius = corner_radius(p, radii);
    let q = abs(p) - half_size + vec2<f32>(radius, radius);
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let rect = rectangles[in.instance_index];
    if rect.size.x <= 0.0 || rect.size.y <= 0.0 {
        discard;
    }
    let half_size = rect.size * 0.5;
    let dist = sdf_rounded_box(in.local_position, half_size, rect.radii);

    let antialias_width = fwidth(dist);
    let outer_coverage = 1.0 - smoothstep(
        -0.5 * antialias_width,
        0.5 * antialias_width,
        dist,
    );
    let has_interior = select(0.0, 1.0, rect.border.x < min(half_size.x, half_size.y));
    let inner_coverage = has_interior * (1.0 - smoothstep(
        -0.5 * antialias_width,
        0.5 * antialias_width,
        dist + rect.border.x,
    ));
    let border_coverage = max(outer_coverage - inner_coverage, 0.0);
    let premultiplied = rect.color.rgb * rect.color.a * inner_coverage
        + rect.border_color.rgb * rect.border_color.a * border_coverage;
    let alpha = rect.color.a * inner_coverage
        + rect.border_color.a * border_coverage;
    let color = premultiplied / max(alpha, 0.000001);

    return vec4<f32>(color, alpha);
}
