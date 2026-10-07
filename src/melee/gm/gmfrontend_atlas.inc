/* gmfrontend_atlas.inc - the Atlas adapter for the menu tree (spec 6.4, 6.7; plan 2026-10-06-atlas-step2).
 * The tree, the cursor, confirm, back and every pending action stay the legacy ones; this file only describes the current
 * FeMenu to the host each frame (the Ui_* shims, pc/platform/gw_script_ui.inc) and turns the host's events into fm_go,
 * fm_confirm and fm_back. MELEE_ATLAS=0 (Ui_Ready() == 0) leaves the legacy menus untouched. */

/* The scene policy's own table (pc/platform/gw_ui_policy.c) names scene kinds by number: the title is 0. */
_Static_assert(GS_TITLE == 0, "gw_ui_policy.c AT_SCENE_TITLE must be GS_TITLE");

/* REPLACE has no user in step 2 (the title is an OVERLAY: retail's own exit logic, attract loop and debug path keep running).
 * This is the extension point: a scene kind that Atlas stands in for returns its own on_enter/on_frame pair here, step 8 first. */
GameScene* gmFrontend_AtlasStandIn(u8 kind)
{
    (void) kind;
    return NULL;
}
