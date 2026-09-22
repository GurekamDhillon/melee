#include "gmfrontend.h"

#if defined(TARGET_PC)

#include "gm_1A3F.h"
#include "gmmain_lib.h"
#include "gmscene.h"
#include <dolphin/gx.h>
#include <dolphin/os.h>
#include <melee/gm/gm_1601.h>
#include <melee/gm/gm_16F1.h>
#include <melee/lb/lbaudio_ax.h>
#include <melee/lb/lbcardgame.h>
#include <melee/lb/lbcardnew.h>
#include <melee/lb/lbdvd.h>
#include <melee/mn/forward.h>
#include <melee/mn/inlines.h>
#include <melee/mn/mnmain.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/gobjgxlink.h>
#include <sysdolphin/baselib/hsd_3915.h>
#include <sysdolphin/baselib/memory.h>
#include <sysdolphin/baselib/sislib.h>
#include <sysdolphin/baselib/state.h>

#include <stdio.h>
#include <string.h>

/* ---- The port's frontend: screens placed anywhere in Melee's menu flow ------------------------
 *
 * HOW MELEE'S FLOW WORKS. A game mode ("major scene": the main menu, VS mode, training...) is a
 * table of states ("minor scenes": CSS, SSS, the match, results), each naming a scene kind whose
 * GameScene supplies on_enter / on_frame / on_exit. The top-level loop (gm_801A4510) runs one mode
 * until it asks for another, then runs that one.
 *
 * HOW A SCREEN IS PLACED. A frontend screen is its own mode, GM_FRONTEND, with one state and one
 * scene, GS_FRONTEND - built exactly like a native screen, so it gets the engine's own per-scene
 * setup, input, audio and transitions. gmFrontend_Route runs on every mode change: when a rule
 * says "between FROM and TO, show SCREEN", the loop enters GM_FRONTEND instead of TO. Confirming
 * continues to TO, exactly as the game intended; backing out returns to FROM. While a screen is up
 * it reports TO as the previous mode, so a native screen that positions itself by where the player
 * came from (the main menu's cursor) behaves exactly as if the player had backed out of TO.
 *
 * THE TOOLKIT. A screen is a data table of items - actions, choices, sliders and toggles - each
 * reading and writing real game data through get/set functions, with optional visibility (a row
 * that only makes sense in one mode) and a help line. Rows scroll when a screen has more than fit.
 * The screen fades in and out, rows slide in one after another, and the highlight glides between
 * rows.
 *
 * DRAWING. Melee's own systems: the text canvas (HSD_SisLib - the menus' font, with its own
 * orthographic 640x480 camera) and GX link callbacks on that camera: the panels below the text
 * (pass 0), the fade above it (pass 2, after the text, which draws in pass 2 at a lower priority).
 * Everything renders through aurora at the window's resolution. */

/* ---- the toolkit's data ------------------------------------------------------------------- */

typedef enum FrontendItemKind {
    FE_ACTION,
    FE_CHOICE,
    FE_SLIDER,
    FE_TOGGLE,
} FrontendItemKind;

typedef enum FrontendAction {
    FE_DO_CONTINUE, ///< go on to the mode the rule interrupted
    FE_DO_BACK,     ///< return to the mode the player came from
} FrontendAction;

typedef struct FrontendItem {
    u8 kind;   ///< ::FrontendItemKind
    u8 action; ///< ::FrontendAction, for FE_ACTION
    const char* label;
    const char* help;
    int (*get)(void);
    void (*set)(int);
    int min, max, step;
    const char* const* options;         ///< FE_CHOICE labels, index = value - min
    void (*format)(int value, char* out); ///< FE_SLIDER text; default "%d"
    int (*visible)(void);               ///< NULL = always shown
} FrontendItem;

typedef struct FrontendScreen {
    const char* title;
    const char* subtitle;
    const FrontendItem* items;
    int n_items;
} FrontendScreen;

typedef struct FrontendRule {
    u8 from;
    u8 to;
    const FrontendScreen* screen;
} FrontendRule;

/* ---- MATCH SETUP: the VS rules, edited in place ------------------------------------------------
 * The same GameRules / GamePrefs the native Rules menu writes (mnmainrule.c). VS mode copies them
 * into the match only when it starts (gmVsMelee_EnterVs -> gm_80167BC8), after the CSS and the
 * SSS, so whatever this screen leaves is what the next match plays. Encodings are the game's own:
 * damage ratio in tenths, time in minutes (0 = none; stock mode has its own limit), item
 * frequency as a signed byte with -1 = none. */

static const char* const fe_opt_mode[] = { "Time", "Stock", "Coin", "Bonus" };
static const char* const fe_opt_handicap[] = { "Off", "Auto", "On" };
static const char* const fe_opt_stage_sel[] = { "On", "Random", "Ordered", "Turns", "Loser" };
static const char* const fe_opt_items[] = { "None", "Very Low", "Low", "Medium", "High",
                                            "Very High" };

static int fe_get_mode(void) { return gmMainLib_GetGameRules()->mode; }
static void fe_set_mode(int v) { gmMainLib_GetGameRules()->mode = (u8) v; }
static int fe_is_stock(void) { return gmMainLib_GetGameRules()->mode == 1; }
static int fe_get_stocks(void) { return gmMainLib_GetGameRules()->stock_count; }
static void fe_set_stocks(int v) { gmMainLib_GetGameRules()->stock_count = (u8) v; }
static int fe_get_time(void)
{
    GameRules* r = gmMainLib_GetGameRules();
    return r->mode == 1 ? r->stock_time_limit : r->time_limit;
}
static void fe_set_time(int v)
{
    GameRules* r = gmMainLib_GetGameRules();
    if (r->mode == 1) {
        r->stock_time_limit = (u8) v;
    } else {
        r->time_limit = (u8) v;
    }
}
static void fe_fmt_time(int v, char* out)
{
    if (v == 0) {
        sprintf(out, "None");
    } else {
        sprintf(out, "%d min", v);
    }
}
static int fe_get_handicap(void) { return gmMainLib_GetGameRules()->handicap; }
static void fe_set_handicap(int v) { gmMainLib_GetGameRules()->handicap = (u8) v; }
static int fe_get_ratio(void) { return gmMainLib_GetGameRules()->damage_ratio; }
static void fe_set_ratio(int v) { gmMainLib_GetGameRules()->damage_ratio = (u8) v; }
static void fe_fmt_ratio(int v, char* out) { sprintf(out, "%d.%dx", v / 10, v % 10); }
static int fe_get_stage_sel(void) { return gmMainLib_GetGameRules()->stage_sel; }
static void fe_set_stage_sel(int v) { gmMainLib_GetGameRules()->stage_sel = (u8) v; }
static int fe_get_items(void) { return (s8) gmMainLib_GetGamePrefs()->item_freq; }
static void fe_set_items(int v) { gmMainLib_GetGamePrefs()->item_freq = (u8) (s8) v; }
static int fe_get_ff(void) { return gmMainLib_GetGameRules()->friendly_fire & 1; }
static void fe_set_ff(int v)
{
    GameRules* r = gmMainLib_GetGameRules();
    r->friendly_fire = (u8) ((r->friendly_fire & ~1) | (v & 1));
}
static int fe_get_pause(void) { return gmMainLib_GetGameRules()->pause != 0; }
static void fe_set_pause(int v) { gmMainLib_GetGameRules()->pause = (u8) (v != 0); }

