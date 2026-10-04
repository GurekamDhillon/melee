// params: lift, gamma, gain (vec4), saturation (float). Scalars use .x.
let c = previous_color(in.uv);
let lifted = max(c.rgb + params.lift.xyz, vec3f(0.0));
let graded = pow(lifted, vec3f(1.0) / max(params.gamma.xyz, vec3f(0.01))) * params.gain.xyz;
let grey = dot(graded, vec3f(0.2126, 0.7152, 0.0722));
return vec4f(mix(vec3f(grey), graded, params.saturation.x), c.a);
