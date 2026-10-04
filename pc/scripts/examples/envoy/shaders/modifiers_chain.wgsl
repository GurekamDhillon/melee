// No strobe: single gentle edge pulse, driven by rewind-restored Lua logic progress.
let source = previous_color(in.uv);
let progress = clamp(params.progress.x, 0.0, 1.0);
let envelope = sin(progress * 3.14159265);
let edge = smoothstep(0.2, 0.7, length(in.uv - vec2f(0.5)));
let strength = clamp(params.strength.x, 0.0, 0.06) * envelope * edge;
return vec4f(mix(source.rgb, vec3f(0.95, 0.65, 0.25), strength), source.a);
