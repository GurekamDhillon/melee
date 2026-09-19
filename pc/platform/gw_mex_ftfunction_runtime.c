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

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

/* Index spaces (see INDEX SPACES below). The port reserves a block of fighter kinds and a block
 * of character kinds for m-ex fighters (melee/ft/forward.h Ft_Kind_Mex0, ChKind_Mex0). Slots are
 * DENSE: slot i is the i-th non-empty m-ex fighter row from internal id GW_MEX_FIRST_NEW up
 * (gw_Mex_SlotInternal), so builds with placeholder rows (ACE's "NONE") still fit. */
#define GW_MEX_SLOTS 31             /* Ft_Kind_Mex0 .. +30: a fighter kind must fit the 6-bit
                                     * x597_bits field the animation code compares it with */
#define GW_PORT_FT_MEX0 0x21        /* Ft_Kind_Mex0 */
#define GW_PORT_CK_MEX0 0x22        /* ChKind_Mex0 (after ChKind_None, 0x21) */
#define GW_MEX_FIRST_NEW 27         /* m-ex internal id of the first added fighter */
#define GW_MEX_INTERNAL_SONIC 31    /* Sonic's m-ex internal id (tests only) */

/* m-ex index-space mappings, defined below */
int gw_Mex_PortCKindToExt(int ckind);
int gw_Mex_ExtToPortCKind(int ext);
int gw_Mex_InternalForPortKind(int fk);
int gw_Mex_PortKindForInternal(int k);
int gw_Mex_InternalForExt(int e);
int gw_Mex_ExtForInternal(int k);
int gw_Mex_SlotInternal(int slot);
int gw_Mex_InternalCount(void);
int gw_Mex_CssIconCount(void);
int gw_Mex_FtCostumeCount(int k);
static int gw_mex_slot_of_internal(int k);

/* Arch_FighterFunc slot indices (Header.s: onLoad 0x00 ... GetTrailData 0xB4). See
 * gw_mex_ftfunction.c's slot_names table; onFrame is slot 23. */
#define GW_MEX_SLOT_ON_LOAD 0
/* Slot 1: the blob's own debug symbols name its target OnRespawn. It overrides vanilla's
 * ftData_OnDeath table, which despite the decomp's name is the (re)spawn initialiser - Fox's
 * entry resets state and sets model-part defaults. Dispatched from fighter.c via
 * GW_MEX_EVENT_ON_DEATH. */
#define GW_MEX_SLOT_ON_RESPAWN 1
#define GW_MEX_SLOT_ON_DESTROY 2            /* ftData_OnUserDataRemove */
#define GW_MEX_SLOT_ON_ITEM_INVISIBLE 14    /* ftData_OnItemInvisible  */
#define GW_MEX_SLOT_ON_ITEM_VISIBLE 15      /* ftData_OnItemVisible    */
#define GW_MEX_SLOT_ON_KNOCKBACK_ENTER 21   /* ftData_OnKnockbackEnter - Sonic: EyeTextureDamaged */
#define GW_MEX_SLOT_ON_KNOCKBACK_EXIT 22    /* ftData_OnKnockbackExit  - Sonic: EyeTextureNormal  */
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
#define GW_MEX_SLOT_ON_ITEM_DROP_EXT 16 /* ftData_OnItemDropExt - m-ex OnItemRelease  */
#define GW_MEX_SLOT_ON_ITEM_PICKUP2 17  /* ftData_OnItemPickup  - m-ex OnItemCatch    */
#define GW_MEX_SLOT_ON_ITEM_DROP 18     /* ftData_OnItemDrop - m-ex onUnknownItemRelated */
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
#define GW_MEX_MEXDATA_SIZE 0x5000u /* 46 slots + 46 per-kind tables of 96 ids */
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

#define GW_MEX_MAX_ARTICLES 16
#define GW_MEX_MAX_ARTICLE_SYMS 64

/* Everything the runtime holds for ONE m-ex fighter. Each engine entry point (a hook dispatched
 * for a fighter kind, a MoveLogic trampoline) selects the fighter it serves into gw_mex_k and
 * restores the previous one on return - one fighter's guest code can trigger another fighter's
 * hook (Sonic's hit puts Wolf into knockback). The macros below keep the single-fighter code that
 * reads gw_mex_ff & co. working unchanged against the current fighter. */
typedef struct gw_mex_kind {
    int port_kind;                /* Ft_Kind_Mex0 + slot */
    int internal;                 /* m-ex internal id */
    gw_ftfunction ff;             /* the loaded, relocated blob (code + overrides) */
    int installed;                /* 1 once installed, -1 on permanent failure */
    uint32_t movelogic_table;     /* guest addr of its MoveLogic MotionState[] */
    int movelogic_entries;
    uint32_t move_cb_guest[GW_MEX_MOVE_MAX_ENTRIES][4]; /* preserved guest anim/input/phys/coll */
    uint32_t art_lo[GW_MEX_MAX_ARTICLES], art_hi[GW_MEX_MAX_ARTICLES];
    int art_count;
    gw_ftfunction_symbol art_syms[GW_MEX_MAX_ARTICLE_SYMS];
    uint32_t art_sym_count;
} gw_mex_kind;

static gw_mex_kind gw_mex_kinds[GW_MEX_SLOTS];
static gw_mex_kind *gw_mex_k = &gw_mex_kinds[0];
static uint32_t gw_mex_r2;            /* shared Arch_FighterFunc holder: every blob's r2 */
static int gw_mex_any_installed;      /* any fighter's code is installed (exec trap guard) */
static uint32_t gw_mex_stack_top;     /* guest stack top (r1), shared */
static uint32_t gw_mex_getdata_buf;   /* guest buffer backing the MEX_GetData(8) shim */

#define gw_mex_ff (gw_mex_k->ff)
#define gw_mex_installed (gw_mex_k->installed)
#define gw_mex_movelogic_table (gw_mex_k->movelogic_table)
#define gw_mex_movelogic_entries (gw_mex_k->movelogic_entries)
#define gw_mex_move_cb_guest (gw_mex_k->move_cb_guest)
#define gw_mex_article_lo (gw_mex_k->art_lo)
#define gw_mex_article_hi (gw_mex_k->art_hi)
#define gw_mex_article_count (gw_mex_k->art_count)
#define gw_mex_article_syms (gw_mex_k->art_syms)
#define gw_mex_article_sym_count (gw_mex_k->art_sym_count)

static int gw_mex_slot_of_port(int fk) {
    return (fk >= GW_PORT_FT_MEX0 && fk < GW_PORT_FT_MEX0 + GW_MEX_SLOTS) ? fk - GW_PORT_FT_MEX0
                                                                           : -1;
}

/* Make port fighter kind `fk` current. Returns the previous selection for gw_Mex_RestoreKind;
 * a non-m-ex kind leaves the selection as it was. Called by gw_runtime.c's hook dispatch. */
void *gw_Mex_SelectKind(int fk) {
    gw_mex_kind *prev = gw_mex_k;
    int s = gw_mex_slot_of_port(fk);
    if (s >= 0) {
        gw_mex_k = &gw_mex_kinds[s];
    }
    return prev;
}

void gw_Mex_RestoreKind(void *prev) {
    if (prev != NULL) {
        gw_mex_k = (gw_mex_kind *) prev;
    }
}

/* The port fighter kind of a fighter gobj (gobj->user_data->kind), or -1. */
static int gw_mex_gobj_kind(void *gobj) {
    uint32_t fd;
    if (gobj == NULL) {
        return -1;
    }
    fd = gw_r32((const void *) (uintptr_t) ((uintptr_t) gobj + 0x2Cu));
    if (fd < 0x80000000u || fd >= 0x80000000u + gw_mem1_size) {
        return -1;
    }
    return (int) gw_r32((const void *) (uintptr_t) (fd + 0x04u));
}

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

/* ---- persistent guest memory -----------------------------------------------------------------
 * Everything the m-ex runtime puts in guest memory (fighter code, its stack, mexData, item article
 * code) comes from here, NOT HSD_MemAlloc: it is loaded once and must outlive the scene that
 * loaded it. See GW_MEX_PERSIST_SIZE in shim_os.c. A bump allocator - nothing is ever freed, which
 * is correct: each blob is loaded exactly once per process. Zero-filled. */
static void *gw_mex_persist_alloc(uint32_t size) {
    extern void gw_mex_persist_region(uint32_t *base, uint32_t *size);
    static uint32_t base, cap, used;
    uint32_t p;
    if (cap == 0u) {
        gw_mex_persist_region(&base, &cap);
    }
    size = (size + 31u) & ~31u;
    if (size > cap - used) {
        gw_panic("mex: persistent guest memory exhausted (%u of %u bytes used, %u requested) - "
                 "raise GW_MEX_PERSIST_SIZE in shim_os.c",
                 used, cap, size);
    }
    p = base + used;
    used += size;
    memset((void *) (uintptr_t) p, 0, size);
    return (void *) (uintptr_t) p;
}

/* ---- mexData: MxDt.dat ---------------------------------------------------------------------
 * m-ex's content tables (fighter/item/stage/...) ship on the disc as MxDt.dat, an HSD archive
 * whose single public symbol is literally `mexData`. The port does not author a layout - it loads
 * the real one. Field paths below were verified against the file (tools/mex_port/dump_mxdt.py),
 * NOT taken from m-ex's mxdt.h, whose MexData.fighter struct is admittedly incomplete:
 *
 *   mexData +0x08 -> fighter;  fighter +0x4C -> item_lookup[]   (stride 8: {s32 count; u16 *ids})
 *   mexData +0x1C -> item;     item +0x10 -> Custom[] (stride 0x3C), +0x14 -> RuntimeIndex[] (4)
 *
 * item +0x00..+0x0C (Common/Fighter/Pokemon/Stages) are ABSOLUTE vanilla guest addresses, not
 * data offsets; they are not in the reloc table, so relocation correctly leaves them alone.
 *
 * INDEX SPACES - the trap here. Two numberings of fighters are in play:
 *   - the PORT's kind (Ft_Kind_Sonic = 33, appended after the vanilla kinds), which is what the
 *     engine stores in fp->kind and therefore what Sonic's own guest code reads and passes;
 *   - Akaneia's m-ex INTERNAL kind (Sonic = 31), which is what item_lookup is indexed by.
 * They differ, so every table access maps port kind -> internal kind first. Only Sonic's mapping
 * is known; any other kind is a hard error, because a guessed index silently returns another
 * fighter's items (item_lookup[33] is somebody else's row). And `fighter.names` is indexed by yet
 * another space (EXTERNAL id) - do not use it with either of these. */

#define GW_MEXDT_OFF_FIGHTER 0x08u
#define GW_MEXDT_OFF_ITEM 0x1Cu
#define GW_MEXDT_FIGHTER_OFF_ITEM_LOOKUP 0x4Cu
#define GW_MEXDT_ITEM_OFF_CUSTOM 0x10u
#define GW_MEXDT_ITEM_OFF_RUNTIME_INDEX 0x14u
#define GW_MEX_CUSTOM_ITEM_START 237u /* m-ex CustomItemStart: global item kinds >= this are custom */

static uint32_t gw_mexdt;        /* guest address of the mexData root; 0 = not loaded */
static uint32_t gw_mexdt_base;   /* guest address of the loaded data section */
static uint32_t gw_mexdt_size;

static int gw_mexdt_in(uint32_t a, uint32_t len) {
    return a >= gw_mexdt_base && (uint64_t) a + len <= (uint64_t) gw_mexdt_base + gw_mexdt_size;
}

/* Load an HSD archive's data section into guest memory and apply its relocation table (each
 * entry is the data offset of a pointer word; the word becomes base + its data-relative value).
 * Returns the guest address of `symbol`, or 0 with a logged reason. `fixed_base` != 0 loads at
 * that guest address instead of allocating from the HSD heap - the in-engine tests use it, since
 * they run before the game's heaps exist. */
