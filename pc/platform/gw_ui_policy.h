/* gw_ui_policy.h - the Atlas scene policy table: how Atlas relates to each scene kind (spec 6.8). Pure C.
 * RETAIL: the scene is the game's own. OVERLAY: the retail scene runs unchanged and Atlas draws an engine screen over it.
 * REPLACE: Atlas stands in for the scene's on_enter/on_frame pair. Everything is RETAIL except the title (an OVERLAY),
 * and MELEE_ATLAS=0 makes everything RETAIL. MELEE_ATLAS_SCENES="<kind>:<retail|overlay|replace>,..." overrides rows for development. */
#ifndef GW_UI_POLICY_H
#define GW_UI_POLICY_H
#ifdef __cplusplus
extern "C" {
#endif

enum { AT_POLICY_RETAIL = 0, AT_POLICY_OVERLAY = 1, AT_POLICY_REPLACE = 2 };
typedef struct { int scene_kind; int policy; const char *screen; } AtPolicyRow;

/* the policy for a scene kind; env_override may be NULL. A word that is not retail, overlay or replace is ignored. */
int at_policy_for(int scene_kind, int atlas_on, const char *env_override);
/* the engine screen id the default table names for that scene kind, or NULL */
const char *at_policy_screen(int scene_kind);
/* 1 when entering this scene kind ends a mod's run: the title, the menus, the game over, the movies and the credits. A screen that says persist = true spans a run's
 * scenes (the ladder's VS-to-VS), so it is closed when one of these begins; every other scene (VS, results, the stage and character selects, the 1P intros and
 * cutscenes) is part of a run. An unknown kind (negative) is no run's end. */
int at_policy_ends_run(int scene_kind);

/* Step 10, the bespoke retail scenes (Trophy Gallery 11, Lottery 12, Collection 13: GameSceneKind, src/melee/gm/forward.h).
 * The retail 2D pieces (retail element ids, gw_ui_retail_ids.h) a scene hides while Atlas frames it: the scene's mask, and nothing
 * unless that scene's policy is OVERLAY (MELEE_ATLAS_SCENES) and Atlas is on. text_ok 0: the decoded retail text is not available, so the
 * retail text objects stay (a partial mask). Opaque plates cover the retail 2D that sits outside the window, so the masks only name
 * what sits inside it. The Lottery's machine and the Collection's room have no 2D piece (their masks are empty: chrome only). */
unsigned at_policy_mask(int scene_kind, int atlas_on, const char *env_override, int text_ok);
/* the chrome screen id of a scene that has one (a bespoke scene whatever its row, or the default table's), else NULL */
const char *at_policy_screen_for(int scene_kind);

#ifdef __cplusplus
}
#endif
#endif
