@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
struct Vertex {
    @location(0) vertex_pos: vec2<f32>,
    @location(1) section: vec4<f32>,
    @location(2) layer_and_weight: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) radii: vec4<f32>,
};
struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) @interpolate(flat) weight: f32,
    @location(2) @interpolate(flat) section: vec4<f32>,
    @location(3) @interpolate(flat) radii: vec4<f32>,
};
@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let position = vec4<f32>(
        vertex.section.xy + vertex.vertex_pos * vertex.section.zw,
        vertex.layer_and_weight.x,
        1.0
    );
    return Fragment(
        viewport * position,
        vertex.color,
        vertex.layer_and_weight.y,
        vertex.section,
        vertex.radii
    );
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    let half_extent = frag.section.zw * 0.5;
    var d = sd_rounded_box(
        frag.position.xy - (frag.section.xy + half_extent),
        half_extent,
        frag.radii
    );
    // A negative weight is `Outline`'s "no outline" -- the only reading that leaves a
    // zero-width outline meaning a hairline rather than a solid fill.
    if frag.weight >= 0.0 {
        d = sd_outline(d, frag.weight);
    }
    return vec4<f32>(frag.color.rgb, frag.color.a * sd_coverage(d));
}