static uint32_t gw_mex_load_hsd(const char *path, const char *symbol, uint32_t fixed_base,
                                uint32_t *out_base, uint32_t *out_size) {
    extern void *gw_HSD_MemAlloc(uint32_t size);
    extern void *gw_DVDReadFileAlloc(const char *path, uint32_t *out_size);
    uint32_t file_len = 0, file_size, data_size, nb_reloc, i, base;
    unsigned char *dat = (unsigned char *) gw_DVDReadFileAlloc(path, &file_len);
    int32_t sym;

    if (dat == NULL) {
        gw_log("mexdata: %s not on this disc", path);
        return 0u;
    }
    if (file_len < 0x20u) {
        gw_log("mexdata: %s too small for an HSD header", path);
        free(dat);
        return 0u;
    }
    file_size = gw_r32(dat + 0x00);
    data_size = gw_r32(dat + 0x04);
    nb_reloc = gw_r32(dat + 0x08);
    if (file_size != file_len || 0x20u + (uint64_t) data_size + (uint64_t) nb_reloc * 4u > file_len) {
        gw_log("mexdata: %s header inconsistent (file 0x%X/0x%X data 0x%X relocs %u)", path,
               file_size, file_len, data_size, nb_reloc);
        free(dat);
        return 0u;
    }
    sym = gw_ftfunction_find_public(dat, file_len, symbol);
    if (sym < 0 || (uint32_t) sym >= data_size) {
        gw_log("mexdata: %s has no public symbol %s", path, symbol);
        free(dat);
        return 0u;
    }
    base = fixed_base != 0u ? fixed_base : (uint32_t) (uintptr_t) gw_mex_persist_alloc(data_size);
    if (base == 0u) {
        gw_log("mexdata: cannot allocate %u bytes for %s", data_size, path);
        free(dat);
        return 0u;
    }
    memcpy((void *) (uintptr_t) base, dat + 0x20, data_size);
    for (i = 0; i < nb_reloc; ++i) {
        uint32_t off = gw_r32(dat + 0x20 + data_size + i * 4u);
        if ((uint64_t) off + 4u > data_size) {
            gw_log("mexdata: %s reloc %u points past the data (0x%X)", path, i, off);
            continue;
        }
        gw_w32((void *) (uintptr_t) (base + off),
               gw_r32((const void *) (uintptr_t) (base + off)) + base);
    }
    free(dat);
    *out_base = base;
    *out_size = data_size;
    return base + (uint32_t) sym;
}

/* Load MxDt.dat once. Safe to call when it is absent (vanilla disc): mexData stays unloaded and
 * anything that needs it fails loudly at the point of use rather than at boot. */
static void gw_mexdt_load(void) {
    uint32_t root, fighter, lookup, item;
    if (gw_mexdt != 0u) {
        return;
    }
    root = gw_mex_load_hsd("MxDt.dat", "mexData", 0u, &gw_mexdt_base, &gw_mexdt_size);
    if (root == 0u) {
        return;
    }
    /* Validate the three pointers everything below depends on, so a layout mismatch is caught
     * here, once, instead of as a wild read inside an item spawn. */
    fighter = gw_r32((const void *) (uintptr_t) (root + GW_MEXDT_OFF_FIGHTER));
    item = gw_r32((const void *) (uintptr_t) (root + GW_MEXDT_OFF_ITEM));
    lookup = gw_mexdt_in(fighter, 0x50u)
                 ? gw_r32((const void *) (uintptr_t) (fighter + GW_MEXDT_FIGHTER_OFF_ITEM_LOOKUP))
                 : 0u;
    if (!gw_mexdt_in(fighter, 0x50u) || !gw_mexdt_in(item, 0x18u) || !gw_mexdt_in(lookup, 8u)) {
        gw_log("mexdata: MxDt.dat loaded but its layout does not match (fighter 0x%08X item "
               "0x%08X item_lookup 0x%08X) - leaving mexData unloaded",
               fighter, item, lookup);
        return;
    }
    gw_mexdt = root;
    gw_log("mexdata: MxDt.dat loaded, mexData @ 0x%08X (data 0x%08X, %u bytes)", root,
           gw_mexdt_base, gw_mexdt_size);
}

/* Port fighter kind -> m-ex internal kind. See INDEX SPACES above. */
static uint32_t gw_mex_internal_kind(uint32_t port_kind, const char *why) {
    extern int gw_Mex_InternalForPortKind(int fk);
    int k = gw_Mex_InternalForPortKind((int) port_kind);
    if (k >= 0) {
        return (uint32_t) k;
    }
    gw_panic("mexdata: %s: no m-ex internal kind is known for port fighter kind %u", why,
             port_kind);
    return 0u;
}

/* item_lookup[internal_kind].ids[n], with every step bounds-checked. */
static uint32_t gw_mex_ft_item_global(uint32_t port_kind, uint32_t n, const char *why) {
    uint32_t mk, fighter, lookup, entry, ids;
    int32_t count;
    if (gw_mexdt == 0u) {
        gw_panic("mexdata: %s needs MxDt.dat, which is not loaded (not on this disc?)", why);
    }
    mk = gw_mex_internal_kind(port_kind, why);
    fighter = gw_r32((const void *) (uintptr_t) (gw_mexdt + GW_MEXDT_OFF_FIGHTER));
    lookup = gw_r32((const void *) (uintptr_t) (fighter + GW_MEXDT_FIGHTER_OFF_ITEM_LOOKUP));
    entry = lookup + mk * 8u;
    if (!gw_mexdt_in(entry, 8u)) {
        gw_panic("mexdata: %s: item_lookup[%u] is outside MxDt.dat", why, mk);
    }
    count = (int32_t) gw_r32((const void *) (uintptr_t) entry);
    ids = gw_r32((const void *) (uintptr_t) (entry + 4u));
    if ((int32_t) n < 0 || (int32_t) n >= count || !gw_mexdt_in(ids + n * 2u, 2u)) {
        gw_panic("mexdata: %s: fighter item %u out of range (kind %u has %d)", why, n, mk, count);
    }
    return gw_r16((const void *) (uintptr_t) (ids + n * 2u));
}

/* MEX_IndexFighterItem(fighter_kind, ItemDesc *desc, item_id): register a fighter article's
 * descriptor in item.RuntimeIndex so item creation can find it. m-ex's Create Item patch ASSERTS
 * if the slot for a custom kind is NULL, so this is not optional. Called from the fighter's own
 * OnLoad, which runs long before any special can spawn the item.
 *
 * NOT YET DONE: whether this call ALSO fills item.Custom (the 0x3C-stride state/function table)
 * is unsettled - see _research/mex-data-layer-design.md section 5. For Sonic's kind that slot
 * ships empty, so something fills it. */
static uint32_t gw_mex_shim_index_item(uint32_t fighter_kind, uint32_t desc, uint32_t item_id,
                                       uint32_t a3, uint32_t a4, uint32_t a5, uint32_t a6,
                                       uint32_t a7) {
    uint32_t global, item, rt, slot;
    (void) a3; (void) a4; (void) a5; (void) a6; (void) a7;
    global = gw_mex_ft_item_global(fighter_kind, item_id, "MEX_IndexFighterItem");
    if (global < GW_MEX_CUSTOM_ITEM_START) {
        gw_panic("mexdata: MEX_IndexFighterItem: global item kind %u is not a custom kind",
                 global);
    }
    item = gw_r32((const void *) (uintptr_t) (gw_mexdt + GW_MEXDT_OFF_ITEM));
    rt = gw_r32((const void *) (uintptr_t) (item + GW_MEXDT_ITEM_OFF_RUNTIME_INDEX));
    slot = rt + (global - GW_MEX_CUSTOM_ITEM_START) * 4u;
    if (!gw_mexdt_in(slot, 4u)) {
        gw_panic("mexdata: MEX_IndexFighterItem: RuntimeIndex[%u] is outside MxDt.dat",
                 global - GW_MEX_CUSTOM_ITEM_START);
    }
    gw_w32((void *) (uintptr_t) slot, desc);
    gw_log("mexdata: MEX_IndexFighterItem(kind %u, item %u) -> global item kind %u, "
           "RuntimeIndex[%u] = desc 0x%08X",
           fighter_kind, item_id, global, global - GW_MEX_CUSTOM_ITEM_START, desc);
    return 0u;
}

/* ---- m-ex CSS accessors (for the data-driven character select, mncharsel.c) ------------------
 * Paths verified against the file with tools/mex_port/dump_css.py:
 *   mexData +0x00 -> metadata, metadata +0x0C = css icon count (s32)
 *   mexData +0x04 -> menu, menu +0x04 -> css, css +0xDC = icon array, stride 0x1C - the SAME
 *   layout as the port's CSSIcon (hud, char, state, timer, joint vs, joint 1p, sfx, bounds)
 * Icon char ids are m-ex EXTERNAL ids. m-ex external 0..25 are exactly the retail CharacterKind
 * order, so they map to themselves; external 30 is Sonic, the port's CharacterKind 0x20. Every
 * other external id is a character the port does not have - reported as -1 ("not available"),
 * never guessed: 0x1E is Crazy Hand to the port but Sonic to m-ex, and 0x20 is the port's Sonic but
 * m-ex's Tails. */
int gw_Mex_CssIconCount(void) {
    uint32_t meta;
    int32_t n;
    /* The CSS runs before any match, i.e. before a fighter install would load mexData. It lives
     * in persistent memory, so loading it here, early, is safe. */
    gw_mexdt_load();
    if (gw_mexdt == 0u) {
        return 0;
    }
    meta = gw_r32((const void *) (uintptr_t) gw_mexdt);
    if (!gw_mexdt_in(meta, 0x10u)) {
        return 0;
    }
    n = (int32_t) gw_r32((const void *) (uintptr_t) (meta + 0x0Cu));
    return (n > 0 && n <= 64) ? n : 0;
}

void *gw_Mex_CssIconTable(void) {
    uint32_t menu, css, tbl;
    int n = gw_Mex_CssIconCount();
    if (n == 0) {
        return NULL;
    }
    menu = gw_r32((const void *) (uintptr_t) (gw_mexdt + 0x04u));
    css = gw_mexdt_in(menu, 8u) ? gw_r32((const void *) (uintptr_t) (menu + 0x04u)) : 0u;
    tbl = css + 0xDCu;
    if (css == 0u || !gw_mexdt_in(tbl, (uint32_t) n * 0x1Cu)) {
        return NULL;
    }
    return (void *) (uintptr_t) tbl;
}

int gw_Mex_ExtToPortCKind(int ext) {
    extern int gw_Mex_InternalForExt(int ext);
    extern int gw_Mex_SlotInternal(int slot);
    int k;
    if (ext >= 0 && ext <= 25) {
        return ext;
    }
    k = gw_Mex_InternalForExt(ext);
    k = k >= 0 ? gw_mex_slot_of_internal(k) : -1;
    return k >= 0 ? GW_PORT_CK_MEX0 + k : -1;
}

/* ---- m-ex sound banks (mexData.ssm, root +0x10) -------------------------------------------------
 * See _research/mex-sound-banks.md. m-ex numbers banks 0..54 exactly as retail and appends its own
 * (55 = null.ssm, 66 = sonic.ssm on Akaneia). mexData.ssm = {Files (char*[]), Flags ({u32 size;
 * u32 flag}[]), LookupTable ({s8 group, load_prio, unload_prio, pitch_thresh}[]), Runtime}. */
#define GW_MEXDT_OFF_SSM 0x10u

/* metadata.ssm_count (78 on Akaneia), or 0 without mexData. */
int gw_Mex_SsmCount(void) {
    uint32_t md;
    if (gw_Mex_CssIconCount() == 0) { /* loads mexData lazily */
        return 0;
    }
    md = gw_r32((const void *) (uintptr_t) gw_mexdt);
    return gw_mexdt_in(md, 0x20u) ? (int) gw_r32((const void *) (uintptr_t) (md + 0x1Cu)) : 0;
}

static uint32_t gw_mex_ssm_table(uint32_t which) {
    uint32_t ssm = gw_r32((const void *) (uintptr_t) (gw_mexdt + GW_MEXDT_OFF_SSM));
    return gw_mexdt_in(ssm + which, 4u) ? gw_r32((const void *) (uintptr_t) (ssm + which)) : 0u;
}

/* Bank i's file name ("sonic.ssm"), its ARAM size, and LookupTable byte k (0 group, 1 load prio,
 * 2 unload prio, 3 pitch threshold). Callers bound i by gw_Mex_SsmCount(). */
const char *gw_Mex_SsmFile(int i) {
    uint32_t t = gw_mex_ssm_table(0x0u);
    return t != 0u ? (const char *) (uintptr_t) gw_r32((const void *) (uintptr_t) (t + 4u * (uint32_t) i))
                   : NULL;
}
/* Bank i's ARAM size: the .ssm header's sample-data word (word 1), read from the file that will
 * actually load - disc or mod. m-ex's table holds the sizes of the files IT shipped (Akaneia's);
 * any other file there (a vanilla disc under a mod, a mod's own bank) differs, and the bank buffer
 * sized from the table overflowed (vanilla main.ssm is 160 bytes larger than Akaneia's). m-ex has
 * the same fix, CalculateBankSizes_Boot, which Akaneia does not ship. 0 when the file is absent;
 * the table's value when the header cannot be read. */
