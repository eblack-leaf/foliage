@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
struct Vertex {
    @location(0) vertex_pos: vec2<f32>,
    @location(1) vertex_tx: vec2<u32>,
    @location(2) position: vec2<f32>,
    @location(3) area: vec2<f32>,
    @location(4) layer: f32,
    @location(5) color: vec4<f32>,
    @location(6) ring: f32,
    @location(7) tx: vec4<f32>,
    @location(8) prog: vec2<f32>,
};
struct VertexFragment {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coordinates: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) ring: f32,
    @location(4) prog: vec2<f32>,
};
@vertex
fn vertex_entry(vertex: Vertex) -> VertexFragment {
    let pos = vec4<f32>(vertex.position + vertex.vertex_pos * vertex.area, vertex.layer, 1.0);
    let tex = vec2<f32>(vertex.tx[vertex.vertex_tx.x], vertex.tx[vertex.vertex_tx.y]);
    return VertexFragment(viewport * pos, tex, vertex.color, vertex.ring, vertex.prog);
}
@group(0)
@binding(1)
var circle_texture: texture_2d<f32>;
@group(0)
@binding(2)
var circle_sampler: sampler;
@fragment
fn fragment_entry (vertex_fragment: VertexFragment) -> @location(0) vec4<f32> {
    let tex_data = textureSample(
        circle_texture,
        circle_sampler,
        vertex_fragment.texture_coordinates,
    ).rgb;
    var coverage = tex_data.r;
    if (vertex_fragment.ring != 0.0) {
        coverage = tex_data.g;
    }
    if (tex_data.b < vertex_fragment.prog.r || tex_data.b > vertex_fragment.prog.g) {
        coverage = 0.0;
    }
    return vec4<f32>(vertex_fragment.color.rgb, vertex_fragment.color.a * coverage);
}