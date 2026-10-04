// Original presentation-clock flash; expires even while logic is frozen.
let c = previous_color(in.uv);
let pulse = (1.0 - params.progress.x) * 0.3;
return vec4f(c.rgb + params.tint.rgb * pulse, c.a);