static const FrontendItem fe_items_vs_setup[] = {
    { FE_ACTION, FE_DO_CONTINUE, "Continue to Character Select",
      "Pick fighters with these rules." },
    { FE_CHOICE, 0, "Mode", "How a match is won.", fe_get_mode, fe_set_mode, 0, 3, 1,
      fe_opt_mode },
    { FE_SLIDER, 0, "Stocks", "Lives each player starts with.", fe_get_stocks, fe_set_stocks, 1,
      99, 1, NULL, NULL, fe_is_stock },
    { FE_SLIDER, 0, "Time Limit", "0 plays without a clock.", fe_get_time, fe_set_time, 0, 99,
      1, NULL, fe_fmt_time },
    { FE_CHOICE, 0, "Items", "How often items appear.", fe_get_items, fe_set_items, -1, 4, 1,
      fe_opt_items },
    { FE_SLIDER, 0, "Damage Ratio", "How far hits send fighters.", fe_get_ratio, fe_set_ratio, 5,
      20, 1, NULL, fe_fmt_ratio },
    { FE_CHOICE, 0, "Handicap", "Auto evens out winners and losers.", fe_get_handicap,
      fe_set_handicap, 0, 2, 1, fe_opt_handicap },
    { FE_CHOICE, 0, "Stage Select", "How the stage is chosen.", fe_get_stage_sel,
      fe_set_stage_sel, 0, 4, 1, fe_opt_stage_sel },
    { FE_TOGGLE, 0, "Friendly Fire", "Team attacks hit teammates.", fe_get_ff, fe_set_ff, 0, 1,
      1 },
    { FE_TOGGLE, 0, "Pause", "Allow pausing during the match.", fe_get_pause, fe_set_pause, 0, 1,
      1 },
};

static const FrontendScreen fe_screen_vs_setup = {
    "VS. MELEE",
    "MATCH SETUP",
    fe_items_vs_setup,
    sizeof fe_items_vs_setup / sizeof fe_items_vs_setup[0],
};

/* THE LOADING SCREEN. No items: a title, a progress bar and a status line. Aurora compiles a
 * render pipeline the first time a draw needs one and skips that draw until it is ready, so a
 * match on cold pipelines arrives piece by piece. It also warms every pipeline in its seed
 * (initial_pipeline_cache.db beside the exe, built from a full sweep) in the background, and
 * counts that work as pending. So this screen holds until nothing is pending - with a minimum
 * so it never flashes, and a ceiling so a stuck queue cannot trap the player - and the match
 * then enters with its pipelines built. */
static const FrontendScreen fe_screen_loading = {
    "GET READY",
    "LOADING",
    NULL,
    0,
};

#define FE_LOAD_MIN_FRAMES 45
#define FE_LOAD_SETTLE_FRAMES 20
#define FE_LOAD_CEILING_FRAMES (8 * 60)

/* WHERE SCREENS GO. One line per placement: Title > VS. Mode > Melee enters GM_VS from GM_MENU,
 * and GM_VS's first state is the CSS - so this screen sits between "Melee" and the CSS. */
static const FrontendRule fe_rules[] = {
    { GM_MENU, GM_VS, &fe_screen_vs_setup },
};

/* ---- state ---------------------------------------------------------------------------------- */

#define FE_MAX_ROWS 7 ///< row slots on screen; longer screens scroll
#define FE_MAX_ITEMS 24
#define FE_STR 48 ///< longest string a row, value or help line shows

static struct {
    const FrontendScreen* screen;
    u8 continue_to;
    u8 back_to;
    int canvas;
    HSD_Text* title;
    HSD_Text* subtitle;
    HSD_Text* help;
    HSD_Text* hints;
    HSD_Text* hint_change;
    HSD_Text* hint_back;
    HSD_Text* label[FE_MAX_ROWS];
    HSD_Text* value[FE_MAX_ROWS];
    char label_str[FE_MAX_ROWS][FE_STR]; ///< what each text currently shows
    char value_str[FE_MAX_ROWS][FE_STR];
    char help_str[FE_STR];
    int vis[FE_MAX_ITEMS]; ///< indices of the visible items: the list, then the button
    int n_vis;
    int n_list;      ///< how many of vis are list rows; vis[n_list] is the button, if any
    bool has_button; ///< the screen's continue action is drawn as the CONTINUE button
    int cursor;  ///< index into vis
    int scroll;  ///< first visible row
    int frames;
    int fade;    ///< 0 = clear .. FE_FADE_FRAMES = black
    int leaving; ///< 0 still here, 1 continue, 2 back
    float hl_y;  ///< the highlight's drawn row position, easing toward the cursor
    int help_item;
    bool loading;    ///< this scene is the loading screen, a state inside a mode
    bool warmed;     ///< the loading screen has run; the in-match hold can stand down
    int load_base;   ///< pipelines already created when the loading screen began
    int load_core;   ///< pipelines built since boot that mean the seed's core is warm
    int load_settled; ///< frames with nothing pending
    float progress;  ///< 0..1, drawn by the bar
    int percent;     ///< what the status line shows
    u8 reported;     ///< the previous mode the loop records when this scene leaves
    bool next_menus; ///< the next GS_FRONTEND scene is the menu tree (gmfrontend_menus.inc)
} fe;

void gmFrontend_BeginLoading(void)
{
    fe.screen = &fe_screen_loading;
    fe.loading = true;
    fe.next_menus = false;
}

bool gmFrontend_TakeWarmed(void)
{
    bool w = fe.warmed;
    fe.warmed = false;
    return w;
}

static int fe_enabled(void)
{
    static int enabled = -1;
    if (enabled < 0) {
        extern int PcFrontendEnabled(void);
        enabled = PcFrontendEnabled();
    }
    return enabled;
}

/* The menu tree replaces GM_MENU's list screens: MELEE_FRONTEND_MENUS (default on). */
static int fe_menus_on(void)
{
    static int on = -1;
    if (on < 0) {
        extern int PcFrontendMenusEnabled(void);
        on = fe_enabled() && PcFrontendMenusEnabled();
    }
    return on;
}

static int fe_rules_enabled(void)
{
    return fe_enabled();
}

static void fm_route_to_menus(u8 kind, u8 sel);
static bool fm_route_native(void);
static bool fm_route_position(u8 previous, u8* kind, u8* sel);

u8 gmFrontend_Route(u8 from, u8 to)
{
    int i;
    if (!fe_enabled()) {
        return to;
    }
    if (fe_menus_on()) {
        if (to == GM_MENU) {
            u8 kind, sel;
            if (fm_route_native()) {
                return to; /* the frontend asked GM_MENU for one of its native screens */
            }
            /* wherever GM_MENU would open, the frontend opens instead - unless that is a screen
               it does not draw (the Event list after an event), which stays native */
            if (fm_route_position(from == GM_FRONTEND ? fe.reported : from, &kind, &sel)) {
                fm_route_to_menus(kind, sel);
                OSReport("frontend: mode %d -> GM_MENU becomes the menus (kind %d, sel %d)\n",
                         from, kind, sel);
                return GM_FRONTEND;
            }
            return to;
        }
        if (to == GM_FRONTEND) {
            return to; /* a native screen backed out to a menu of ours, or MATCH SETUP */
        }
    }
    if (from == GM_FRONTEND) {
        return to; /* leaving a screen goes exactly where it chose */
    }
    for (i = 0; i < (int) (sizeof fe_rules / sizeof fe_rules[0]); i++) {
        if (fe_rules[i].from == from && fe_rules[i].to == to) {
            fe.screen = fe_rules[i].screen;
            fe.continue_to = to;
            fe.back_to = from;
            fe.reported = to;
            fe.next_menus = false;
            OSReport("frontend: mode %d -> %d, showing \"%s\" first\n", from, to,
                     fe.screen->subtitle);
            return GM_FRONTEND;
        }
    }
    return to;
}