uint32_t gw_Mex_SsmSize(int i) {
    extern uint32_t gw_DVDReadPrefix(const char *path, void *dst, uint32_t length);
    uint32_t t = gw_mex_ssm_table(0x4u);
    uint32_t table = t != 0u ? gw_r32((const void *) (uintptr_t) (t + 8u * (uint32_t) i)) : 0u;
    const char *name = gw_Mex_SsmFile(i);
    char path[96];
    unsigned char hdr[8];
    if (name == NULL || name[0] == 0) {
        return table;
    }
    snprintf(path, sizeof path, "/audio/us/%s", name);
    switch (gw_DVDReadPrefix(path, hdr, sizeof hdr)) {
    case 0:
        return 0u; /* not on the disc or in a mod */
    case 8:
        return gw_r32(hdr + 4);
    default:
        return table;
    }
}
int gw_Mex_SsmLookup(int i, int k) {
    uint32_t t = gw_mex_ssm_table(0x8u);
    return t != 0u ? (int) *(const int8_t *) (uintptr_t) (t + 4u * (uint32_t) i + (uint32_t) k) : 0;
}

/* fighter.ssm_files[ext].ssm_id (mexData.fighter +0x38, stride 0x10, EXTERNAL id), or -1. */
static int gw_mex_ssm_for_ext(int ext) {
    uint32_t fighter, tbl, e;
    if (ext < 0 || gw_Mex_SsmCount() == 0) {
        return -1;
    }
    fighter = gw_r32((const void *) (uintptr_t) (gw_mexdt + GW_MEXDT_OFF_FIGHTER));
    tbl = gw_mexdt_in(fighter + 0x38u, 4u) ? gw_r32((const void *) (uintptr_t) (fighter + 0x38u)) : 0u;
    e = tbl + (uint32_t) ext * 0x10u;
    if (tbl == 0u || !gw_mexdt_in(e, 1u)) {
        return -1;
    }
    e = *(const uint8_t *) (uintptr_t) e;
    return (e == 0xFFu || (int) e >= gw_Mex_SsmCount()) ? -1 : (int) e;
}

/* The sound bank of a player's CharacterKind (port numbering), or -1. */
int gw_Mex_SsmForPortCKind(int ck) {
    extern int gw_Mex_PortCKindToExt(int);
    return gw_mex_ssm_for_ext(gw_Mex_PortCKindToExt(ck));
}

/* The bank that a fighter's RELATIVE sound ids (5000..9999) index, by port FighterKind, or -1.
 * Only m-ex fighters use relative ids; vanilla fighters' data holds absolute ids. */
int gw_Mex_SsmForPortKind(int fk) {
    extern int gw_Mex_ExtForInternal(int k);
    int s = gw_mex_slot_of_port(fk);
    int k = s >= 0 ? gw_Mex_SlotInternal(s) : -1;
    return k >= 0 ? gw_mex_ssm_for_ext(gw_Mex_ExtForInternal(k)) : -1;
}

/* ---- m-ex music and per-fighter audio -----------------------------------------------------------
 * mexData.music (root +0x14) +0x00 = the BGM file names (char*[bgm_count]; 0..97 are retail's
 * hps_files, in order). fighter +0x30 victory_theme[ext] (a BGM index) and +0x34
 * announcer_call[ext] (an SFX id in nr_name.ssm), both s32 by EXTERNAL id. Values verified on
 * Akaneia: Sonic (ext 30) = BGM 125 ff_sonic.hps, announcer 510059; ext 0..25 equal retail. */
#define GW_MEXDT_OFF_MUSIC 0x14u

int gw_Mex_BgmCount(void) {
    uint32_t md;
    if (gw_Mex_CssIconCount() == 0) {
        return 0;
    }
    md = gw_r32((const void *) (uintptr_t) gw_mexdt);
    return gw_mexdt_in(md, 0x24u) ? (int) gw_r32((const void *) (uintptr_t) (md + 0x20u)) : 0;
}

const char *gw_Mex_BgmFile(int i) {
    uint32_t music, names;
    if (i < 0 || i >= gw_Mex_BgmCount()) {
        return NULL;
    }
    music = gw_r32((const void *) (uintptr_t) (gw_mexdt + GW_MEXDT_OFF_MUSIC));
    names = gw_mexdt_in(music, 4u) ? gw_r32((const void *) (uintptr_t) music) : 0u;
    if (names == 0u || !gw_mexdt_in(names + 4u * (uint32_t) i, 4u)) {
        return NULL;
    }
    return (const char *) (uintptr_t) gw_r32((const void *) (uintptr_t) (names + 4u * (uint32_t) i));
}

/* fighter-table s32 at `field_off`, indexed by the port CharacterKind's external id; `fallback`
 * when unavailable. */
static int gw_mex_fighter_s32_for_ck(int ck, uint32_t field_off, int fallback) {
    extern int gw_Mex_PortCKindToExt(int);
    int ext = gw_Mex_PortCKindToExt(ck);
    uint32_t fighter, tbl;
    if (ext < 0 || gw_Mex_CssIconCount() == 0) {
        return fallback;
    }
    fighter = gw_r32((const void *) (uintptr_t) (gw_mexdt + GW_MEXDT_OFF_FIGHTER));
    tbl = gw_mexdt_in(fighter + field_off, 4u) ? gw_r32((const void *) (uintptr_t) (fighter + field_off)) : 0u;
    if (tbl == 0u || !gw_mexdt_in(tbl + 4u * (uint32_t) ext, 4u)) {
        return fallback;
    }
    return (int) gw_r32((const void *) (uintptr_t) (tbl + 4u * (uint32_t) ext));
}

int gw_Mex_VictoryThemeForPortCKind(int ck) { return gw_mex_fighter_s32_for_ck(ck, 0x30u, -1); }

/* fighter +0x28 result_file[ext] (char*, "GmRstMSn.dat") and +0x2C result_scale[ext] (f32). */
const char *gw_Mex_ResultFileForPortCKind(int ck) {
    uint32_t p = (uint32_t) gw_mex_fighter_s32_for_ck(ck, 0x28u, 0);
    return (p != 0u && gw_mexdt_in(p, 1u) && *(const char *) (uintptr_t) p != 0)
               ? (const char *) (uintptr_t) p
               : NULL;
}
float gw_Mex_ResultScaleForPortCKind(int ck) {
    union { uint32_t u; float f; } v;
    v.u = (uint32_t) gw_mex_fighter_s32_for_ck(ck, 0x2Cu, 0x3F800000);
    return v.f;
}
int gw_Mex_AnnouncerForPortCKind(int ck) { return gw_mex_fighter_s32_for_ck(ck, 0x34u, -1); }

/* ---- m-ex menu params (mexData.menu +0x00) ---------------------------------------------------
 * params[0] is the CSS cursor scale m-ex's CursorScale patches apply (Akaneia: 0.95 - its icon
 * grid is denser than retail's 25, so the retail hand covers too much of it). params[2] is the
 * 1P level-text Y offset (AdjustLevelTextOffset). 1.0 when there is no mexData, so a caller can
 * multiply unconditionally. */
float gw_Mex_MenuParamF(int i) {
    uint32_t menu, params, p;
    if (i < 0 || i > 7 || gw_Mex_CssIconCount() == 0) { /* loads mexData lazily */
        return 1.0f;
    }
    menu = gw_r32((const void *) (uintptr_t) (gw_mexdt + 0x04u));
    params = gw_mexdt_in(menu, 4u) ? gw_r32((const void *) (uintptr_t) menu) : 0u;
    p = params + (uint32_t) i * 4u;
    if (params == 0u || !gw_mexdt_in(p, 4u)) {
        return 1.0f;
    }
    return gw_rf32((const void *) (uintptr_t) p);
}

/* ---- m-ex menu playlist (mexData.music +0x04 / +0x08) ----------------------------------------
 * Ported from m-ex (https://github.com/akaneia/m-ex): BGM/MenuPlaylist.asm. The main-menu theme
 * becomes a weighted draw over MexPlaylistEntry { u16 bgm_id; u16 chance }, stride 4, instead of
 * retail's hard-coded 1-in-4 between menu01 and menu3. The draw itself stays in game code so it
 * uses the game's own RNG stream; these accessors only expose the table. Akaneia ships 2 entries:
 * bgm 52 at weight 45, bgm 54 at weight 19. */
static uint32_t gw_mex_music_field(uint32_t off) {
    uint32_t music;
    if (gw_Mex_CssIconCount() == 0) { /* loads mexData lazily */
        return 0u;
    }
    music = gw_r32((const void *) (uintptr_t) (gw_mexdt + GW_MEXDT_OFF_MUSIC));
    return gw_mexdt_in(music + off, 4u) ? gw_r32((const void *) (uintptr_t) (music + off)) : 0u;
}

int gw_Mex_MenuPlaylistCount(void) {
    uint32_t tbl = gw_mex_music_field(0x04u);
    int32_t n = (int32_t) gw_mex_music_field(0x08u);
    if (tbl == 0u || n <= 0 || n > 256 || !gw_mexdt_in(tbl, (uint32_t) n * 4u)) {
        return 0;
    }
    return (int) n;
}

static uint32_t gw_mex_menu_playlist_entry(int i) {
    uint32_t tbl = gw_mex_music_field(0x04u);
    return (i >= 0 && i < gw_Mex_MenuPlaylistCount()) ? tbl + (uint32_t) i * 4u : 0u;
}

/* Entry i's BGM id, or -1 when it is out of range or names no BGM the port's table has. */
int gw_Mex_MenuPlaylistBgm(int i) {
    uint32_t e = gw_mex_menu_playlist_entry(i);
    int bgm = e != 0u ? (int) gw_r16((const void *) (uintptr_t) e) : -1;
    return (bgm >= 0 && bgm < gw_Mex_BgmCount()) ? bgm : -1;
}

/* Entry i's weight (m-ex calls it a percent, but it is really a share of their sum), or 0. */
int gw_Mex_MenuPlaylistChance(int i) {
    uint32_t e = gw_mex_menu_playlist_entry(i);
    return e != 0u ? (int) gw_r16((const void *) (uintptr_t) (e + 2u)) : 0;
}

/* ---- m-ex per-fighter BGM (mexData.fighter +0x54) --------------------------------------------
 * Ported from m-ex (https://github.com/akaneia/m-ex): Fighter BGM/GetFighterBGM.asm, which
 * replaces the vanilla 0x21-entry table behind lbAudioAx_8002305C. Two u16 BGM ids per fighter,
 * stride 4, indexed by m-ex EXTERNAL id - the same space as the CharacterKind the retail call
 * sites pass. One big-endian word holds both, `which` 0 in the high half. Verified on Akaneia:
 * ext 30 (Sonic) = { 105, 134 }, ext 0..25 equal the retail table. Returns -1 when mexData is
 * absent, the port has no external id for the character, or m-ex marks it 0xFFFF ("no music",
 * the bosses) - in each case the caller keeps retail's value. */
int gw_Mex_FighterBgmForPortCKind(int ck, int which) {
    int word, bgm;
    if (which < 0 || which > 1) {
        return -1;
    }
    word = gw_mex_fighter_s32_for_ck(ck, 0x54u, -1);
    bgm = (which == 0) ? ((word >> 16) & 0xFFFF) : (word & 0xFFFF);
    return (bgm == 0xFFFF || bgm >= gw_Mex_BgmCount()) ? -1 : bgm;
}

/* Port FighterKind -> m-ex INTERNAL id, or -1. FK 0..26 are the same in both. m-ex appends its
 * new fighters after the vanilla playables and moves the six specials to the end (internal
 * internal_id_count-6 .. -1 = 35..40 on Akaneia), where the port keeps them at 27..32 and adds
 * Sonic at 33 (m-ex 31). Verified: _research/mex-stock-icons.md. */
int gw_Mex_InternalForPortKind(int fk) {
    int n = gw_Mex_InternalCount();
    int s = gw_mex_slot_of_port(fk);
    if (fk >= 0 && fk <= 26) {
        return fk;
    }
    if (fk >= 27 && fk <= 32) {
        /* the six bosses: m-ex moves them to the last six internal ids */
        return n > 6 ? n - 6 + (fk - 27) : fk;
    }
    return s >= 0 ? gw_Mex_SlotInternal(s) : -1;
}

/* ---- m-ex fighter rows ---------------------------------------------------------------------------
 * What ftData_MexInitKinds (melee/ft/ftdata.c) fills the port's per-kind tables from. Every value
 * is returned as a plain scalar or pointer: game code and platform code must not share structs
 * (game memory is big-endian). Strings point into the loaded MxDt.dat. Layouts: m-ex
 * MexTK/include/mxdt.h MexData.fighter / fighter_function; field order verified by
 * tools/mex_port/dump_fighters.py on Akaneia and ACE. Tables are indexed by INTERNAL id except
 * names/costume_info/ssm/results/victory/announcer (EXTERNAL id). */

static uint32_t gw_mex_ftfield(uint32_t off) {
    uint32_t fighter;
    if (gw_Mex_CssIconCount() == 0) { /* loads mexData lazily */
        return 0u;
    }
    fighter = gw_r32((const void *) (uintptr_t) (gw_mexdt + GW_MEXDT_OFF_FIGHTER));
    return gw_mexdt_in(fighter + off, 4u) ? gw_r32((const void *) (uintptr_t) (fighter + off)) : 0u;
}

