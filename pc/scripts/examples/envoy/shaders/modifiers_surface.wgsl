// Real GX surface composite: two equipment bands first, then seven status treatments.
// params: seconds,intensity,burn,shock / chill,curse,haste,guarded /
// momentum,equip1 look+hue,equip1 strength,equip2 look+hue / equip2 strength,guard flash,combined hue,combined strength.
fn em1_hue(h: f32) -> vec3f {
    return clamp(abs(fract(vec3f(h) + vec3f(0.0, 0.666667, 0.333333)) * 6.0 - 3.0) - 1.0, vec3f(0.0), vec3f(1.0));
}
fn em1_blend(base: vec3f, color: vec3f, strength: f32) -> vec3f {
    return mix(base, color, clamp(strength, 0.0, 0.7));
}
fn em1_burn(s: GdSurfaceInput, rim: f32, t: f32) -> vec3f {
    // flame: deep red body with yellow tongues that rise over time (the only status that moves upward and flickers red-to-yellow)
    let tongue = pow(0.5 + 0.5 * sin(s.uv0.y * 16.0 - t * 5.0 + sin(s.uv0.x * 9.0 + t * 2.0) * 2.0), 2.0);
    return mix(vec3f(0.95, 0.08, 0.0), vec3f(1.0, 0.78, 0.1), tongue) * (0.65 + 0.55 * rim);
}
fn em1_shock(s: GdSurfaceInput, rim: f32, t: f32) -> vec3f {
    // electric cyan with white-hot arcs: the only status with bright white lines moving over a saturated cyan body
    let arc = pow(0.5 + 0.5 * sin(s.uv0.y * 22.0 + sin(s.uv0.x * 9.0 + t * 6.0) * 3.0), 6.0);
    return vec3f(0.0, 0.80, 1.0) * (0.55 + 0.25 * rim) + vec3f(0.9) * arc;
}
fn em1_chill(base: vec3f, s: GdSurfaceInput, rim: f32) -> vec3f {
    // frost: drained of colour, lifted toward pale ice-white, whitest at the rim and on the edges of the model
    let gray = dot(base, vec3f(0.2126, 0.7152, 0.0722));
    let frost = smoothstep(0.25, 0.8, abs(s.uv0.y * 2.0 - 1.0));
    return mix(vec3f(gray * 0.9 + 0.22), vec3f(0.86, 0.95, 1.0), clamp(0.35 + frost * 0.4 + rim * 0.5, 0.0, 1.0));
}
fn em1_curse(rim: f32) -> vec3f { return mix(vec3f(0.20, 0.08, 0.32), vec3f(0.65, 0.23, 0.7), 1.0 - rim); }
fn em1_haste(s: GdSurfaceInput, t: f32) -> vec3f {
    let streak = pow(0.5 + 0.5 * sin(s.uv0.y * 35.0 - t * 3.0), 6.0);
    return vec3f(0.15, 0.95, 0.57) * (0.65 + streak * 0.35);
}
fn em1_guard(s: GdSurfaceInput, rim: f32, flash: f32) -> vec3f {
    // royal blue hex facets with a lighter rim: a plated look, not a glow
    let facet = floor(fract(s.uv0.x * 5.0 + s.uv0.y * 3.0) * 3.0) / 3.0;
    return vec3f(0.12, 0.30, 1.0) * (0.55 + facet * 0.45 + rim * 0.2) + vec3f(0.25, 0.4, 0.7) * (rim * 0.5 + clamp(flash, 0.0, 1.0) * 0.1);
}
fn em1_momentum(s: GdSurfaceInput, stacks: f32, spend: f32) -> vec3f {
    // gold that fills from the feet up with the stack count (static: it does not flicker like fire)
    let climb = smoothstep(1.0 - clamp(stacks, 0.0, 1.0) - 0.2, 1.0 - clamp(stacks, 0.0, 1.0), 1.0 - s.uv0.y);
    return vec3f(1.0, 0.88, 0.12) * (0.55 + 0.4 * climb + 0.15 * clamp(spend, 0.0, 1.0));
}
fn em1_equipment(encoded: f32, base: vec3f, s: GdSurfaceInput, rim: f32, t: f32) -> vec3f {
    let look = u32(clamp(floor(encoded), 0.0, 7.0));
    var treatment = em1_hue(fract(encoded));
    switch look {
        case 1u: { treatment = em1_burn(s, rim, t); }
        case 2u: { treatment = em1_shock(s, rim, t); }
        case 3u: { treatment = em1_chill(base, s, rim); }
        case 4u: { treatment = em1_curse(rim); }
        case 5u: { treatment = em1_haste(s, t); }
        case 6u: { treatment = em1_guard(s, rim, 0.0); }
        case 7u: { treatment = em1_momentum(s, 0.5, 0.0); }
        default: {}
    }
    return mix(treatment, em1_hue(fract(encoded)), 0.25);
}
fn gd_surface(base: vec4f, s: GdSurfaceInput) -> vec4f {
    let p = s.params;
    let t = p[0].x; // explicit logic clock; never s.time (host time)
    let intensity = clamp(p[0].y, 0.0, 1.0);
    if (intensity == 0.0) { return base; }
    let n = normalize(s.normal + vec3f(0.00001));
    let eye = normalize(s.eye_position + vec3f(0.00001));
    let rim = select(0.35, pow(1.0 - abs(dot(n, eye)), 2.0), s.normal_valid > 0.5);
    var active_statuses = 0u;
    for (var j = 0u; j < 7u; j = j + 1u) {
        let lane = j + 2u;
        if (p[lane / 4u][lane % 4u] > 0.0) { active_statuses = active_statuses + 1u; }
    }
    let equipment_slots = 3u - min(active_statuses, 3u);
    let equip1_scale = select(0.025, 0.10 + 0.16 * rim, equipment_slots > 0u);
    let equip2_scale = select(0.025, 0.08 + 0.16 * (1.0 - rim), equipment_slots > 1u || (equipment_slots > 0u && p[2].z <= 0.0));
    var rgb = base.rgb;
    rgb = em1_blend(rgb, em1_equipment(p[2].y, rgb, s, rim, t), min(p[2].z * intensity * equip1_scale, 0.28));
    rgb = em1_blend(rgb, em1_equipment(p[2].w, rgb, s, rim, t), min(p[3].x * intensity * equip2_scale, 0.28));
    rgb = em1_blend(rgb, em1_hue(p[3].z), min(clamp(p[3].w, 0.0, 1.0) * intensity * 0.025, 0.28));
    // At most three statuses get full strength. Remaining statuses fold into faint tint.
    var shown = 0u;
    for (var i = 0u; i < 7u; i = i + 1u) {
        var value = 0.0;
        var color = rgb;
        switch i {
            case 0u: { value = p[0].z; color = em1_burn(s, rim, t); }
            case 1u: { value = p[0].w; color = em1_shock(s, rim, t); }
            case 2u: { value = p[1].x; color = em1_chill(rgb, s, rim); }
            case 3u: { value = p[1].y; color = em1_curse(rim); }
            case 4u: { value = p[1].z; color = em1_haste(s, t); }
            case 5u: { value = p[1].w; color = em1_guard(s, rim, p[3].y); }
            default: { value = p[2].x; color = em1_momentum(s, value, 0.0); }
        }
        if (value > 0.0) {
            // A status is the loudest thing on a fighter: full strength for three, a faint tint for the rest. The rim adds a lit edge so it reads on dark and light fighters.
            let strength = select(0.03, 0.80, shown < 3u);
            // With two or three statuses each owns a diagonal band of the model, so none is washed out by the others.
            var own = 1.0;
            let crowd = min(active_statuses, 3u);
            if (crowd > 1u && shown < 3u) {
                let share = 1.0 / f32(crowd);
                let phase = fract(s.uv0.x * 3.0 + s.uv0.y * 2.0);
                let lo = f32(shown) * share;
                own = select(0.25, 1.0, phase >= lo && phase < lo + share);
            }
            rgb = em1_blend(rgb, color, clamp(value, 0.0, 1.0) * intensity * strength * own * (0.75 + 0.25 * rim));
            if (shown < 3u) { rgb = rgb + color * rim * clamp(value, 0.0, 1.0) * intensity * 0.30; }
            shown = shown + 1u;
        }
    }
    return vec4f(clamp(rgb, vec3f(0.0), vec3f(1.0)), base.a);
}
