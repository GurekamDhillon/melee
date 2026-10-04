let base = previous_color(in.uv);
return vec4f(mix(base.rgb, params.tint.rgb, params.strength.x), base.a);
