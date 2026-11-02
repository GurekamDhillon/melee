/* SPDX-License-Identifier: MIT
 * Copyright (c) 2026 GurekamDhillon */
/* gdmod.h - mod-c-v1 (ABI 1.0). Spec: docs/specs/mod-c-v1.md. Hook table: docs/specs/hooks-v1.tsv.
 *
 * One C source, two targets: native (compiled into the host build, x86-64) and WebAssembly (wasm32, AOT, sandboxed).
 * Freestanding: this header includes nothing but <stdint.h>-equivalent builtin types and needs no libc.
 *
 * Required compile flags: -ffp-contract=off (clang/gcc) or /fp:precise (MSVC), -fwrapv, -fno-math-errno,
 * no -ffast-math, no x87 (x86-64 SSE only), wasm: also -mno-simd128 is NOT required (SIMD is rejected by the loader).
 */
#ifndef GDMOD_H
#define GDMOD_H

/* ---- fixed-width types, no libc ----------------------------------------------------------------------------- */
typedef signed char gd_i8;
typedef unsigned char gd_u8;
typedef short gd_i16;
typedef unsigned short gd_u16;
typedef int gd_i32;
typedef unsigned int gd_u32;
typedef long long gd_i64;
typedef unsigned long long gd_u64;
typedef float gd_f32;
typedef gd_u32 gd_bool; /* 0 or 1 only */

_Static_assert(sizeof(gd_i8) == 1 && sizeof(gd_i16) == 2 && sizeof(gd_i32) == 4 && sizeof(gd_i64) == 8, "int widths");
_Static_assert(sizeof(gd_f32) == 4, "f32");
_Static_assert(sizeof(void*) == 4 || sizeof(void*) == 8, "pointer width");

#define GD_ABI_MAJOR 1u
#define GD_ABI_MINOR 0u
#define GD_ABI_VERSION ((GD_ABI_MAJOR << 16) | GD_ABI_MINOR)
#define GD_HOOK_TABLE_VERSION 1u /* hooks-v1.tsv */

/* ---- import/export plumbing --------------------------------------------------------------------------------- */
#if defined(__wasm__)
#define GD_WASM 1
#define GD_IMPORT(name) __attribute__((import_module("gd"), import_name(#name)))
#define GD_EXPORT(name) __attribute__((export_name(#name), used))
/* data objects are exported by symbol visibility (wasm-ld --export-dynamic exports them as globals holding the address) */
#define GD_EXPORT_DATA(name) __attribute__((visibility("default"), used))
#else
#define GD_WASM 0
#define GD_IMPORT(name)
#if defined(_MSC_VER)
#define GD_EXPORT(name) __declspec(dllexport)
#define GD_EXPORT_DATA(name) __declspec(dllexport)
#else
#define GD_EXPORT(name) __attribute__((visibility("default"), used))
#define GD_EXPORT_DATA(name) __attribute__((visibility("default"), used))
#endif
#endif

/* A host build may supply the accessors as static inline functions: define GD_HOST_INLINE_HEADER to a quoted header
 * path (it is included below, and every gd_* function it defines wins) and GD_ACCESSORS_INLINE to 1. Otherwise the
 * accessors are ordinary external functions (a native build resolves them at link/load time; wasm imports them from
 * module "gd"). The semantics are identical either way. */
#if defined(GD_ACCESSORS_INLINE) && GD_ACCESSORS_INLINE
#define GD_API(ret, name, args) /* defined by the host header as static inline */
#ifdef GD_HOST_INLINE_HEADER
#include GD_HOST_INLINE_HEADER
#endif
#else
#define GD_API(ret, name, args) GD_IMPORT(name) ret name args
#endif

/* ---- handles ------------------------------------------------------------------------------------------------ */
/* A handle is an opaque u32: (generation << 16) | index, 0 = invalid. Never a pointer, never persisted in mod state
 * across a match (slots are reused; the generation catches stale use: a stale handle reads 0 / writes nothing and the
 * host counts it). Handle values are NOT stable between builds; do not hash or compare them across builds. */