u8 gmFrontend_ReportedMode(void)
{
    return fe.reported;
}

/* ---- layout and drawing -------------------------------------------------------------------- */

#define FE_GX_LINK 14
#define FE_W 640.0F
#define FE_H 480.0F
#define FE_FRAME_M 8.0F ///< the 9-slice frame's inset from the screen edge
#define FE_PANEL_X 28.0F
#define FE_PANEL_Y 68.0F
#define FE_PANEL_W 584.0F
#define FE_PANEL_H 320.0F
#define FE_ROW_X 64.0F
#define FE_ROW_Y 114.0F
#define FE_ROW_W 512.0F ///< row_ng/row_sel at 1x; the plate is the top 28px, a shadow the rest
#define FE_ROW_H 32.0F
#define FE_ROW_STEP 34.0F
#define FE_VALUE_X (FE_ROW_X + FE_ROW_W - 40.0F) ///< right edge of the value column
#define FE_BTN_W 204.0F ///< btn_continue at 0.8 of its 1x size
#define FE_BTN_H 51.0F
#define FE_BTN_X (FE_PANEL_X + FE_PANEL_W - FE_BTN_W - 18.0F)
#define FE_BTN_Y (FE_PANEL_Y + FE_PANEL_H - 26.0F)
#define FE_FADE_FRAMES 10
#define FE_HINT_Y 424.0F
#define FE_HINT_A_X 44.0F
#define FE_HINT_LR_X 196.0F
#define FE_HINT_B_X 372.0F
/* the art pack's palette (its style.css) */
#define FE_INK fe_rgba(10, 14, 24, 255)
#define FE_COBALT_DK fe_rgba(20, 38, 92, 255)
#define FE_COBALT fe_rgba(30, 58, 140, 255)
#define FE_COBALT_LT fe_rgba(47, 85, 184, 255)
#define FE_GOLD fe_rgba(240, 180, 41, 255)
#define FE_GOLD_LT fe_rgba(255, 215, 102, 255)
#define FE_BONE fe_rgba(242, 239, 228, 255)
#define FE_LABEL_COLOR FE_BONE
#define FE_VALUE_COLOR FE_GOLD_LT
#define FE_HELP_COLOR fe_rgba(150, 162, 184, 255)

static GXColor fe_rgba(u8 r, u8 g, u8 b, u8 a)
{
    GXColor c;
    c.r = r;
    c.g = g;
    c.b = b;
    c.a = a;
    return c;
}

/* A rectangle in 640x480 screen space, y down, with a vertical gradient. The canvas camera is
 * ortho with y running 0..-480, hence the negation. */
static void fe_rect(float x, float y, float w, float h, GXColor top, GXColor bottom)
{
    GXBegin(GX_QUADS, GX_VTXFMT0, 4);
    GXPosition2f32(x, -y);
    GXColor4u8(top.r, top.g, top.b, top.a);
    GXPosition2f32(x + w, -y);
    GXColor4u8(top.r, top.g, top.b, top.a);
    GXPosition2f32(x + w, -(y + h));
    GXColor4u8(bottom.r, bottom.g, bottom.b, bottom.a);
    GXPosition2f32(x, -(y + h));
    GXColor4u8(bottom.r, bottom.g, bottom.b, bottom.a);
}

static void fe_solid(float x, float y, float w, float h, GXColor c)
{
    fe_rect(x, y, w, h, c, c);
}

/* A small arrowhead pointing left (dir < 0) or right, centred on (x, y). A quad with two
 * coincident corners is a triangle. The SIS font has no '<' or '>', so choices draw theirs. */
static void fe_arrow(float x, float y, float size, int dir, GXColor c)
{
    float tip = x + (dir < 0 ? -size : size) * 0.5F;
    float base = x - (dir < 0 ? -size : size) * 0.5F;
    GXBegin(GX_QUADS, GX_VTXFMT0, 4);
    GXPosition2f32(base, -(y - size * 0.6F));
    GXColor4u8(c.r, c.g, c.b, c.a);
    GXPosition2f32(tip, -y);
    GXColor4u8(c.r, c.g, c.b, c.a);
    GXPosition2f32(tip, -y);
    GXColor4u8(c.r, c.g, c.b, c.a);
    GXPosition2f32(base, -(y + size * 0.6F));
    GXColor4u8(c.r, c.g, c.b, c.a);
}

/* ---- HD art --------------------------------------------------------------------------------
 * The menu art pack (cobalt/gold: panel, 9-slice frame, rows, buttons, glyphs, cursor) is made
 * by its own HTML/CSS generator and converted, each element at its manifest format, with
 *     python pc/tools/png2gx.py --manifest <menu>/out/manifest.json --outdir _build/ui
 * then found beside the exe (gw_GxTex_OpenUI). Everything is authored at 2x, i.e. 1:1 with a
 * 1280x960 window, and drawn here at its 1x size. Loaded into the scene heap on every enter,
 * since the heap is rebuilt per scene; a missing texture falls back to a flat fill. */
typedef struct FeTex {
    void* data;
    void* lut;
    u16 w, h;
    bool ok;
    GXTexObj obj;
    GXTlutObj tlut;
} FeTex;

enum {
    FT_FRAME_TL, FT_FRAME_TR, FT_FRAME_BL, FT_FRAME_BR, FT_EDGE_H, FT_EDGE_V,
    FT_PANEL, FT_ROW, FT_ROW_SEL, FT_BTN, FT_BTN_HOVER, FT_BTN_PRESS,
    FT_GLYPH_A, FT_GLYPH_B, FT_CURSOR, FT_COUNT
};

static const char* const fe_tex_names[FT_COUNT] = {
    "frame_corner_tl", "frame_corner_tr", "frame_corner_bl", "frame_corner_br",
    "frame_edge_h", "frame_edge_v", "panel_bg", "row_ng", "row_sel",
    "btn_continue_ng", "btn_continue_hover", "btn_continue_press",
    "glyph_a", "glyph_b", "cursor_hand",
};

static FeTex fe_tex[FT_COUNT];

