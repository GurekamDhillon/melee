// Original stage tour colour look, one parameter float per RGB channel.
fn gd_surface(color: vec4f, s: GdSurfaceInput) -> vec4f {
  let tint = vec3f(s.params[0].x, s.params[0].y, s.params[0].z);
  return vec4f(color.rgb * tint, color.a);
}
