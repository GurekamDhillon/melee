/* gw_mex_ftfunction_runtime.c - phase 3: run a fighter's m-ex ftFunction blob at runtime.
 *
 * This is the bridge between the phase-1 PPC interpreter and the phase-2 blob loader on one side,
 * and the engine's per-kind dispatch surface (gw_Mex_GObjDispatch) on the other. For Ft_Kind_Sonic
 * it:
 *
 *   1. allocates the relocated code + the Arch_FighterFunc (mexData) tables + a guest stack from
 *      the fighter heap (HSD_MemAlloc), instead of the fixed 0x802F0000 pin the test suite uses;
 *   2. loads + relocates PlSn.dat's `ftFunction` into that region (gw_ftfunction_load_at);
 *   3. installs a guest->native resolver over the gw_mex_bridge table so the interpreter's absolute
 *      `bl` targets reach the port's native `gw_` functions;
 *   4. registers ONE native shim for onLoad (slot 0), so the engine's existing onLoad dispatch
 *      (gw_Mex_OnLoadDispatch) runs Sonic's PPC onLoad through the interpreter with r2 = mexData.
 *
 * The engine's onLoad dispatch (fighter.c -> Mex_OnLoadDispatch -> gw_Mex_OnLoadDispatch ->
 * gw_Mex_GObjDispatch) is already wired; the previous phase only needed the blob to be loaded and
 * a per-(event,kind) hook installed. This file also bridges the handful of onLoad's absolute call
 * targets that are m-ex-specific (MEX_IndexFighterItem, MEX_GetData) or that install GUEST-code
 * callbacks the native engine cannot invoke (GObj_SetupGXLink, HSD_GObj_SetupProc,
 * HSD_GObjGXLink_8039084C), by re-expressing them as logged native no-ops. This is the minimum to
 * let onLoad complete without crashing; the guest callbacks it would install (render/proc) are the
 * "incoming-call" problem and are deferred (see _research/mex-ppc-interpreter.md).
 *
 * This is NATIVE platform code (compiled directly with the i686 clang, not through gwtool).
 */
#include "gw.h"
#include "gw_test.h"
#include "gw_ppc.h"
#include "gw_mex_ftfunction.h"
#include "gw_mex_bridge.h"

#include <stdlib.h>
#include <string.h>

#define GW_MEX_KIND_SONIC 33      /* Ft_Kind_Sonic in the port (melee/ft/forward.h) */
#define GW_MEX_INTERNAL_SONIC 31  /* Sonic's m-ex internal character id */
#define GW_MEX_FTFUNC_DAT "PlSn.dat"

/* Arch_FighterFunc slot indices (Header.s: onLoad 0x00 ... GetTrailData 0xB4). See
 * gw_mex_ftfunction.c's slot_names table; onFrame is slot 23. */
#define GW_MEX_SLOT_ON_LOAD 0
#define GW_MEX_SLOT_MOVE_LOGIC 3
#define GW_MEX_SLOT_SPECIAL_N 4
#define GW_MEX_SLOT_SPECIAL_N_AIR 5
#define GW_MEX_SLOT_SPECIAL_S 6
#define GW_MEX_SLOT_SPECIAL_S_AIR 7
#define GW_MEX_SLOT_SPECIAL_HI 8
#define GW_MEX_SLOT_SPECIAL_HI_AIR 9
#define GW_MEX_SLOT_SPECIAL_LW 10
#define GW_MEX_SLOT_SPECIAL_LW_AIR 11
#define GW_MEX_SLOT_ON_FRAME 23
#define GW_MEX_SLOT_ON_ACTION_STATE_CHANGE 24
#define GW_MEX_SLOT_ON_REAPPLY_ATTR 25
#define GW_MEX_SLOT_ON_ITEM_PICKUP 13
#define GW_MEX_SLOT_ON_DOUBLE_JUMP 32
#define GW_MEX_SLOT_ON_USMASH 36

