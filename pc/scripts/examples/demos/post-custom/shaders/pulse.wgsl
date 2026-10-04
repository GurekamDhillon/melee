// Original one-sample pulse. Save an edit to see hot reload.
let color = previous_color(in.uv);
let amount = params.amount.x * (0.5 + 0.5 * sin(time_seconds()));
return vec4f(mix(color.rgb, color.rgb * params.tint.rgb, amount), color.a);
