// Where a point laid out in logical pixels is drawn, once the element -- and everything it is drawn
// inside of -- has been turned and scaled: the linear part's two columns, then the offset. Applied
// after a quad is laid out, so what a fragment measures its own edge in is still the element's own
// space. A panel's and a polygon's edges, smoothed over a screen-space derivative, stay one pixel
// wide whatever this does; a mark's and a stroke's, stated by the backend, are stated as drawn. A
// glyph is a coverage bitmap cut at its unscaled size, and is resampled through this -- soft while
// it is scaled, sharp again once the scale is back to one.
fn posed(at: vec2<f32>, linear: vec4<f32>, offset: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(linear.x * at.x + linear.z * at.y, linear.y * at.x + linear.w * at.y) + offset;
}