/* onLoad's absolute bl targets (see the disassembly in the evidence log). */
#define GW_MEX_GUEST_INDEX_ITEM   0x803D7058u /* m-ex MEX_IndexFighterItem (no vanilla symbol) */
#define GW_MEX_GUEST_GET_DATA     0x803D7094u /* m-ex MEX_GetData (no vanilla symbol) */
#define GW_MEX_GUEST_GXLINK_CLEAR 0x8039084Cu /* HSD_GObjGXLink_8039084C (gobj->gx_link is NONE) */
#define GW_MEX_GUEST_SETUP_GXLINK 0x8039069Cu /* GObj_SetupGXLink(cb=guest 0x80000BDC) */
#define GW_MEX_GUEST_SETUP_PROC   0x8038FD54u /* HSD_GObj_SetupProc(cb=guest 0x80000C74) */

/* Arch_FighterFunc layout the loader writes (see gw_mex_ftfunction.c): a 46-slot pointer array
 * followed by the per-kind tables. 0x2000 bytes covers the slot array + 25 overridden slots'
 * 64-entry per-kind tables with margin. */
#define GW_MEX_MEXDATA_SIZE 0x2000u
#define GW_MEX_STACK_SIZE 0x10000u
#define GW_MEX_GETDATA_SIZE 0x1000u /* synthetic safe buffer MEX_GetData(8) hands back */

static gw_ftfunction gw_mex_ff;      /* the loaded, relocated blob (code + overrides) */
static uint32_t gw_mex_stack_top;    /* guest stack top (r1) */
static uint32_t gw_mex_getdata_buf;  /* guest buffer backing the MEX_GetData(8) shim */
static int gw_mex_installed;         /* 1 once installed, -1 on failure */

static uint32_t gw_mex_override_target(uint32_t slot) {
    int i;
    for (i = 0; i < gw_mex_ff.override_count; ++i) {
        if (!gw_mex_ff.overrides[i].is_func_addr && gw_mex_ff.overrides[i].slot == slot) {
            return gw_mex_ff.overrides[i].target;
        }
    }
    return 0;
}

/* ---- native shims for onLoad's m-ex / guest-callback call targets ---------------------
 * These have the gw_ppc_native_fn shape (8 word args, word return) the bridge marshals into.
 * Each is a logged no-op: the real work (mexData item tables, GXLink/proc guest callbacks) is
 * not built in the port yet, so the shim documents the call and returns safely. */

static uint32_t gw_mex_shim_index_item(uint32_t fighter_id, uint32_t article_data,
                                       uint32_t article_id, uint32_t a3, uint32_t a4, uint32_t a5,
                                       uint32_t a6, uint32_t a7) {
    static int logged;
    (void)a3; (void)a4; (void)a5; (void)a6; (void)a7;
    if (!logged) {
        logged = 1;
        gw_log("interp: MEX_IndexFighterItem(fighter=%u article=0x%08X id=%u) -> no-op (mexData "
               "item tables not built in the port)", fighter_id, article_data, article_id);
    }
    return 0;
}

static uint32_t gw_mex_shim_get_data(uint32_t id, uint32_t a1, uint32_t a2, uint32_t a3,
                                     uint32_t a4, uint32_t a5, uint32_t a6, uint32_t a7) {
    static int logged;
    (void)a1; (void)a2; (void)a3; (void)a4; (void)a5; (void)a6; (void)a7;
    if (!logged) {
        logged = 1;
        gw_log("interp: MEX_GetData(id=%u) -> synthetic buffer 0x%08X (OFST_* metadata not built)",
               id, gw_mex_getdata_buf);
    }
    /* id 8 = CostumeSymbol = OFST_Char_CostumeRuntimePointers. Hand back the synthetic buffer,
     * whose per-kind slots point at a zeroed sub-region so onLoad's costume lookup reads NULL and
     * skips (rather than faulting on an unbuilt table). Other ids are unused by onLoad. */
    return (id == 8u) ? gw_mex_getdata_buf : 0u;
}

