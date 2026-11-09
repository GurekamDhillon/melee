/* stage_switch.c - the C twin of rust/mods/stage_switch (same id, hooks, rotation and refusal rules), for a machine
 * without the wasm32 Rust target: clang builds it (build.sh beside this file). A loop FD -> BF -> Yoshi's Story ->
 * Dream Land -> Fountain -> FD, PERIOD logic frames per stage; the switch is native (gd_stage_switch), applied between
 * logic frames inside the simulation, so it rolls back and resimulates with the match.
 *   stage.on_start post: (re)starts the clock (match start and inside every switch).
 *   frame.run_procs post: at PERIOD, request the next stage after the live one; -9 (a fighter grabbed / thrown /
 *   respawning) retries next frame, any other refusal stops the rotation for this match. */
#include "../../../../docs/specs/gdmod.h"

#define GD_ADDR_STAGE_ON_START 0x801C0FB8u /* stage.on_start  void(ptr) */
#ifndef PERIOD
#define PERIOD 600
#endif
#define BUSY (-9)

/* Gr_Kind_Battle, Story, OldPupupu, Izumi, Last: the C demo's order after its initial FD */
static const gd_i32 k_rotation[5] = {36, 10, 28, 12, 37};

static gd_i32 g_clock = -1; /* -1: no match / rotation stopped */
static gd_i32 g_warned;

static void stage_start(gd_u32 p)
{
    (void)p;
    g_clock = 0;
    g_warned = 0;
}

static void on_frame_end(void)
{
    gd_i32 cur, next, rc, i;
    if (g_clock < 0) return;
    g_clock += 1;
    if (g_clock < PERIOD) return;
    cur = gd_stage_grkind();
    next = -1;
    for (i = 0; i < 5; ++i) {
        if (k_rotation[i] == cur) { next = k_rotation[(i + 1) % 5]; break; }
    }
    if (next < 0) {
        if (!g_warned) { gd_log("stage_switch: host stage not supported, rotation off; grkind", cur); g_warned = 1; }
        g_clock = -1;
        return;
    }
    rc = gd_stage_switch(next);
    if (rc == 0) {
        gd_log("stage_switch: switching to grkind", next);
        g_clock = -1; /* stage.on_start restarts it inside the switch */
    } else if (rc != BUSY) {
        gd_log("stage_switch: refused, rotation off; rc", rc);
        g_clock = -1;
    }
}

GD_HOOKS_BEGIN
    GD_HOOK_POST(GD_ADDR_STAGE_ON_START, 0, stage_start)
    GD_HOOK_POST(GD_ADDR_FRAME_RUN_PROCS, 100, on_frame_end)
GD_HOOKS_END

GD_MOD_MANIFEST("gd.stage_switch", "Stage switch", 0x00010000u, GD_ABI_VERSION, GD_ABI_VERSION, GD_FEATURE_NONE,
                GD_FEATURE_NONE, (gd_u32)(sizeof(g_clock) + sizeof(g_warned)));