typedef gd_u32 gd_fighter; /* a fighter instance (Fighter + its GObj) */
typedef gd_u32 gd_item;    /* an item instance */
typedef gd_u32 gd_gobj;    /* any GObj: only gd_gobj_kind / gd_gobj_to_* are valid on it */
typedef gd_u32 gd_player;  /* a player slot object; also valid as a small id: use gd_player_from_slot */
typedef gd_u32 gd_stage;   /* the stage (a single handle for the match) */
typedef gd_u32 gd_spawn;   /* an item spawn request (SpawnItem) alive for the duration of its hook call only */
typedef gd_u32 gd_ptr;     /* a pointer the ABI does not expose: always 0 in v1 */

#define GD_NULL_HANDLE 0u

enum { GD_MAX_PLAYERS = 4, GD_SLOT_NONE = -1 };

enum gd_gobj_kind_e { GD_GOBJ_OTHER = 0, GD_GOBJ_FIGHTER = 1, GD_GOBJ_ITEM = 2, GD_GOBJ_STAGE = 3 };

/* ---- value types (little-endian, natural alignment, 4-byte aligned, no padding, identical on both targets) ----- */
typedef struct gd_vec3 { gd_f32 x, y, z; } gd_vec3;
typedef struct gd_vec2 { gd_f32 x, y; } gd_vec2;

/* Controller snapshot of a fighter for this logic frame (Fighter.input slot 0: after the game's stick processing). */
typedef struct gd_input {
    gd_f32 lstick_x, lstick_y; /* -1..1 */
    gd_f32 cstick_x, cstick_y;
    gd_f32 trigger;            /* analog shoulder, 0..1 */
    gd_u32 held;               /* HSD_Pad bits held */
    gd_u32 pressed;            /* newly pressed this frame */
    gd_u32 released;           /* newly released this frame */
} gd_input;
_Static_assert(sizeof(gd_input) == 32, "gd_input layout");

/* Bulk copy-out of the fighter fields most mods read; one call instead of ten. Copy-in (write) takes a mask. */
typedef struct gd_fighter_view {
    gd_vec3 pos;        /* Fighter.cur_pos */
    gd_vec3 vel;        /* Fighter.self_vel */
    gd_vec3 kb_vel;     /* Fighter.x8c_kb_vel */
    gd_f32 facing;      /* Fighter.facing_dir, +1 or -1 */
    gd_f32 percent;     /* Fighter.dmg.x1830_percent */
    gd_i32 action_state;/* Fighter.motion_id */
    gd_i32 kind;        /* Fighter.kind (internal character id) */
    gd_i32 slot;        /* Fighter.player_id, 0..3 */
    gd_u32 reserved[6]; /* zero on read, ignored on write */
} gd_fighter_view;
_Static_assert(sizeof(gd_fighter_view) == 80, "gd_fighter_view layout");

enum { /* gd_fighter_write mask bits; only the named fields are written */
    GD_FV_POS = 1u << 0, GD_FV_VEL = 1u << 1, GD_FV_KB_VEL = 1u << 2, GD_FV_FACING = 1u << 3, GD_FV_PERCENT = 1u << 4
};

typedef struct gd_item_view {
    gd_vec3 pos;      /* Item.pos */
    gd_vec3 vel;      /* Item.x40_vel */
    gd_f32 facing;    /* Item.facing_dir */
    gd_i32 kind;      /* Item.kind */
    gd_i32 owner_slot;/* slot of Item.owner if it is a fighter, else GD_SLOT_NONE */
    gd_u32 reserved[4];
} gd_item_view;
_Static_assert(sizeof(gd_item_view) == 52, "gd_item_view layout");

/* ---- accessors (all deterministic, no side effect beyond the named write, never invoke hooks) -------------------
 * A write through an invalid or stale handle is a counted no-op. Floats written are NaN-canonicalised by the host. */
GD_API(gd_u32, gd_abi_host_version, (void));
GD_API(gd_u32, gd_gobj_kind, (gd_gobj g));
GD_API(gd_fighter, gd_gobj_to_fighter, (gd_gobj g)); /* 0 if g is not a fighter */
GD_API(gd_item, gd_gobj_to_item, (gd_gobj g));