static uint32_t gw_mex_shim_gxlink_clear(uint32_t gobj, uint32_t a1, uint32_t a2, uint32_t a3,
                                         uint32_t a4, uint32_t a5, uint32_t a6, uint32_t a7) {
    static int logged;
    (void)a1; (void)a2; (void)a3; (void)a4; (void)a5; (void)a6; (void)a7;
    if (!logged) {
        logged = 1;
        gw_log("interp: HSD_GObjGXLink_8039084C(gobj=0x%08X) -> skipped (gobj->gx_link is "
               "HSD_GOBJ_GXLINK_NONE at onLoad time)", gobj);
    }
    return 0;
}

static uint32_t gw_mex_shim_setup_gxlink(uint32_t gobj, uint32_t cb, uint32_t gx_link,
                                         uint32_t priority, uint32_t a4, uint32_t a5, uint32_t a6,
                                         uint32_t a7) {
    static int logged;
    (void)a4; (void)a5; (void)a6; (void)a7;
    if (!logged) {
        logged = 1;
        gw_log("interp: GObj_SetupGXLink(gobj=0x%08X cb=0x%08X link=%u pri=%u) -> guest render "
               "callback deferred (incoming-call problem)", gobj, cb, gx_link, priority);
    }
    return 0;
}

static uint32_t gw_mex_shim_setup_proc(uint32_t gobj, uint32_t cb, uint32_t priority, uint32_t a3,
                                       uint32_t a4, uint32_t a5, uint32_t a6, uint32_t a7) {
    static int logged;
    (void)a3; (void)a4; (void)a5; (void)a6; (void)a7;
    if (!logged) {
        logged = 1;
        gw_log("interp: HSD_GObj_SetupProc(gobj=0x%08X cb=0x%08X pri=%u) -> guest proc callback "
               "deferred (incoming-call problem)", gobj, cb, priority);
    }
    return 0;
}

/* guest -> native resolver: m-ex-only helpers and guest-callback installers resolve to the native
 * shims above; everything else resolves through the build-time bridge table. Fighter_ChangeMotionState
 * (0x800693AC) is tagged with its float signature so the bridge marshals its f1..f3 float args. */
static gw_ppc_native_fn gw_mex_interp_resolve(uint32_t guest_addr, void *ctx, gw_ppc_sig *sig) {
    int kind;
    uint32_t native;
    (void)ctx;
    switch (guest_addr) {
    case GW_MEX_GUEST_INDEX_ITEM:
        return gw_mex_shim_index_item;
    case GW_MEX_GUEST_GET_DATA:
        return gw_mex_shim_get_data;
    case GW_MEX_GUEST_GXLINK_CLEAR:
        return gw_mex_shim_gxlink_clear;
    case GW_MEX_GUEST_SETUP_GXLINK:
        return gw_mex_shim_setup_gxlink;
    case GW_MEX_GUEST_SETUP_PROC:
        return gw_mex_shim_setup_proc;
    default:
        break;
    }
    native = gw_mex_bridge_lookup(guest_addr, &kind);
    if (native != 0 && kind == 1) {
        if (guest_addr == 0x800693ACu) {
            /* Fighter_ChangeMotionState(gobj, msid, flags, f32 anim_start, f32 anim_speed,
             * f32 anim_blend, arg3): ints in r3-r5, floats in f1-f3, then arg3 in r6. */
            sig->float_args = (1u << 3) | (1u << 4) | (1u << 5);
            sig->n_args = 7;
            sig->ret_float = 0;
        }
        return (gw_ppc_native_fn)(uintptr_t)native;
    }
    gw_log("interp: unresolved guest call target 0x%08X (no bridge entry)", guest_addr);
    return NULL;
}

/* Run one override slot through the interpreter: gobj -> r3, r2 = mexData, r1 = guest stack. */
static uint32_t gw_mex_interp_run(uint32_t slot, void *gobj) {
    uint32_t target = gw_mex_override_target(slot);
    uint32_t args[1];
    if (target == 0) {
        return 0;
    }
    args[0] = (uint32_t)(uintptr_t)gobj;
    return gw_ppc_call(target, args, 1, gw_mex_ff.mexdata_base, gw_mex_stack_top);
}

