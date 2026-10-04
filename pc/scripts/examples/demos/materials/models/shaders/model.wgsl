// Original untextured normal-based colour, for the user's own mesh.
let light = 0.3 + 0.7 * abs(normalize(in.normal).y);
return vec4f(params.tint.rgb * light, 1.0);