static uint32_t gw_mex_word(uint32_t tbl, uint32_t index, uint32_t stride) {
    uint32_t a = tbl + index * stride;
    return (tbl != 0u && gw_mexdt_in(a, 4u)) ? gw_r32((const void *) (uintptr_t) a) : 0u;
}

static const char *gw_mex_cstr(uint32_t p) {
    return (p != 0u && gw_mexdt_in(p, 1u)) ? (const char *) (uintptr_t) p : NULL;
}

/* The slot table: slot i -> the i-th non-empty m-ex fighter row (internal id >= 27, before the
 * six bosses m-ex keeps last). Built once, when mexData is first available. */
static int gw_mex_slot_internal[GW_MEX_SLOTS];
static int gw_mex_slot_count = -1;

static void gw_mex_slots_build(void) {
    int n, k;
    if (gw_mex_slot_count >= 0 || gw_Mex_CssIconCount() == 0) {
        return;
    }
    n = gw_Mex_InternalCount();
    gw_mex_slot_count = 0;
    for (k = GW_MEX_FIRST_NEW; n > 6 && k < n - 6; ++k) {
        extern int gw_DVDConvertPathToEntrynum(const char *path);
        const char *pl = gw_mex_cstr(gw_mex_word(gw_mex_ftfield(0x04u), (uint32_t) k, 8u));
        if (pl == NULL || pl[0] == '\0') {
            continue; /* a placeholder row */
        }
        if (gw_DVDConvertPathToEntrynum(pl) < 0) {
            gw_log("mexdata: m-ex fighter %d's %s is not on the disc or in a mod - skipped", k, pl);
            continue;
        }
        if (gw_mex_slot_count == GW_MEX_SLOTS) {
            gw_log("mexdata: more than %d m-ex fighters - m-ex %d (%s) and later are left out",
                   GW_MEX_SLOTS, k, pl);
            break;
        }
        gw_mex_slot_internal[gw_mex_slot_count++] = k;
    }
    gw_log("mexdata: %d m-ex fighter slots", gw_mex_slot_count);
}

/* m-ex internal id of port slot `slot`, or -1. */
int gw_Mex_SlotInternal(int slot) {
    gw_mex_slots_build();
    return (slot >= 0 && slot < gw_mex_slot_count) ? gw_mex_slot_internal[slot] : -1;
}

/* Port slot of m-ex internal id `k`, or -1. */
static int gw_mex_slot_of_internal(int k) {
    int i;
    gw_mex_slots_build();
    for (i = 0; i < gw_mex_slot_count; ++i) {
        if (gw_mex_slot_internal[i] == k) {
            return i;
        }
    }
    return -1;
}

/* Port fighter kind of m-ex internal id `k` (retail kinds map to themselves), or -1. */
int gw_Mex_PortKindForInternal(int k) {
    int n = gw_Mex_InternalCount(), s;
    if (k >= 0 && k <= 26) {
        return k;
    }
    if (n > 6 && k >= n - 6 && k < n) {
        return 27 + (k - (n - 6));
    }
    s = gw_mex_slot_of_internal(k);
    return s >= 0 ? GW_PORT_FT_MEX0 + s : -1;
}

/* External id whose ft_kind_desc names internal `k` ({u8 internal, extra, transform}, stride 3). */
int gw_Mex_ExtForInternal(int k) {
    uint32_t desc = gw_mex_ftfield(0x0Cu);
    int e, n = gw_Mex_InternalCount();
    for (e = 0; desc != 0u && e < n; ++e) {
        if (gw_mexdt_in(desc + (uint32_t) e * 3u, 1u) &&
            *(const uint8_t *) (uintptr_t) (desc + (uint32_t) e * 3u) == (uint8_t) k) {
            return e;
        }
    }
    return -1;
}

int gw_Mex_InternalForExt(int e) {
    uint32_t desc = gw_mex_ftfield(0x0Cu);
    if (e < 0 || desc == 0u || !gw_mexdt_in(desc + (uint32_t) e * 3u, 1u)) {
        return -1;
    }
    return *(const uint8_t *) (uintptr_t) (desc + (uint32_t) e * 3u);
}

const char *gw_Mex_FtPlFile(int k) { return gw_mex_cstr(gw_mex_word(gw_mex_ftfield(0x04u), (uint32_t) k, 8u)); }
const char *gw_Mex_FtPlSymbol(int k) {
    uint32_t t = gw_mex_ftfield(0x04u);
    return t != 0u ? gw_mex_cstr(gw_mex_word(t + 4u, (uint32_t) k, 8u)) : NULL;
}
const char *gw_Mex_FtAnimFile(int k) { return gw_mex_cstr(gw_mex_word(gw_mex_ftfield(0x1Cu), (uint32_t) k, 4u)); }
int gw_Mex_FtAnimCount(int k) { return (int) gw_mex_word(gw_mex_ftfield(0x20u), (uint32_t) k, 4u); }
int gw_Mex_FtEffectIndex(int k) {
    uint32_t t = gw_mex_ftfield(0x24u);
    return (t != 0u && gw_mexdt_in(t + (uint32_t) k, 1u)) ? *(const uint8_t *) (uintptr_t) (t + (uint32_t) k) : -1;
}
int gw_Mex_FtCostumeCount(int k) {
    uint32_t t = gw_mex_ftfield(0x10u);
    int e = gw_Mex_ExtForInternal(k);
    return (e >= 0 && t != 0u && gw_mexdt_in(t + (uint32_t) e * 4u, 1u))
               ? *(const uint8_t *) (uintptr_t) (t + (uint32_t) e * 4u)
               : 0;
}
/* Costume c of internal k: which 0 = file, 1 = joint symbol, 2 = matanim symbol. */
const char *gw_Mex_FtCostumeString(int k, int c, int which) {
    uint32_t tbl = gw_mex_word(gw_mex_ftfield(0x14u), (uint32_t) k, 4u);
    return tbl != 0u ? gw_mex_cstr(gw_mex_word(tbl + (uint32_t) which * 4u, (uint32_t) c, 16u)) : NULL;
}
/* Guest address of the {result, intro, ending, wait} symbol-name block (Fighter_DemoStrings). */
void *gw_Mex_FtDemoStrings(int k) {
    uint32_t p = gw_mex_word(gw_mex_ftfield(0x18u), (uint32_t) k, 4u);
    return (p != 0u && gw_mexdt_in(p, 16u)) ? (void *) (uintptr_t) p : NULL;
}

/* fighter_function[slot][k] as a native pointer: the vanilla function (or data table, for
 * MoveLogic) m-ex gives this fighter by default, through the bridge. NULL when empty or not
 * bridged (logged once per address). The ftFunction's overrides are NOT here - those are guest
 * code, dispatched by the runtime's hooks once the fighter's file loads. */
void *gw_Mex_FtFunc(int slot, int k) {
    uint32_t ff, tbl, g, native;
    int kind = -1;
    if (gw_Mex_CssIconCount() == 0) {
        return NULL;
    }
    ff = gw_r32((const void *) (uintptr_t) (gw_mexdt + 0x0Cu));
    tbl = gw_mex_word(ff, (uint32_t) slot, 4u);
    g = gw_mex_word(tbl, (uint32_t) k, 4u);
    if (g == 0u) {
        return NULL;
    }
    native = gw_mex_bridge_lookup(g, &kind);
    if (native == 0u) {
        gw_log("mexdata: fighter_function[%d][%d] = 0x%08X has no native counterpart - left empty",
               slot, k, g);
        return NULL;
    }
    return (void *) (uintptr_t) native;
}

/* fighter.names[ext] (plain ASCII, e.g. "Sonic"), or NULL. */
const char *gw_Mex_FighterName(int ext) {
    return ext >= 0 ? gw_mex_cstr(gw_mex_word(gw_mex_ftfield(0x00u), (uint32_t) ext, 4u)) : NULL;
}

/* costume_info[ext of port character kind ck].{num, red_idx, blue_idx, green_idx}[field], or 0
 * (no mexData, or ck has no m-ex row). */
int gw_Mex_CostumeInfo(int ck, int field) {
    uint32_t t = gw_mex_ftfield(0x10u);
    int e = gw_Mex_PortCKindToExt(ck);
    if (e < 0 || t == 0u || field < 0 || field > 3 || !gw_mexdt_in(t + (uint32_t) e * 4u, 4u)) {
        return 0;
    }
    return *(const uint8_t *) (uintptr_t) (t + (uint32_t) e * 4u + (uint32_t) field);
}

/* The part-visibility table a fighter's costume uses: costume_file[k][costume]
 * .visibility_lookup_idx (m-ex lets costumes share one; Sonic's all use 0). The costume id itself
 * when there is no mexData or no row for it. */
int gw_Mex_CostumeVisIdx(int fk, int costume) {
    int k = gw_Mex_InternalForPortKind(fk);
    uint32_t tbl;
    int32_t v;
    if (k < 0 || costume < 0 || gw_Mex_CssIconCount() == 0 ||
        costume >= gw_Mex_FtCostumeCount(k)) {
        return costume;
    }
    tbl = gw_mex_word(gw_mex_ftfield(0x14u), (uint32_t) k, 4u);
    if (tbl == 0u || !gw_mexdt_in(tbl + (uint32_t) costume * 16u + 12u, 4u)) {
        return costume;
    }
    v = (int32_t) gw_r32((const void *) (uintptr_t) (tbl + (uint32_t) costume * 16u + 12u));
    return (v >= 0 && v < 16) ? v : costume;
}

/* The retail fighter this one was cloned from: the retail kind whose default onLoad it shares
 * (m-ex fighters keep their base's vanilla callbacks until their ftFunction overrides them). Used
 * for the per-kind tables m-ex does not describe. Mario when nothing matches. */
int gw_Mex_FtBaseKind(int k) {
    uint32_t ff, tbl, g;
    int r;
    if (gw_Mex_CssIconCount() == 0) {
        return 0;
    }
    ff = gw_r32((const void *) (uintptr_t) (gw_mexdt + 0x0Cu));
    tbl = gw_mex_word(ff, 0u, 4u);
    g = gw_mex_word(tbl, (uint32_t) k, 4u);
    for (r = 0; g != 0u && r <= 26; ++r) {
        if (gw_mex_word(tbl, (uint32_t) r, 4u) == g) {
            return r;
        }
    }
    return 0;
}

/* mexData metadata.internal_id_count (41 on Akaneia), or 0 without mexData. */
int gw_Mex_InternalCount(void) {
    uint32_t md;
    if (gw_Mex_CssIconCount() == 0) { /* loads mexData lazily */
        return 0;
    }
    md = gw_r32((const void *) (uintptr_t) gw_mexdt);
    return gw_mexdt_in(md, 8u) ? (int) gw_r32((const void *) (uintptr_t) (md + 0x04u)) : 0;
}

/* Franchise-emblem index for an m-ex external id: mexData fighter +0x08 -> insignia_idx[ext]
 * (u8). For ext 0..25 it equals retail's own emblem table (lbl_803B7B18); Sonic (30) is 17, an
 * emblem that exists only in Akaneia's IfAll Eblm_matanim_joint. -1 when unavailable. */
int gw_Mex_InsigniaForExt(int ext) {
    uint32_t fighter, tbl;
    if (ext < 0 || gw_Mex_CssIconCount() == 0) {
        return -1;
    }
    fighter = gw_r32((const void *) (uintptr_t) (gw_mexdt + GW_MEXDT_OFF_FIGHTER));
    tbl = gw_mexdt_in(fighter, 0x0Cu) ? gw_r32((const void *) (uintptr_t) (fighter + 0x08u)) : 0u;
    if (tbl == 0u || !gw_mexdt_in(tbl + (uint32_t) ext, 1u)) {
        return -1;
    }
    return *(const uint8_t *) (uintptr_t) (tbl + (uint32_t) ext);
}

/* The reverse, for portraits: the port's CharacterKind -> m-ex external id (-1 if none). */
int gw_Mex_PortCKindToExt(int ckind) {
    extern int gw_Mex_ExtForInternal(int k);
    if (ckind >= 0 && ckind <= 25) {
        return ckind;
    }
    if (ckind >= GW_PORT_CK_MEX0 && ckind < GW_PORT_CK_MEX0 + GW_MEX_SLOTS) {
        int k = gw_Mex_SlotInternal(ckind - GW_PORT_CK_MEX0);
        return k >= 0 ? gw_Mex_ExtForInternal(k) : -1;
    }
    return -1;
}

