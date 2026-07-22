@group(1)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
@group(1)
@binding(1)
var icon_sampler: sampler;
@group(0)
@binding(0)
var icon_texture: texture_2d<f32>;
struct Vertex {
    @location(0) vertex_pos: vec2<f32>,
    @location(1) section: vec4<f32>,
    @location(2) layer: f32,
    @location(3) color: vec4<f32>,
    @location(4) screen_px_range: f32,
    @location(5) opacity: f32,
};
struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex: vec2<f32>,
    @location(2) screen_px_range: f32,
};
@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let position = vec4<f32>(vertex.section.xy + vertex.section.zw * vertex.vertex_pos, vertex.layer, 1.0);
    return Fragment(
        viewport * position,
        vertex.color * vec4f(1.0, 1.0, 1.0, vertex.opacity),
        vertex.vertex_pos,
        vertex.screen_px_range,
    );
}
fn median3(a: f32, b: f32, c: f32) -> f32 {
    return max(min(a, b), min(max(a, b), c));
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    // Multi-channel signed distance: the median of R/G/B reconstructs the true distance while
    // preserving sharp corners (a single channel would round them off). 0.5 is the edge.
    let msdf = textureSample(icon_texture, icon_sampler, frag.tex);
    let dist = median3(msdf.r, msdf.g, msdf.b) - 0.5;
    // Convert the normalized distance to a screen-space, ~1px-wide anti-aliased edge using how
    // many screen pixels the field's range spans at this instance's on-screen size.
    let coverage = clamp(dist * max(frag.screen_px_range, 1.0) + 0.5, 0.0, 1.0);
    return vec4<f32>(frag.color.rgb, frag.color.a * coverage);
}
