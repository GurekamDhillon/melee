// params[0]: threshold (0 intact, 1 gone), edge width, noise scale, unused.
// params[1]: glowing edge RGB, strength. Noise is stable in the raw UV domain.
fn gd_surface(color: vec4f, s: GdSurfaceInput) -> vec4f {
    let cell = floor(s.uv0 * max(s.params[0].z, 1.0));
    let noise = fract(sin(dot(cell, vec2f(12.9898, 78.233))) * 43758.5453);
    let threshold = clamp(s.params[0].x, 0.0, 1.0);
    if (threshold >= 1.0 || noise < threshold) { discard; }
    let edge = 1.0 - smoothstep(threshold, threshold + max(s.params[0].y, 0.0001), noise);
    return vec4f(color.rgb + edge * s.params[1].rgb * s.params[1].a, color.a);
}
