@group(1)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
@group(1)
@binding(1)
var image_sampler: sampler;
@group(0)
@binding(0)
var image_texture: texture_2d<f32>;
struct Vertex{
    @location(0) vertex_pos: vec2<f32>,
    @location(1) tx_index: vec2<u32>,
    @location(2) section: vec4<f32>,
    @location(3) layer: f32,
    @location(4) tx_coords: vec4<f32>,
    @location(5) opacity: f32,
    @location(6) radii: vec4<f32>,
};
struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) tx_coords: vec2<f32>,
    @location(1) opacity: f32,
    @location(2) @interpolate(flat) section: vec4<f32>,
    @location(3) @interpolate(flat) radii: vec4<f32>,
};
@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let position = vec4<f32>(vertex.section.xy + vertex.vertex_pos * vertex.section.zw, vertex.layer, 1.0);
    let tx_coords = vec2<f32>(vertex.tx_coords[vertex.tx_index.x], vertex.tx_coords[vertex.tx_index.y]);
    return Fragment(viewport * position, tx_coords, vertex.opacity, vertex.section, vertex.radii);
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    // The pixels still cover the whole box -- the corners are cut by alpha, not by
    // cropping the sample, so a `Crop` view stays full-bleed right up to the curve.
    let half_extent = frag.section.zw * 0.5;
    let d = sd_rounded_box(
        frag.position.xy - (frag.section.xy + half_extent),
        half_extent,
        frag.radii
    );
    let color = textureSample(image_texture, image_sampler, frag.tx_coords);
    return color * vec4f(1.0, 1.0, 1.0, frag.opacity * sd_coverage(d));
}
