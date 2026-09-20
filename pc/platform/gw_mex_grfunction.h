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

/* Must equal ARRAY_SIZE(stage_datas) in src/melee/gr/ground.c: this bound exists to reject a disc
 * whose internal stage count would index past that table.
 *
 * SIZED AGAINST ACE, NOT AKANEIA. The vanilla 111 rows look "already big enough" at Akaneia's 96
 * internal stages and are not at ACE's 155 - which is exactly why ACE's stages were silently off.
 * 160 = ACE's 155 plus a little slack, and ground.c's stage_datas[] is declared with that many
 * rows under TARGET_PC. Grow both together or the guard stops meaning anything. */
#define GW_MEX_GR_MAX 160

/* Must equal ST_MEX_EXT_MAX in src/melee/gr/stage.c, i.e. ARRAY_SIZE(stage_id_map). Every
 * external->internal lookup indexes that array by a raw StKind taken from mexData, so a disc
 * declaring more external ids than the port compiled rows for has to be refused here rather than
 * read off the end later. Akaneia declares 313, ACE 372. */
#define GW_MEX_GR_EXT_MAX 384

/* First m-ex-ADDED internal stage id: 0..70 are the vanilla stages the port compiles in. */
#define GW_MEX_GR_FIRST_NEW 71

/* ---- tables ------------------------------------------------------------------------------
 * All return 0/-1/NULL when MxDt.dat is absent (a vanilla disc), so game code can call them
 * unconditionally. */

/* mexData.metadata internal/external stage counts, or 0. */
/* Drop the cached stage tables; the next call re-resolves them. See
 * gw_Mex_InvalidateAfterMem1Restore() - the tables live in MEM1 but the pointers do not. */
void gw_Mex_GrInvalidate(void);
/* Stage BGM, from Arch_Map_Playlists[INTERNAL grkind]. Count is 0 when this stage has no
 * playlist (or on a disc with no mexData). gw_Mex_GrBgmEntry fills *bgm and *chance for entry
 * `i`; the weighted draw itself stays in game code so it uses the game's own RNG. */
int gw_Mex_GrBgmCount(int grkind);
int gw_Mex_GrBgmId(int grkind, int i);     /* -1 when there is no such entry */
int gw_Mex_GrBgmChance(int grkind, int i); /*  0 when there is no such entry */

int gw_Mex_GrInternalCount(void);
int gw_Mex_GrExternalCount(void);

/* Arch_Map_StageIDs[ext].grkind, or -1 when out of range / no mexData. */
int gw_Mex_GrKindForExt(int ext);

/* The stage's "/GrXx.dat" path as the MxDt row names it, or NULL when the row is empty. This
 * does NOT say the file is on this disc - gw_Mex_GrIsMex() below is what adds the disc check,
 * and it is the one that decides whether a row gets synthesised. The distinction is real and
 * easy to get wrong: a VANILLA disc run with the mods folder on reads Akaneia's full 96-row
 * MxDt.dat out of the sonic mod and carries none of the stage files, so every added row here
 * names a path that cannot be read. */
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

/* MELEE_GR_TRACE=1: narrate the m-ex stage bring-up. Diagnostics only. */
int gw_Mex_GrTrace(void);

/* Registers this module's self-contained tests with the in-engine suite. */
void gw_mex_grfunction_tests_register(void);

#ifdef __cplusplus
}
#endif
#endif /* GW_MEX_GRFUNCTION_H */