/* Two-argument variant (OnItemPickup): gobj -> r3, arg1 -> r4. */
static uint32_t gw_mex_interp_run2(uint32_t slot, void *gobj, uint32_t arg1) {
    uint32_t target = gw_mex_override_target(slot);
    uint32_t args[2];
    if (target == 0) {
        return 0;
    }
    args[0] = (uint32_t)(uintptr_t)gobj;
    args[1] = arg1;
    return gw_ppc_call(target, args, 2, gw_mex_ff.mexdata_base, gw_mex_stack_top);
}

/* onLoad (slot 0) - actually runs Sonic's PPC onLoad through the interpreter. */
static void gw_mex_interp_onload(void *gobj) {
    uint32_t target = gw_mex_override_target(GW_MEX_SLOT_ON_LOAD);
    uint32_t r3;
    gw_log("interp: onLoad kind=%d entry=0x%08X gobj=%p running", GW_MEX_KIND_SONIC, target, gobj);
    r3 = gw_mex_interp_run(GW_MEX_SLOT_ON_LOAD, gobj);
    gw_log("interp: onLoad kind=%d entry=0x%08X ran, r3=0x%08X", GW_MEX_KIND_SONIC, target, r3);
}

/* onFrame (slot 23) - runs Sonic's PPC onFrame per fighter per frame (Fighter_8006A360 ->
 * Mex_OnFrameDispatch -> gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_FRAME)). The log is throttled to one
 * line per second (60 invocations), since OnFrame fires at 60 Hz per fighter. */
static void gw_mex_interp_onframe(void *gobj) {
    uint32_t target = gw_mex_override_target(GW_MEX_SLOT_ON_FRAME);
    static int first = 1;
    static uint32_t count;
    uint32_t r3;

    if (target == 0) {
        return;
    }
    if (first) {
        first = 0;
        gw_log("interp: onFrame kind=%d entry=0x%08X gobj=%p running (per-frame)",
               GW_MEX_KIND_SONIC, target, gobj);
    }
    r3 = gw_mex_interp_run(GW_MEX_SLOT_ON_FRAME, gobj);
    ++count;
    if ((count % 60u) == 1u) {
        gw_log("interp: onFrame kind=%d invocation %u ran, r3=0x%08X", GW_MEX_KIND_SONIC, count,
               r3);
    }
}

/* Run one override slot, logging its first invocation and a throttled progress line. Used by the
 * auto-firing overrides (onActionStateChange / onReapplyAttr) so their execution is provable in the
 * log without flooding. Each slot's first invocation is always logged; later ones every 60. */
static uint32_t gw_mex_interp_run_logged(uint32_t slot, const char *name, void *gobj) {
    uint32_t target = gw_mex_override_target(slot);
    static int first_logged[64];
    static uint32_t count[64];
    uint32_t r3;

    if (target == 0) {
        return 0;
    }
    ++count[slot];
    if (slot < 64u && !first_logged[slot]) {
        first_logged[slot] = 1;
        gw_log("interp: %s kind=%d entry=0x%08X gobj=%p invocation 1 running", name,
               GW_MEX_KIND_SONIC, target, gobj);
    }
    r3 = gw_mex_interp_run(slot, gobj);
    if (slot < 64u && (count[slot] == 1u || (count[slot] % 60u) == 1u)) {
        gw_log("interp: %s kind=%d invocation %u ran, r3=0x%08X", name, GW_MEX_KIND_SONIC,
               count[slot], r3);
    }
    return r3;
}

/* Two-argument logged variant (OnItemPickup): logs its first invocation with the item gobj, then
 * throttles to one line per 60 invocations. */
