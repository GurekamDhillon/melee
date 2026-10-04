// params: width, threshold, strength (float), tint (vec4). Full resolution recommended.
let c = previous_color(in.uv);
let step = vec2f(max(params.width.x, 1.0)) / resolution();
let z = linear_depth(scene_depth(in.uv));
let a = linear_depth(scene_depth(in.uv + vec2f(step.x, 0.0)));
let b = linear_depth(scene_depth(in.uv - vec2f(step.x, 0.0)));
let d = linear_depth(scene_depth(in.uv + vec2f(0.0, step.y)));
let e = linear_depth(scene_depth(in.uv - vec2f(0.0, step.y)));
let edge = max(max(abs(a-z), abs(b-z)), max(abs(d-z), abs(e-z))) / max(z, 1.0);
let amount = smoothstep(params.threshold.x, max(params.threshold.x * 2.0, 0.001), edge) * params.strength.x;
return vec4f(mix(c.rgb, params.tint.xyz, clamp(amount, 0.0, 1.0)), c.a);
