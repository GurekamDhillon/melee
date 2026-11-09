/* hashword_probe.c - Envoy M0 probe (b): a sim mod with state that rolls back and is hashed.
 * Every frame it folds each present player's percent, position and action state into a running digest (state that
 * depends on the whole history, like Envoy's run record) and hands the digest to gd_hash_word. It changes nothing in
 * the game. Built with -DSKEW=N (the desync test's variant, loaded by one peer only), its word differs from logic frame
 * N on while the game itself stays identical.
 * Build: rust/mods/hashword_probe/build.sh (clang --target=wasm32). */
#include "../../../docs/specs/gdmod.h"

#ifndef SKEW
#define SKEW (-1)
#endif

static gd_u32 g_digest = 0x811C9DC5u;
static gd_u32 g_frames;

static gd_u32 mix(gd_u32 h, gd_u32 v)
{
    h ^= v;
    h *= 0x01000193u;
    return h ^ (h >> 15);
}

static gd_u32 fbits(gd_f32 f)
{
    union { gd_f32 f; gd_u32 u; } x;
    x.f = f;
    return x.u;
}

/* frame.run_procs post: after the logic procs, so the fighters are this frame's */
static void on_frame_end(void)
{
    gd_i32 s;
    for (s = 0; s < 4; ++s) {
        gd_fighter f;
        if (!gd_player_present(s)) continue;
        f = gd_fighter_from_slot(s);
        g_digest = mix(g_digest, (gd_u32)s);
        g_digest = mix(g_digest, fbits(gd_fighter_get_percent(f)));
        g_digest = mix(g_digest, fbits(gd_fighter_get_pos_x(f)));
        g_digest = mix(g_digest, fbits(gd_fighter_get_pos_y(f)));
        g_digest = mix(g_digest, (gd_u32)gd_fighter_get_action_state(f));
    }
    g_frames += 1u;
    if (SKEW >= 0 && gd_match_frame() >= SKEW) g_digest ^= 0x5A5A5A5Au;
    gd_hash_word(g_digest);
}

GD_HOOKS_BEGIN
    GD_HOOK_POST(GD_ADDR_FRAME_RUN_PROCS, 0, on_frame_end)
GD_HOOKS_END

GD_MOD_MANIFEST("gd.hashword_probe", "Hash word probe", 0x00010000u, GD_ABI_VERSION, GD_ABI_VERSION, GD_FEATURE_NONE,
                GD_FEATURE_NONE, (gd_u32)(sizeof(g_digest) + sizeof(g_frames)));