static uint32_t gw_mex_interp_run_logged2(uint32_t slot, const char *name, void *gobj,
                                          uint32_t arg1) {
    uint32_t target = gw_mex_override_target(slot);
    static int first_logged2[64];
    static uint32_t count2[64];
    uint32_t r3;

    if (target == 0) {
        return 0;
    }
    ++count2[slot];
    if (slot < 64u && !first_logged2[slot]) {
        first_logged2[slot] = 1;
        gw_log("interp: %s kind=%d entry=0x%08X gobj=%p item=0x%08X invocation 1 running", name,
               GW_MEX_KIND_SONIC, target, gobj, arg1);
    }
    r3 = gw_mex_interp_run2(slot, gobj, arg1);
    if (slot < 64u && (count2[slot] == 1u || (count2[slot] % 60u) == 1u)) {
        gw_log("interp: %s kind=%d invocation %u ran, r3=0x%08X", name, GW_MEX_KIND_SONIC,
               count2[slot], r3);
    }
    return r3;
}

/* onActionStateChange (slot 24) - dispatched from ftcolanim.c's four ftData_UnkMotionStates4 call
 * sites (ftCo_800C0134 / ftCo_800C0200 / ftCo_800C0408). Fires when the fighter's colour-anim
 * action state changes, so it runs during spawn/respawn without any input. */
static void gw_mex_interp_action_state_change(void *gobj) {
    gw_mex_interp_run_logged(GW_MEX_SLOT_ON_ACTION_STATE_CHANGE, "onActionStateChange", gobj);
}

/* onReapplyAttr (slot 25) - dispatched from ftCo_800D105C (ftchangeparam.c), which re-applies the
 * fighter's DAT attributes (plus metal/bunny-hood modifiers) at spawn and on item effects. */
static void gw_mex_interp_reapply_attr(void *gobj) {
    gw_mex_interp_run_logged(GW_MEX_SLOT_ON_REAPPLY_ATTR, "onReapplyAttr", gobj);
}

/* The 8 specials (slots 4-11) - dispatched from the existing ftData_Special* call sites in
 * ftCo_Attack100.c / ftCo_SpecialS.c / ftCo_SpecialAir.c. They fire on B-button input, so in a
 * headless Target Test they only run when MELEE_PAD_SCRIPT drives a B press. */
#define GW_MEX_SPECIAL_INTERP(slot, NAME)                                          \
    static void gw_mex_interp_##NAME(void *gobj) {                                 \
        gw_mex_interp_run_logged(slot, #NAME, gobj);                               \
    }
GW_MEX_SPECIAL_INTERP(GW_MEX_SLOT_SPECIAL_N, special_n)
GW_MEX_SPECIAL_INTERP(GW_MEX_SLOT_SPECIAL_N_AIR, special_n_air)
GW_MEX_SPECIAL_INTERP(GW_MEX_SLOT_SPECIAL_S, special_s)
GW_MEX_SPECIAL_INTERP(GW_MEX_SLOT_SPECIAL_S_AIR, special_s_air)
GW_MEX_SPECIAL_INTERP(GW_MEX_SLOT_SPECIAL_HI, special_hi)
GW_MEX_SPECIAL_INTERP(GW_MEX_SLOT_SPECIAL_HI_AIR, special_hi_air)
GW_MEX_SPECIAL_INTERP(GW_MEX_SLOT_SPECIAL_LW, special_lw)
GW_MEX_SPECIAL_INTERP(GW_MEX_SLOT_SPECIAL_LW_AIR, special_lw_air)

/* onDoubleJump (slot 32) - dispatched from ftCo_800CBAC4 (ftCo_JumpAerial.c), the common aerial-
 * jump enter. Fires when the fighter double/multi-jumps, so it needs an air jump input. */
static void gw_mex_interp_double_jump(void *gobj) {
    gw_mex_interp_run_logged(GW_MEX_SLOT_ON_DOUBLE_JUMP, "onDoubleJump", gobj);
}

/* onUSmash (slot 36) - dispatched from doEnter (ftCo_AttackHi4.c), the up-smash enter. Fires on
 * up+A input. */
