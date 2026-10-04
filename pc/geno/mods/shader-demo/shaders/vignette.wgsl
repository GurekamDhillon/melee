// params: strength, radius, softness (floats).
let c = previous_color(in.uv);
let delta = in.uv - vec2f(0.5);
let fade = smoothstep(params.radius.x, params.radius.x + max(params.softness.x, 0.001), length(delta));
return vec4f(c.rgb * (1.0 - clamp(params.strength.x, 0.0, 1.0) * fade), c.a);