/* ---- item articles: itFunction -------------------------------------------------------------
 * A fighter's own item code ships in its .dat as the public symbol `itFunction` - Sonic's spring
 * lives there. Layout, verified across all 7 Akaneia custom fighters (tools/mex_port/
 * dump_itfunction.py, _research/mex-item-spawn.md):
 *     { u32 count; u32 article[count]; }   article[n] = data offset of a MEXFunction, 0 = hole
 * Each article is an ORDINARY MEXFunction - the same struct as ftFunction - which is why parsing
 * the whole symbol as one MEXFunction (the earlier attempt) produced nonsense.
 *
 * What m-ex does with it (its loader hook @0x80068B40, run right after ftFunction): relocate each
 * article's code, then write `Custom[global-237][slot] = code + off` for each FUNCTION reloc
 * {slot, code_offset}, where slot is a word index into the 0x3C-byte ItemLogicTable. Slots it does
 * not list keep their shipped values. That is what fills item.Custom - MEX_IndexFighterItem only
 * ever writes RuntimeIndex (confirmed against the code Akaneia actually ships).
 *
 * The state table the logic table points at lives INSIDE the article's code (at code+0), and its
 * function pointers are fixed up by the article's own abs32 instruction relocs. So every function
 * reachable from Custom[] is guest code; native item code calling it lands in the execute trap. */

#define GW_MEX_ITEM_LOGIC_STRIDE 0x3Cu /* sizeof ItemLogicTable (identical redefinition below) */
#define GW_MEX_ITEM_LOGIC_SLOTS 15u   /* 0x3C / 4 */

/* Article symbols and code ranges live in the per-fighter state (gw_mex_kind). */

static void gw_mex_article_symbols(const unsigned char *dat, uint32_t dat_size, uint32_t mf,
                                   uint32_t code) {
    /* +0x18 count, +0x1C table data offset; entries {start, END, name}, stride 12. */
    uint32_t n = gw_r32(dat + mf + 0x18), off = gw_r32(dat + mf + 0x1C), i;
    uint32_t tbl = 0x20u + off;
    if (n == 0u || (uint64_t) tbl + (uint64_t) n * 12u > dat_size) {
        return;
    }
    for (i = 0; i < n && gw_mex_article_sym_count < GW_MEX_MAX_ARTICLE_SYMS; ++i) {
        const unsigned char *e = dat + tbl + i * 12u;
        uint32_t np = 0x20u + gw_r32(e + 8), len = 0;
        char *copy;
        if (np >= dat_size) {
            continue;
        }
        while (np + len < dat_size && dat[np + len] != 0 && len < 127u) {
            ++len;
        }
        copy = (char *) malloc(len + 1u);
        if (copy == NULL) {
            return;
        }
        memcpy(copy, dat + np, len);
        copy[len] = 0;
        gw_mex_article_syms[gw_mex_article_sym_count].start = code + gw_r32(e + 0);
        gw_mex_article_syms[gw_mex_article_sym_count].end = code + gw_r32(e + 4);
        gw_mex_article_syms[gw_mex_article_sym_count].name = copy;
        ++gw_mex_article_sym_count;
    }
}

static const char *gw_mex_article_symbol_name(uint32_t a) {
    uint32_t i;
    for (i = 0; i < gw_mex_article_sym_count; ++i) {
        if (a >= gw_mex_article_syms[i].start && a < gw_mex_article_syms[i].end) {
            return gw_mex_article_syms[i].name;
        }
    }
    return NULL;
}


static void gw_mex_unload_items(void) {
    uint32_t i;
    int k;
    for (k = 0; k < gw_mex_article_count; ++k) {
        gw_ppc_remove_code_range(gw_mex_article_lo[k], gw_mex_article_hi[k]);
    }
    gw_mex_article_count = 0;
    for (i = 0; i < gw_mex_article_sym_count; ++i) {
        free((void *) gw_mex_article_syms[i].name);
    }
    gw_mex_article_sym_count = 0;
}

/* Load every article of `dat_path`'s itFunction for port fighter `port_kind`. Needs mexData.
 * With `arch_data` (the game's loaded copy of the same file, data section) each article's code is
 * relocated in place there, as m-ex does; otherwise it is copied into persistent memory. */
static void gw_mex_load_items(const char *dat_path, uint32_t port_kind, uint32_t arch_data,
                              uint32_t arch_data_size) {
    extern void *gw_HSD_MemAlloc(uint32_t size);
    extern void *gw_DVDReadFileAlloc(const char *path, uint32_t *out_size);
    uint32_t dat_size = 0, data_size, top, count, n;
    unsigned char *dat;
    int32_t pub;

    if (gw_mexdt == 0u) {
        gw_log("itfunction: mexData not loaded - %s's items cannot be registered", dat_path);
        return;
    }
    dat = (unsigned char *) gw_DVDReadFileAlloc(dat_path, &dat_size);
    if (dat == NULL || dat_size < 0x20u) {
        free(dat);
        return;
    }
    data_size = gw_r32(dat + 0x04);
    pub = gw_ftfunction_find_public(dat, dat_size, "itFunction");
    if (pub < 0) {
        gw_log("itfunction: %s has no itFunction (no fighter items)", dat_path);
        free(dat);
        return;
    }
    top = 0x20u + (uint32_t) pub;
    count = gw_r32(dat + top);
    if (count > 64u || (uint64_t) top + 4u + (uint64_t) count * 4u > dat_size) {
        gw_log("itfunction: %s implausible article count %u", dat_path, count);
        free(dat);
        return;
    }
    for (n = 0; n < count; ++n) {
        uint32_t art = gw_r32(dat + top + 4u + n * 4u);
        uint32_t mf, code_off, irt_off, irt_count, frt_off, frt_count, code_size, code;
        uint32_t global, item, custom, entry, i;
        if (art == 0u) {
            continue; /* a hole in the article array */
        }
        mf = 0x20u + art;
        if ((uint64_t) mf + 0x20u > dat_size) {
            gw_log("itfunction: article %u struct past the file", n);
            continue;
        }
        code_off = gw_r32(dat + mf + 0x00);
        irt_off = gw_r32(dat + mf + 0x04); /* NB: 0 is a VALID offset (Sonic's spring uses it) */
        irt_count = gw_r32(dat + mf + 0x08);
        frt_off = gw_r32(dat + mf + 0x0C);
        frt_count = gw_r32(dat + mf + 0x10);
        code_size = gw_r32(dat + mf + 0x14);
        if (code_size == 0u || code_off > data_size || code_size > data_size - code_off ||
            (uint64_t) frt_off + (uint64_t) frt_count * 8u > data_size) {
            gw_log("itfunction: article %u has an out-of-range code/reloc table", n);
            continue;
        }
        if (arch_data != 0u) {
            if (data_size != arch_data_size) {
                gw_log("itfunction: %s on disc is not the loaded archive - articles skipped",
                       dat_path);
                break;
            }
            code = arch_data + code_off;
        } else {
            code = (uint32_t) (uintptr_t) gw_mex_persist_alloc(code_size);
        }
        if (code == 0u) {
            gw_log("itfunction: cannot allocate %u bytes for article %u", code_size, n);
            continue;
        }
        memcpy((void *) (uintptr_t) code, dat + 0x20u + code_off, code_size);
        if (gw_ftfunction_reloc(dat, dat_size, irt_off, irt_count, code, code_size) != 0) {
            gw_log("itfunction: article %u relocation failed", n);
            continue;
        }
        /* Register BEFORE anything can run it: the interpreter must treat a bl between two
         * functions of this article as in-guest, not as a native call. */
        gw_ppc_add_code_range(code, code + code_size);
        if (gw_mex_article_count < GW_MEX_MAX_ARTICLES) {
            gw_mex_article_lo[gw_mex_article_count] = code;
            gw_mex_article_hi[gw_mex_article_count] = code + code_size;
            ++gw_mex_article_count;
        }
        gw_mex_article_symbols(dat, dat_size, mf, code);

        global = gw_mex_ft_item_global(port_kind, n, "itFunction");
        if (global < GW_MEX_CUSTOM_ITEM_START) {
            gw_log("itfunction: article %u maps to vanilla item kind %u - not a custom item", n,
                   global);
            continue;
        }
        item = gw_r32((const void *) (uintptr_t) (gw_mexdt + GW_MEXDT_OFF_ITEM));
        custom = gw_r32((const void *) (uintptr_t) (item + GW_MEXDT_ITEM_OFF_CUSTOM));
        entry = custom + (global - GW_MEX_CUSTOM_ITEM_START) * GW_MEX_ITEM_LOGIC_STRIDE;
        if (!gw_mexdt_in(entry, GW_MEX_ITEM_LOGIC_STRIDE)) {
            gw_log("itfunction: item.Custom[%u] is outside MxDt.dat", global - 237u);
            continue;
        }
        for (i = 0; i < frt_count; ++i) {
            uint32_t slot = gw_r32(dat + 0x20u + frt_off + i * 8u);
            uint32_t off = gw_r32(dat + 0x20u + frt_off + i * 8u + 4u);
            if (slot >= GW_MEX_ITEM_LOGIC_SLOTS || off >= code_size) {
                gw_log("itfunction: article %u function reloc %u out of range (slot %u off 0x%X)",
                       n, i, slot, off);
                continue;
            }
            gw_w32((void *) (uintptr_t) (entry + slot * 4u), code + off);
        }
        gw_log("itfunction: %s article %u -> item kind %u: code 0x%08X (+0x%X), %u relocs, "
               "%u logic slots into item.Custom[%u]",
               dat_path, n, global, code, code_size, irt_count, frt_count, global - 237u);
    }
    free(dat);
}

/* ---- custom item creation (the native half of m-ex's Create Item patch) --------------------
 * Called from Item_80267978 (src/melee/it/item.c), which picks an item's descriptor (xC4) and
 * logic table (xB8) by kind range. Vanilla has four ranges and routes everything >= 208 to the
 * STAGE tables, so a custom kind like Sonic's spring (277) indexed the stage table at 69 - far off
 * its end - got NULL model data and asserted "not found zako model data!". m-ex patches that site
 * (Create Item.asm @0x80267990) to add a fifth range, kind >= 237, backed by mexData:
 *   descriptor  = item.RuntimeIndex[kind - 237]           (filled by MEX_IndexFighterItem)
 *   logic table = &item.Custom[kind - 237], stride 0x3C   (sizeof ItemLogicTable)
 * Both fail loudly rather than return something plausible: m-ex itself asserts
 * "ItemNotInitialized" on a NULL descriptor, and an all-zero logic table would give the item no
 * states and crash somewhere unrelated much later. The logic table's callbacks (spawned,
 * destroyed, ...) may be guest code; native item code calling them is covered by the execute
 * trap. */

#define GW_MEX_ITEM_LOGIC_STRIDE 0x3Cu

static uint32_t gw_mex_item_table(uint32_t off, const char *what) {
    uint32_t item;
    if (gw_mexdt == 0u) {
        gw_panic("mexdata: custom item %s needs MxDt.dat, which is not loaded", what);
    }
    item = gw_r32((const void *) (uintptr_t) (gw_mexdt + GW_MEXDT_OFF_ITEM));
    return gw_r32((const void *) (uintptr_t) (item + off));
}

/* Descriptor (ItemGObjData+0xC4) for a custom item kind. Native pointer == guest address. */
void *gw_Mex_ItemCustomDesc(int kind) {
    uint32_t idx, slot, desc;
    if (kind < (int) GW_MEX_CUSTOM_ITEM_START) {
        gw_panic("mexdata: Mex_ItemCustomDesc(%d) called for a vanilla item kind", kind);
    }
    idx = (uint32_t) kind - GW_MEX_CUSTOM_ITEM_START;
    slot = gw_mex_item_table(GW_MEXDT_ITEM_OFF_RUNTIME_INDEX, "descriptor") + idx * 4u;
    if (!gw_mexdt_in(slot, 4u)) {
        gw_panic("mexdata: item kind %d: RuntimeIndex[%u] is outside MxDt.dat", kind, idx);
    }
    desc = gw_r32((const void *) (uintptr_t) slot);
    if (desc == 0u) {
        gw_panic("mexdata: item kind %d not initialized - RuntimeIndex[%u] is NULL. The owning "
                 "fighter's OnLoad must call MEX_IndexFighterItem before the item can spawn "
                 "(m-ex asserts ItemNotInitialized here too)",
                 kind, idx);
    }
    return (void *) (uintptr_t) desc;
}

