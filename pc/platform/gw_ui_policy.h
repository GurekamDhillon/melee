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

#ifdef __cplusplus
}
#endif
#endif