static void fe_tex_load(FeTex* t, const char* name)
{
    extern int GxTex_OpenUI(const char* name);
    extern int GxTex_Width(int h);
    extern int GxTex_Height(int h);
    extern int GxTex_Format(int h);
    extern int GxTex_ImageSize(int h);
    extern int GxTex_TlutSize(int h);
    extern int GxTex_TlutFormat(int h);
    extern int GxTex_TlutEntries(int h);
    extern void GxTex_CopyImage(int h, void* dst);
    extern void GxTex_CopyTlut(int h, void* dst);
    extern void GxTex_Close(int h);
    int h = GxTex_OpenUI(name);
    int tfmt;
    t->ok = false;
    t->lut = NULL;
    if (h < 0) {
        return;
    }
    t->data = HSD_MemAlloc(GxTex_ImageSize(h));
    tfmt = GxTex_TlutFormat(h);
    if (tfmt >= 0) {
        t->lut = HSD_MemAlloc(GxTex_TlutSize(h));
    }
    if (t->data != NULL && (tfmt < 0 || t->lut != NULL)) {
        GxTex_CopyImage(h, t->data);
        t->w = (u16) GxTex_Width(h);
        t->h = (u16) GxTex_Height(h);
        if (tfmt >= 0) {
            /* a colour-indexed texture: its palette loads into TLUT0 before each draw */
            GxTex_CopyTlut(h, t->lut);
            GXInitTlutObj(&t->tlut, t->lut, (GXTlutFmt) tfmt, (u16) GxTex_TlutEntries(h));
            GXInitTexObjCI(&t->obj, t->data, t->w, t->h, (GXTexFmt) GxTex_Format(h),
                           GX_CLAMP, GX_CLAMP, GX_FALSE, GX_TLUT0);
        } else {
            GXInitTexObj(&t->obj, t->data, t->w, t->h, (GXTexFmt) GxTex_Format(h), GX_CLAMP,
                         GX_CLAMP, GX_FALSE);
        }
        GXInitTexObjLOD(&t->obj, GX_LINEAR, GX_LINEAR, 0.0F, 0.0F, 0.0F, GX_FALSE, GX_FALSE,
                        GX_ANISO_1);
        t->ok = true;
    }
    GxTex_Close(h);
}

/* A textured quad with explicit texture coordinates (a flipped 9-slice edge swaps them), tinted
 * by `c`. It sets its own GX state (one texture stage, modulated by the vertex colour) and then
 * invalidates HSD's state cache and restores the untextured vertex-colour setup the rest of the
 * panels draw with - raw GX calls go around that cache. */
static void fe_tex_quad_uv(FeTex* t, float x, float y, float w, float h, float u0, float v0,
                           float u1, float v1, GXColor c)
{
    if (!t->ok) {
        return;
    }
    GXClearVtxDesc();
    GXSetVtxDesc(GX_VA_POS, GX_DIRECT);
    GXSetVtxDesc(GX_VA_CLR0, GX_DIRECT);
    GXSetVtxDesc(GX_VA_TEX0, GX_DIRECT);
    GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_POS, GX_POS_XY, GX_F32, 0);
    GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_CLR0, GX_CLR_RGBA, GX_RGBA8, 0);
    GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_TEX0, GX_TEX_ST, GX_F32, 0);
    GXSetNumTexGens(1);
    GXSetTexCoordGen(GX_TEXCOORD0, GX_TG_MTX2x4, GX_TG_TEX0, GX_IDENTITY);
    GXSetNumIndStages(0);
    GXSetNumTevStages(1);
    GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
    GXSetTevOp(GX_TEVSTAGE0, GX_MODULATE);
    GXSetNumChans(1);
    GXSetChanCtrl(GX_COLOR0A0, GX_FALSE, GX_SRC_REG, GX_SRC_VTX, GX_LIGHT_NULL, GX_DF_NONE,
                  GX_AF_NONE);
    GXSetBlendMode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_NOOP);
    GXSetAlphaCompare(GX_ALWAYS, 0, GX_AOP_AND, GX_ALWAYS, 0);
    GXSetZMode(GX_FALSE, GX_ALWAYS, GX_FALSE);
    GXSetCullMode(GX_CULL_NONE);
    if (t->lut != NULL) {
        GXLoadTlut(&t->tlut, GX_TLUT0);
    }
    GXLoadTexObj(&t->obj, GX_TEXMAP0);
    GXBegin(GX_QUADS, GX_VTXFMT0, 4);
    GXPosition2f32(x, -y);
    GXColor4u8(c.r, c.g, c.b, c.a);
    GXTexCoord2f32(u0, v0);
    GXPosition2f32(x + w, -y);
    GXColor4u8(c.r, c.g, c.b, c.a);
    GXTexCoord2f32(u1, v0);
    GXPosition2f32(x + w, -(y + h));
    GXColor4u8(c.r, c.g, c.b, c.a);
    GXTexCoord2f32(u1, v1);
    GXPosition2f32(x, -(y + h));
    GXColor4u8(c.r, c.g, c.b, c.a);
    GXTexCoord2f32(u0, v1);
    HSD_StateInvalidate(-1);
    hsd_80391A04(1.0F, 1.0F, 1);
}

static void fe_tex_quad(FeTex* t, float x, float y, float w, float h, GXColor c)
{
    fe_tex_quad_uv(t, x, y, w, h, 0.0F, 0.0F, 1.0F, 1.0F, c);
}

/* An element at its authored size (1x = half its 2x texel size), or a flat stand-in. */
static void fe_tex_or_solid(int which, float x, float y, float w, float h, GXColor fallback)
{
    if (fe_tex[which].ok) {
        fe_tex_quad(&fe_tex[which], x, y, w, h, fe_rgba(255, 255, 255, 255));
    } else {
        fe_solid(x, y, w, h, fallback);
    }
}

/* ---- the layout + motion player and the menu tree (split out for size; same TU) ---------- */

static void fe_match_setup_from_menus(void);

#include "gmfrontend_player.inc"
#include "gmfrontend_kit.inc"
#include "gmfrontend_menus.inc"

static void fm_route_to_menus(u8 kind, u8 sel)
{
    fm.start_kind = kind;
    fm.start_sel = sel;
    fe.next_menus = true;
}

static bool fm_route_native(void)
{
    return fm.native_req;
}

static bool fm_route_position(u8 previous, u8* kind, u8* sel)
{
    fm_position_for(previous, kind, sel);
    return fm_replaced(*kind);
}

/* VS > Melee from the menus: the MATCH SETUP screen as its own GS_FRONTEND scene, continuing to
 * GM_VS; backing out goes to GM_MENU, which routes back to the VS hub on Melee (the reported
 * mode is GM_VS, as it was when the native menu led here). */
static void fe_match_setup_from_menus(void)
{
    fe.screen = &fe_screen_vs_setup;
    fe.continue_to = GM_VS;
    fe.back_to = GM_MENU;
    fe.reported = GM_MENU;
    fe.next_menus = false;
    fm_leave_scene(GM_FRONTEND);
}

bool gmFrontend_NativeRequest(u8* kind, u8* sel)
{
    if (!fm.native_req) {
        return false;
    }
    *kind = fm.native_kind;
    *sel = fm.native_sel;
    return true;
}

bool gmFrontend_TakeNativeRequest(u8* kind, u8* sel)
{
    if (!gmFrontend_NativeRequest(kind, sel)) {
        return false;
    }
    fm.native_req = false;
    return true;
}

bool gmFrontend_NativeReturn(int kind, int sel)
{
    if (!fe_menus_on() || !fm_replaced(kind)) {
        return false;
    }
    OSReport("frontend: native screen backs out to (kind %d, sel %d) - back to the menus\n", kind,
             sel);
    fm_route_to_menus((u8) kind, (u8) sel);
    return true;
}