/* Logic table (ItemGObjData+0xB8) for a custom item kind: &item.Custom[idx]. */
void *gw_Mex_ItemCustomLogic(int kind) {
    uint32_t idx, entry;
    if (kind < (int) GW_MEX_CUSTOM_ITEM_START) {
        gw_panic("mexdata: Mex_ItemCustomLogic(%d) called for a vanilla item kind", kind);
    }
    idx = (uint32_t) kind - GW_MEX_CUSTOM_ITEM_START;
    entry = gw_mex_item_table(GW_MEXDT_ITEM_OFF_CUSTOM, "logic table") +
            idx * GW_MEX_ITEM_LOGIC_STRIDE;
    if (!gw_mexdt_in(entry, GW_MEX_ITEM_LOGIC_STRIDE)) {
        gw_panic("mexdata: item kind %d: item.Custom[%u] is outside MxDt.dat", kind, idx);
    }
    if (gw_r32((const void *) (uintptr_t) entry) == 0u) { /* ItemLogicTable.states */
        gw_panic("mexdata: item kind %d: item.Custom[%u] is EMPTY (no state table). It ships "
                 "zeroed in MxDt.dat and nothing has filled it yet - the item's logic presumably "
                 "comes from the fighter's itFunction, which the port does not load yet",
                 kind, idx);
    }
    return (void *) (uintptr_t) entry;
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

/* MEX_GetFtItemID(fighter_gobj, n): the fighter's n-th article as a GLOBAL item kind. A pure
 * lookup into static file data - item_lookup ships correct, so there is no build step. For Sonic
 * the only correct answer is MEX_GetFtItemID(sonic, 0) == 277 (test mex_ft_item_id_sonic).
 * Previously this returned 0 forever, which is item kind 0 - a real, different item. */
static uint32_t gw_mex_shim_get_ft_item_id(uint32_t gobj, uint32_t item_id, uint32_t a2,
                                           uint32_t a3, uint32_t a4, uint32_t a5, uint32_t a6,
                                           uint32_t a7) {
    uint32_t fd, kind;
    (void) a2; (void) a3; (void) a4; (void) a5; (void) a6; (void) a7;
    fd = gw_r32((const void *) (uintptr_t) (gobj + 0x2Cu)); /* gobj->user_data */
    if (fd < 0x80000000u || fd >= 0x80000000u + gw_mem1_size) {
        gw_panic("mexdata: MEX_GetFtItemID: gobj 0x%08X has no fighter data", gobj);
    }
    kind = gw_r32((const void *) (uintptr_t) (fd + 0x4u)); /* fp->kind */
    return gw_mex_ft_item_global(kind, item_id, "MEX_GetFtItemID");
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
    return gw_ppc_call(gw_mex_thunk_guest[k], args, 4, gw_mex_r2, gw_mex_stack_top);
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

/* Guest code = ftFunction or any registered item article (see gw_ppc_add_code_range). */
static int gw_mex_in_blob(uint32_t a) {
    return gw_ppc_is_guest_code(a);
}

/* Release every thunk bound to guest code in [lo, hi) - that code is being unloaded. Anything
 * native still holding such a thunk belonged to the scene that just ended (its GObjs are gone). */
static void gw_mex_release_thunks(uint32_t lo, uint32_t hi) {
    int k;
    for (k = 0; k < gw_mex_thunk_count; ++k) {
        if (gw_mex_thunk_guest[k] >= lo && gw_mex_thunk_guest[k] < hi) {
            gw_mex_thunk_guest[k] = 0u;
        }
    }
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
        for (k = 0; k < gw_mex_thunk_count; ++k) {
            if (gw_mex_thunk_guest[k] == 0u) {
                break; /* a slot released by gw_mex_release_thunks */
            }
        }
        if (k == gw_mex_thunk_count) {
            if (gw_mex_thunk_count >= GW_MEX_THUNK_MAX) {
                gw_panic("interp: out of native thunks (%d) binding %s for %s", GW_MEX_THUNK_MAX,
                         gw_ppc_describe(guest), why);
            }
            ++gw_mex_thunk_count;
        }
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
    uint32_t args[4], r3;
    static int logged_returns;
    args[0] = a0;
    args[1] = a1;
    args[2] = a2;
    args[3] = a3;
    r3 = gw_ppc_call(target, args, 4, gw_mex_r2, gw_mex_stack_top);
    /* The return value matters to native callers more than it looks: an item state's animated /
     * collided callbacks are predicates, and returning TRUE tells the engine to DESTROY the item.
     * Log the first few so "why did this die on frame 1" is answerable. */
    if (logged_returns < 48) {
        ++logged_returns;
        gw_log("interp: trap: %s returned r3=0x%08X", gw_ppc_describe(target), r3);
    }
    return r3;
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
    gw_log("interp: trap: native code called guest %s directly - %s", gw_ppc_describe(eip),
           gw_mex_in_blob(eip) ? "interpreting" : "redirected to its native build");
}

static LONG CALLBACK gw_mex_exec_trap(PEXCEPTION_POINTERS ep) {
    PEXCEPTION_RECORD er = ep->ExceptionRecord;
    uint32_t eip;
    if (er->ExceptionCode != EXCEPTION_ACCESS_VIOLATION || er->NumberParameters < 2 ||
        er->ExceptionInformation[0] != 8 /* execute */) {
        return EXCEPTION_CONTINUE_SEARCH;
    }
    eip = (uint32_t) ep->ContextRecord->Eip;
    if ((uint32_t) er->ExceptionInformation[1] != eip || !gw_mex_any_installed) {
        return EXCEPTION_CONTINUE_SEARCH;
    }
    if (!gw_mex_in_blob(eip)) {
        /* A VANILLA engine function called through its GameCube address. m-ex code does this
         * constantly: Sonic alone has 64 raw stores like `fp->pre_hitlag_cb = efLib_PauseAll`
         * (and post_hitlag_cb = efLib_ResumeAll, take_dmg_2_cb = efLib_DestroyAll) across his
         * side-B and down-B states. On hardware that address IS the function; in the port the
         * function is the gwtool-built gw_ one, so the native call faulted on hitlag during a spin
         * move (user-found, VS match: execute fault at 0x8005BA40 = efLib_PauseAll).
         * Redirect to the native implementation. No marshalling: the caller is native code
         * calling through a native function pointer, and the target is the native build of the
         * same function, so both sides already agree on the cdecl arguments. */
        int kind = 0;
        uint32_t native = gw_mex_bridge_lookup(eip, &kind);
        if (native == 0u || kind != 1) {
            return EXCEPTION_CONTINUE_SEARCH; /* not a known function: a real crash */
        }
        gw_mex_trap_note(eip);
        ep->ContextRecord->Eip = (DWORD) native;
        return EXCEPTION_CONTINUE_EXECUTION;
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
    {   /* MELEE_MEX_TRACE_CALLS=1: the first shimmed engine calls a blob makes, with args. */
        static int trace = -1, n_logged;
        if (trace < 0) {
            const char *v = getenv("MELEE_MEX_TRACE_CALLS");
            trace = (v != NULL && v[0] == '1');
        }
        if (trace && n_logged < 40) {
            ++n_logged;
            gw_log("interp: shim call guest 0x%08X -> native 0x%08X (a0=0x%08X a1=0x%08X "
                   "a2=0x%08X a3=0x%08X)",
                   guest_addr, native, a0, a1, a2, a3);
        }
    }
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

/* The prototype-derived set (tools/mex_port/gen_sigs.py), covering 96 of the blob's 101 bridged
 * targets versus the 12 above. Regenerate with:
 *   python tools/mex_port/gen_sigs.py --blob _build/sonic_ftfunction_reloc.bin \
 *          --blob-base 0x807F4D60 --out-c melee/pc/platform/gw_mex_sigs_gen.inc
 * It refuses varargs and doubles rather than guessing, and exits non-zero if it ever disagrees
 * with the hand table above. */
#include "gw_mex_sigs_gen.inc"

/* Hand table first, so a deliberate hand entry always wins; then the generated table. A target in
 * neither keeps gw_ppc_bridge_call's integer default. */
static int gw_mex_sig_lookup(uint32_t guest_addr, gw_ppc_sig *sig) {
    unsigned i, lo, hi;
    for (i = 0; i < sizeof gw_mex_sigs / sizeof gw_mex_sigs[0]; ++i) {
        if (gw_mex_sigs[i].guest == guest_addr) {
            sig->float_args = gw_mex_sigs[i].float_args;
            sig->n_args = gw_mex_sigs[i].n_args;
            sig->ret_float = gw_mex_sigs[i].ret_float;
            return 1;
        }
    }
    lo = 0;
    hi = (unsigned) (sizeof gw_mex_gen_sigs / sizeof gw_mex_gen_sigs[0]);
    while (lo < hi) {
        unsigned mid = lo + (hi - lo) / 2u;
        if (gw_mex_gen_sigs[mid].guest < guest_addr) {
            lo = mid + 1u;
        } else {
            hi = mid;
        }
    }
    if (lo < (unsigned) (sizeof gw_mex_gen_sigs / sizeof gw_mex_gen_sigs[0]) &&
        gw_mex_gen_sigs[lo].guest == guest_addr) {
        sig->float_args = gw_mex_gen_sigs[lo].float_args;
        sig->n_args = gw_mex_gen_sigs[lo].n_args;
        sig->ret_float = gw_mex_gen_sigs[lo].ret_float;
        return 1;
    }
    return 0;
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
        (void) gw_mex_sig_lookup(guest_addr, sig);
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
    return gw_ppc_call(target, args, 1, gw_mex_r2, gw_mex_stack_top);
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
    return gw_ppc_call(target, args, 2, gw_mex_r2, gw_mex_stack_top);
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
    void *prev = gw_Mex_SelectKind(gw_mex_gobj_kind(gobj));
    uint32_t target = gw_mex_move_cb_guest_addr(gobj, which);
    uint32_t args[1];
    static const char *const names[4] = {"anim", "input", "phys", "coll"};
    static int first[4];
    static uint32_t count[4];
    uint32_t fd;

    if (target == 0) {
        gw_Mex_RestoreKind(prev);
        return;
    }
    if (!first[which]) {
        first[which] = 1;
        fd = gw_r32((const void *)(uintptr_t)((uintptr_t)gobj + 0x2Cu));
        gw_log("interp: MoveLogic %s_cb kind=%d gobj=%p motion_id=0x%X anim_id=0x%X -> guest "
               "0x%08X (interpreting)",
               names[which], gw_mex_k->port_kind, gobj, gw_r32((const void *)(uintptr_t)(fd + 0x10u)),
               gw_r32((const void *)(uintptr_t)(fd + 0x14u)), target);
    }
    args[0] = (uint32_t)(uintptr_t)gobj;
    gw_ppc_call(target, args, 1, gw_mex_r2, gw_mex_stack_top);
    ++count[which];
    if (count[which] == 1u || (count[which] % 60u) == 1u) {
        gw_log("interp: MoveLogic %s_cb invocation %u ran", names[which], count[which]);
    }
    gw_Mex_RestoreKind(prev);
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
    if (a >= 0x80000000u && a < 0x80000000u + gw_mem1_size && gw_mex_r2 != 0) {
        args[0] = (uint32_t)(uintptr_t)gobj;
        gw_ppc_call(a, args, 1, gw_mex_r2, gw_mex_stack_top);
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
    int s = gw_mex_slot_of_port(kind);
    if (s >= 0 && gw_mex_kinds[s].installed == 1 && gw_mex_kinds[s].movelogic_table != 0) {
        return (void *)(uintptr_t)gw_mex_kinds[s].movelogic_table;
    }
    return vanilla;
}

/* onLoad (slot 0) - actually runs Sonic's PPC onLoad through the interpreter. */
static void gw_mex_interp_onload(void *gobj) {
    uint32_t target = gw_mex_override_target(GW_MEX_SLOT_ON_LOAD);
    uint32_t r3;
    gw_log("interp: onLoad kind=%d entry=0x%08X gobj=%p running", gw_mex_k->port_kind, target, gobj);
    r3 = gw_mex_interp_run(GW_MEX_SLOT_ON_LOAD, gobj);
    gw_log("interp: onLoad kind=%d entry=0x%08X ran, r3=0x%08X", gw_mex_k->port_kind, target, r3);
}

/* onFrame (slot 23) - runs Sonic's PPC onFrame per fighter per frame (Fighter_8006A360 ->
 * Mex_OnFrameDispatch -> gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_FRAME)). The log is throttled to one
 * line per second (60 invocations), since OnFrame fires at 60 Hz per fighter. */
/* MELEE_MEX_TRACE_SCALE=1: each frame, walk the fighter's joint tree (gobj->hsd_obj, HSD_JObj:
 * next +0x08, child +0x10, local scale Vec3 +0x2C, world Mtx +0x44) and log any joint whose LOCAL
 * scale or WORLD matrix row length is abnormally large, naming it by depth-first index. Written to
 * find the joint behind "the fists go huge for a few frames during jab". */
static void gw_mex_trace_scale(void *gobj, uint32_t fd) {
    static int trace = -1;
    uint32_t stack[256];
    int sp = 0, idx = 0;
    uint32_t msid, root;
    if (trace < 0) {
        const char *t = getenv("MELEE_MEX_TRACE_SCALE");
        trace = (t != NULL && t[0] == '1');
    }
    if (!trace) {
        return;
    }
    msid = gw_r32((const void *) (uintptr_t) (fd + 0x10u));
    root = gw_r32((const void *) (uintptr_t) ((uintptr_t) gobj + 0x28u));
    if (root < 0x80000000u || root >= 0x80000000u + gw_mem1_size) {
        return;
    }
    stack[sp++] = root;
    while (sp > 0 && idx < 400) {
        uint32_t j = stack[--sp];
        uint32_t nx, ch;
        float sx, sy, sz, wmax = 0.0f;
        int r;
        if (j < 0x80000000u || j >= 0x80000000u + gw_mem1_size) {
            continue;
        }
        sx = gw_rf32((const void *) (uintptr_t) (j + 0x2Cu));
        sy = gw_rf32((const void *) (uintptr_t) (j + 0x30u));
        sz = gw_rf32((const void *) (uintptr_t) (j + 0x34u));
        for (r = 0; r < 3; ++r) { /* row length of the 3x3 part of the world matrix */
            float a = gw_rf32((const void *) (uintptr_t) (j + 0x44u + r * 16u + 0u));
            float b = gw_rf32((const void *) (uintptr_t) (j + 0x44u + r * 16u + 4u));
            float c = gw_rf32((const void *) (uintptr_t) (j + 0x44u + r * 16u + 8u));
            float l = (float) sqrt((double) (a * a + b * b + c * c));
            if (l > wmax) {
                wmax = l;
            }
        }
        if (sx > 3.0f || sy > 3.0f || sz > 3.0f || sx < -3.0f || sy < -3.0f || sz < -3.0f ||
            wmax > 3.0f) {
            /* Which animation TRACKS does this joint actually have? HSD_JObj.aobj +0x7C;
             * HSD_AObj.fobj +0x14; HSD_FObj: next +0x00, obj_type +0x13 (SCAX/Y/Z = 8/9/10),
             * frac_value +0x14 (quantisation format). A joint that shows a large local scale with
             * NO scale track means something other than its animation wrote it. */
            char tracks[96];
            int tl = 0, nf = 0;
            uint32_t aobj = gw_r32((const void *) (uintptr_t) (j + 0x7Cu));
            uint32_t fo = (aobj >= 0x80000000u && aobj < 0x80000000u + gw_mem1_size)
                              ? gw_r32((const void *) (uintptr_t) (aobj + 0x14u))
                              : 0u;
            tracks[0] = 0;
            while (fo >= 0x80000000u && fo < 0x80000000u + gw_mem1_size && nf < 16 &&
                   tl < (int) sizeof tracks - 12) {
                uint8_t ty = *(const uint8_t *) (uintptr_t) (fo + 0x13u);
                uint8_t fr = *(const uint8_t *) (uintptr_t) (fo + 0x14u);
                tl += snprintf(tracks + tl, sizeof tracks - (size_t) tl, "%u/%02X ", ty, fr);
                fo = gw_r32((const void *) (uintptr_t) fo);
                ++nf;
            }
            gw_log("scale: msid=0x%03X joint %d local=(%.2f,%.2f,%.2f) world_rowmax=%.2f "
                   "tracks[type/frac]: %s",
                   msid, idx, (double) sx, (double) sy, (double) sz, (double) wmax,
                   aobj ? (nf ? tracks : "(aobj, no fobj)") : "(no aobj)");
        }
        ++idx;
        nx = gw_r32((const void *) (uintptr_t) (j + 0x08u));
        ch = gw_r32((const void *) (uintptr_t) (j + 0x10u));
        /* depth-first: visit the child before the sibling, so indices follow the bone order */
        if (nx != 0u && sp < 255) {
            stack[sp++] = nx;
        }
        if (ch != 0u && sp < 255) {
            stack[sp++] = ch;
        }
    }
}

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
               gw_mex_k->port_kind, target, gobj);
    }
    r3 = gw_mex_interp_run(GW_MEX_SLOT_ON_FRAME, gobj);
    /* MELEE_MEX_TRACE_PARTS=1: log the model-part visibility table (fp->x5F4_arr[0..3], {prev,
     * idx} pairs) whenever it changes, with the action state. Groups 2/3 are Sonic's left/right
     * mouth; ProcessMouth copies [2].prev into whichever one matches the facing direction. */
    {
        static int trace = -1;
        static uint8_t last[8];
        static uint32_t last_msid = 0xFFFFFFFFu;
        uint32_t fd;
        if (trace < 0) {
            const char *t = getenv("MELEE_MEX_TRACE_PARTS");
            trace = (t != NULL && t[0] == '1');
        }
        fd = gw_r32((const void *) (uintptr_t) ((uintptr_t) gobj + 0x2Cu));
        if (fd >= 0x80000000u && fd < 0x80000000u + gw_mem1_size) {
            gw_mex_trace_scale(gobj, fd);
        }
        if (trace && fd >= 0x80000000u && fd < 0x80000000u + gw_mem1_size) {
            uint8_t cur[8];
            uint32_t msid = gw_r32((const void *) (uintptr_t) (fd + 0x10u));
            memcpy(cur, (const void *) (uintptr_t) (fd + 0x5F4u), 8);
            if (memcmp(cur, last, 8) != 0 || msid != last_msid) {
                memcpy(last, cur, 8);
                last_msid = msid;
                gw_log("parts: msid=0x%03X facing=%.0f  [0]%d/%d [1]%d/%d [2]%d/%d [3]%d/%d "
                       "(prev/idx)",
                       msid, (double) gw_rf32((const void *) (uintptr_t) (fd + 0x2Cu)),
                       (int8_t) cur[0], (int8_t) cur[1], (int8_t) cur[2], (int8_t) cur[3],
                       (int8_t) cur[4], (int8_t) cur[5], (int8_t) cur[6], (int8_t) cur[7]);
            }
        }
    }
    ++count;
    if ((count % 60u) == 1u) {
        gw_log("interp: onFrame kind=%d invocation %u ran, r3=0x%08X", gw_mex_k->port_kind, count,
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
               gw_mex_k->port_kind, target, gobj);
    }
    r3 = gw_mex_interp_run(slot, gobj);
    if (slot < 64u && (count[slot] == 1u || (count[slot] % 60u) == 1u)) {
        gw_log("interp: %s kind=%d invocation %u ran, r3=0x%08X", name, gw_mex_k->port_kind,
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
               gw_mex_k->port_kind, target, gobj, arg1);
    }
    r3 = gw_mex_interp_run2(slot, gobj, arg1);
    if (slot < 64u && (count2[slot] == 1u || (count2[slot] % 60u) == 1u)) {
        gw_log("interp: %s kind=%d invocation %u ran, r3=0x%08X", name, gw_mex_k->port_kind,
               count2[slot], r3);
    }
    return r3;
}

/* OnRespawn (slot 1) - dispatched from fighter.c's ftData_OnDeath call site (GW_MEX_EVENT_ON_DEATH),
 * which runs at every (re)spawn, including the first. Sonic's sets his model-part DEFAULTS here:
 * ftParts_80074A4C(gobj, 2, 0) makes variant 0 the default for mesh group 2, his mouth. Unregistered,
 * the Fox-clone fallback ran instead and only set group 0, so x5F4_arr[2].prev stayed -1;
 * ProcessMouth refills the mouth from that default whenever an animation is not driving it, so the
 * mouth vanished in idle and only appeared when an attack animation set it directly. */
static void gw_mex_interp_respawn(void *gobj) {
    gw_mex_interp_run_logged(GW_MEX_SLOT_ON_RESPAWN, "OnRespawn", gobj);
}

/* Slots that override a vanilla per-kind table with a dispatch site already in the game. Wired by
 * the TABLE each slot replaces (the Header.s names), not by the function name in the blob's debug
 * symbols: the two are different namespaces and both are correct - slot 21 overrides
 * ftData_OnKnockbackEnter, and Sonic's handler for it is called EyeTextureDamaged. An unregistered
 * slot does not fail; it silently runs the Fox clone's handler instead, which is exactly how his
 * idle mouth went missing (slot 1). */
static void gw_mex_interp_destroy(void *gobj) {
    gw_mex_interp_run_logged(GW_MEX_SLOT_ON_DESTROY, "onDestroy", gobj);
}
static void gw_mex_interp_item_invisible(void *gobj) {
    gw_mex_interp_run_logged(GW_MEX_SLOT_ON_ITEM_INVISIBLE, "onItemInvisible", gobj);
}
static void gw_mex_interp_item_visible(void *gobj) {
    gw_mex_interp_run_logged(GW_MEX_SLOT_ON_ITEM_VISIBLE, "onItemVisible", gobj);
}
static void gw_mex_interp_knockback_enter(void *gobj) {
    gw_mex_interp_run_logged(GW_MEX_SLOT_ON_KNOCKBACK_ENTER, "onKnockbackEnter", gobj);
}
static void gw_mex_interp_knockback_exit(void *gobj) {
    gw_mex_interp_run_logged(GW_MEX_SLOT_ON_KNOCKBACK_EXIT, "onKnockbackExit", gobj);
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

/* The three remaining item slots (16/17/18), dispatched from ftcommon.c's ftCommon_8007E6DC /
 * ftCommon_8007E7E4 / ftCommon_8007E79C - the same three call sites m-ex's "Fighter OnItem"
 * patches hook. Each passes the call site's s32 argument on in r4. Every Akaneia fighter with an
 * ftFunction overrides all three, and while they were unregistered the clone base's handler ran
 * silently in their place: a thrown or caught item behaved as the BASE character's. */
static void gw_mex_interp_item_drop_ext(void *gobj, void *arg1) {
    gw_mex_interp_run_logged2(GW_MEX_SLOT_ON_ITEM_DROP_EXT, "onItemRelease", gobj,
                              (uint32_t) (uintptr_t) arg1);
}
static void gw_mex_interp_item_pickup2(void *gobj, void *arg1) {
    gw_mex_interp_run_logged2(GW_MEX_SLOT_ON_ITEM_PICKUP2, "onItemCatch", gobj,
                              (uint32_t) (uintptr_t) arg1);
}
static void gw_mex_interp_item_drop(void *gobj, void *arg1) {
    gw_mex_interp_run_logged2(GW_MEX_SLOT_ON_ITEM_DROP, "onItemDrop", gobj,
                              (uint32_t) (uintptr_t) arg1);
}

/* Symbolizer for gw_ppc: guest address -> the containing blob function's name, or NULL. */
static const char *gw_mex_symbolize(uint32_t guest_addr) {
    gw_mex_kind *prev = gw_mex_k;
    const char *n = NULL;
    int i;
    for (i = 0; i < GW_MEX_SLOTS && n == NULL; ++i) {
        gw_mex_k = &gw_mex_kinds[i];
        n = gw_ftfunction_symbol_name(&gw_mex_ff, guest_addr);
        if (n == NULL) {
            n = gw_mex_article_symbol_name(guest_addr);
        }
    }
    gw_mex_k = prev;
    return n;
}

/* Called from game code (ftData_8008572C) every time Sonic's fighter file is (re)loaded - once per
 * residency, i.e. per scene that uses him. `kind` is the port's Ft_Kind_Sonic (33); `arch_data` /
 * `arch_data_size` are the loaded PlSn archive's data section.
 *
 * Ported from m-ex (https://github.com/akaneia/m-ex): Init ftFunction.asm relocates a fighter's
 * code IN PLACE inside its loaded Pl file, so the code lives exactly as long as the file and costs
 * no memory of its own. The port does the same, which is what lets any number of m-ex fighters
 * load match by match instead of each claiming persistent memory forever. Only the runtime's
 * shared state (mexData, the guest stack, the Arch_FighterFunc holder) is persistent.
 *
 * A reload first unwinds the previous install: its code ranges, thunks and symbols point into the
 * previous (now freed) copy of the file. */
void gw_Mex_FtFunctionInstall(int kind, void *arch_data, uint32_t arch_data_size) {
    static uint32_t mexdata_base, stack_base;
    uint32_t getdata_base;
    int rc, slot = gw_mex_slot_of_port(kind), internal;
    const char *dat;
    void *prev;

    if (slot < 0) {
        return;
    }
    internal = gw_Mex_SlotInternal(slot);
    dat = internal >= 0 ? gw_Mex_FtPlFile(internal) : NULL;
    if (dat == NULL) {
        gw_log("interp: kind %d has no m-ex fighter row - nothing to install", kind);
        return;
    }
    prev = gw_Mex_SelectKind(kind);
    gw_mex_k->port_kind = kind;
    gw_mex_k->internal = internal;
    if (gw_mex_installed < 0) {
        gw_Mex_RestoreKind(prev);
        return;
    }

    if (mexdata_base == 0u) {
        /* One-time, process-lifetime setup. */
        mexdata_base = (uint32_t) (uintptr_t) gw_mex_persist_alloc(GW_MEX_MEXDATA_SIZE);
        stack_base = (uint32_t) (uintptr_t) gw_mex_persist_alloc(GW_MEX_STACK_SIZE);
        getdata_base = (uint32_t) (uintptr_t) gw_mex_persist_alloc(GW_MEX_GETDATA_SIZE);
        gw_mex_getdata_buf = getdata_base;
        {
            /* Make every per-kind slot of the synthetic costume table point at a zeroed
             * sub-region, so onLoad's costume lookup dereferences valid guest memory and reads
             * NULL (skips). */
            uint32_t i;
            for (i = 0; i < 0x400u / 4u; ++i) {
                gw_w32((void *)(uintptr_t)(getdata_base + 4u * i), getdata_base + 0x400u);
            }
        }
        gw_mex_stack_top = stack_base + GW_MEX_STACK_SIZE - 0x100u;
        gw_mex_r2 = mexdata_base;
        gw_ppc_set_bridge(gw_mex_interp_resolve, NULL, 0u, 0u); /* code lives in added ranges */
        gw_mexdt_load();
        /* Back the interpreter's symbolizer with the blob's own debug symbol table, so every
         * panic, budget dump and trace names a guest function instead of a bare address. */
        gw_ppc_set_symbolizer(gw_mex_symbolize);
        /* First in the chain, so it runs before the port's own crash handling - which it defers
         * to for anything that is not an execute fault inside the blob. */
        if (AddVectoredExceptionHandler(1, gw_mex_exec_trap) == NULL) {
            gw_log("interp: could not install the guest execute trap - direct native calls into "
                   "guest code will crash");
        }
    }

    if (gw_mex_installed == 1) {
        /* Unwind the previous residency. Its file is gone; nothing may reach its code. */
        gw_log("interp: reinstalling %s's ftFunction (previous code 0x%08X was freed with its "
               "file)",
               dat, gw_mex_ff.code_base);
        gw_ppc_remove_code_range(gw_mex_ff.code_base, gw_mex_ff.code_base + gw_mex_ff.code_size);
        gw_mex_release_thunks(gw_mex_ff.code_base, gw_mex_ff.code_base + gw_mex_ff.code_size);
        {
            int k;
            for (k = 0; k < gw_mex_article_count; ++k) {
                gw_mex_release_thunks(gw_mex_article_lo[k], gw_mex_article_hi[k]);
            }
        }
        gw_mex_unload_items();
        gw_ftfunction_free(&gw_mex_ff);
        gw_mex_movelogic_table = 0u;
        gw_mex_installed = 0;
    }
    if (arch_data == NULL) {
        gw_log("interp: ftFunction install failed: no loaded archive");
        gw_Mex_RestoreKind(prev);
        return;
    }
    gw_mex_installed = -1;

    rc = gw_ftfunction_load_in_archive(dat, (uint32_t) internal,
                                       (uint32_t) (uintptr_t) arch_data, arch_data_size,
                                       mexdata_base, &gw_mex_ff);
    if (rc == GW_FTFUNC_ERR_NO_SYMBOL) {
        /* A fighter with no ftFunction runs entirely on its base's vanilla callbacks. */
        gw_log("interp: %s has no ftFunction - vanilla callbacks only", dat);
        gw_mex_installed = 0;
        gw_Mex_RestoreKind(prev);
        return;
    }
    if (rc != GW_FTFUNC_OK) {
        gw_log("interp: %s ftFunction install failed: load returned %d", dat, rc);
        gw_Mex_RestoreKind(prev);
        return;
    }

    gw_ppc_add_code_range(gw_mex_ff.code_base, gw_mex_ff.code_base + gw_mex_ff.code_size);
    gw_mex_any_installed = 1;
    gw_mex_load_items(dat, (uint32_t) kind, (uint32_t) (uintptr_t) arch_data, arch_data_size);

    gw_mex_movelogic_setup();

    /* Install the onLoad and onFrame overrides (this phase's deliverables). The other engine
     * events (onDeath/onDestroy/...) are not registered so they keep their vanilla behaviour. */
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_LOAD, kind, gw_mex_interp_onload);
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_FRAME, kind, gw_mex_interp_onframe);
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_DEATH, kind, gw_mex_interp_respawn);
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_DESTROY, kind, gw_mex_interp_destroy);
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_ITEM_INVISIBLE, kind,
                        gw_mex_interp_item_invisible);
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_ITEM_VISIBLE, kind,
                        gw_mex_interp_item_visible);
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_KNOCKBACK_ENTER, kind,
                        gw_mex_interp_knockback_enter);
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_KNOCKBACK_EXIT, kind,
                        gw_mex_interp_knockback_exit);
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_ACTION_STATE_CHANGE, kind,
                        gw_mex_interp_action_state_change);
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_REAPPLY_ATTR, kind,
                        gw_mex_interp_reapply_attr);
    gw_Mex_HookRegister(GW_MEX_EVENT_SPECIAL_N, kind, gw_mex_interp_special_n);
    gw_Mex_HookRegister(GW_MEX_EVENT_SPECIAL_N_AIR, kind,
                        gw_mex_interp_special_n_air);
    gw_Mex_HookRegister(GW_MEX_EVENT_SPECIAL_S, kind, gw_mex_interp_special_s);
    gw_Mex_HookRegister(GW_MEX_EVENT_SPECIAL_S_AIR, kind,
                        gw_mex_interp_special_s_air);
    gw_Mex_HookRegister(GW_MEX_EVENT_SPECIAL_HI, kind, gw_mex_interp_special_hi);
    gw_Mex_HookRegister(GW_MEX_EVENT_SPECIAL_HI_AIR, kind,
                        gw_mex_interp_special_hi_air);
    gw_Mex_HookRegister(GW_MEX_EVENT_SPECIAL_LW, kind, gw_mex_interp_special_lw);
    gw_Mex_HookRegister(GW_MEX_EVENT_SPECIAL_LW_AIR, kind,
                        gw_mex_interp_special_lw_air);
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_DOUBLE_JUMP, kind,
                        gw_mex_interp_double_jump);
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_USMASH, kind, gw_mex_interp_usmash);
    gw_Mex_HookRegister2(GW_MEX_EVENT_ON_ITEM_PICKUP, kind,
                         gw_mex_interp_item_pickup);
    gw_Mex_HookRegister2(GW_MEX_EVENT_ON_ITEM_DROP_EXT, kind,
                         gw_mex_interp_item_drop_ext);
    gw_Mex_HookRegister2(GW_MEX_EVENT_ON_ITEM_PICKUP2, kind,
                         gw_mex_interp_item_pickup2);
    gw_Mex_HookRegister2(GW_MEX_EVENT_ON_ITEM_DROP, kind,
                         gw_mex_interp_item_drop);

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

    gw_log("interp: installed %s ftFunction for kind %d / m-ex %d (code 0x%08X..0x%08X, "
           "mexData 0x%08X, stack 0x%08X, %d overrides)",
           dat, kind, internal, gw_mex_ff.code_base, gw_mex_ff.code_base + gw_mex_ff.code_size,
           mexdata_base, stack_base, gw_mex_ff.override_count);
    gw_Mex_RestoreKind(prev);
}

