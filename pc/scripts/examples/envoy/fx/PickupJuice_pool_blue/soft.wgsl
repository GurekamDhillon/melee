let q = in.uv * 2.0 - vec2f(1.0);
let a = pow(max(0.0, 1.0 - dot(q,q)), 2.0);
return vec4f(params.tint.rgb * in.color.rgb, a * in.color.a);
