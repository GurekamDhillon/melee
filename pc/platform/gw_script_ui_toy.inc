/* gw_script_ui_toy.inc - the host half of the framed trophy scenes (Trophy Gallery 11, Lottery 12, Collection 13; spec 13.10).
 * Included at the end of gw_script_ui.inc. The retail scene runs unchanged; the game side (src/melee/gm/gmfrontend_atlas_toy.inc) wraps the scene's
 * on_frame and, AFTER retail's own frame, submits one framed screen per frame through these shims (game code calls them as Ui_*):
 *   FrameBegin -> Trail -> a chrome shim (the explainer and the counter) -> Key -> Commit.
 * A framed screen is presentation only: no focus, no hit rectangles, no events (gs_ui_engine_event returns for it, input_feed is 1), so retail's own
 * pad polling is the only reader. It is never drawn online (the game side checks Ui_NetplayActive; the mask is 0 online as well).
 *
 * THE MASK IS SET BEFORE THE SCENE'S on_enter. Script_SceneBegin (and so gw_Ui_SceneBegin) runs in gm_801A4D34, AFTER on_enter, so a creation-time guard
 * would read an empty mask. gm_801A4014 therefore calls Ui_ScenePolicyMask(kind) first, for every scene: it sets (or clears) the scene-policy source
 * of the retail mask, so the next scene's call replaces the last one and nothing is left over. */
#include "gw_ui_frame.h"
#include "gw_ui_toy.h"

/* MELEE_ATLAS_FRAME_WIN="x,y,w,h" overrides every framed scene's window (tuning, in the 640x480 retail canvas); MELEE_ATLAS_FRAME_OUTLINE=1 draws its
 * edge in the ember colour so it can be compared with where retail puts the model by looking. Both read once. */
static int gs_ui_frame_env_done, gs_ui_frame_win_set, gs_ui_frame_outline;
static float gs_ui_frame_win[4];

static void gs_ui_frame_env(void) {
    const char *e;
    if (gs_ui_frame_env_done) return;
    gs_ui_frame_env_done = 1;
    e = getenv("MELEE_ATLAS_FRAME_OUTLINE");
    gs_ui_frame_outline = e != NULL && strcmp(e, "1") == 0;
    e = getenv("MELEE_ATLAS_FRAME_WIN");
    if (e != NULL && e[0] != '\0') {
        float v[4];
        int n = sscanf(e, "%f,%f,%f,%f", &v[0], &v[1], &v[2], &v[3]);
        if (n == 4 && v[0] >= 0.0f && v[1] >= 0.0f && v[2] > 0.0f && v[3] > 0.0f && v[0] + v[2] <= 640.0f && v[1] + v[3] <= 480.0f) {
            memcpy(gs_ui_frame_win, v, sizeof v);
            gs_ui_frame_win_set = 1;
            gw_log("ui: framed scenes use the window %.0f,%.0f,%.0f,%.0f (MELEE_ATLAS_FRAME_WIN)", v[0], v[1], v[2], v[3]);
        } else gw_log("ui: MELEE_ATLAS_FRAME_WIN ignored: want x,y,w,h inside 640x480");
    }
    if (gs_ui_frame_outline) gw_log("ui: framed scenes draw their window's edge in ember (MELEE_ATLAS_FRAME_OUTLINE)");
}

/* Step 8's gate decides whether the retail text of the trophy scenes can be shown (the decoder reads the glyph stream back to UTF-8). It has not
 * been run in this tree, so the retail text objects stay visible and the explainer uses authored fallbacks. 0 means no. */
#define GS_UI_TOY_TEXT_OK 0

/* window_known: the game side has a measured window for this scene (or MELEE_ATLAS_FRAME_WIN gives one). Without a window the chrome does not draw
 * (gw_Ui_FrameBegin refuses), so nothing is hidden for it either: the mask stays empty and the scene is exactly retail. */
void gw_Ui_ScenePolicyMask(int scene_kind, int window_known) {
    unsigned m;
    gs_ui_frame_env();
    m = (window_known || gs_ui_frame_win_set) ? at_policy_mask(scene_kind, atlas_env_on(), getenv("MELEE_ATLAS_SCENES"), GS_UI_TOY_TEXT_OK) : 0u;
    unsigned was = gs_ui_retail.src[AT_RS_POLICY];
    at_retail_set(&gs_ui_retail, AT_RS_POLICY, m);
    if (m != was) gw_log("ui: scene %d: the policy hides retail elements %u (was %u)", scene_kind, m, was);
}

/* gw_Ui_NetplayActive (nonzero while netplay or rollback is on: a framed scene is not drawn then) is defined once, in gw_script_ui_data.inc (same test) */

