/* gw_mods.h - the mods folder: which mods exist, which are enabled, the order they mount in.
 *
 * LAYOUT (next to melee-pc.exe, or MELEE_MODS_DIR):
 *
 *   mods/
 *     enabled.txt            the enabled set, one mod id per line ('#' comments). ABSENT = every
 *                            mod is enabled (the pre-D1 behaviour). Read once at boot; the toggle
 *                            API below rewrites it for the NEXT boot.
 *     <id>/                  one mod; <id> is the folder name and is the mod's identity
 *       mod.json             optional metadata (below). Never a disc file.
 *       files/               the payload: files/PlWf.dat answers disc path /PlWf.dat,
 *                            files/audio/us/wolf.ssm answers /audio/us/wolf.ssm
 *       (no files/ folder)   then the mod folder itself is the payload root (legacy layout)
 *     targettest/            not a mod (Target Test layouts) - skipped
 *
 * mod.json is a FLAT object; every field is optional:
 *   { "id": "ace-wolf", "name": "Wolf", "version": "2.0.0", "kind": "fighter",
 *     "pack": "ace", "description": "...", "requires": ["ace-base"], "conflicts": [],
 *     "hash": "<content digest written by the packer>" }
 * kind is one of base | fighter | stage | misc | script. A "script" mod carries only Lua scripts
 * (<id>/scripts/*.lua, run by gw_script.c; any kind of mod may also ship a scripts/ folder, which
 * is never mounted as disc files). At most ONE "base" mod mounts: a base carries
 * MxDt.dat (m-ex's content tables), and there is exactly one of those per boot.
 *
 * RESOLUTION at boot: start from the enabled set; drop a mod whose "requires" are not all
 * mounting (MISSING_DEP); drop a mod that conflicts with one already kept (CONFLICT: an explicit
 * "conflicts" entry either way, or a second base); repeat until stable. Mount order: requirements
 * first, then base < misc < fighter/stage, then id. A later mod wins a disc path both provide.
 *
 * All ints and const char* - no structs - so gwtool-compiled game code (the in-game mods menu)
 * can call these directly: declare them WITHOUT the gw_ prefix, e.g. `extern int Mods_Count(void);`
 * Strings are owned here and stay valid until the next boot. Index i is 0..Mods_Count()-1, in id
 * order, and is stable for the whole session.
 */
#ifndef GW_MODS_H
#define GW_MODS_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define GW_MODS_MAX 256

/* gw_Mods_Status values */
enum {
    GW_MOD_ACTIVE = 0,      /* mounted this boot */
    GW_MOD_OFF = 1,         /* not in the enabled set (or MELEE_MODS=0) */
    GW_MOD_MISSING_DEP = 2, /* enabled, but something it requires is not mounting */
    GW_MOD_CONFLICT = 3     /* enabled, but it conflicts with a mod that is mounting */
};

/* ---- the toggle-menu API ------------------------------------------------------------------ */

int gw_Mods_Count(void);                 /* mods found in the folder, enabled or not */
const char *gw_Mods_Id(int i);           /* folder name; "" when i is out of range */
const char *gw_Mods_Name(int i);         /* mod.json name, else the id */
const char *gw_Mods_Version(int i);      /* "" when unknown */
const char *gw_Mods_Kind(int i);         /* "base" | "fighter" | "stage" | "misc" | "script" */
const char *gw_Mods_Pack(int i);         /* e.g. "ace", "akaneia"; "" when unknown */
const char *gw_Mods_Description(int i);  /* "" when none */
const char *gw_Mods_Requires(int i);     /* comma-separated ids, "" when none */
int gw_Mods_Find(const char *id);        /* index of `id` (case-insensitive), or -1 */

int gw_Mods_IsActive(int i);             /* 1 = mounted THIS boot */
int gw_Mods_Status(int i);               /* GW_MOD_* for this boot */
const char *gw_Mods_StatusText(int i);   /* short human text for the status, e.g. "needs ace-base" */

int gw_Mods_IsEnabled(int i);            /* 1 = in the enabled set for the NEXT boot */
/* Enable/disable mod i for the next boot. Enabling also enables what it requires and disables
 * enabled mods that conflict with it (and their dependents); disabling also disables every
 * enabled mod that requires it. Returns how many mods changed state (0 = nothing changed or bad
 * index). Nothing is written until gw_Mods_Save. */
int gw_Mods_SetEnabled(int i, int on);
int gw_Mods_Save(void);                  /* write mods/enabled.txt; 0 = ok, -1 = failed */
int gw_Mods_RestartNeeded(void);         /* 1 = the next-boot set differs from what is mounted */
const char *gw_Mods_Dir(void);           /* the mods folder in use ("" when none could be found) */

/* ---- the file layer (shim_dvd.c) ----------------------------------------------------------- */

int gw_Mods_ActiveCount(void);           /* mounting mods */
int gw_Mods_ActiveAt(int n);             /* index of the n-th mounting mod, in mount order */
const char *gw_Mods_PayloadDir(int i);   /* host folder whose contents are disc paths */
int gw_Mods_PayloadIsModDir(int i);      /* 1 = legacy layout: skip mod.json at its top level */
/* The overlay reports every disc path it finally mounted, with the mod that won it, so the
 * netplay fingerprint covers the exact files the game will read. Order-independent. */
void gw_Mods_NoteFile(int i, const char *disc_path, uint32_t size);

/* ---- the mounted set, summarised -----------------------------------------------------------
 * Netplay does NOT require equal mod sets: it matches fighters and stages one by one by content
 * identity and refuses only on different global game data (gw_mexid.h). These describe the set
 * for logs and UI ("the other player runs ..."). */

/* 64-bit digest of the MOUNTED set: every mounting mod's id, version, mod.json hash and every
 * disc path it provides (with size). 0 when no mod is mounted, so vanilla == vanilla. */
uint64_t gw_Mods_Fingerprint(void);
/* The mounted set as "id#hhhh,id#hhhh" (sorted by id; hhhh = 16 bits of that mod's own digest),
 * "" when none. Short enough to travel in a refusal message for a handful of mods. */
const char *gw_Mods_Describe(void);
/* Compare a peer's gw_Mods_Describe string with ours and write a human-readable difference into
 * out ("you lack X; peer lacks Y; files differ: Z"). A truncated peer list (ending in "...")
 * suppresses "peer lacks". Returns the number of differing mods (0 = same). */
int gw_Mods_DiffDescribe(const char *peer_desc, char *out, int cap);

void gw_mods_tests_register(void);

#ifdef __cplusplus
}
#endif
#endif /* GW_MODS_H */