/* The state's on_enter: when this scene is the menu tree, what gmmenumode.c's onEnter does
 * before the menu scene - the memory-card work area and archive (the scene saves on arrival,
 * as mnMain_Scene_OnEnter does) and the preload-cache bookkeeping of a mode change. */
static void fe_state_enter(GameModeState* state)
{
    (void) state;
    if (!fe.next_menus) {
        return;
    }
    lbCardNew_AllocWorkArea();
    lbCardGame_LoadArchive(0);
    lbDvd_80018C6C();
    lbDvd_8001823C();
    lbDvd_80018254();
}

/* Ease-out cubic over [0,1]. */
static float fe_ease(float t)
{
    float u;
    if (t <= 0.0F) {
        return 0.0F;
    }
    if (t >= 1.0F) {
        return 1.0F;
    }
    u = 1.0F - t;
    return 1.0F - u * u * u;
}

/* How far row `slot` has slid in: rows arrive one after another. */
static float fe_row_offset(int slot)
{
    float t = (float) (fe.frames - 2 * slot) / 12.0F;
    return (1.0F - fe_ease(t)) * 48.0F;
}

static float fe_row_y(int slot)
{
    return FE_ROW_Y + FE_ROW_STEP * slot;
}

static int fe_rows_shown(void)
{
    return fe.n_list < FE_MAX_ROWS ? fe.n_list : FE_MAX_ROWS;
}

/* The 9-slice frame around the screen: four corners at 64x64 and the two edge pieces stretched
 * between them, the bottom and right ones mirrored - the art pack's own preview composes it so. */
static void fe_draw_frame(void)
{
    const float m = FE_FRAME_M, c = 64.0F, e = 16.0F;
    const float sx = FE_W - 2 * m - 2 * c, sy = FE_H - 2 * m - 2 * c;
    GXColor w = fe_rgba(255, 255, 255, 255);
    if (!fe_tex[FT_EDGE_H].ok || !fe_tex[FT_FRAME_TL].ok) {
        return;
    }
    fe_tex_quad_uv(&fe_tex[FT_EDGE_H], m + c, m, sx, e, 0, 0, 1, 1, w);
    fe_tex_quad_uv(&fe_tex[FT_EDGE_H], m + c, FE_H - m - e, sx, e, 0, 1, 1, 0, w);
    fe_tex_quad_uv(&fe_tex[FT_EDGE_V], m, m + c, e, sy, 0, 0, 1, 1, w);
    fe_tex_quad_uv(&fe_tex[FT_EDGE_V], FE_W - m - e, m + c, e, sy, 1, 0, 0, 1, w);
    fe_tex_quad(&fe_tex[FT_FRAME_TL], m, m, c, c, w);
    fe_tex_quad(&fe_tex[FT_FRAME_TR], FE_W - m - c, m, c, c, w);
    fe_tex_quad(&fe_tex[FT_FRAME_BL], m, FE_H - m - c, c, c, w);
    fe_tex_quad(&fe_tex[FT_FRAME_BR], FE_W - m - c, FE_H - m - c, c, c, w);
}

static void fe_draw_panels(HSD_GObj* gobj, int pass)
{
    int slot;
    float pulse;
    (void) gobj;
    if (pass != 0) {
        return;
    }
    if (fm.active) {
        fp_draw();
        return;
    }
    if (fe.screen == NULL) {
        return;
    }
    hsd_80391A04(1.0F, 1.0F, 1);

    fe_solid(0, 0, FE_W, FE_H, FE_INK);
    fe_draw_frame();

    pulse = (float) (fe.frames % 60) / 60.0F;
    pulse = pulse < 0.5F ? pulse * 2.0F : (1.0F - pulse) * 2.0F;

    if (fe.loading) {
        /* the panel at its native size, and a hard-edged progress bar on it */
        float bx = 128, by = 268, bw = 384, bh = 12;
        float fill = bw * fe.progress;
        fe_tex_or_solid(FT_PANEL, 64, 112, 512, 256, FE_COBALT);
        fe_solid(bx - 4, by - 4, bw + 8, bh + 8, FE_INK);
        fe_solid(bx, by, bw, bh, FE_COBALT_DK);
        fe_solid(bx, by, fill, bh, FE_GOLD);
        fe_solid(bx, by, fill, 3, FE_GOLD_LT);
        if (fill > 4) {
            fe_solid(bx + fill - 4, by - 2, 4, bh + 4, fe_rgba(242, 239, 228, (u8) (150 + 105 * pulse)));
        }
        return;
    }

    fe_tex_or_solid(FT_PANEL, FE_PANEL_X, FE_PANEL_Y, FE_PANEL_W, FE_PANEL_H, FE_COBALT);

    for (slot = 0; slot < fe_rows_shown(); slot++) {
        fe_tex_or_solid(FT_ROW, FE_ROW_X + fe_row_offset(slot), fe_row_y(slot), FE_ROW_W,
                        FE_ROW_H, FE_COBALT_DK);
    }
    /* the cursor's plate, drawn where it currently is on its way to the cursor */
    if (fe.cursor < fe.n_list) {
        float y = FE_ROW_Y + FE_ROW_STEP * fe.hl_y;
        int s0 = (int) (fe.hl_y + 0.5F);
        fe_tex_or_solid(FT_ROW_SEL, FE_ROW_X + fe_row_offset(s0), y, FE_ROW_W, FE_ROW_H,
                        FE_COBALT_LT);
    }

    for (slot = 0; slot < fe_rows_shown(); slot++) {
        int idx = fe.vis[fe.scroll + slot];
        const FrontendItem* it = &fe.screen->items[idx];
        float x = FE_ROW_X + fe_row_offset(slot);
        float y = fe_row_y(slot);
        float cy = y + 14.0F; /* the plate's middle */
        int selected = (fe.scroll + slot) == fe.cursor;

        /* change arrows on the selected value */
        if (selected && (it->kind == FE_CHOICE || it->kind == FE_SLIDER)) {
            fe_arrow(x + FE_ROW_W - 28, cy, 10, +1, FE_GOLD_LT);
            fe_arrow(it->kind == FE_SLIDER ? x + 226 : x + 318, cy, 10, -1, FE_GOLD_LT);
        }

        /* the value's own furniture: a slider track, a toggle pill - ink-outlined, hard stops */
        if (it->kind == FE_SLIDER && it->get != NULL && it->max > it->min) {
            float frac = (float) (it->get() - it->min) / (float) (it->max - it->min);
            float tx = x + 240, tw = 150, ty = cy - 3;
            fe_solid(tx - 2, ty - 2, tw + 4, 10, FE_INK);
            fe_solid(tx, ty, tw * frac, 6, FE_GOLD);
            fe_solid(tx + tw * frac - 5, ty - 6, 10, 18, FE_INK);
            fe_solid(tx + tw * frac - 3, ty - 4, 6, 14, FE_BONE);
        } else if (it->kind == FE_TOGGLE && it->get != NULL) {
            int on = it->get() != 0;
            float px = x + 240, py = cy - 9;
            fe_solid(px - 2, py - 2, 44, 22, FE_INK);
            fe_solid(px, py, 40, 18, on ? FE_GOLD : FE_COBALT_DK);
            fe_solid(on ? px + 23 : px + 3, py + 3, 14, 12, FE_BONE);
        }
    }

    /* CONTINUE, overlapping the panel's corner: hover while the cursor is on it, pressed as the
       screen leaves through it; the hand points at it */
    if (fe.has_button) {
        int on = fe.cursor == fe.n_vis - 1;
        int which = fe.leaving == 1 ? FT_BTN_PRESS : on ? FT_BTN_HOVER : FT_BTN;
        fe_tex_or_solid(which, FE_BTN_X, FE_BTN_Y, FE_BTN_W, FE_BTN_H,
                        on ? FE_COBALT_LT : FE_COBALT);
        if (on && fe.leaving == 0) {
            float bob = 3.0F * pulse;
            fe_tex_quad(&fe_tex[FT_CURSOR], FE_BTN_X + FE_BTN_W * 0.82F,
                        FE_BTN_Y + FE_BTN_H * 0.48F + bob, 32, 32, fe_rgba(255, 255, 255, 255));
        }
    }

    /* footer hints: the controller's own buttons beside their labels */
    fe_tex_quad(&fe_tex[FT_GLYPH_A], FE_HINT_A_X, FE_HINT_Y, 24, 24, fe_rgba(255, 255, 255, 255));
    fe_arrow(FE_HINT_LR_X + 6, FE_HINT_Y + 12, 12, -1, FE_GOLD);
    fe_arrow(FE_HINT_LR_X + 24, FE_HINT_Y + 12, 12, +1, FE_GOLD);
    fe_tex_quad(&fe_tex[FT_GLYPH_B], FE_HINT_B_X, FE_HINT_Y, 24, 24, fe_rgba(255, 255, 255, 255));

    /* scroll marks, when the list runs past the rows shown */
    if (fe.scroll > 0) {
        fe_solid(FE_W * 0.5F - 12, FE_ROW_Y - 9, 24, 4, FE_GOLD);
    }
    if (fe.scroll + fe_rows_shown() < fe.n_list) {
        fe_solid(FE_W * 0.5F - 12, fe_row_y(FE_MAX_ROWS) + 1, 24, 4, FE_GOLD);
    }
}

