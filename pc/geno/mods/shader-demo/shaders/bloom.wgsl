// params: threshold, intensity, radius (floats). Bright extraction + compact blur, at half res.
var glow = vec3f(0.0);
let step = vec2f(max(params.radius.x, 1.0)) / resolution();
for (var y = -2; y <= 2; y++) {
  for (var x = -2; x <= 2; x++) {
    let c = previous_color(in.uv + vec2f(f32(x), f32(y)) * step).rgb;
    glow += max(c - vec3f(params.threshold.x), vec3f(0.0));
  }
}
return vec4f(glow * (params.intensity.x / 25.0), 1.0);