static void gw_mex_interp_usmash(void *gobj) {
    gw_mex_interp_run_logged(GW_MEX_SLOT_ON_USMASH, "onUSmash", gobj);
}

/* onItemPickup (slot 13) - dispatched from ftpickupitem_800948A8 (ftpickupitem.c) with the picked
 * item gobj. Fires only when the fighter picks up an item, so in Target Test it cannot fire. */
static void gw_mex_interp_item_pickup(void *gobj, void *arg1) {
    gw_mex_interp_run_logged2(GW_MEX_SLOT_ON_ITEM_PICKUP, "onItemPickup", gobj,
                              (uint32_t)(uintptr_t)arg1);
}

/* Called from game code (ftData_8008572C) once Sonic's data is on the disc. `kind` is the port's
 * Ft_Kind_Sonic (33). */
void gw_Mex_FtFunctionInstall(int kind) {
    uint32_t code_base, mexdata_base, stack_base, getdata_base;
    extern void *gw_HSD_MemAlloc(uint32_t size);
    int rc;

    if (kind != GW_MEX_KIND_SONIC || gw_mex_installed != 0) {
        return;
    }
    gw_mex_installed = -1;

    code_base = (uint32_t)(uintptr_t)gw_HSD_MemAlloc(0x6000u);
    mexdata_base = (uint32_t)(uintptr_t)gw_HSD_MemAlloc(GW_MEX_MEXDATA_SIZE);
    stack_base = (uint32_t)(uintptr_t)gw_HSD_MemAlloc(GW_MEX_STACK_SIZE);
    getdata_base = (uint32_t)(uintptr_t)gw_HSD_MemAlloc(GW_MEX_GETDATA_SIZE);
    if (code_base == 0 || mexdata_base == 0 || stack_base == 0 || getdata_base == 0) {
        gw_log("interp: ftFunction install failed: heap allocation returned NULL");
        return;
    }
    memset((void *)(uintptr_t)mexdata_base, 0, GW_MEX_MEXDATA_SIZE);
    memset((void *)(uintptr_t)getdata_base, 0, GW_MEX_GETDATA_SIZE);
    gw_mex_getdata_buf = getdata_base;
    {
        /* Make every per-kind slot of the synthetic costume table point at a zeroed sub-region,
         * so onLoad's costume lookup dereferences valid guest memory and reads NULL (skips). */
        uint32_t i;
        for (i = 0; i < 0x400u / 4u; ++i) {
            gw_w32((void *)(uintptr_t)(getdata_base + 4u * i), getdata_base + 0x400u);
        }
    }

    rc = gw_ftfunction_load_at(GW_MEX_FTFUNC_DAT, GW_MEX_INTERNAL_SONIC, code_base, mexdata_base,
                               &gw_mex_ff);
    if (rc != GW_FTFUNC_OK) {
        gw_log("interp: ftFunction install failed: load returned %d", rc);
        return;
    }

    gw_mex_stack_top = stack_base + GW_MEX_STACK_SIZE - 0x100u;
    gw_ppc_set_bridge(gw_mex_interp_resolve, NULL, gw_mex_ff.code_base,
                      gw_mex_ff.code_base + gw_mex_ff.code_size);

    /* Install the onLoad and onFrame overrides (this phase's deliverables). The other engine
     * events (onDeath/onDestroy/...) are not registered so they keep their vanilla behaviour. */
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_LOAD, GW_MEX_KIND_SONIC, gw_mex_interp_onload);
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_FRAME, GW_MEX_KIND_SONIC, gw_mex_interp_onframe);
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_ACTION_STATE_CHANGE, GW_MEX_KIND_SONIC,
                        gw_mex_interp_action_state_change);
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_REAPPLY_ATTR, GW_MEX_KIND_SONIC,
                        gw_mex_interp_reapply_attr);
    gw_Mex_HookRegister(GW_MEX_EVENT_SPECIAL_N, GW_MEX_KIND_SONIC, gw_mex_interp_special_n);
    gw_Mex_HookRegister(GW_MEX_EVENT_SPECIAL_N_AIR, GW_MEX_KIND_SONIC,
                        gw_mex_interp_special_n_air);
    gw_Mex_HookRegister(GW_MEX_EVENT_SPECIAL_S, GW_MEX_KIND_SONIC, gw_mex_interp_special_s);
    gw_Mex_HookRegister(GW_MEX_EVENT_SPECIAL_S_AIR, GW_MEX_KIND_SONIC,
                        gw_mex_interp_special_s_air);
    gw_Mex_HookRegister(GW_MEX_EVENT_SPECIAL_HI, GW_MEX_KIND_SONIC, gw_mex_interp_special_hi);
    gw_Mex_HookRegister(GW_MEX_EVENT_SPECIAL_HI_AIR, GW_MEX_KIND_SONIC,
                        gw_mex_interp_special_hi_air);
    gw_Mex_HookRegister(GW_MEX_EVENT_SPECIAL_LW, GW_MEX_KIND_SONIC, gw_mex_interp_special_lw);
    gw_Mex_HookRegister(GW_MEX_EVENT_SPECIAL_LW_AIR, GW_MEX_KIND_SONIC,
                        gw_mex_interp_special_lw_air);
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_DOUBLE_JUMP, GW_MEX_KIND_SONIC,
                        gw_mex_interp_double_jump);
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_USMASH, GW_MEX_KIND_SONIC, gw_mex_interp_usmash);
    gw_Mex_HookRegister2(GW_MEX_EVENT_ON_ITEM_PICKUP, GW_MEX_KIND_SONIC,
                         gw_mex_interp_item_pickup);

    gw_mex_installed = 1;
    gw_log("interp: installed Sonic ftFunction (code 0x%08X..0x%08X, mexData 0x%08X, stack "
           "0x%08X, %d overrides); onLoad/onFrame/onActionStateChange/onReapplyAttr/8 specials/"
           "onDoubleJump/onUSmash/onItemPickup active",
           gw_mex_ff.code_base, gw_mex_ff.code_base + gw_mex_ff.code_size, mexdata_base,
           stack_base, gw_mex_ff.override_count);
}