static void fe_draw_fade(HSD_GObj* gobj, int pass)
{
    (void) gobj;
    if (pass != 2) {
        return;
    }
    if (fm.active) {
        if (fm.fade > 0) {
            hsd_80391A04(1.0F, 1.0F, 1);
            fe_solid(0, 0, FE_W, FE_H, fe_rgba(0, 0, 0, (u8) (255 * fm.fade / FM_FADE)));
        }
        return;
    }
    if (fe.fade <= 0) {
        return;
    }
    hsd_80391A04(1.0F, 1.0F, 1);
    fe_solid(0, 0, FE_W, FE_H, fe_rgba(0, 0, 0, (u8) (255 * fe.fade / FE_FADE_FRAMES)));
}

/* ---- text ---------------------------------------------------------------------------------- */

static HSD_Text* fe_text(float x, float y, float scale, u8 align, GXColor color, const char* s)
{
    HSD_Text* t = HSD_SisLib_803A6754(0, fe.canvas);
    if (t == NULL) {
        return NULL;
    }
    t->pos_x = x;
    t->pos_y = y;
    t->pos_z = 0.0F;
    t->font_size.x = scale;
    t->font_size.y = scale;
    t->default_kerning = 1;
    t->default_alignment = align;
    HSD_SisLib_803A74F0(t, HSD_SisLib_803A6B98(t, 0.0F, 0.0F, s), &color);
    return t;
}

/* Replace a text's string. HSD_SisLib_803A70A0 takes SIS-encoded text rather than a C string and
 * prints garbage from ASCII, so a changed string gets a fresh text object instead - only when it
 * actually changes (`cur` keeps what is shown), with the old object's position, size and colour. */
static void fe_set_text(HSD_Text** pt, char* cur, int cap, const char* s, GXColor color)
{
    HSD_Text* old = *pt;
    HSD_Text* t;
    int i;
    if (old == NULL) {
        return;
    }
    for (i = 0; cur[i] == s[i] && s[i] != '\0' && i < cap - 1; i++) {
    }
    if (cur[i] == s[i]) {
        return; /* unchanged */
    }
    t = fe_text(old->pos_x, old->pos_y, old->font_size.x, old->default_alignment, color,
                s[0] != '\0' ? s : " ");
    if (t == NULL) {
        return;
    }
    t->hidden = old->hidden;
    HSD_SisLib_803A5CC4(old);
    *pt = t;
    for (i = 0; s[i] != '\0' && i < cap - 1; i++) {
        cur[i] = s[i];
    }
    cur[i] = '\0';
}

static void fe_value_string(const FrontendItem* it, char* out)
{
    int v;
    out[0] = '\0';
    if (it->get == NULL) {
        return;
    }
    v = it->get();
    switch (it->kind) {
    case FE_CHOICE:
        if (it->options != NULL && v >= it->min && v <= it->max) {
            sprintf(out, "%s", it->options[v - it->min]);
        }
        break;
    case FE_SLIDER:
        if (it->format != NULL) {
            it->format(v, out);
        } else {
            sprintf(out, "%d", v);
        }
        break;
    case FE_TOGGLE:
        sprintf(out, v ? "On" : "Off");
        break;
    }
}

/* Put the visible items into the row slots: labels, values, positions (rows slide in). */
static void fe_refresh_rows(void)
{
    int slot;
    char buf[48];
    for (slot = 0; slot < FE_MAX_ROWS; slot++) {
        HSD_Text* l = fe.label[slot];
        HSD_Text* v = fe.value[slot];
        if (l == NULL || v == NULL) {
            continue;
        }
        if (slot >= fe_rows_shown()) {
            l->hidden = 1;
            v->hidden = 1;
            continue;
        }
        {
            const FrontendItem* it = &fe.screen->items[fe.vis[fe.scroll + slot]];
            float dx = fe_row_offset(slot);
            fe_set_text(&fe.label[slot], fe.label_str[slot], FE_STR, it->label, FE_LABEL_COLOR);
            fe_value_string(it, buf);
            fe_set_text(&fe.value[slot], fe.value_str[slot], FE_STR, buf, FE_VALUE_COLOR);
            l = fe.label[slot];
            v = fe.value[slot];
            l->hidden = 0;
            l->pos_x = FE_ROW_X + 32 + dx;
            l->pos_y = fe_row_y(slot) + 4;
            v->hidden = buf[0] == '\0';
            v->pos_x = FE_VALUE_X + dx;
            v->pos_y = fe_row_y(slot) + 4;
        }
    }
    if (fe.n_vis > 0) {
        int idx = fe.vis[fe.cursor];
        const char* h = fe.screen->items[idx].help;
        fe_set_text(&fe.help, fe.help_str, FE_STR, h != NULL ? h : "", FE_HELP_COLOR);
    }
}

/* Recompute which items are visible (a value can hide or show others), keeping the cursor on the
 * same item where it can. */
