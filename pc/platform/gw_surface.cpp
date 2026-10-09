/* Visual-only surface selection. Game-thread ownership/scopes; copied FIFO data
 * is consumed by Aurora, including interpolated replay. No simulation reads. */
#include "gw_surface.h"
#include "gw_surface_state.hpp"
#include <dolphin/gx/GXAurora.h>
#include <chrono>
#include <cstdio>
#include <cstring>
extern "C" int gw_gx_suppress_draws;

static gw_surface::Selection selections;
static bool enabled[8];
static unsigned enabled_count;
static uint64_t load_calls, fifo_commands;
static void submit(gw_surface::Binding binding) {
#ifdef AURORA_GD_SURFACE_VERSION
    static const auto start = std::chrono::steady_clock::now();
    binding.params.data[0] = std::chrono::duration<float>(std::chrono::steady_clock::now() - start).count();
    GXAuroraSurface(binding.program, binding.params.data.data());
    ++fifo_commands;
#else
    (void)binding;
#endif
}
extern "C" uint32_t gw_surface_register(const char *source, const char *label, char *error, unsigned size) {
    ++load_calls;
#ifdef AURORA_GD_SURFACE_VERSION
    return GXAuroraSurfaceRegister(source, label, error, size);
#else
    (void)source; (void)label;
    if (error && size) snprintf(error, size, "Aurora surface patch is not built");
    return 0;
#endif
}
extern "C" void gw_surface_stats(uint64_t out[3]) {
    out[0] = load_calls; out[1] = fifo_commands; out[2] = enabled_count;
}
/* Geno "toon" (cel shading for base "none" defines): an engine-owned selection under a
 * reserved owner. A script's own gd.fighter_shader on that port evicts it (scripts win);
 * any release re-arms it, so it is re-asserted on the fighter's next draw. Visual only. */
static const unsigned kToonOwner = 0x544F4F4Eu; /* 'TOON' - script owners are 1..N */
static uint32_t toon_program;
static bool toon_failed, toon_held[8];
static int toon_kind[8]; /* kind + 1 already handled on this slot, 0 = re-check */
extern "C" int gw_Geno_ToonParams(int kind, float out16[16]);
extern "C" void gw_log(const char *fmt, ...);
/* params: [0] bands [1] shadow level [2] rim [3] outline width | [4..6] key light (eye space)
 * [7] outline darkness | [8..10] shadow tint [11] band softness | [15] unlit base (engine side). */
static const char *kToonSurface = R"(
fn gd_surface(color: vec4f, s: GdSurfaceInput) -> vec4f {
    if (s.normal_valid < 0.5) { return color; }
    let n = normalize(s.normal);
    let l = normalize(s.params[1].xyz);
    let v = -s.eye_position / max(length(s.eye_position), 0.0001);
    let bands = max(s.params[0].x, 2.0);
    let soft = max(s.params[2].w, 0.001);
    let ndl = dot(n, l) * 0.5 + 0.5;
    let x = ndl * (bands - 1.0);
    let step_lo = floor(x);
    let q = (step_lo + smoothstep(0.5 - soft * bands, 0.5 + soft * bands, fract(x))) / (bands - 1.0);
    let shade = mix(s.params[2].rgb * s.params[0].y, vec3f(1.0), clamp(q, 0.0, 1.0));
    var rgb = color.rgb * shade;
    let facing = abs(dot(n, v));
    let rim = smoothstep(0.62, 0.72, 1.0 - facing) * step(0.55, ndl) * s.params[0].z;
    rgb = rgb + vec3f(rim);
    let edge = 1.0 - smoothstep(0.0, max(s.params[0].w, 0.0001), facing);
    rgb = rgb * (1.0 - edge * s.params[1].w);
    return vec4f(clamp(rgb, vec3f(0.0), vec3f(1.0)), color.a);
}
)";
static void toon_rearm() { for (int &k : toon_kind) k = 0; }
extern "C" int gw_surface_select(unsigned slot, uint32_t program, unsigned owner, const float *params16);
extern "C" void gw_GenoToonEnsure(int slot, int kind) {
    if (slot < 1 || slot > 6 || toon_kind[slot] == kind + 1) return;
    float p[16] = {0};
    const int on = gw_Geno_ToonParams(kind, p);
    if (toon_held[slot]) { gw_surface_select((unsigned)slot, 0, kToonOwner, nullptr); toon_held[slot] = false; }
    toon_kind[slot] = kind + 1;
    if (!on) return;
    if (!toon_program && !toon_failed) {
        char why[512] = {0};
        toon_program = gw_surface_register(kToonSurface, "builtin/geno-toon", why, sizeof why);
        if (!toon_program) { toon_failed = true; gw_log("geno toon: built-in surface unavailable: %s", why); return; }
    }
    if (!toon_program) return;
    if (gw_surface_select((unsigned)slot, toon_program, kToonOwner, p)) {
        toon_held[slot] = true;
        gw_log("geno toon: P%d kind %d cel shading on (bands %g shadow %g outline %g)", slot, kind, p[0], p[1], p[3]);
    } else {
        gw_log("geno toon: P%d kind %d keeps a script's surface shader", slot, kind);
    }
}
extern "C" int gw_surface_select(unsigned slot, uint32_t program, unsigned owner, const float *params16) {
    if (owner != kToonOwner && slot < 8 && toon_held[slot]) {
        selections.set(slot, 0, kToonOwner, gw_surface::Params{}); /* a script selection outranks the define's toon */
        toon_held[slot] = false;
        if (enabled[slot]) { --enabled_count; enabled[slot] = false; }
    }
    gw_surface::Params params{};
    if (params16) memcpy(params.data.data() + 4, params16, sizeof(float) * 16);
    params.data[1] = static_cast<float>(slot);
    if (!selections.set(slot, program, owner, params)) return 0;
    if (enabled[slot]) --enabled_count;
    enabled[slot] = program != 0;
    if (enabled[slot]) ++enabled_count;
    return 1;
}
extern "C" int gw_surface_update(unsigned slot, unsigned owner, const float *params16) {
    gw_surface::Params params{};
    if (params16) memcpy(params.data.data() + 4, params16, sizeof(float) * 16);
    params.data[1] = static_cast<float>(slot);
    return selections.update(slot, owner, params) ? 1 : 0;
}
extern "C" void gw_surface_release(unsigned owner) {
    selections.release(owner);
    if (!owner) for (bool &h : toon_held) h = false;
    toon_rearm();
    // Scopes are empty at script lifecycle boundaries. Count remaining selections.
    enabled_count = 0;
    for (unsigned slot = 1; slot < 8; ++slot) {
        enabled[slot] = selections.enter(slot).program != 0;
        selections.leave();
        if (enabled[slot]) ++enabled_count;
    }
}
extern "C" void gw_SurfaceBegin(int slot) {
    if (enabled_count && !gw_gx_suppress_draws) submit(selections.enter(static_cast<unsigned>(slot)));
}
extern "C" void gw_SurfaceEnd(void) {
    if (enabled_count && !gw_gx_suppress_draws) submit(selections.leave());
}