/* the text gate for a scene kind: 0 until step 8's probe has recorded GO for SdToy and SdToyExp */
int gw_Ui_ToyTextOk(int scene_kind) { (void) scene_kind; return GS_UI_TOY_TEXT_OK; }

/* MELEE_ATLAS_TOYPROBE=1: the game side logs numbers about a trophy scene at entry (never text) */
int gw_Ui_ToyProbe(void) {
    static int on = -1;
    if (on < 0) { const char *e = getenv("MELEE_ATLAS_TOYPROBE"); on = e != NULL && strcmp(e, "1") == 0; }
    return on;
}

/* Begin a framed screen. The window (x, y, w, h in the 640x480 retail canvas; w <= 0 means unmeasured: nothing is framed) is where retail's camera
 * frames its model. 0 when Atlas is off or its roles are missing, in which case the game side submits nothing. */
int gw_Ui_FrameBegin(const char *id, const char *title, int x, int y, int w, int h) {
    gs_ui_frame_env();
    if (!(w > 0 && h > 0) && !gs_ui_frame_win_set) {                 /* unmeasured: draw nothing rather than guess (a whole-canvas plate would blank the scene) */
        static char noted[4][AT_ID * 2];
        int i;
        if (id == NULL) return 0;
        for (i = 0; i < 4 && noted[i][0] != '\0'; i++) if (strcmp(noted[i], id) == 0) return 0;
        if (i < 4) { snprintf(noted[i], sizeof noted[i], "%s", id); gw_log("ui: framed scene %s has no window yet: nothing is drawn (MELEE_ATLAS_FRAME_WIN=x,y,w,h tunes one)", id); }
        return 0;
    }
    if (!gw_Ui_Begin(id, AT_PRIMARY_FRAME, 0, title)) return 0;
    gs_ui_scratch.preset = AT_PRESET_NARROW;
    gs_ui_scratch.input_feed = 1;                                   /* Atlas reads no pad here: retail does */
    gs_ui_scratch.has_frame = 1;
    gs_ui_scratch.frame_x = (float) x; gs_ui_scratch.frame_y = (float) y; gs_ui_scratch.frame_w = (float) w; gs_ui_scratch.frame_h = (float) h;
    if (gs_ui_frame_win_set) {
        gs_ui_scratch.has_frame = 1;
        gs_ui_scratch.frame_x = gs_ui_frame_win[0]; gs_ui_scratch.frame_y = gs_ui_frame_win[1];
        gs_ui_scratch.frame_w = gs_ui_frame_win[2]; gs_ui_scratch.frame_h = gs_ui_frame_win[3];
    }
    gs_ui_scratch.frame_outline = gs_ui_frame_outline;
    return 1;
}

static void gs_ui_frame_explain(const char *kicker, const char *title, const char *what, const char *from, const char *counter) {
    gs_ui_scratch_ex.has = 1;
    gs_ui_scratch_ex.no_well = 1;                                   /* the trophy itself is the picture: no media well */
    snprintf(gs_ui_scratch_ex.kicker, sizeof gs_ui_scratch_ex.kicker, "%s", kicker);
    snprintf(gs_ui_scratch_ex.title, sizeof gs_ui_scratch_ex.title, "%s", title);
    snprintf(gs_ui_scratch_ex.what, sizeof gs_ui_scratch_ex.what, "%s", what);
    snprintf(gs_ui_scratch_ex.from_text, sizeof gs_ui_scratch_ex.from_text, "%s", from);
    snprintf(gs_ui_scratch.counter, sizeof gs_ui_scratch.counter, "%s", counter);
}

/* the Gallery's chrome: the numbers are retail's (the selected index and the count); name, description and series are decoded text, or empty */
void gw_Ui_ToyChrome(int index, int count, const char *name, const char *desc, const char *series) {
    AtToyChrome c;
    if (!gs_ui_scratch_live) return;
    at_toy_chrome(&c, index, count, name, desc, series);
    gs_ui_frame_explain("TROPHY", c.title, c.what, c.from, c.counter);
}

/* the Lottery's chrome: the coin balance (gm_801623D8() / 10) and the bet retail has chosen */
void gw_Ui_LotChrome(int coins, int bet) {
    AtLotChrome c;
    if (!gs_ui_scratch_live) return;
    at_lot_chrome(&c, coins, bet);
    gs_ui_frame_explain("LOTTERY", c.title, c.what, "", c.counter);
}

/* the Collection's chrome: how many trophies the room holds */
void gw_Ui_CollChrome(int total) {
    AtCollChrome c;
    if (!gs_ui_scratch_live) return;
    at_coll_chrome(&c, total);
    gs_ui_frame_explain("COLLECTION", c.title, c.what, "", c.counter);
}