static void fe_rebuild_visible(void)
{
    int keep = fe.n_vis > 0 ? fe.vis[fe.cursor] : -1;
    int button = -1;
    int i;
    fe.n_list = 0;
    for (i = 0; i < fe.screen->n_items && fe.n_list < FE_MAX_ITEMS - 1; i++) {
        const FrontendItem* it = &fe.screen->items[i];
        if (it->visible != NULL && !it->visible()) {
            continue;
        }
        if (it->kind == FE_ACTION && it->action == FE_DO_CONTINUE) {
            button = i; /* drawn as the CONTINUE button, not a row */
        } else {
            fe.vis[fe.n_list++] = i;
        }
    }
    fe.n_vis = fe.n_list;
    fe.has_button = button >= 0;
    if (fe.has_button) {
        fe.vis[fe.n_vis++] = button;
    }
    /* the same item where it is still visible, else the nearest list row above it */
    fe.cursor = 0;
    for (i = 0; i < fe.n_vis; i++) {
        if (fe.vis[i] == keep) {
            fe.cursor = i;
            break;
        }
        if (i < fe.n_list && fe.vis[i] <= keep) {
            fe.cursor = i;
        }
    }
    if (fe.cursor < fe.n_list) {
        if (fe.cursor < fe.scroll) {
            fe.scroll = fe.cursor;
        }
        if (fe.cursor >= fe.scroll + FE_MAX_ROWS) {
            fe.scroll = fe.cursor - FE_MAX_ROWS + 1;
        }
    }
    if (fe.scroll > fe.n_list - fe_rows_shown()) {
        fe.scroll = fe.n_list - fe_rows_shown();
    }
    if (fe.scroll < 0) {
        fe.scroll = 0;
    }
}

/* ---- the scene ----------------------------------------------------------------------------- */

void gm_Scene_Frontend_OnEnter(void* enter_data)
{
    HSD_GObj* gobj;
    int slot;
    (void) enter_data;

    fe.cursor = 0;
    fe.scroll = 0;
    fe.frames = 0;
    fe.fade = FE_FADE_FRAMES;
    fe.leaving = 0;
    fe.hl_y = 0.0F;
    fe.n_vis = 0;
    fe.help_item = -1;
    if (fe.next_menus) {
        fe.next_menus = false;
        fe.canvas = HSD_SisLib_803A611C(0, NULL, 0x13, 0x14, 0, FE_GX_LINK, 10, 0);
        gobj = GObj_Create(0xE, 0xF, 0);
        if (gobj != NULL) {
            GObj_SetupGXLink(gobj, fe_draw_panels, FE_GX_LINK, 0);
        }
        gobj = GObj_Create(0xE, 0xF, 0);
        if (gobj != NULL) {
            GObj_SetupGXLink(gobj, fe_draw_fade, FE_GX_LINK, 20);
        }
        fm_scene_enter();
        return;
    }
    fm.active = false;
    if (fe.screen == NULL) {
        return;
    }
    fe_rebuild_visible();
    if (fe.has_button) {
        fe.cursor = fe.n_vis - 1; /* A straight away continues, as the menus do */
    }

    /* The text canvas makes its own 640x480 orthographic camera; the panels draw on its link
       below the text, the fade above it. */
    fe.canvas = HSD_SisLib_803A611C(0, NULL, 0x13, 0x14, 0, FE_GX_LINK, 10, 0);
    gobj = GObj_Create(0xE, 0xF, 0);
    if (gobj != NULL) {
        GObj_SetupGXLink(gobj, fe_draw_panels, FE_GX_LINK, 0);
    }
    gobj = GObj_Create(0xE, 0xF, 0);
    if (gobj != NULL) {
        GObj_SetupGXLink(gobj, fe_draw_fade, FE_GX_LINK, 20);
    }
    {
        int i, n = 0;
        for (i = 0; i < FT_COUNT; i++) {
            fe_tex_load(&fe_tex[i], fe_tex_names[i]);
            n += fe_tex[i].ok;
        }
        OSReport("frontend: %d of %d art elements loaded%s\n", n, FT_COUNT,
                 n < FT_COUNT ? " - the missing ones draw flat" : "");
    }

    if (fe.loading) {
        extern int Gfx_PipelinesCreated(void);
        fe.title = fe_text(320, 196, 1.2F, 1, FE_BONE, fe.screen->title);
        fe.subtitle = fe_text(76, 114, 0.55F, 0, FE_INK, fe.screen->subtitle); /* on the tab */
        fe.help = fe_text(320, 296, 0.5F, 1, FE_BONE, " ");
        fe.help_str[0] = ' ';
        fe.help_str[1] = '\0';
        fe.load_base = Gfx_PipelinesCreated();
        {
            extern int Gfx_SeedCoreCount(void);
            extern int Gfx_PipelinesPending(void);
            extern int Gfx_SeedPipelinesBuilt(void);
            fe.load_core = Gfx_SeedCoreCount();
            OSReport("frontend: loading screen - %d pipelines built so far (%d from the seed), "
                     "%d pending, seed core %d\n",
                     fe.load_base, Gfx_SeedPipelinesBuilt(), Gfx_PipelinesPending(),
                     fe.load_core);
        }
        fe.load_settled = 0;
        fe.progress = 0.0F;
        fe.percent = -1;
        return;
    }
    fe.title = fe_text(44, 24, 0.9F, 0, FE_BONE, fe.screen->title);
    fe.subtitle = /* on the panel's gold tab */
        fe_text(FE_PANEL_X + 44, FE_PANEL_Y + 7, 0.55F, 0, FE_INK, fe.screen->subtitle);
    for (slot = 0; slot < FE_MAX_ROWS; slot++) {
        fe.label[slot] =
            fe_text(FE_ROW_X + 32, fe_row_y(slot) + 4, 0.56F, 0, FE_LABEL_COLOR, " ");
        fe.value[slot] = fe_text(FE_VALUE_X, fe_row_y(slot) + 4, 0.56F, 2, FE_VALUE_COLOR, " ");
        fe.label_str[slot][0] = ' ';
        fe.label_str[slot][1] = '\0';
        fe.value_str[slot][0] = ' ';
        fe.value_str[slot][1] = '\0';
    }
    fe.help = fe_text(44, FE_PANEL_Y + FE_PANEL_H + 8, 0.5F, 0, FE_HELP_COLOR, " ");
    fe.help_str[0] = ' ';
    fe.help_str[1] = '\0';
    fe.hints = fe_text(FE_HINT_A_X + 32, FE_HINT_Y + 2, 0.5F, 0, FE_BONE, "Select");
    fe.hint_change = fe_text(FE_HINT_LR_X + 40, FE_HINT_Y + 2, 0.5F, 0, FE_BONE, "Change");
    fe.hint_back = fe_text(FE_HINT_B_X + 32, FE_HINT_Y + 2, 0.5F, 0, FE_BONE, "Back");
    fe_refresh_rows();
}

static void fe_change(const FrontendItem* it, int dir)
{
    int v, n;
    if (it->get == NULL || it->set == NULL) {
        return;
    }
    v = it->get();
    n = it->max - it->min + 1;
    switch (it->kind) {
    case FE_CHOICE:
        v = it->min + ((v - it->min + dir) % n + n) % n; /* wraps */
        break;
    case FE_SLIDER:
        v += dir * (it->step > 0 ? it->step : 1);
        v = v < it->min ? it->min : v > it->max ? it->max : v; /* clamps */
        break;
    case FE_TOGGLE:
        v = !v;
        break;
    default:
        return;
    }
    it->set(v);
    sfxMove();
    fe_rebuild_visible(); /* a value can show or hide other rows */
}

