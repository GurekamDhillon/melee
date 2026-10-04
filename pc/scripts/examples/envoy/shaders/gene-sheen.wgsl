// Surface functions receive RGB after the texture-preserving TEV tint.
fn gd_surface(color: vec4f, s: GdSurfaceInput) -> vec4f {
    if (s.normal_valid < 0.5) { return color; }
    let n = s.normal / max(length(s.normal), 0.0001);
    let v = -s.eye_position / max(length(s.eye_position), 0.0001);
    let rim = pow(1.0 - clamp(abs(dot(n, v)), 0.0, 1.0), max(s.params[1].x, 1.0));
    return vec4f(color.rgb + s.params[0].rgb * s.params[0].a * rim, color.a);
}
