/* gw.h - the boundary between melee's big-endian "game world" and native Windows code.
 *
 * Game code is compiled by clang for PowerPC and then rewritten by gwtool so that every memory
 * access is byte-swapped: values live big-endian in memory (exactly as on a GameCube) and native
 * in registers. Pointers are 32-bit and mean the same addresses natively, so a game pointer can
 * be dereferenced directly here -- but every multi-byte field behind it is big-endian, so it must
 * be read and written with the accessors below.
 *
 * Every SDK/libc function the game calls is an undefined symbol named gw_<name>; the shims in
 * this directory define them. Rules for writing a shim, derived from the PowerPC ABI clang used:
 *   - scalars (integers, floats, pointers) are passed natively: declare them normally;
 *   - a struct passed by value arrives as a byte-for-byte copy, so its fields are big-endian
 *     (GXColor is four bytes, so it is unaffected; GXColorS10 is four s16 and is not);
 *   - a struct return of <= 8 bytes comes back packed into an integer in big-endian order;
 *   - a larger struct return uses a hidden first pointer to a big-endian destination (sret).
 */
#ifndef GW_H
#define GW_H

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ---- byte-swapped access to game memory ------------------------------------------------- */

static inline uint16_t gw_bswap16(uint16_t v) { return (uint16_t)((v >> 8) | (v << 8)); }
static inline uint32_t gw_bswap32(uint32_t v) {
  return (v >> 24) | ((v >> 8) & 0xFF00u) | ((v << 8) & 0xFF0000u) | (v << 24);
}
static inline uint64_t gw_bswap64(uint64_t v) {
  return ((uint64_t)gw_bswap32((uint32_t)v) << 32) | gw_bswap32((uint32_t)(v >> 32));
}

static inline uint8_t gw_r8(const void *p) { return *(const uint8_t *)p; }
static inline void gw_w8(void *p, uint8_t v) { *(uint8_t *)p = v; }

static inline uint16_t gw_r16(const void *p) {
  uint16_t v;
  memcpy(&v, p, 2);
  return gw_bswap16(v);
}
static inline void gw_w16(void *p, uint16_t v) {
  v = gw_bswap16(v);
  memcpy(p, &v, 2);
}
static inline uint32_t gw_r32(const void *p) {
  uint32_t v;
  memcpy(&v, p, 4);
  return gw_bswap32(v);
}
static inline void gw_w32(void *p, uint32_t v) {
  v = gw_bswap32(v);
  memcpy(p, &v, 4);
}
static inline uint64_t gw_r64(const void *p) {
  uint64_t v;
  memcpy(&v, p, 8);
  return gw_bswap64(v);
}
static inline void gw_w64(void *p, uint64_t v) {
  v = gw_bswap64(v);
  memcpy(p, &v, 8);
}

static inline float gw_rf32(const void *p) {
  uint32_t v = gw_r32(p);
  float f;
  memcpy(&f, &v, 4);
  return f;
}
static inline void gw_wf32(void *p, float f) {
  uint32_t v;
  memcpy(&v, &f, 4);
  gw_w32(p, v);
}
static inline double gw_rf64(const void *p) {
  uint64_t v = gw_r64(p);
  double d;
  memcpy(&d, &v, 8);
  return d;
}
static inline void gw_wf64(void *p, double d) {
  uint64_t v;
  memcpy(&v, &d, 8);
  gw_w64(p, v);
}

/* A pointer stored in game memory is a big-endian 32-bit value naming the same address space. */
static inline void *gw_rptr(const void *p) { return (void *)(uintptr_t)gw_r32(p); }
static inline void gw_wptr(void *p, const void *v) { gw_w32(p, (uint32_t)(uintptr_t)v); }

/* Bulk helpers for the float arrays the GX API passes around (matrices, viewports, ...). */
void gw_read_f32v(float *dst, const void *src, int count);
void gw_write_f32v(void *dst, const float *src, int count);
void gw_read_mtx(float dst[3][4], const void *src);
void gw_write_mtx(void *dst, const float src[3][4]);
void gw_read_mtx44(float dst[4][4], const void *src);
void gw_write_mtx44(void *dst, const float src[4][4]);

/* ---- diagnostics ------------------------------------------------------------------------ */

void gw_log(const char *fmt, ...);
void gw_logv(const char *fmt, va_list ap);
/* Logs two strings with no format expansion; safe for printing a suspect format string. */
void gw_log_raw(const char *prefix, const char *text);
/* True when MELEE_PC_TRACE_OSREPORT is set; gates the OSReport format trace. */
bool gw_trace_osreport(void);
/* Logs the first call of an unimplemented entry point, then counts silently. Whatever appears
 * during boot is the real to-do list, in the order the game needs it. */
void gw_stub_hit(const char *name);
#define GW_STUB() gw_stub_hit(__FUNCTION__)
void gw_panic(const char *fmt, ...);
void gw_dump_stub_summary(void);
void gw_archive_crash_log(const char *reason);
/* Turns a fault from "the log stops" into a logged address, .map RVA and frame list. Install
 * before anything else so faults during startup are reported too. */