/* fighter */
GD_API(gd_fighter, gd_fighter_from_slot, (gd_i32 slot)); /* 0 if none; sub-fighters (Nana) are not addressable in v1 */
GD_API(gd_i32, gd_fighter_slot, (gd_fighter f));
GD_API(gd_i32, gd_fighter_kind, (gd_fighter f));
GD_API(gd_gobj, gd_fighter_gobj, (gd_fighter f));
GD_API(gd_f32, gd_fighter_get_pos_x, (gd_fighter f));
GD_API(gd_f32, gd_fighter_get_pos_y, (gd_fighter f));
GD_API(gd_f32, gd_fighter_get_pos_z, (gd_fighter f));
GD_API(void, gd_fighter_set_pos, (gd_fighter f, gd_f32 x, gd_f32 y, gd_f32 z));
GD_API(gd_f32, gd_fighter_get_vel_x, (gd_fighter f));
GD_API(gd_f32, gd_fighter_get_vel_y, (gd_fighter f));
GD_API(gd_f32, gd_fighter_get_vel_z, (gd_fighter f));
GD_API(void, gd_fighter_set_vel, (gd_fighter f, gd_f32 x, gd_f32 y, gd_f32 z));
GD_API(gd_i32, gd_fighter_get_action_state, (gd_fighter f));
/* Request a state change through the game's own state-change path at the end of the current hook; not applied inside
 * a replace of that state's own function. v1: no flags, game defaults. Returns 0 if rejected (stale handle). */
GD_API(gd_bool, gd_fighter_request_action_state, (gd_fighter f, gd_i32 state));
GD_API(gd_f32, gd_fighter_get_percent, (gd_fighter f));
GD_API(void, gd_fighter_set_percent, (gd_fighter f, gd_f32 percent));
GD_API(gd_f32, gd_fighter_get_facing, (gd_fighter f));
GD_API(void, gd_fighter_set_facing, (gd_fighter f, gd_f32 facing)); /* only +1.0f / -1.0f are accepted */
GD_API(void, gd_fighter_get_input, (gd_fighter f, gd_input* out));
GD_API(void, gd_fighter_read, (gd_fighter f, gd_fighter_view* out));
GD_API(void, gd_fighter_write, (gd_fighter f, const gd_fighter_view* in, gd_u32 mask));

/* item */
GD_API(gd_i32, gd_item_kind, (gd_item it));
GD_API(gd_gobj, gd_item_gobj, (gd_item it));
GD_API(void, gd_item_read, (gd_item it, gd_item_view* out));
GD_API(void, gd_item_write, (gd_item it, const gd_item_view* in, gd_u32 mask)); /* mask: GD_FV_POS | GD_FV_VEL | GD_FV_FACING */
/* number of live items; iterate by index in the game's item list order (deterministic) */
GD_API(gd_i32, gd_item_count, (void));
GD_API(gd_item, gd_item_at, (gd_i32 index));

/* spawn request (valid only inside item.spawn* hooks) */
GD_API(gd_i32, gd_spawn_get_kind, (gd_spawn s));
GD_API(void, gd_spawn_set_kind, (gd_spawn s, gd_i32 kind)); /* pre hooks only */
GD_API(void, gd_spawn_get_pos, (gd_spawn s, gd_vec3* out));
GD_API(void, gd_spawn_set_pos, (gd_spawn s, const gd_vec3* in)); /* pre hooks only */

/* player */
GD_API(gd_i32, gd_player_stocks, (gd_i32 slot));
GD_API(gd_i32, gd_player_character, (gd_i32 slot));
GD_API(gd_i32, gd_player_present, (gd_i32 slot)); /* 1 if the slot is a fighter in this match */

/* stage and match */
GD_API(gd_i32, gd_stage_kind, (void));
GD_API(gd_i32, gd_match_frame, (void)); /* NHV1 frame column: scene frame counter minus its value at the first row */

/* randomness: ONLY through the game's RNG (HSD_Randi / HSD_Randf); advances and is part of the game state */
GD_API(gd_u32, gd_rand_u32, (gd_u32 bound)); /* [0, bound); bound 0 returns 0 without advancing */
GD_API(gd_f32, gd_rand_f32, (void));         /* [0, 1) */

/* order-sensitive / non-IEEE-exact math: implemented by the host with ONE implementation for both builds (the game's
 * own tables where it has them), so a mod never links libm. sqrt and abs are exact IEEE and are inline below. */