static void fe_loading_frame(void)
{
    extern int Gfx_PipelinesPending(void);
    extern int Gfx_PipelinesCreated(void);
    extern int Gfx_LoadScreenEnabled(void);
    extern int Gfx_SeedPipelinesBuilt(void);
    int pending, done;
    float target;
    char buf[FE_STR];

    if (fe.leaving != 0) {
        if (++fe.fade >= FE_FADE_FRAMES) {
            fe.warmed = true;
            gm_801A4B60(); /* the state's on_exit picks the match */
        }
        return;
    }
    if (fe.fade > 0) {
        fe.fade--;
    }

    /* Warm means the seed's core is built - aurora compiles the seed in its own order, which is
       most-used first, and counts what it has built (Gfx_SeedPipelinesBuilt) - or that nothing
       is pending at all. */
    pending = Gfx_PipelinesPending();
    done = Gfx_PipelinesCreated();
    if (pending > 0 && fe.load_core > 0 && Gfx_SeedPipelinesBuilt() < fe.load_core) {
        target = (float) Gfx_SeedPipelinesBuilt() / (float) fe.load_core;
        fe.load_settled = 0;
    } else if (pending > 0 && fe.load_core == 0) {
        target = (float) (done - fe.load_base) / (float) (done - fe.load_base + pending);
        fe.load_settled = 0;
    } else {
        target = 1.0F;
        fe.load_settled++;
    }
    if (target > fe.progress) {
        fe.progress += (target - fe.progress) * 0.2F;
    }
    if (fe.load_settled > 0 && fe.progress > 0.995F) {
        fe.progress = 1.0F;
    }
    if (fe.percent != (int) (fe.progress * 100.0F)) {
        fe.percent = (int) (fe.progress * 100.0F);
        if (fe.progress < 1.0F) {
            sprintf(buf, "Warming up the renderer");
        } else {
            sprintf(buf, "Ready");
        }
        fe_set_text(&fe.help, fe.help_str, FE_STR, buf, FE_HELP_COLOR);
    }

    if (!Gfx_LoadScreenEnabled() || fe.frames >= FE_LOAD_CEILING_FRAMES ||
        (fe.frames >= FE_LOAD_MIN_FRAMES && fe.load_settled >= FE_LOAD_SETTLE_FRAMES))
    {
        OSReport("frontend: loading screen done after %d frames (%d pipelines built since boot, "
                 "%d while here, %d still pending, core target %d, %s)\n",
                 fe.frames, done, done - fe.load_base, pending, fe.load_core,
                 fe.load_settled > 0 ? "warm" : "ceiling");
        fe.leaving = 1;
    }
}

void gm_Scene_Frontend_OnFrame(void)
{
    u32 in;
    const FrontendItem* it;

    if (fm.active) {
        fm_scene_frame();
        return;
    }
    fe.frames++;
    if (fe.loading) {
        fe_loading_frame();
        return;
    }
    if (fe.screen == NULL) {
        gm_ChangeGameModeAfterCurrentScene(fe.continue_to);
        gm_801A4B60();
        return;
    }

    /* fades: in on arrival, out before leaving */
    if (fe.leaving != 0) {
        if (++fe.fade >= FE_FADE_FRAMES) {
            /* continuing, the next mode sees GM_MENU before it, as from the native menu; backing
               out, the menus position themselves by the mode this screen stood in for */
            fe.reported = (fe.leaving == 1 && fe_menus_on()) ? GM_MENU : fe.continue_to;
            gm_ChangeGameModeAfterCurrentScene(fe.leaving == 1 ? fe.continue_to : fe.back_to);
            gm_801A4B60();
        }
        fe_refresh_rows();
        return;
    }
    if (fe.fade > 0) {
        fe.fade--;
    }

    if (fe.cursor < fe.n_list) {
        fe.hl_y += ((float) (fe.cursor - fe.scroll) - fe.hl_y) * 0.35F;
    }

    if (fe.frames >= 4 && fe.n_vis > 0) { /* let the button that brought us here go */
        in = mn_80229624(4);
        it = &fe.screen->items[fe.vis[fe.cursor]];
        if (in & MenuInput_Up) {
            fe.cursor = (fe.cursor + fe.n_vis - 1) % fe.n_vis;
            sfxMove();
        } else if (in & MenuInput_Down) {
            fe.cursor = (fe.cursor + 1) % fe.n_vis;
            sfxMove();
        } else if (in & MenuInput_Left) {
            fe_change(it, -1);
        } else if (in & MenuInput_Right) {
            fe_change(it, +1);
        } else if (in & MenuInput_Confirm) {
            if (it->kind == FE_ACTION) {
                if (it->action == FE_DO_CONTINUE) {
                    sfxForward();
                    fe.leaving = 1;
                } else {
                    sfxBack();
                    fe.leaving = 2;
                }
            } else {
                fe_change(it, +1);
            }
        } else if (in & MenuInput_Back) {
            sfxBack();
            fe.leaving = 2;
        }
        if (fe.cursor >= fe.n_list) {
            /* on the button: the plate waits on the row the cursor will come back to */
        } else if (fe.cursor < fe.scroll) {
            fe.scroll = fe.cursor;
        } else if (fe.cursor >= fe.scroll + FE_MAX_ROWS) {
            fe.scroll = fe.cursor - FE_MAX_ROWS + 1;
        }
    }
    fe_refresh_rows();
}

void gm_Scene_Frontend_OnExit(void* exit_data)
{
    int slot;
    (void) exit_data;
    if (fm.active) {
        fm_scene_exit();
        return;
    }
    for (slot = 0; slot < FE_MAX_ROWS; slot++) {
        if (fe.label[slot] != NULL) {
            HSD_SisLib_803A5CC4(fe.label[slot]);
        }
        if (fe.value[slot] != NULL) {
            HSD_SisLib_803A5CC4(fe.value[slot]);
        }
        fe.label[slot] = fe.value[slot] = NULL;
    }
    if (fe.title != NULL) {
        HSD_SisLib_803A5CC4(fe.title);
    }
    if (fe.subtitle != NULL) {
        HSD_SisLib_803A5CC4(fe.subtitle);
    }
    if (fe.help != NULL) {
        HSD_SisLib_803A5CC4(fe.help);
    }
    if (fe.hints != NULL) {
        HSD_SisLib_803A5CC4(fe.hints);
    }
    if (fe.hint_change != NULL) {
        HSD_SisLib_803A5CC4(fe.hint_change);
    }
    if (fe.hint_back != NULL) {
        HSD_SisLib_803A5CC4(fe.hint_back);
    }
    fe.title = fe.subtitle = fe.help = fe.hints = fe.hint_change = fe.hint_back = NULL;
    {
        int i;
        for (i = 0; i < FT_COUNT; i++) {
            fe_tex[i].ok = false; /* the scene heap goes */
        }
    }
    fe.loading = false;
}

static u8 fe_enter_data[4];

GameModeState gm_Mode_Frontend_States[] = {
    {
        0,
        lbDvdPreload_2,
        0,
        fe_state_enter,
        NULL,
        {
            GS_FRONTEND,
            fe_enter_data,
            NULL,
        },
    },
    { GM_GAMEMODESTATE_TERMINATE },
};

#endif
