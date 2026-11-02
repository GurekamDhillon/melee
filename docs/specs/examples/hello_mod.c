/* SPDX-License-Identifier: MIT
 * Copyright (c) 2026 GurekamDhillon */
/* hello_mod.c - mod-c-v1 example. Port 1's fighter (slot 0) takes +1% extra every time damage is applied to it.
 * All state is in globals below (no heap, no statics with dynamic init); that is what makes rollback correct with no
 * save/load code. Build:
 *   native: clang -std=c11 -O2 -ffp-contract=off -fwrapv -fno-math-errno -ffreestanding -fsyntax-only hello_mod.c
 *   wasm:   clang --target=wasm32 -O2 -ffp-contract=off -nostdlib -Wl,--no-entry -Wl,--export-dynamic -o hello_mod.wasm hello_mod.c
 */
#include "../gdmod.h"

#define TARGET_SLOT 0

/* mod state: plain globals, zero-initialised, no pointers */
static gd_u32 g_hits;
static gd_f32 g_bonus_total;

/* damage.fighter_take_damage  0x8006CC7C  void(gd_fighter, f32): post, runs after the game applied `damage_amount` */
static void on_take_damage_post(gd_fighter victim, gd_f32 damage_amount)
{
    if (gd_fighter_slot(victim) != TARGET_SLOT || !(damage_amount > 0.0f)) {
        return;
    }
    gd_f32 pct = gd_fighter_get_percent(victim);
    gd_fighter_set_percent(victim, pct + 1.0f);
    g_hits += 1u;
    g_bonus_total += 1.0f;
}

GD_HOOKS_BEGIN
    GD_HOOK_POST(GD_ADDR_FIGHTER_TAKE_DAMAGE, 0, on_take_damage_post)
GD_HOOKS_END

GD_MOD_MANIFEST("gd.hello", "Hello: +1% on port 1", 0x00010000u, GD_ABI_VERSION, GD_ABI_VERSION, GD_FEATURE_NONE,
                GD_FEATURE_NONE, (gd_u32)(sizeof(g_hits) + sizeof(g_bonus_total)));