GD_API(gd_f32, gd_sinf, (gd_f32 x));
GD_API(gd_f32, gd_cosf, (gd_f32 x));
GD_API(gd_f32, gd_atan2f, (gd_f32 y, gd_f32 x));
GD_API(gd_f32, gd_powf, (gd_f32 x, gd_f32 y));

/* diagnostics: a host log line, never part of the game state; may be a no-op in a release build */
GD_API(void, gd_log, (const char* msg, gd_i32 value));

/* ---- inline deterministic helpers (no libc, no FMA, fixed evaluation order) ---------------------------------- */
static inline gd_f32 gd_fabsf(gd_f32 x) { return __builtin_fabsf(x); }
static inline gd_f32 gd_sqrtf(gd_f32 x) { return __builtin_sqrtf(x); } /* correctly rounded in IEEE; -fno-math-errno */
static inline gd_bool gd_isnan(gd_f32 x) { return x != x; }
static inline gd_f32 gd_minf(gd_f32 a, gd_f32 b) { return (b < a) ? b : a; }
static inline gd_f32 gd_maxf(gd_f32 a, gd_f32 b) { return (a < b) ? b : a; }
static inline gd_f32 gd_clampf(gd_f32 x, gd_f32 lo, gd_f32 hi) { return gd_minf(gd_maxf(x, lo), hi); }
/* a*b+c as two separately rounded operations (volatile-free: -ffp-contract=off forbids the fusion) */
static inline gd_f32 gd_mul_add(gd_f32 a, gd_f32 b, gd_f32 c) { gd_f32 p = a * b; return p + c; }
static inline gd_f32 gd_dot3(const gd_vec3* a, const gd_vec3* b) { gd_f32 xx = a->x * b->x; gd_f32 yy = a->y * b->y; gd_f32 zz = a->z * b->z; return (xx + yy) + zz; }
/* float -> int32 with a defined result for every input: truncate toward zero, saturate, NaN -> 0. The only allowed
 * float->int conversion (the C cast is undefined out of range; wasm's trapping trunc would abort the match). */
static inline gd_i32 gd_f2i(gd_f32 x) {
    if (x != x) return 0;
    if (x >= 2147483648.0f) return 2147483647;
    if (x <= -2147483648.0f) return (-2147483647 - 1);
    return (gd_i32)x;
}
/* integer division and modulo with a defined result (divisor 0 -> 0; INT_MIN / -1 -> INT_MIN) */
static inline gd_i32 gd_div(gd_i32 a, gd_i32 b) { if (b == 0) return 0; if (b == -1) return (gd_i32)(0u - (gd_u32)a); return a / b; }
static inline gd_i32 gd_mod(gd_i32 a, gd_i32 b) { if (b == 0 || b == -1) return 0; return a % b; }

/* ---- hooks -------------------------------------------------------------------------------------------------- */
enum gd_hook_mode { GD_PRE = 1, GD_POST = 2, GD_REPLACE = 3 };
enum gd_pre_result { GD_CONTINUE = 0, GD_SKIP = 1 }; /* pre-hook return: SKIP omits the original and lower-priority pre/replace */

/* The original call's arguments and result, for a hook that needs to call or replace it. Valid only inside a hook. */
GD_API(void, gd_orig_call_void, (void));
GD_API(gd_i32, gd_orig_call_i32, (void));
GD_API(gd_f32, gd_orig_call_f32, (void));
/* On GD_SKIP of a non-void function, the value the call returns (the last SKIPping hook's value wins). */
GD_API(void, gd_set_return_i32, (gd_i32 v));
GD_API(void, gd_set_return_f32, (gd_f32 v));

typedef void (*gd_fn)(void); /* real signature per hooks-v1.tsv row; the host calls it with exactly that signature */

typedef struct gd_hook_decl {
    gd_u32 address;  /* retail GALE01 address, the key (hooks-v1.tsv column 2) */
    gd_u32 mode;     /* enum gd_hook_mode */
    gd_i32 priority; /* -1000..1000, lower runs first for pre and replace selection, last for post; default 0 */
    gd_u32 flags;    /* reserved, 0 */
    gd_fn fn;        /* wasm: an index into the exported function table; native: a code pointer */
} gd_hook_decl;

