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

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

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
#define GW_MEX_GUEST_GET_FT_ITEM_ID 0x803D7088u /* m-ex MEX_GetFtItemID (no vanilla symbol) */
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

/* MoveLogic (slot 3) is a MotionState[] table the engine indexes by `motion_id - fp->x18`. Its
 * layout (mirroring src/melee/ft/types.h MotionState, 0x20 bytes) is:
 *   0x00 anim_id, 0x04 x4_flags, 0x08 move_id<<24, 0x0C anim_cb, 0x10 input_cb,
 *   0x14 phys_cb, 0x18 coll_cb, 0x1C cam_cb.
 * Sonic's table (PlSn.dat) has 31 entries (code +0x1D4 .. +0x5B4) and every cam_cb is the vanilla
 * ftCamera_UpdateCameraBox (0x800761C8). */
#define GW_MEX_MOVE_MAX_ENTRIES 64
#define GW_MEX_MOVE_CAM_CB_GUEST 0x800761C8u /* ftCamera_UpdateCameraBox */

static gw_ftfunction gw_mex_ff;      /* the loaded, relocated blob (code + overrides) */
static uint32_t gw_mex_stack_top;    /* guest stack top (r1) */
static uint32_t gw_mex_getdata_buf;  /* guest buffer backing the MEX_GetData(8) shim */
static int gw_mex_installed;         /* 1 once installed, -1 on failure */

static uint32_t gw_mex_movelogic_table;  /* guest addr of Sonic's MoveLogic MotionState[] */
static int gw_mex_movelogic_entries;     /* number of MotionState entries (31) */
static uint32_t gw_mex_move_cb_guest[GW_MEX_MOVE_MAX_ENTRIES][4]; /* preserved guest anim/input/phys/coll */

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

static uint32_t gw_mex_shim_get_ft_item_id(uint32_t gobj, uint32_t item_id, uint32_t a2,
                                           uint32_t a3, uint32_t a4, uint32_t a5, uint32_t a6,
                                           uint32_t a7) {
    static int logged;
    static uint32_t count;
    uint32_t fighter_id = 0;
    (void)a2; (void)a3; (void)a4; (void)a5; (void)a6; (void)a7;
    ++count;
    if (!logged) {
        uint32_t fd = gw_r32((const void *)(uintptr_t)(gobj + 0x2Cu));
        if (fd >= 0x80000000u && fd < 0x80000000u + gw_mem1_size) {
            fighter_id = gw_r32((const void *)(uintptr_t)(fd + 0x4u));
        }
        logged = 1;
        gw_log("interp: MEX_GetFtItemID(gobj=0x%08X item=%u fighter=%u) -> 0 (mexData item "
               "lookup table not built in the port)", gobj, item_id, fighter_id);
    } else if ((count % 60u) == 1u) {
        gw_log("interp: MEX_GetFtItemID hit %u -> 0 (no-op)", count);
    }
    return 0;
}

/* ---- incoming calls: native code calling guest code ---------------------------------------
 * The "incoming-call problem". Guest code hands a guest function address to a native engine API
 * (GObj_SetupGXLink, HSD_GObj_SetupProc, ...), which stores it and later CALLS it as a native
 * function pointer. A guest address is not x86 code, so that call executes MEM1 bytes and dies
 * with ACCESS_VIOLATION at the guest address (seen: 0x807FA400 = SpecialNHit_Enter).
 *
 * Fix: a pool of native thunks. Each thunk is a real x86 function bound to one guest address;
 * when native code calls it, it interprets that guest function. gw_mex_callable() maps whatever
 * the guest passes to something native code can call:
 *   - an address inside the blob       -> a thunk that interprets it
 *   - a VANILLA engine function's guest address (e.g. a stock callback the guest reuses)
 *                                      -> that function's native gw_ address, via the bridge;
 *                                         thunking it would interpret engine code we do not have
 *   - 0                                -> 0
 * Thunks are deduplicated by guest address and never freed: the set of distinct callbacks a
 * fighter installs is small and fixed, and a callback may be invoked long after it was set.
 *
 * Arity: each thunk forwards FOUR integer arguments. Callers here pass 1 (proc/event callbacks)
 * or 2 (render callbacks: gobj + pass). Under cdecl the caller pushes and pops its own arguments,
 * so reading two extra slots is harmless - they are caller-frame words the guest never reads.
 * Forwarding matters: GXLink_Sonic returns immediately unless pass == 2, so a thunk that dropped
 * the second argument would silently draw nothing and look like a rendering bug. */

