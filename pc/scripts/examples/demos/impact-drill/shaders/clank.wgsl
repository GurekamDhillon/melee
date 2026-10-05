// @module
// Faithful owner v3 port; source: docs/prompts/reference/owner-impact-frame-v3.html.
// Geometry is height-normalized and GL-y-up; scene UVs remain top-left.
const PI: f32 = 3.141592653589793;
const TAU: f32 = 6.283185307179586;
fn metric(p:vec2f)->vec2f { return p*vec2f(resolution().x/resolution().y,-1.0); }
fn unmetric(p:vec2f)->vec2f { return p*vec2f(resolution().y/resolution().x,-1.0); }
fn sat(x: f32) -> f32 { return clamp(x, 0.0, 1.0); }

fn hash11(p: f32) -> f32 {
  return fract(sin(p * 127.1) * 43758.5453123);
}
fn hash21(p: vec2f) -> f32 {
  return fract(sin(dot(p, vec2f(127.1,311.7))) * 43758.5453123);
}
fn hash31(p: vec3f) -> f32 {
  return fract(sin(dot(p, vec3f(127.1,311.7,74.7))) * 43758.5453123);
}

fn noise(p: vec2f) -> f32 {
  var i: vec2f = floor(p);
  var f: vec2f = fract(p);
  f = f*f*(vec2f(3.0)-2.0*f);
  var a: f32 = hash21(i);
  var b: f32 = hash21(i+vec2f(1,0));
  var c: f32 = hash21(i+vec2f(0,1));
  var d: f32 = hash21(i+vec2f(1,1));
  return mix(mix(a,b,f.x), mix(c,d,f.x), f.y);
}

fn sdSegment(p: vec2f, a: vec2f, b: vec2f) -> f32 {
  // All calls have <=0.0065 radius. Outside this box the contribution is zero.
  if (any(p < min(a,b)-vec2f(0.007)) || any(p > max(a,b)+vec2f(0.007))) { return 1.0; }
  var pa: vec2f = p-a;
  var ba: vec2f = b-a;
  var h: f32 = sat(dot(pa,ba)/max(dot(ba,ba), 1e-5));
  return length(pa-ba*h);
}

// hard triangular spike
fn spike(p: vec2f, ang: f32, len: f32, halfWidth: f32) -> f32 {
  var q: vec2f = vec2f(cos(ang)*p.x+sin(ang)*p.y, -sin(ang)*p.x+cos(ang)*p.y);
  var x: f32 = q.x;
  var y: f32 = abs(q.y);
  var side: f32 = halfWidth * max(0.0, 1.0 - x/len);
  var inside: f32 = step(0.0, x) * step(x, len) * step(y, side);
  return inside;
}

// deterministic jagged polyline, intentionally angular
fn bolt(uv: vec2f, id: f32, thickness: f32, phase: f32) -> f32 {
  var p: vec2f = metric(uv-params.center.xy);
  var baseAng: f32 = hash11(id*11.37) * TAU;
  baseAng += sin(params.elapsed.x*4.0 + id*7.0) * 0.015;

  var total: f32 = 0.0;
  var a: vec2f = vec2f(0.0);
  var ang: f32 = baseAng;
  var travelled: f32 = 0.0;

  for (var i: i32 =0; i<9; i=i+1) {
    var fi: f32 = f32(i);
    var rndA: f32 = hash31(vec3f(id, fi, 2.17));
    var rndB: f32 = hash31(vec3f(id, fi, 7.91));

    var segLen: f32 = 0.055 + 0.025*rndB + fi*0.0035;
    var zig: f32 = (rndA - 0.5) * (0.78 + fi*0.045);
    ang += zig;

    var b: vec2f = a + vec2f(cos(ang), sin(ang)) * segLen;
    var d: f32 = sdSegment(p, a, b);

    // narrow white core + slightly wider colored sheath
    var core: f32 = 1.0 - smoothstep(thickness*0.22, thickness*0.46, d);
    var body: f32 = 1.0 - smoothstep(thickness*0.55, thickness, d);

    // intermittent "cut" pattern gives manga-like broken energy
    var cut: f32 = step(0.15, hash31(vec3f(id*2.7, fi, floor(params.elapsed.x*15.0 + phase))));
    total = max(total, max(core*1.6, body*cut));

    // small fork on selected segments
    if (i == 2 || i == 5 || i == 7) {
      var forkGate: f32 = step(0.42, hash31(vec3f(id, fi, 91.0)));
      var fAng: f32 = ang + (hash31(vec3f(id,fi,31.0))-0.5)*1.8;
      var fb: vec2f = a + vec2f(cos(fAng),sin(fAng)) * segLen * (0.8 + rndB);
      var fd: f32 = sdSegment(p, a, fb);
      var fork: f32 = 1.0 - smoothstep(thickness*0.38, thickness*0.8, fd);
      total = max(total, fork * forkGate);
    }

    a = b;
    travelled += segLen;
  }

  var radial: f32 = length(p);
  var startGate: f32 = smoothstep(0.018, 0.055, radial);
  var endGate: f32 = 1.0 - smoothstep(0.72, 0.95, radial);
  return total * startGate * endGate;
}