/* Register the module's self-contained tests (bridge lookup + resolver). */
void gw_mex_ftfunction_runtime_tests_register(void);

/* ---- tests --------------------------------------------------------------------------- */

/* MEX_GetFtItemID(sonic, 0) must be 277: Sonic is m-ex internal kind 31 with one article, global
 * ItemKind 277 - reproduced independently by tools/mex_port/dump_mxdt.py. This pins the whole chain
 * the spring depends on: the HSD relocation, the verified field paths, AND the port-kind (33) ->
 * internal-kind (31) mapping. Get the mapping wrong and it returns another fighter's item. Skips
 * on a disc without MxDt.dat (vanilla). Loads at a fixed scratch address, not the HSD heap. */
#define GW_MEXDT_TEST_BASE 0x80600000u
static int test_mex_ft_item_id_sonic(void) {
    uint32_t saved = gw_mexdt, saved_base = gw_mexdt_base, saved_size = gw_mexdt_size;
    uint32_t root, got;
    int rc = 0;
    root = gw_mex_load_hsd("MxDt.dat", "mexData", GW_MEXDT_TEST_BASE, &gw_mexdt_base,
                           &gw_mexdt_size);
    if (root == 0u) {
        gw_log("test mex_ft_item_id_sonic: no MxDt.dat on this disc - skipped");
        gw_mexdt = saved; gw_mexdt_base = saved_base; gw_mexdt_size = saved_size;
        return 0;
    }
    gw_mexdt = root;
    got = gw_mex_ft_item_global(
        (uint32_t) (GW_PORT_FT_MEX0 + gw_mex_slot_of_internal(GW_MEX_INTERNAL_SONIC)), 0u, "test");
    if (got != 277u) {
        gw_test_fail("MEX_GetFtItemID(sonic, 0) = %u, expected 277", got);
        rc = 1;
    }
    gw_mexdt = saved; gw_mexdt_base = saved_base; gw_mexdt_size = saved_size;
    return rc;
}

