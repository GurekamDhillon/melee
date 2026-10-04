// Whole-scene look also covers static destinations with no surface hook.
let c = previous_color(in.uv);
return vec4f(c.rgb * mix(vec3f(1.0), params.tint.rgb, 0.3), c.a);