// radial razor cracks in the pressure field
fn crackField(uv: vec2f, id: f32) -> f32 {
  var p: vec2f = metric(uv-params.center.xy);
  var r: f32 = length(p);
  var a: f32 = atan2(p.y,p.x);
  var target_angle: f32 = hash11(id*9.73)*TAU;
  var da: f32 = abs(atan2(sin(a-target_angle),cos(a-target_angle)));

  var teeth: f32 = abs(fract(r*28.0 + id*0.37) - 0.5);
  var angular: f32 = 1.0 - smoothstep(0.006, 0.019, da + teeth*0.016);
  var radialGate: f32 = smoothstep(0.13,0.18,r) * (1.0-smoothstep(0.82,0.96,r));
  return angular * radialGate;
}

fn mod_fragment(in: Input) -> vec4f {
  var uv: vec2f = in.uv;
  var p: vec2f = metric(uv-params.center.xy);
  var r: f32 = length(p);
  var a: f32 = atan2(p.y,p.x);

  // impact timing
  var hit: f32 = 1.0-params.progress.x;
  var early: f32 = 1.0-smoothstep(0.0,0.18,params.progress.x);
  var radius: f32 = 0.035 + 0.76*(1.0-pow(1.0-params.progress.x,2.4));

  // pressure displacement: narrow, violent, not soft
  var ringDist: f32 = abs(r-radius);
  var ring: f32 = 1.0-smoothstep(0.006,0.025,ringDist);
  var dir: vec2f = normalize(p + vec2f(1e-6));
  var tangent: vec2f = vec2f(-dir.y,dir.x);
  var duv: vec2f = uv + unmetric(-dir*ring*0.035*hit + tangent*ring*sin(a*13.0)*0.007*hit)*params.intensity.x;

  // RGB split is short-lived and hard
  var rgbSplit: f32 = (0.002 + early*0.009 + ring*0.004)*params.intensity.x;
  var base: vec3f;
  base.r = scene_color(duv + unmetric(dir*rgbSplit)).r;
  base.g = scene_color(duv).g;
  base.b = scene_color(duv - unmetric(dir*rgbSplit)).b;

  // brutal manga/poster thresholding
  var lum: f32 = dot(base,vec3f(0.299,0.587,0.114));
  var poster: vec3f;
  if (lum < 0.075) { poster = vec3f(0.0); }
  else if (lum < 0.18) { poster = base*0.32; }
  else if (lum < 0.42) { poster = base*0.72; }
  else { poster = min(base*1.32,vec3f(1.0)); }

  // central black void with jagged perimeter
  var edgeNoise: f32 = noise(vec2f(cos(a),sin(a))*13.0 + vec2f(r*34.0 + floor(params.elapsed.x*14.0)*0.23));
  var voidR: f32 = 0.055 + 0.075*hit;
  var jag: f32 = voidR + (edgeNoise-0.5)*0.055;
  var voidMask: f32 = 1.0-smoothstep(jag-0.004,jag+0.008,r);

  // blade-like radial explosion shards
  var shards: f32 = 0.0;
  for(var i: i32 =0;i<18;i=i+1) {
    var fi: f32 =f32(i);
    var ang: f32 =hash11(fi*17.13)*TAU;
    var len: f32 =0.13 + hash11(fi*7.7)*0.34;
    var wid: f32 =0.003 + hash11(fi*4.1)*0.012;
    shards=max(shards, spike(p,ang,len,wid));
  }
  shards *= hit * smoothstep(0.01,0.10,r);

  // layered lightning network
  var l: f32 = 0.0;
  if (r < 1.05 && hit > 0.0) {
  for(var i: i32 =0;i<12;i=i+1) {
    var fi: f32 =f32(i);
    var flicker: f32 = 0.62 + 0.38*step(0.45,hash31(vec3f(fi,floor(params.elapsed.x*22.0),3.0)));
    l = max(l, bolt(uv,fi+0.31,0.0065,fi)*flicker);
  }

  }
  var cracks: f32 =0.0;
  for(var i: i32 =0;i<14;i=i+1) { cracks=max(cracks,crackField(uv,f32(i)+0.6)); }
  cracks *= ring*hit;

  // razor shockwave: broken concentric wedges rather than smooth halo
  var sector: f32 = floor((a+PI)/TAU*30.0);
  var sectorRand: f32 = hash11(sector*8.1);
  var brokenRing: f32 = ring * step(0.22,sectorRand);
  var ring2: f32 = (1.0-smoothstep(0.003,0.013,abs(r-(radius-0.035)))) * step(0.53,sectorRand);

  // thin black "haki fissures" around white/red energy
  var darkSheath: f32 = smoothstep(0.0,0.028,l) * (1.0-smoothstep(0.08,0.2,l));

  // color
  var col: vec3f = poster;
  col = mix(col,vec3f(0.004,0.0,0.008),voidMask);
  col -= darkSheath*vec3f(0.20,0.02,0.035);

  // saturated red-violet, but with white knife-edge cores
  var lightningBody: f32 = sat(l);
  var lightningCore: f32 = smoothstep(0.88,1.55,l);
  var red: vec3f = vec3f(1.0,0.015,0.055);
  var magenta: vec3f = vec3f(0.78,0.02,0.92);
  var boltColor: vec3f = mix(red,magenta, noise(vec2f(cos(a),sin(a))*9.0+vec2f(5.0)));
  col += boltColor * lightningBody * 2.2 * hit;
  col += vec3f(1.45,1.22,1.38) * lightningCore * 2.1 * hit;

  col += vec3f(0.95,0.0,0.08) * brokenRing * 1.65 * hit;
  col += vec3f(0.55,0.02,0.88) * ring2 * 1.35 * hit;
  col += vec3f(0.9,0.01,0.05) * cracks * 1.5;
  col += vec3f(0.8,0.02,0.12) * shards * 1.2;

  // initial white "frame tear" at impact
  var flash: f32 = early * (1.0-smoothstep(0.0,0.24,r));
  var star: f32 = 0.0;
  for(var i: i32 =0;i<8;i=i+1) {
    var fi: f32 =f32(i);
    star=max(star,spike(p,fi*TAU/8.0,0.22,0.014));
  }
  if (params.flash_limit.x < 0.5) { col += vec3f(1.25) * star * flash * 1.6; }
  else { col *= 1.0-star*flash*0.25; }

  // inverted wedge cuts during first few frames
  var invWedge: f32 = step(0.82, fract((a+PI)/TAU*12.0 + floor(params.elapsed.x*10.0)*0.07));
  invWedge *= early * (1.0-smoothstep(0.08,0.42,r));
  if (params.flash_limit.x < 0.5) { col = mix(col,vec3f(1.0)-col,invWedge*0.72); }
  else { col *= 1.0-invWedge*0.3; }

  // hard vignette + scan-grain
  var q: vec2f =uv*(vec2f(1.0)-uv.yx);
  var vig: f32 =pow(sat(q.x*q.y*23.0),0.28);
  col*=vig;
  var grain: f32 =hash21(in.screen_uv*resolution()+floor(params.elapsed.x*45.0))-0.5;
  col+=grain*0.022;

  // crush black one final time for graphic separation
  col = max(col,vec3f(0.0));
  col *= mix(0.82,1.12,step(0.11,dot(col,vec3f(0.333))));
  // Fade only the final 10% back to the unchanged frozen scene.
  let original = scene_color(uv).rgb;
  col = mix(original,col,params.intensity.x*(1.0-smoothstep(0.9,1.0,params.progress.x)));
  if (params.flash_limit.x > 0.5) {
    // Fixed scene during native freeze; restrained RGB excursions with a slow
    // onset/release replace temporal history (the chain's previous is spatial).
    col = original + clamp(col-original,vec3f(-0.04),vec3f(0.04))*params.limiter_strength.x;
  }
  return vec4f(col,1.0);
}
