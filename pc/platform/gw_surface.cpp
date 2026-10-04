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
extern "C" int gw_surface_select(unsigned slot, uint32_t program, unsigned owner, const float *params16) {
    gw_surface::Params params{};
    if (params16) memcpy(params.data.data() + 4, params16, sizeof(float) * 16);
    params.data[1] = static_cast<float>(slot);
    if (!selections.set(slot, program, owner, params)) return 0;
    if (enabled[slot]) --enabled_count;
    enabled[slot] = program != 0;
    if (enabled[slot]) ++enabled_count;
    return 1;
}
extern "C" void gw_surface_release(unsigned owner) {
    selections.release(owner);
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
