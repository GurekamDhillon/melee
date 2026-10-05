// No strobe: a single gentle pulse, driven by rewind-restored Lua logic progress. The tint and the shape come from the archetype of the chain that fired
// it (synergy_fx.lua): 0 edge ring (the plain chain), 1 rising from below (flame, momentum), 2 a sweep across (haste web, shock arcs),
// 3 an expanding ring (crit), 4 crystalline edge (chill, armour). Strength is capped well below a flash.
let source = previous_color(in.uv);
let progress = clamp(params.progress.x, 0.0, 1.0);
let envelope = sin(progress * 3.14159265);
let shape = params.shape.x;
let c = in.uv - vec2f(0.5);
var mask = smoothstep(0.2, 0.7, length(c));
if (shape > 0.5 && shape < 1.5) {
    mask = smoothstep(0.45, 1.0, in.uv.y) * (0.7 + 0.3 * sin(in.uv.x * 18.0 + progress * 9.0));
} else if (shape > 1.5 && shape < 2.5) {
    let d = (in.uv.x - progress) * 5.0;
    mask = exp(-d * d) * 0.9 + 0.25 * smoothstep(0.3, 0.7, length(c));
} else if (shape > 2.5 && shape < 3.5) {
    let d = (length(c) - progress * 0.62) * 9.0;
    mask = exp(-d * d);
} else if (shape > 3.5) {
    mask = smoothstep(0.25, 0.7, length(c)) * (0.65 + 0.35 * step(0.5, fract(in.uv.x * 9.0 + in.uv.y * 9.0)));
}
let strength = clamp(params.strength.x, 0.0, 0.1) * envelope * mask;
return vec4f(mix(source.rgb, params.tint.xyz, strength), source.a);