void gw_install_crash_handler(void);
/* Logs a code address as a melee-pc.map rva, or as module+offset when it is not in the exe. */
void gw_log_code_addr(const char *label, const void *addr);
/* PAGE_GUARD a region and log the faulting instruction of accesses, to find what overwrites a
 * field. gw_watch_tick re-arms it; call that once per frame from the frame pump. */
void gw_watch_page(void *addr, size_t size);
void gw_watch_tick(void);
/* Samples the game thread's pc every few seconds. Finds loops that never present a frame,
 * which produce no log output and look exactly like a hang from outside. */
void gw_start_watchdog(void);

/* ---- process-wide runtime ---------------------------------------------------------------- */

/* MEM1: the game's 24 MB main memory, mapped at 0x80000000 when the OS allows it, because game
 * code tells main memory from ARAM by comparing addresses against 0x80000000. */
extern unsigned char *gw_mem1;
extern uint32_t gw_mem1_size;
extern unsigned char *gw_aram;
extern uint32_t gw_aram_size;

bool gw_mem_init(void);
/* Fills the OS globals at the bottom of MEM1 that the console's IPL would have written; without
 * them OS_BUS_CLOCK is zero and every tick conversion collapses to zero. Called by gw_mem_init. */
void gw_init_lomem(void);
void gw_apply_fixups(void); /* swap link-time pointers in game globals; must run first */
const char *gw_iso_path(void);

/* Synchronous read of a whole disc file into a malloc'd buffer (caller frees). NULL on failure;
 * *out_size is 0 unless the read succeeded. Headless path for the m-ex ftFunction loader. */
void *gw_DVDReadFileAlloc(const char *path, uint32_t *out_size);

/* Whether a ported m-ex behavior is enabled (MELEE_MEX env / mods\mex.txt). See tools/mex_port/. */
int gw_Mex_Enabled(const char *name);

/* melee's own main(), renamed by gwtool. */
void gw_main(void);

/* ---- m-ex Tier C fighter hook surface (native re-expression) ----------------------------
 * Ported from m-ex (https://github.com/akaneia/m-ex): each "Fighter On*" patch replaces one
 * `addi` that computes a per-character callback-table base, and the decomp already calls
 * `ftData_<X>[fp->kind](gobj)` at those sites. A hook is therefore a table-slot override: one
 * native function pointer per (event, kind); NULL means "no override, run the vanilla entry".
 * See _research/mex-tier-c-hooks.md. `kind` is the internal FighterKind (melee/ft/forward.h
 * Ft_Kind_*), 0..Ft_Kind_Max-1. Dispatch is a flat array, not a chain, because OnFrame runs
 * per-fighter per-frame and m-ex is single-slot (last registration wins). */

typedef void (*gwmex_gobj_fn)(void* gobj);
/* Two-argument hook (gobj + one extra word, e.g. OnItemPickup's item_gobj). */
typedef void (*gwmex_gobj_fn2)(void* gobj, void* arg1);
/* Predicates (Category 2, e.g. OnFloat) return a value deciding whether the behaviour fires. It is
 * `int`, not `bool`, for the same boundary reason as gw_Mex_Enabled: game code's MSL `bool` is
 * `int`, the native layer's is `_Bool`. */
typedef int (*gwmex_gobj_pred)(void* gobj);

/* Event ids mirror the ftData_* table families (one id per table). The game-side call sites use
 * the per-event dispatch wrappers below, so these ids only cross the boundary at registration. */
