/* gw_stageskin.h - stage skins: several enabled mods that replace the same retail stage file, and the
 * player's pick among them (Original, skin 1, skin 2, ...) without a restart.
 *
 * WHY: the file overlay (shim_dvd.c) used to keep ONE provider per disc path (the last mounted mod),
 * so a second skin for Pokemon Stadium either conflicted (Nucleus lists the skins of one stage as
 * mutual conflicts) or silently won by mount order. Now every provider of a stage's files is kept, the
 * stage-kind mods that are skins of the same retail stage no longer conflict, and the stage select
 * picks one per load. The model is in gw_stageskin.inc (compiled into gw_mods.c, which keeps the mods
 * registry's statics); the overlay's provider switch is in shim_dvd.c.
 *
 * VOCABULARY. A "retail stage" is a row of the retail table (external stage id, file stem). A "skin"
 * of it is a MOUNTED mod of kind "stage" that provides a file of that stage (GrPs.dat, GrPs.usd,
 * GrPs1.dat ... at the payload root). "Original" is the stage as it is without any skin: the last
 * mounted provider that is not a stage-kind mod, else the disc. A choice is an index: 0 Original,
 * k+1 the k-th skin (mount order).
 *
 * THE CHOICE. settings.cfg "skin_<external stage id>" = a mod id, "orig", or absent/"auto". Absent
 * keeps what the game did before the picker existed: the last mounted skin. MELEE_SKIN_<ext> in the
 * environment wins (tests). It is applied when the stage loads (Ground_801C0754 -> StageSkin_ApplyForLoad),
 * never in the middle of a match, so the heap and the rewind snapshots of a match see one file.
 *
 * ONLINE. The host's choice decides. The host sends ";skin=<12 hex digits of the skin's content digest>"
 * or ";skin=orig" in the match setup's extras; a guest applies exactly that (a skin it does not have: Original,
 * logged). The host sends a skin only when the guest announced it (identity entries of kind GW_MEXID_SKIN),
 * so both load the same file or both load Original. A match whose setup has no skin= (a public random
 * match, an older host) is played with Original on both sides. Replays record the identity.
 *
 * Game-callable functions (the retargeted game calls them WITHOUT the gw_ prefix) take scalars and
 * byte buffers only. */
#ifndef GW_STAGESKIN_H
#define GW_STAGESKIN_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define GW_SKIN_MAX 16        /* skins kept per retail stage */
#define GW_SKIN_RETAIL_MAX 32 /* rows of the retail table */

/* what the overlay is told to serve for a stage's files */
#define GW_SKIN_DEFAULT (-2)  /* mount order: the last mounted provider (the pre-picker behaviour) */
#define GW_SKIN_ORIGINAL (-1) /* the last non-skin provider, else the disc */

/* ---- the retail stage table (also gw_mexid.c's list of vanilla stages) ---- */
int gw_Skin_RetailCount(void);
int gw_Skin_RetailExt(int r);          /* external stage id of row r */
const char *gw_Skin_RetailFile(int r); /* the file stem as the table spells it ("GrPs", "GrIz.dat") */
const char *gw_Skin_RetailName(int r);
int gw_Skin_RetailForExt(int ext);     /* row of an external stage id, -1 when it is not a retail stage */
/* Row of the retail stage a disc path belongs to: a root-level file named <stem>[digits].<ext>. -1 otherwise. */
int gw_Skin_FileRetail(const char *disc_path);
/* Bit r set for every retail stage a payload folder's root files belong to. */
uint32_t gw_Skin_PayloadMask(const char *payload_dir);

/* ---- pure model (tested natively) ---- */
/* The choice a stored setting means among skins with these ids (mount order): 0 Original, 1..n a skin.
 * NULL, "" and "auto" mean the last mounted skin (n; 0 when there are none); "orig" Original; an id that
 * is not installed falls back to auto. */
int gw_Skin_Resolve(const char *setting, const char *const *ids, int n);
/* Cycle a choice through 0..n (n+1 entries) by dir (+1 / -1), wrapping. */
int gw_Skin_Step(int cur, int n, int dir);
/* One provider of a disc path, in mount order. */
typedef struct gw_skin_prov {
    int mod_index; /* gw_Mods_* index */
    int is_skin;   /* the mod is of kind "stage" */
} gw_skin_prov;
/* Which provider serves a path for `want` (GW_SKIN_DEFAULT, GW_SKIN_ORIGINAL or a gw_Mods_* index of a skin):
 * its index in prov[], or -1 for the disc. on_disc: the disc has the path too. A path nothing but skins
 * provide (no disc copy) keeps the default provider whatever is asked. */
int gw_skin_select(const gw_skin_prov *prov, int nprov, int on_disc, int want);

/* ---- installed skins and the choice ---- */
int gw_StageSkin_Count(int ext);               /* skins of a retail stage (not counting Original); 0 for any other */
const char *gw_StageSkin_Id(int ext, int k);   /* the mod id of skin k (0-based) */
const char *gw_StageSkin_Name(int ext, int k);
uint64_t gw_StageSkin_Digest(int ext, int k);  /* content digest of the skin's stage files */
int gw_StageSkin_Choice(int ext);              /* the local choice now: 0 Original, k+1 */
int gw_StageSkin_Cycle(int ext, int dir);      /* step the choice (+1 next, -1 previous), store it, return it */
/* The SSS label for this stage: copies the chosen name ("Original" or the skin's name) to out; returns the number of
 * skins (0 = no picker for this stage, out empty). */
int gw_StageSkin_Label(int ext, char *out, int cap);

/* ---- applying ---- */
/* At a stage load (grkind = the internal id): serve the files of that stage from the right provider. The match's
 * override (scene skin=) wins, then Original for an online match without one, then the local choice. */
void gw_StageSkin_ApplyForLoad(int grkind);
/* The scene key skin=<value>: "orig", 12+ hex digits of a content digest, or a mod id. 0 = understood. */
int gw_StageSkin_SceneSet(const char *value);
/* An online match is about to launch: a scene without skin= means Original. */
void gw_StageSkin_NetBegin(void);
/* What the last load served, for replays: the external stage id and the name ("" = Original / none) and digest (0 = none). */
int gw_StageSkin_Last(int *ext, char *name, int cap, uint64_t *digest);
/* A replay's recorded skin, applied on the next load: digest of the skin ("" = Original). Same as SceneSet. */
/* The host's extras for the scene: ";skin=<12 hex>" or ";skin=orig" into out; peer_has(ext, digest) tells whether the guest
 * announced it. Returns 1 when the guest lacks the host's choice (Original is sent instead). */
int gw_StageSkin_HostToken(int ext, int (*peer_has)(int ext, uint64_t digest), char *out, int cap);

void gw_stageskin_tests_register(void);

#ifdef __cplusplus
}
#endif
#endif /* GW_STAGESKIN_H */
