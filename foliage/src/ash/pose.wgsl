// Where a point laid out in logical pixels is drawn, once the element -- and everything it is drawn
// inside of -- has been turned and scaled: the linear part's two columns, then the offset. Applied
// after a quad is laid out, so what a fragment measures its own edge in is still the element's own
// space, and the screen-space derivative keeps that edge one pixel wide whatever this does.
fn posed(at: vec2<f32>, linear: vec4<f32>, offset: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(linear.x * at.x + linear.z * at.y, linear.y * at.x + linear.w * at.y) + offset;
}