enum {
    GW_MEX_EVENT_ON_LOAD = 0,         /* ftData_OnLoad            */
    GW_MEX_EVENT_ON_DEATH,            /* ftData_OnDeath           */
    GW_MEX_EVENT_ON_DESTROY,          /* ftData_OnUserDataRemove  */
    GW_MEX_EVENT_ON_FRAME,            /* ftData_UnkMotionStates3  */
    GW_MEX_EVENT_ON_ABSORB,           /* ftData_OnAbsorb          */
    GW_MEX_EVENT_ON_APPLY_HEAD_ITEM,  /* ftData_UnkMotionStates1  */
    GW_MEX_EVENT_ON_REMOVE_HEAD_ITEM, /* ftData_UnkMotionStates2  */
    GW_MEX_EVENT_ON_ITEM_INVISIBLE,   /* ftData_OnItemInvisible   */
    GW_MEX_EVENT_ON_ITEM_VISIBLE,     /* ftData_OnItemVisible     */
    GW_MEX_EVENT_ON_KNOCKBACK_ENTER,  /* ftData_OnKnockbackEnter  */
    GW_MEX_EVENT_ON_KNOCKBACK_EXIT,   /* ftData_OnKnockbackExit   */
    GW_MEX_EVENT_ON_ACTION_STATE_CHANGE, /* ftData_UnkMotionStates4  */
    GW_MEX_EVENT_ON_REAPPLY_ATTR,     /* ftKindCalcIndiviParamTable */
    GW_MEX_EVENT_SPECIAL_N,           /* ftData_SpecialN            */
    GW_MEX_EVENT_SPECIAL_N_AIR,       /* ftData_SpecialAirN         */
    GW_MEX_EVENT_SPECIAL_S,           /* ftData_SpecialS            */
    GW_MEX_EVENT_SPECIAL_S_AIR,       /* ftData_SpecialAirS         */
    GW_MEX_EVENT_SPECIAL_HI,          /* ftData_SpecialHi           */
    GW_MEX_EVENT_SPECIAL_HI_AIR,      /* ftData_SpecialAirHi        */
    GW_MEX_EVENT_SPECIAL_LW,          /* ftData_SpecialLw           */
    GW_MEX_EVENT_SPECIAL_LW_AIR,      /* ftData_SpecialAirLw        */
    GW_MEX_EVENT_MOVE_LOGIC,          /* MoveLogic (m-ex Arch_FighterFunc slot 3) */
    GW_MEX_EVENT_ON_DOUBLE_JUMP,      /* ftCo_800CBAC4 (m-ex onDoubleJump, slot 32) */
    GW_MEX_EVENT_ON_USMASH,           /* ftCo_AttackHi4 doEnter (m-ex onUSmash, slot 36) */
    GW_MEX_EVENT_ON_ITEM_PICKUP,      /* ftpickupitem_800948A8 (m-ex OnItemPickup, slot 13) */
    GW_MEX_EVENT_COUNT
};

#define GW_MEX_KIND_MAX 34 /* Ft_Kind_Max (melee/ft/forward.h) */

/* Register/clear a per-(event, kind) override. fn == NULL clears the slot, so the vanilla entry
 * runs again. Last registration wins; m-ex does not chain. Returns 0 on a bad event/kind (int, not
 * bool: game code's MSL `bool` is `int`, while the native layer's is `_Bool`, so the boundary uses
 * int like gw_Mex_Enabled). */
int gw_Mex_HookRegister(int event, int kind, gwmex_gobj_fn fn);
int gw_Mex_HookRegister2(int event, int kind, gwmex_gobj_fn2 fn);
int gw_Mex_PredicateRegister(int event, int kind, gwmex_gobj_pred fn);

/* Call override[event][kind](gobj) if registered, else vanilla(gobj) if non-NULL. `vanilla` is the
 * decomp table entry already byte-swapped to native by the game call site. */
void gw_Mex_GObjDispatch(int event, int kind, void* gobj, void* vanilla);
void gw_Mex_GObjDispatch2(int event, int kind, void* gobj, void* arg1, void* vanilla);

/* Predicate dispatch for the Category 2 events (OnFloat and friends), whose return value decides
 * whether the behaviour fires: returns the registered predicate's result, else `vanilla`'s, else 0.
 * No game call site yet - Category 2 is deferred (_research/mex-tier-c-hooks.md section 7.3) - but
 * the surface is complete and tested, so a future site has a proven entry point. */
int gw_Mex_GObjPredDispatch(int event, int kind, void* gobj, void* vanilla);

/* Per-event dispatch wrappers for the game-side call sites (keeps the event id out of game code). */
void gw_Mex_OnLoadDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_OnDeathDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_OnDestroyDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_OnFrameDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_OnAbsorbDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_OnApplyHeadItemDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_OnRemoveHeadItemDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_OnItemInvisibleDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_OnItemVisibleDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_OnKnockbackEnterDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_OnKnockbackExitDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_OnActionStateChangeDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_OnReapplyAttrDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_SpecialNDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_SpecialNAirDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_SpecialSDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_SpecialSAirDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_SpecialHiDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_SpecialHiAirDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_SpecialLwDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_SpecialLwAirDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_MoveLogicDispatch(int kind, void* gobj, void* vanilla);
/* MoveLogic (m-ex Arch_FighterFunc slot 3) is a per-kind MotionState[] table, not a callback: the
 * engine site (Fighter_UnkInitLoad_80068914) asks for the per-kind character-state table and this
 * returns the interpreted MoveLogic table for Sonic, else `vanilla` (ftData_CharacterStateTables). */
void* gw_Mex_MoveLogicTable(int kind, void* vanilla);
/* Dispatch a fighter callback field (accessory1_cb/accessory4_cb/deal_dmg_cb/...) that m-ex guest
 * code may have overwritten with a guest PPC pointer: interpret a guest address, else call it. */
void gw_Mex_FighterCallbackDispatch(void* gobj, void* cb);
void gw_Mex_OnDoubleJumpDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_OnUSmashDispatch(int kind, void* gobj, void* vanilla);
void gw_Mex_OnItemPickupDispatch(int kind, void* gobj, void* arg1, void* vanilla);

#ifdef __cplusplus
}
#endif
#endif /* GW_H */
