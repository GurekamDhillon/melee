// Example effect body: the first package texture, animated tint, authored colour ramps.
// Explicit LOD remains legal inside per-particle control flow.
let tex = textureSampleLevel(t0, s0, uv_of(parts[in.ii], 0u, in.uv), 0.0);
return tex * in.color * in.color1;
