/* gw_mex_grfunction.h - m-ex custom STAGES: the grFunction blob and the expanded stage tables.
 *
 * grFunction is to stages exactly what ftFunction is to fighters. A stage .dat (e.g. Akaneia's
 * GrOMc.dat) carries a public symbol `grFunction` holding a relocatable PPC MEXFunction whose
 * `functionRelocTable` names, per entry, a WORD INDEX INTO `StageData` (melee/gr/types.h) and the
 * code offset to put there. m-ex's "Init grFunction.asm" runs Reloc + Overload on it at stage-file
 * load time, against `mexData.stage_desc[internal_stage_id]`.
 *
 * The four things this file owns:
 *
 *  1. THE STAGE TABLES. `mexData +0x28` (m-ex `Arch_Map`) holds StageIDs / Audio / LineTypeData /
 *     StageItemLookup / StageNames / Playlists; `mexData +0x2C` (m-ex `Arch_grFunction`) is an
 *     array of `internal_stage_count` pointers to StageData rows. On Akaneia that is 96 internal
 *     stages (0..70 vanilla, 71..95 added) and 313 external ids.
 *
 *  2. THE TWO INDEX SPACES. INTERNAL (= the port's `GrKind`; what `stage_datas[]` and
 *     `stage_info.grkind` use) and EXTERNAL (= the port's `StKind`; what the stage-select screen
 *     and `stage_id_map[]` use). `Arch_Map_StageIDs` is the external -> internal map, stride 12,
 *     the same `struct StageIdMapEntry` the port already has - so it is a straight table
 *     replacement, not a new space. External 288..312 map to internal 71..95.
 *
 *  3. THE BLOB. Loaded and relocated IN PLACE inside the stage's own loaded archive, as m-ex does
 *     on hardware, so it lives exactly as long as the file.
 *
 *  4. THE SEVEN CALL SLOTS. `StageData`'s function fields are called from gwtool-compiled game
 *     code, which would call a guest address as if it were native x86. Each m-ex stage row is
 *     therefore given native trampolines (below) that interpret the blob's override when there is
 *     one, and otherwise call the vanilla function the MxDt row names (its CLONE BASE - an
 *     unregistered slot silently runs the base's handler, exactly as on hardware).
 *
 * This is NATIVE platform code (compiled directly with the i686 clang, not through gwtool).
 */
#ifndef GW_MEX_GRFUNCTION_H
#define GW_MEX_GRFUNCTION_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Word indices into `struct StageData` (melee/gr/types.h), which is what a grFunction entry's
 * `ReplaceThis` names. Verified against the shipped table: every added stage overrides 1/3/5 and
 * most also 4/6/7/8/9; 0, 2, 10, 11 and 12 are never overridden. */
enum {
    GW_MEX_GR_SLOT_GRKIND = 0,
    GW_MEX_GR_SLOT_CALLBACKS = 1,      /* StageCallbacks[] - DATA, not code */
    GW_MEX_GR_SLOT_FILE = 2,           /* char* "/GrXx.dat" */
    GW_MEX_GR_SLOT_ON_INIT = 3,
    GW_MEX_GR_SLOT_ON_DEMO_INIT = 4,
    GW_MEX_GR_SLOT_ON_LOAD = 5,
    GW_MEX_GR_SLOT_ON_START = 6,
    GW_MEX_GR_SLOT_CALLBACK4 = 7,
    GW_MEX_GR_SLOT_ON_TOUCH_LINE = 8,
    GW_MEX_GR_SLOT_ON_CHECK_SHADOW = 9,
    GW_MEX_GR_SLOT_FLAGS2 = 10,
    GW_MEX_GR_SLOT_JOINTS = 11,
    GW_MEX_GR_SLOT_JOINT_COUNT = 12,
    GW_MEX_GR_SLOT_COUNT = 13
};

/* The port's `stage_datas[]` has 111 rows, so every m-ex internal stage id (0..95 on Akaneia)
 * already fits. This bound exists to reject a build whose table grew past it. */
