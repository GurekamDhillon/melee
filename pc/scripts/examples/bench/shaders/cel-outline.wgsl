// params[0]: bands, edge width (0..1), edge darkness (0..1), unused.
// This draws a dark surface silhouette edge; it does not expand geometry.
fn gd_surface(color: vec4f, s: GdSurfaceInput) -> vec4f {
    let bands = max(s.params[0].x, 2.0);
    let luma = max(dot(color.rgb, vec3f(0.2126, 0.7152, 0.0722)), 0.0001);
    let quantized = floor(clamp(luma, 0.0, 1.0) * bands) / bands;
    var rgb = color.rgb * (quantized / luma);
    if (s.normal_valid > 0.5) {
        let n = s.normal / max(length(s.normal), 0.0001);
        let view = -s.eye_position / max(length(s.eye_position), 0.0001);
        let edge = 1.0 - smoothstep(0.0, max(s.params[0].y, 0.0001), abs(dot(n, view)));
        rgb *= 1.0 - edge * clamp(s.params[0].z, 0.0, 1.0);
    }
    return vec4f(rgb, color.a);
}