#define GW_MEX_THUNK_MAX 64

static uint32_t gw_mex_thunk_guest[GW_MEX_THUNK_MAX];
static int gw_mex_thunk_count;

static uint32_t gw_mex_thunk_run(int k, uint32_t a0, uint32_t a1, uint32_t a2, uint32_t a3) {
    uint32_t args[4];
    args[0] = a0;
    args[1] = a1;
    args[2] = a2;
    args[3] = a3;
    return gw_ppc_call(gw_mex_thunk_guest[k], args, 4, gw_mex_ff.mexdata_base, gw_mex_stack_top);
}

#define GW_MEX_THUNK(k) \
    static uint32_t gw_mex_thunk_##k(uint32_t a0, uint32_t a1, uint32_t a2, uint32_t a3) { \
        return gw_mex_thunk_run(k, a0, a1, a2, a3); \
    }
#define GW_MEX_THUNK8(b) \
    GW_MEX_THUNK(b##0) GW_MEX_THUNK(b##1) GW_MEX_THUNK(b##2) GW_MEX_THUNK(b##3) \
    GW_MEX_THUNK(b##4) GW_MEX_THUNK(b##5) GW_MEX_THUNK(b##6) GW_MEX_THUNK(b##7)
GW_MEX_THUNK8(0) GW_MEX_THUNK8(1) GW_MEX_THUNK8(2) GW_MEX_THUNK8(3)
GW_MEX_THUNK8(4) GW_MEX_THUNK8(5) GW_MEX_THUNK8(6) GW_MEX_THUNK8(7)

typedef uint32_t (*gw_mex_thunk_fn)(uint32_t, uint32_t, uint32_t, uint32_t);
#define GW_MEX_THUNK_REF8(b) \
    gw_mex_thunk_##b##0, gw_mex_thunk_##b##1, gw_mex_thunk_##b##2, gw_mex_thunk_##b##3, \
    gw_mex_thunk_##b##4, gw_mex_thunk_##b##5, gw_mex_thunk_##b##6, gw_mex_thunk_##b##7
static const gw_mex_thunk_fn gw_mex_thunks[GW_MEX_THUNK_MAX] = {
    GW_MEX_THUNK_REF8(0), GW_MEX_THUNK_REF8(1), GW_MEX_THUNK_REF8(2), GW_MEX_THUNK_REF8(3),
    GW_MEX_THUNK_REF8(4), GW_MEX_THUNK_REF8(5), GW_MEX_THUNK_REF8(6), GW_MEX_THUNK_REF8(7),
};

static int gw_mex_in_blob(uint32_t a) {
    return a >= gw_mex_ff.code_base && a < gw_mex_ff.code_base + gw_mex_ff.code_size;
}

/* Guest function address -> something native code can call. See the block comment above. */
static uint32_t gw_mex_callable(uint32_t guest, const char *why) {
    int k, kind = 0;
    uint32_t native;
    if (guest == 0u) {
        return 0u;
    }
    if (gw_mex_in_blob(guest)) {
        for (k = 0; k < gw_mex_thunk_count; ++k) {
            if (gw_mex_thunk_guest[k] == guest) {
                return (uint32_t) (uintptr_t) gw_mex_thunks[k];
            }
        }
        if (gw_mex_thunk_count >= GW_MEX_THUNK_MAX) {
            gw_panic("interp: out of native thunks (%d) binding %s for %s", GW_MEX_THUNK_MAX,
                     gw_ppc_describe(guest), why);
        }
        k = gw_mex_thunk_count++;
        gw_mex_thunk_guest[k] = guest;
        gw_log("interp: thunk %d -> guest %s (%s)", k, gw_ppc_describe(guest), why);
        return (uint32_t) (uintptr_t) gw_mex_thunks[k];
    }
    native = gw_mex_bridge_lookup(guest, &kind);
    if (native != 0u && kind == 1) {
        return native; /* a vanilla engine function the guest is reusing as a callback */
    }
    gw_panic("interp: %s: callback 0x%08X is neither blob code nor a bridged engine function",
             why, guest);
    return 0u;
}

/* ---- incoming calls, general case: trap and emulate ---------------------------------------
 * The thunk pool above covers callbacks the guest hands to an engine API. It cannot cover the
 * other route: guest code STORING a code address straight into engine data. Sonic does exactly
 * that - `stw r9,0x21C0(r31)` puts SpecialNHit_Enter into fp->deal_dmg_cb - and the engine later
 * calls `fp->deal_dmg_cb(gobj)` with no API in between. Fighter alone has 19 such HSD_GObjEvent
 * fields (fp+0x21B0..0x21F8) with ~29 native call sites, of which one was routed; item and
 * effect code will add more. Patching call sites one by one would always miss the next.
 *
 * So catch it at the only point every such call has in common. Guest MEM1 is not executable, so
 * a native call to a guest address faults CLEANLY on the instruction fetch (DEP), with EIP equal
 * to the guest address and the stack exactly as the caller left it: return address at [esp],
 * arguments above it. A vectored handler that sees an execute fault inside the blob rewrites EIP
 * to gw_mex_trap_trampoline and resumes. Because nothing on the stack moved, the trampoline
 * receives the caller's arguments as though it had been called directly, interprets the guest
 * function, and `ret`s straight back to the original caller - which cleans up its own arguments
 * (cdecl). No game-source edits; covers every such call site, including ones not found yet.
 *
 * Deliberately narrow: only EXCEPTION_ACCESS_VIOLATION, only an EXECUTE fault (information[0] ==
 * 8), only when the faulting address is EIP and lies inside the blob. Everything else returns
 * CONTINUE_SEARCH untouched, so real crashes still reach the port's crash logger.
 *
 * Cost is one exception round trip per call. That is fine for event callbacks (on hit, on death)
 * but not for anything per-frame; the first trap per target is logged so a hot one can be given an
 * explicit route (as accessory4_cb has in fighter.c) instead. */

static __declspec(thread) uint32_t gw_mex_trap_target;

#define GW_MEX_TRAP_SEEN_MAX 32
static uint32_t gw_mex_trap_seen[GW_MEX_TRAP_SEEN_MAX];
static uint32_t gw_mex_trap_seen_count[GW_MEX_TRAP_SEEN_MAX];
static int gw_mex_trap_seen_n;

static uint32_t gw_mex_trap_trampoline(uint32_t a0, uint32_t a1, uint32_t a2, uint32_t a3) {
    /* Capture the target FIRST: the guest function may itself make a native call that traps
     * again, and that nested trap overwrites gw_mex_trap_target. */
    uint32_t target = gw_mex_trap_target;
    uint32_t args[4];
    args[0] = a0;
    args[1] = a1;
    args[2] = a2;
    args[3] = a3;
    return gw_ppc_call(target, args, 4, gw_mex_ff.mexdata_base, gw_mex_stack_top);
}

static void gw_mex_trap_note(uint32_t eip) {
    int i;
    for (i = 0; i < gw_mex_trap_seen_n; ++i) {
        if (gw_mex_trap_seen[i] == eip) {
            uint32_t n = ++gw_mex_trap_seen_count[i];
            if ((n % 600u) == 0u) {
                gw_log("interp: trap: %s has been called natively %u times - consider an "
                       "explicit route if this is per-frame", gw_ppc_describe(eip), n);
            }
            return;
        }
    }
    if (gw_mex_trap_seen_n < GW_MEX_TRAP_SEEN_MAX) {
        gw_mex_trap_seen[gw_mex_trap_seen_n] = eip;
        gw_mex_trap_seen_count[gw_mex_trap_seen_n] = 1u;
        ++gw_mex_trap_seen_n;
    }
    gw_log("interp: trap: native code called guest %s directly - emulating", gw_ppc_describe(eip));
}

static LONG CALLBACK gw_mex_exec_trap(PEXCEPTION_POINTERS ep) {
    PEXCEPTION_RECORD er = ep->ExceptionRecord;
    uint32_t eip;
    if (er->ExceptionCode != EXCEPTION_ACCESS_VIOLATION || er->NumberParameters < 2 ||
        er->ExceptionInformation[0] != 8 /* execute */) {
        return EXCEPTION_CONTINUE_SEARCH;
    }
    eip = (uint32_t) ep->ContextRecord->Eip;
    if ((uint32_t) er->ExceptionInformation[1] != eip || gw_mex_ff.code_size == 0u ||
        !gw_mex_in_blob(eip)) {
        return EXCEPTION_CONTINUE_SEARCH;
    }
    gw_mex_trap_note(eip);
    gw_mex_trap_target = eip;
    ep->ContextRecord->Eip = (DWORD) (uintptr_t) gw_mex_trap_trampoline;
    return EXCEPTION_CONTINUE_EXECUTION;
}

/* Call the real native engine function for `guest_addr` with the generic bridge shape. */
static uint32_t gw_mex_call_native(uint32_t guest_addr, uint32_t a0, uint32_t a1, uint32_t a2,
                                   uint32_t a3) {
    int kind = 0;
    uint32_t native = gw_mex_bridge_lookup(guest_addr, &kind);
    if (native == 0u || kind != 1) {
        gw_panic("interp: no native engine function for guest 0x%08X", guest_addr);
    }
    return ((gw_ppc_native_fn) (uintptr_t) native)(a0, a1, a2, a3, 0, 0, 0, 0);
}

/* HSD_GObjGXLink_8039084C(gobj): no callback argument, so no translation is needed. It used to
 * be skipped outright; it is a plain engine call and now runs for real. */
static uint32_t gw_mex_shim_gxlink_clear(uint32_t gobj, uint32_t a1, uint32_t a2, uint32_t a3,
                                         uint32_t a4, uint32_t a5, uint32_t a6, uint32_t a7) {
    (void) a1; (void) a2; (void) a3; (void) a4; (void) a5; (void) a6; (void) a7;
    return gw_mex_call_native(GW_MEX_GUEST_GXLINK_CLEAR, gobj, 0, 0, 0);
}

/* GObj_SetupGXLink(gobj, render_cb, gx_link, priority): translate render_cb, then call for real. */
static uint32_t gw_mex_shim_setup_gxlink(uint32_t gobj, uint32_t cb, uint32_t gx_link,
                                         uint32_t priority, uint32_t a4, uint32_t a5, uint32_t a6,
                                         uint32_t a7) {
    (void) a4; (void) a5; (void) a6; (void) a7;
    return gw_mex_call_native(GW_MEX_GUEST_SETUP_GXLINK, gobj,
                              gw_mex_callable(cb, "GObj_SetupGXLink render_cb"), gx_link,
                              priority);
}

/* HSD_GObj_SetupProc(gobj, cb, priority): translate cb, call for real, and return the proc the
 * engine hands back - the guest may keep it. */
static uint32_t gw_mex_shim_setup_proc(uint32_t gobj, uint32_t cb, uint32_t priority, uint32_t a3,
                                       uint32_t a4, uint32_t a5, uint32_t a6, uint32_t a7) {
    (void) a3; (void) a4; (void) a5; (void) a6; (void) a7;
    return gw_mex_call_native(GW_MEX_GUEST_SETUP_PROC, gobj,
                              gw_mex_callable(cb, "HSD_GObj_SetupProc cb"), priority, 0);
}

/* ---- bridged-call signatures -----------------------------------------------------------
 * The bridge defaults every target to "8 integer args, integer return" (gw_ppc_bridge_call).
 * That is correct for the majority of engine calls, which pass pointers and ints, but it is
 * silently WRONG for anything using the FPRs: float arguments get pulled from r3.. instead of
 * f1.., and a float return is dropped on the floor because the callee's f1 is never written back.
 *
 * That is not a crash - it is garbage that propagates. It cost a real bug: Sonic's neutral
 * special calls atan2f and then normalises the angle with
 *     while (a < 0) a += 2*PI;
 * With atan2f's return discarded, the guest read a stale f1; when that happened to be hugely
 * negative the loop could never reach zero and the game hung inside the interpreter with no
 * diagnostic (found via the GW_PPC_MAX_INSNS budget panic).
 *
 * This table is the float-signature set, keyed by guest address. It is deliberately explicit
 * rather than inferred: a wrong signature here is another silent-garbage bug, so each entry is
 * justified by the function's decomp prototype. NOTE this covers the math library and the one
 * engine function already known; the blob bridges ~101 distinct targets in total and the rest
 * still ride the integer default, so any engine function taking or returning a float is still
 * a latent instance of this bug. Deriving the full set from the decomp prototypes is the
 * principled fix. */
typedef struct gw_mex_sig_entry {
    uint32_t guest;
    uint32_t float_args; /* bit i => native arg slot i is a float from the next FPR */
    uint32_t n_args;
    int ret_float;
} gw_mex_sig_entry;

#define GW_MEX_SIG_F1 0x1u  /* (float) */
#define GW_MEX_SIG_F2 0x3u  /* (float, float) */

static const gw_mex_sig_entry gw_mex_sigs[] = {
    /* libm: one or two float args, float return. Addresses from config/GALE01/symbols.txt. */
    {0x8000CE50u, GW_MEX_SIG_F1, 1, 1}, /* expf   */
    {0x8000CEE0u, GW_MEX_SIG_F2, 2, 1}, /* powf   */
    {0x80022C30u, GW_MEX_SIG_F2, 2, 1}, /* atan2f <- the neutral-B hang */
    {0x80022D1Cu, GW_MEX_SIG_F1, 1, 1}, /* acosf  */
    {0x80022DBCu, GW_MEX_SIG_F1, 1, 1}, /* asinf  */
    {0x80022E68u, GW_MEX_SIG_F1, 1, 1}, /* atanf  */
    {0x803261BCu, GW_MEX_SIG_F1, 1, 1}, /* tanf   */
    {0x80326240u, GW_MEX_SIG_F1, 1, 1}, /* cosf   */
    {0x803263D4u, GW_MEX_SIG_F1, 1, 1}, /* sinf   */
    {0x803265A8u, GW_MEX_SIG_F1, 1, 1}, /* logf   */
    {0x80364340u, GW_MEX_SIG_F2, 2, 1}, /* fmodf  */
    /* __cvt_fp2unsigned(double) was here as a float arg. That is WRONG: its parameter is a
     * DOUBLE, and the bridge marshals 4-byte argument slots only, so neither a float slot nor
     * the integer default can express it. Removed rather than left wrong - supporting a double
     * argument is a real gap in gw_ppc_bridge_call, not a table entry. */

    /* Fighter_ChangeMotionState(gobj, msid, flags, f32 anim_start, f32 anim_speed,
     * f32 anim_blend, arg3): ints in r3-r5, floats in f1-f3, then arg3 in r6. */
    {0x800693ACu, (1u << 3) | (1u << 4) | (1u << 5), 7, 0},
};

static const gw_mex_sig_entry *gw_mex_sig_lookup(uint32_t guest_addr) {
    unsigned i;
    for (i = 0; i < sizeof gw_mex_sigs / sizeof gw_mex_sigs[0]; ++i) {
        if (gw_mex_sigs[i].guest == guest_addr) {
            return &gw_mex_sigs[i];
        }
    }
    return NULL;
}

/* guest -> native resolver: m-ex-only helpers and guest-callback installers resolve to the native
 * shims above; everything else resolves through the build-time bridge table, tagged with its
 * float signature when gw_mex_sigs has one. */
static gw_ppc_native_fn gw_mex_interp_resolve(uint32_t guest_addr, void *ctx, gw_ppc_sig *sig) {
    int kind;
    uint32_t native;
    (void)ctx;
    switch (guest_addr) {
    case GW_MEX_GUEST_INDEX_ITEM:
        return gw_mex_shim_index_item;
    case GW_MEX_GUEST_GET_FT_ITEM_ID:
        return gw_mex_shim_get_ft_item_id;
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
        const gw_mex_sig_entry *e = gw_mex_sig_lookup(guest_addr);
        if (e != NULL) {
            sig->float_args = e->float_args;
            sig->n_args = e->n_args;
            sig->ret_float = e->ret_float;
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

/* ---- MoveLogic (slot 3) move-table runtime -------------------------------------------
 * MoveLogic is NOT code: it is a MotionState[] table the engine indexes by `motion_id - fp->x18`
 * (fighter.c:1235) to drive each action state's anim/phys/coll/cam callbacks. m-ex swaps
 * ftData_CharacterStateTables[kind] for this table (PlayerBlockInit.asm @ 0x80068B60, replacing
 * `addi r6,r3,4832` = the ftData_CharacterStateTables base). The table's anim/input/phys/coll
 * callbacks are guest PPC code, so each is rewritten to a native trampoline that interprets the
 * preserved guest address; cam_cb is the vanilla ftCamera_UpdateCameraBox, bridged to native. */

enum {
    GW_MEX_MOVE_CB_ANIM = 0,
    GW_MEX_MOVE_CB_INPUT,
    GW_MEX_MOVE_CB_PHYS,
    GW_MEX_MOVE_CB_COLL,
};

static uint32_t gw_mex_move_cb_guest_addr(void *gobj, int which) {
    uint32_t fd = gw_r32((const void *)(uintptr_t)((uintptr_t)gobj + 0x2Cu)); /* gobj->user_data */
    uint32_t motion_id, x18;
    int idx;
    if (fd < 0x80000000u || fd >= 0x80000000u + gw_mem1_size) {
        return 0;
    }
    motion_id = gw_r32((const void *)(uintptr_t)(fd + 0x10u)); /* fp->motion_id */
    x18 = gw_r32((const void *)(uintptr_t)(fd + 0x18u));       /* fp->x18 (341) */
    idx = (int)(motion_id - x18);
    if (idx < 0 || idx >= gw_mex_movelogic_entries) {
        return 0;
    }
    return gw_mex_move_cb_guest[idx][which];
}

static void gw_mex_move_call(void *gobj, int which) {
    uint32_t target = gw_mex_move_cb_guest_addr(gobj, which);
    uint32_t args[1];
    static const char *const names[4] = {"anim", "input", "phys", "coll"};
    static int first[4];
    static uint32_t count[4];
    uint32_t fd;

    if (target == 0) {
        return;
    }
    if (!first[which]) {
        first[which] = 1;
        fd = gw_r32((const void *)(uintptr_t)((uintptr_t)gobj + 0x2Cu));
        gw_log("interp: MoveLogic %s_cb kind=33 gobj=%p motion_id=0x%X anim_id=0x%X -> guest "
               "0x%08X (interpreting)",
               names[which], gobj, gw_r32((const void *)(uintptr_t)(fd + 0x10u)),
               gw_r32((const void *)(uintptr_t)(fd + 0x14u)), target);
    }
    args[0] = (uint32_t)(uintptr_t)gobj;
    gw_ppc_call(target, args, 1, gw_mex_ff.mexdata_base, gw_mex_stack_top);
    ++count[which];
    if (count[which] == 1u || (count[which] % 60u) == 1u) {
        gw_log("interp: MoveLogic %s_cb invocation %u ran", names[which], count[which]);
    }
}

static void gw_mex_move_anim_cb(void *gobj) { gw_mex_move_call(gobj, GW_MEX_MOVE_CB_ANIM); }
static void gw_mex_move_input_cb(void *gobj) { gw_mex_move_call(gobj, GW_MEX_MOVE_CB_INPUT); }
static void gw_mex_move_phys_cb(void *gobj) { gw_mex_move_call(gobj, GW_MEX_MOVE_CB_PHYS); }
static void gw_mex_move_coll_cb(void *gobj) { gw_mex_move_call(gobj, GW_MEX_MOVE_CB_COLL); }

/* General fighter-callback dispatch for the "incoming-call" slots (accessory1_cb/accessory4_cb/
 * deal_dmg_cb/...) that m-ex guest code overwrites with guest PPC function pointers. A guest
 * address (in MEM1) is interpreted; a native pointer is called directly. */
void gw_Mex_FighterCallbackDispatch(void *gobj, void *cb) {
    uint32_t a = (uint32_t)(uintptr_t)cb;
    uint32_t args[1];
    if (a == 0) {
        return;
    }
    if (a >= 0x80000000u && a < 0x80000000u + gw_mem1_size && gw_mex_ff.mexdata_base != 0) {
        args[0] = (uint32_t)(uintptr_t)gobj;
        gw_ppc_call(a, args, 1, gw_mex_ff.mexdata_base, gw_mex_stack_top);
        return;
    }
    ((gwmex_gobj_fn)cb)(gobj);
}

/* Rewrite Sonic's MoveLogic table in place: preserve the guest anim/input/phys/coll callbacks and
 * replace them with native trampolines; replace cam_cb with the bridged ftCamera_UpdateCameraBox.
 * Runs once, at install, before the engine ever reads the table. */
static void gw_mex_movelogic_setup(void) {
    uint32_t table = gw_mex_override_target(GW_MEX_SLOT_MOVE_LOGIC);
    uint32_t next = gw_mex_override_target(GW_MEX_SLOT_SPECIAL_N);
    uint32_t cam_native;
    int cam_kind = -1;
    int entries, i;

    if (table == 0 || next <= table) {
        gw_log("interp: MoveLogic table missing (slot3=0x%08X slot4=0x%08X)", table, next);
        return;
    }
    entries = (int)((next - table) / 0x20u);
    if (entries < 1 || entries > GW_MEX_MOVE_MAX_ENTRIES) {
        gw_log("interp: MoveLogic table size implausible (%d entries)", entries);
        return;
    }
    cam_native = gw_mex_bridge_lookup(GW_MEX_MOVE_CAM_CB_GUEST, &cam_kind);
    if (cam_native == 0 || cam_kind != 1) {
        gw_log("interp: MoveLogic cam_cb 0x%08X not bridged -> table left unwired",
               GW_MEX_MOVE_CAM_CB_GUEST);
        return;
    }

    gw_mex_movelogic_entries = entries;
    for (i = 0; i < entries; ++i) {
        uint32_t e = table + (uint32_t)i * 0x20u;
        gw_mex_move_cb_guest[i][GW_MEX_MOVE_CB_ANIM] =
            gw_r32((const void *)(uintptr_t)(e + 0x0Cu));
        gw_mex_move_cb_guest[i][GW_MEX_MOVE_CB_INPUT] =
            gw_r32((const void *)(uintptr_t)(e + 0x10u));
        gw_mex_move_cb_guest[i][GW_MEX_MOVE_CB_PHYS] =
            gw_r32((const void *)(uintptr_t)(e + 0x14u));
        gw_mex_move_cb_guest[i][GW_MEX_MOVE_CB_COLL] =
            gw_r32((const void *)(uintptr_t)(e + 0x18u));
        gw_w32((void *)(uintptr_t)(e + 0x0Cu), (uint32_t)(uintptr_t)gw_mex_move_anim_cb);
        gw_w32((void *)(uintptr_t)(e + 0x10u), (uint32_t)(uintptr_t)gw_mex_move_input_cb);
        gw_w32((void *)(uintptr_t)(e + 0x14u), (uint32_t)(uintptr_t)gw_mex_move_phys_cb);
        gw_w32((void *)(uintptr_t)(e + 0x18u), (uint32_t)(uintptr_t)gw_mex_move_coll_cb);
        gw_w32((void *)(uintptr_t)(e + 0x1Cu), cam_native);
    }
    gw_mex_movelogic_table = table;
    gw_log("interp: MoveLogic table @ guest 0x%08X (%d MotionState entries): anim/input/phys/coll "
           "-> trampolines, cam -> gw_ftCamera_UpdateCameraBox 0x%08X",
           table, entries, cam_native);
}

/* The engine site (fighter.c Fighter_UnkInitLoad_80068914) asks for the per-kind character-state
 * table; for Sonic we hand back the interpreted MoveLogic table instead of Fox's vanilla one. */
void *gw_Mex_MoveLogicTable(int kind, void *vanilla) {
    if (kind == GW_MEX_KIND_SONIC && gw_mex_installed == 1 && gw_mex_movelogic_table != 0) {
        return (void *)(uintptr_t)gw_mex_movelogic_table;
    }
    return vanilla;
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

/* Symbolizer for gw_ppc: guest address -> the containing blob function's name, or NULL. */
static const char *gw_mex_symbolize(uint32_t guest_addr) {
    return gw_ftfunction_symbol_name(&gw_mex_ff, guest_addr);
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
    /* Back the interpreter's symbolizer with the blob's own debug symbol table, so every panic,
     * budget dump and trace names a guest function instead of printing a bare address. */
    gw_ppc_set_symbolizer(gw_mex_symbolize);
    /* First in the chain, so it runs before the port's own crash handling - which it defers to
     * for anything that is not an execute fault inside the blob. */
    if (AddVectoredExceptionHandler(1, gw_mex_exec_trap) == NULL) {
        gw_log("interp: could not install the guest execute trap - direct native calls into "
               "guest code will crash");
    }

    gw_mex_movelogic_setup();

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
    /* MELEE_MEX_DUMP_CODE=<path> writes the RELOCATED blob (what the interpreter actually
     * executes, after Reloc/Overload have been applied) so it can be disassembled offline:
     *   python tools/mex_port/ppc_disasm.py --raw <path> --base <code_base> --start <va> --count N
     * Dumping the relocated image rather than the on-disc section matters - the disc bytes still
     * have unrelocated branch/address operands, so they disassemble into misleading targets. */
    {
        const char *dump_path = getenv("MELEE_MEX_DUMP_CODE");
        if (dump_path != NULL && dump_path[0] != '\0') {
            FILE *df = fopen(dump_path, "wb");
            if (df == NULL) {
                gw_log("interp: MELEE_MEX_DUMP_CODE: cannot open %s", dump_path);
            } else {
                size_t wrote = fwrite((const void *)(uintptr_t)gw_mex_ff.code_base, 1,
                                      gw_mex_ff.code_size, df);
                fclose(df);
                gw_log("interp: dumped %u bytes of relocated code (base 0x%08X) to %s",
                       (unsigned)wrote, gw_mex_ff.code_base, dump_path);
            }
        }
    }

    gw_log("interp: installed Sonic ftFunction (code 0x%08X..0x%08X, mexData 0x%08X, stack "
           "0x%08X, %d overrides); onLoad/onFrame/onActionStateChange/onReapplyAttr/8 specials/"
           "onDoubleJump/onUSmash/onItemPickup + MoveLogic table active",
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

static int test_resolver_mex_shims(void) {
    /* Every MEX_* call target in Sonic's ftFunction must resolve to a native shim, never NULL. */
    static const uint32_t mex_targets[] = {GW_MEX_GUEST_INDEX_ITEM, GW_MEX_GUEST_GET_FT_ITEM_ID,
                                           GW_MEX_GUEST_GET_DATA};
    unsigned i;
    for (i = 0; i < sizeof mex_targets / sizeof mex_targets[0]; ++i) {
        gw_ppc_sig sig;
        memset(&sig, 0, sizeof sig);
        if (gw_mex_interp_resolve(mex_targets[i], NULL, &sig) == NULL) {
            gw_test_fail("resolver returned NULL for MEX_* target 0x%08X", mex_targets[i]);
            return 1;
        }
    }
    return 0;
}

void gw_mex_ftfunction_runtime_tests_register(void) {
    gw_test_register("bridge_lookup_memcpy", test_bridge_lookup_memcpy);
    gw_test_register("bridge_lookup_global", test_bridge_lookup_global);
    gw_test_register("bridge_lookup_miss", test_bridge_lookup_miss);
    gw_test_register("resolver_mex_shims", test_resolver_mex_shims);
}
