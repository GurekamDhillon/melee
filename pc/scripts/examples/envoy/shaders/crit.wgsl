// @module
// Crit impact frame: a toned-down, strength-scaled cousin of the clank look (clank.wgsl).
// One full-screen pass. Lua computes every amount from one `strength`; this file only draws.
// Geometry is height-normalised (round at any window shape); scene UVs stay top-left.
// Layers (each can be zero): hard two-tone tear, speed lines, radial blur, decaying
// chromatic aberration, shockwave ring, contrast push. All noise is keyed on an integer
// angular bucket, so there is no atan seam.
const TAU: f32 = 6.283185307179586;
fn metric(p:vec2f)->vec2f { return p*vec2f(resolution().x/resolution().y,-1.0); }
fn unmetric(p:vec2f)->vec2f { return p*vec2f(resolution().y/resolution().x,-1.0); }
fn sat(x: f32) -> f32 { return clamp(x, 0.0, 1.0); }
fn hash11(p: f32) -> f32 { return fract(sin(p * 127.1) * 43758.5453123); }
fn lum(c: vec3f) -> f32 { return dot(c, vec3f(0.299,0.587,0.114)); }

fn mod_fragment(in: Input) -> vec4f {
  let uv = in.uv;
  let p = metric(uv - params.center.xy);
  let r = length(p);
  let dir = p / max(r, 1e-5);
  let prog = params.progress.x;
  let t = params.elapsed.x;
  // ease-out envelope: full at once, long soft tail
  let env = pow(1.0 - sat(prog), 2.2);

  // shockwave ring: radius grows with an ease-out, thin, displaces pixels outward
  let ring_r = 0.03 + 0.62 * (1.0 - pow(1.0 - sat(prog), 2.2));
  let ring = 1.0 - smoothstep(0.004, 0.03, abs(r - ring_r));
  let ring_amt = params.ring.x * env;
  var duv = uv - unmetric(dir) * ring * 0.022 * ring_amt;

  // crisp accent: a thin bright ring edge. It is what a light crit reads as (strong crits have the tear and the lines on top of it).
  let ring_edge = 1.0 - smoothstep(0.0, 0.010, abs(r - ring_r));
  // radial blur toward the contact point (5 taps, constant bound)
  let blur = params.blur.x * env * smoothstep(0.05, 0.5, r);
  // decaying chromatic aberration along the radial
  let ca = params.ca.x * env * (0.4 + smoothstep(0.0, 0.6, r));
  let ca_off = unmetric(dir) * ca;
  var base = vec3f(0.0);
  for (var i: i32 = 0; i < 5; i = i + 1) {
    let k = f32(i) / 4.0;
    let suv = duv + (params.center.xy - duv) * blur * k;
    base.r = base.r + previous_color(suv + ca_off).r * 0.2;
    base.g = base.g + previous_color(suv).g * 0.2;
    base.b = base.b + previous_color(suv - ca_off).b * 0.2;
  }
  let original = previous_color(uv).rgb;
  var col = base;

  // contrast / saturation push, decays with the envelope
  let cp = params.contrast.x * env;
  col = (col - vec3f(0.5)) * (1.0 + 0.6 * cp) + vec3f(0.5);
  col = mix(vec3f(lum(col)), col, 1.0 + 0.5 * cp);

  // speed lines: radial bands, angular bucket hashed (periodic), flicker 30 Hz
  let n = 72.0;
  let bucket = floor((atan2(p.y, p.x) / TAU + 0.5) * n);
  let b = ((bucket % n) + n) % n;
  let fl = floor(t * 30.0);
  let h = hash11(b * 3.7 + fl * 1.9);
  let on = step(0.82, h);
  let len = 0.2 + 0.5 * hash11(b * 5.3 + 1.0);
  let inner = 0.11 + 0.1 * (1.0 - env);
  let band = on * smoothstep(inner, inner + 0.08, r) * (1.0 - smoothstep(len, len + 0.45, r));
  col = col + vec3f(1.0) * band * params.lines.x * env;

  // impact tear: a hard two-tone frame (optionally inverted), inside a radius
  let in_tear = step(t, params.tear_s.x) * (1.0 - smoothstep(params.tear_r.x * 0.85, params.tear_r.x, r));
  let l = lum(base);
  let thr = step(params.tear_thr.x, l);
  var two = mix(vec3f(0.02, 0.0, 0.04), vec3f(1.0), thr);
  two = mix(two, vec3f(1.0) - two, params.tear_inv.x);
  col = mix(col, two, in_tear * params.tear_amt.x);

  col = col + vec3f(1.0, 0.96, 0.82) * ring_edge * params.accent.x * sat(1.6 * env);
  col = max(col, vec3f(0.0));
  // never leave a residue: the tail fades to the untouched scene
  let fade = 1.0 - smoothstep(0.85, 1.0, prog);
  col = mix(original, col, fade);
  return vec4f(min(col, vec3f(1.5)), 1.0);
}