/* Register the module's self-contained tests (bridge lookup + resolver). */
void gw_mex_ftfunction_runtime_tests_register(void);

/* ---- tests --------------------------------------------------------------------------- */

static int test_bridge_lookup_memcpy(void) {
    int kind = -1;
    uint32_t native = gw_mex_bridge_lookup(0x800031F4u, &kind); /* memcpy */
    if (native == 0 || kind != 1) {
        gw_test_fail("bridge lookup of memcpy (0x800031F4) failed: native=0x%08X kind=%d", native,
                     kind);
        return 1;
    }
    if (native < 0x10000000u || native > 0x20000000u) {
        gw_test_fail("bridge memcpy native 0x%08X not in the image range", native);
        return 1;
    }
    return 0;
}

static int test_bridge_lookup_global(void) {
    int kind = -1;
    uint32_t native = gw_mex_bridge_lookup(0x803C1154u, &kind); /* ftData_OnLoad (object) */
    if (native == 0 || kind != 0) {
        gw_test_fail("bridge lookup of ftData_OnLoad (0x803C1154) failed: native=0x%08X kind=%d",
                     native, kind);
        return 1;
    }
    return 0;
}

static int test_bridge_lookup_miss(void) {
    int kind = -1;
    if (gw_mex_bridge_lookup(0x81234567u, &kind) != 0) {
        gw_test_fail("bridge lookup of an unmapped guest address should return 0");
        return 1;
    }
    return 0;
}

void gw_mex_ftfunction_runtime_tests_register(void) {
    gw_test_register("bridge_lookup_memcpy", test_bridge_lookup_memcpy);
    gw_test_register("bridge_lookup_global", test_bridge_lookup_global);
    gw_test_register("bridge_lookup_miss", test_bridge_lookup_miss);
}