enum { GD_FEATURE_NONE = 0u,
       GD_FEATURE_ITEM_WRITE = 1u << 0,   /* gd_item_write / gd_spawn_set_* */
       GD_FEATURE_ACTION_REQUEST = 1u << 1, /* gd_fighter_request_action_state */
       GD_FEATURE_REPLACE = 1u << 2,      /* uses GD_REPLACE */
       GD_FEATURE_LOG = 1u << 3 };

#define GD_MANIFEST_MAGIC 0x444D4447u /* 'GDMD' little-endian */
#define GD_MOD_ID_MAX 32u
#define GD_MOD_NAME_MAX 48u

typedef struct gd_manifest {
    gd_u32 magic;             /* GD_MANIFEST_MAGIC */
    gd_u32 struct_size;       /* sizeof(gd_manifest) */
    gd_u32 abi_min;           /* lowest host ABI this mod runs on  (GD_ABI_VERSION format) */
    gd_u32 abi_max;           /* highest host ABI this mod was tested on; same major required */
    gd_u32 hook_table_version;/* hooks-vN the addresses come from */
    gd_u32 mod_version;       /* (major << 16) | (minor << 8) | patch */
    gd_u32 required_features; /* GD_FEATURE_* the mod cannot run without */
    gd_u32 optional_features;
    gd_u32 state_bytes;       /* size of the mod's mutable state (.data + .bss), measured by the author; host verifies */
    gd_u32 hook_count;
    const gd_hook_decl* hooks;
    char mod_id[GD_MOD_ID_MAX];   /* [a-z0-9._-], NUL-terminated, globally unique, e.g. "gd.hello" */
    char mod_name[GD_MOD_NAME_MAX];
} gd_manifest;

/* Define the manifest. The hook list is built by GD_HOOK_* below; the manifest object is exported as `gd_mod_manifest`
 * (wasm: an exported global holding its address; native: a symbol the loader looks up). Use once per mod. */
#define GD_MOD_MANIFEST(id, name, version, abi_min_, abi_max_, req_feat, opt_feat, state_sz)                          \
    GD_EXPORT_DATA(gd_mod_manifest) const gd_manifest gd_mod_manifest = {                                                 \
        GD_MANIFEST_MAGIC, (gd_u32)sizeof(gd_manifest), (abi_min_), (abi_max_), GD_HOOK_TABLE_VERSION, (version),     \
        (req_feat), (opt_feat), (state_sz), (gd_u32)(sizeof(gd_hooks_) / sizeof(gd_hooks_[0])), gd_hooks_, id, name }

/* Hook declaration list: GD_HOOKS_BEGIN; GD_HOOK_POST(addr, prio, fn); ... GD_HOOKS_END; then GD_MOD_MANIFEST. */
#define GD_HOOKS_BEGIN static const gd_hook_decl gd_hooks_[] = {
#define GD_HOOKS_END };
#define GD_HOOK_DECL_(addr, m, prio, f) { (addr), (m), (prio), 0u, (gd_fn)(f) },
#define GD_HOOK_PRE(addr, prio, f) GD_HOOK_DECL_(addr, GD_PRE, prio, f)
#define GD_HOOK_POST(addr, prio, f) GD_HOOK_DECL_(addr, GD_POST, prio, f)
#define GD_HOOK_REPLACE(addr, prio, f) GD_HOOK_DECL_(addr, GD_REPLACE, prio, f)

/* The ABI version this mod was compiled against, exported so a loader can refuse before reading the manifest.
 * Defined here, so include gdmod.h from exactly one translation unit of a mod. */
GD_EXPORT_DATA(gdmod_abi_version) const gd_u32 gdmod_abi_version = GD_ABI_VERSION;

/* Named addresses for the hooks the examples use (full list: hooks-v1.tsv). A mod may use any 0x8xxxxxxx literal. */
#define GD_ADDR_FIGHTER_TAKE_DAMAGE 0x8006CC7Cu /* damage.fighter_take_damage  void(gd_fighter,f32) */
#define GD_ADDR_FIGHTER_PROC_UPDATE 0x8006B82Cu /* fighter.proc04              void(gd_fighter) */
#define GD_ADDR_FRAME_RUN_PROCS 0x80390CFCu     /* frame.run_procs             void() */

#endif /* GDMOD_H */