/* The two music tables this port reads out of mexData: the menu playlist (music +0x04/+0x08)
 * and per-fighter BGM (fighter +0x54). Both are pure layout, so the failure mode of a wrong
 * offset is garbage that happens not to crash - the checks are therefore range checks over the
 * whole table, not a single spot value, and the real values are logged for eyeballing. Skipped on
 * a disc without MxDt.dat. Disc-agnostic on purpose: Akaneia and ACE ship different playlists. */
static int test_mex_music_tables(void) {
    uint32_t saved = gw_mexdt, saved_base = gw_mexdt_base, saved_size = gw_mexdt_size;
    uint32_t root;
    int rc = 0, n, i, total = 0, nbgm, sonic_ck, slot;
    root = gw_mex_load_hsd("MxDt.dat", "mexData", GW_MEXDT_TEST_BASE, &gw_mexdt_base,
                           &gw_mexdt_size);
    if (root == 0u) {
        gw_log("test mex_music_tables: no MxDt.dat on this disc - skipped");
        gw_mexdt = saved; gw_mexdt_base = saved_base; gw_mexdt_size = saved_size;
        return 0;
    }
    gw_mexdt = root;
    nbgm = gw_Mex_BgmCount();
    n = gw_Mex_MenuPlaylistCount();
    if (n <= 0) {
        gw_test_fail("mexData has no menu playlist (count %d)", n);
        rc = 1;
    }
    for (i = 0; i < n; ++i) {
        int bgm = gw_Mex_MenuPlaylistBgm(i), chance = gw_Mex_MenuPlaylistChance(i);
        gw_log("test mex_music_tables: menu playlist[%d] = bgm %d, weight %d", i, bgm, chance);
        if (bgm < 0 || bgm >= nbgm) {
            gw_test_fail("menu playlist[%d] names bgm %d, outside the %d-entry table", i, bgm,
                         nbgm);
            rc = 1;
        }
        total += chance;
    }
    if (rc == 0 && total <= 0) {
        gw_test_fail("menu playlist weights sum to %d - no entry could ever be drawn", total);
        rc = 1;
    }
    slot = gw_mex_slot_of_internal(GW_MEX_INTERNAL_SONIC);
    sonic_ck = slot >= 0 ? GW_PORT_CK_MEX0 + slot : -1;
    if (sonic_ck >= 0) {
        int a = gw_Mex_FighterBgmForPortCKind(sonic_ck, 0);
        int b = gw_Mex_FighterBgmForPortCKind(sonic_ck, 1);
        gw_log("test mex_music_tables: fighter BGM for CKind %d = { %d, %d }", sonic_ck, a, b);
        if (a < 0 || a >= nbgm || b < 0 || b >= nbgm) {
            gw_test_fail("fighter BGM { %d, %d } outside the %d-entry table", a, b, nbgm);
            rc = 1;
        }
    }
    /* Falcon is m-ex external 0 on every m-ex disc and CharacterKind 0 in the port. */
    if (gw_Mex_FighterBgmForPortCKind(0, 0) < 0) {
        gw_test_fail("fighter BGM has no row for external id 0");
        rc = 1;
    }
    gw_mexdt = saved; gw_mexdt_base = saved_base; gw_mexdt_size = saved_size;
    return rc;
}

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
    gw_test_register("mex_ft_item_id_sonic", test_mex_ft_item_id_sonic);
    gw_test_register("mex_music_tables", test_mex_music_tables);
    gw_test_register("bridge_lookup_memcpy", test_bridge_lookup_memcpy);
    gw_test_register("bridge_lookup_global", test_bridge_lookup_global);
    gw_test_register("bridge_lookup_miss", test_bridge_lookup_miss);
    gw_test_register("resolver_mex_shims", test_resolver_mex_shims);
}
