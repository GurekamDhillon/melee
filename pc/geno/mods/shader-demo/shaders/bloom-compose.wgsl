// Full resolution composition retains the original scene's detail.
let base = scene_color(in.uv);
return vec4f(base.rgb + previous_color(in.uv).rgb, base.a);