#define GW_MEX_GR_MAX 111

/* First m-ex-ADDED internal stage id: 0..70 are the vanilla stages the port compiles in. */
#define GW_MEX_GR_FIRST_NEW 71

/* ---- tables ------------------------------------------------------------------------------
 * All return 0/-1/NULL when MxDt.dat is absent (a vanilla disc), so game code can call them
 * unconditionally. */

/* mexData.metadata internal/external stage counts, or 0. */
/* Drop the cached stage tables; the next call re-resolves them. See
 * gw_Mex_InvalidateAfterMem1Restore() - the tables live in MEM1 but the pointers do not. */
void gw_Mex_GrInvalidate(void);
int gw_Mex_GrInternalCount(void);
int gw_Mex_GrExternalCount(void);

/* Arch_Map_StageIDs[ext].grkind, or -1 when out of range / no mexData. */
int gw_Mex_GrKindForExt(int ext);

/* The stage's "/GrXx.dat" path, or NULL when the row is empty OR the file is not on this disc.
 * The disc check is what makes the added rows DENSE in practice: a build that declares a stage it
 * does not ship must not get a StageData row, or the first load walks off into a missing file. */
const char *gw_Mex_GrFile(int grkind);

/* Non-zero when `grkind` is an m-ex-added stage whose file is on this disc, i.e. a row the port
 * must synthesise rather than one it compiled in. */
int gw_Mex_GrIsMex(int grkind);

/* The MxDt row's word `slot` translated to something game code can store:
 *  - a function slot (3..9) -> the native address of the vanilla function the row names, via the
 *    guest->native bridge; NULL when it is 0 or -1 or has no bridge entry (logged once);
 *  - flags2 (10) -> the raw value.
 * The row's CALLBACKS and FILE words are reached through gw_Mex_GrFile / the blob instead. */
void *gw_Mex_GrVanillaFn(int grkind, int slot);
uint32_t gw_Mex_GrFlags2(int grkind);

/* ---- per-load ---------------------------------------------------------------------------- */

/* Tell the runtime which stage is being brought up. Called from Ground_801C0754 before the file
 * loads, so the trampolines below know whose row they belong to even if the blob never installs. */
void gw_Mex_GrSelect(int grkind);

/* Load + Reloc + Overload the stage archive's `grFunction`, in place inside `archive` (an
 * HSD_Archive*). No-op for a vanilla stage, for a stage with no grFunction symbol, or with no
 * mexData. Never fails hard: a failure is logged and the stage keeps its clone-base handlers. */
void gw_Mex_GrFunctionInit(void *archive, int grkind);

/* The current stage's StageCallbacks[] - the blob's `map_gobjs` table when it overrode slot 1,
 * else NULL so the caller keeps its own compiled-in table. This is m-ex's `Get grFunction`
 * standalone function (guest 0x803D7068), which every custom stage's map-gobj creator calls. */
void *gw_Mex_GrCallbacks(void);

/* Bind a StageCallbacks entry (on_init / gobj_proc / callback3) to something native code can
 * call: blob code becomes a thunk, a native pointer passes through unchanged. */
void *gw_Mex_GrBind(void *fn);

/* ---- the seven StageData trampolines ------------------------------------------------------
 * Game code stores these in a synthesised StageData row. Each runs the current stage's blob
 * override for its slot, or the vanilla clone-base function from the MxDt row, or nothing. */
void gw_Mex_GrOnInit(void);
void gw_Mex_GrOnDemoInit(int arg);
void gw_Mex_GrOnLoad(void);
void gw_Mex_GrOnStart(void);
int gw_Mex_GrCallback4(void);
void *gw_Mex_GrOnTouchLine(int index);
int gw_Mex_GrOnCheckShadowRender(void *pos, int arg1, void *jobj);

/* Registers this module's self-contained tests with the in-engine suite. */
void gw_mex_grfunction_tests_register(void);

#ifdef __cplusplus
}
#endif
#endif /* GW_MEX_GRFUNCTION_H */
