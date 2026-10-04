// Original texture-preserving RGB tint; params[0].x controls strength.
fn gd_surface(color: vec4f, s: GdSurfaceInput) -> vec4f {
  return vec4f(mix(color.rgb, color.rgb * vec3f(0.4,0.85,1.0), s.params[0].x), color.a);
}
