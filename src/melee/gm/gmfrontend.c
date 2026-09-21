#include "gmfrontend.h"

#if defined(TARGET_PC)

#include "gm_1A3F.h"
#include "gmmain_lib.h"
#include "gmscene.h"
#include <dolphin/gx.h>
#include <dolphin/os.h>
#include <melee/lb/lbaudio_ax.h>
#include <melee/mn/forward.h>
#include <melee/mn/inlines.h>
#include <melee/mn/mnmain.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/gobjgxlink.h>
#include <sysdolphin/baselib/hsd_3915.h>
#include <sysdolphin/baselib/sislib.h>

#include <stdio.h>

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
    { FE_ACTION, FE_DO_BACK, "Back to Main Menu", "Return without starting." },
};

static const FrontendScreen fe_screen_vs_setup = {
    "VS. MELEE",
    "MATCH SETUP",
    fe_items_vs_setup,
    sizeof fe_items_vs_setup / sizeof fe_items_vs_setup[0],
};

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
    HSD_Text* label[FE_MAX_ROWS];
    HSD_Text* value[FE_MAX_ROWS];
    char label_str[FE_MAX_ROWS][FE_STR]; ///< what each text currently shows
    char value_str[FE_MAX_ROWS][FE_STR];
    char help_str[FE_STR];
    int vis[FE_MAX_ITEMS]; ///< indices of the visible items, in order
    int n_vis;
    int cursor;  ///< index into vis
    int scroll;  ///< first visible row
    int frames;
    int fade;    ///< 0 = clear .. FE_FADE_FRAMES = black
    int leaving; ///< 0 still here, 1 continue, 2 back
    float hl_y;  ///< the highlight's drawn row position, easing toward the cursor
    int help_item;
} fe;

u8 gmFrontend_Route(u8 from, u8 to)
{
    static int enabled = -1;
    int i;
    if (enabled < 0) {
        extern int PcFrontendEnabled(void);
        enabled = PcFrontendEnabled();
    }
    if (!enabled || from == GM_FRONTEND) {
        return to; /* leaving a screen goes exactly where it chose */
    }
    for (i = 0; i < (int) (sizeof fe_rules / sizeof fe_rules[0]); i++) {
        if (fe_rules[i].from == from && fe_rules[i].to == to) {
            fe.screen = fe_rules[i].screen;
            fe.continue_to = to;
            fe.back_to = from;
            OSReport("frontend: mode %d -> %d, showing \"%s\" first\n", from, to,
                     fe.screen->subtitle);
            return GM_FRONTEND;
        }
    }
    return to;
}

u8 gmFrontend_ReportedMode(void)
{
    return fe.continue_to;
}

/* ---- layout and drawing -------------------------------------------------------------------- */

#define FE_GX_LINK 14
#define FE_W 640.0F
#define FE_H 480.0F
#define FE_ROW_X 56.0F
#define FE_ROW_Y 112.0F
#define FE_ROW_W 528.0F
#define FE_ROW_H 34.0F
#define FE_ROW_STEP 40.0F
#define FE_VALUE_X 548.0F ///< right edge of the value column
#define FE_FADE_FRAMES 10
#define FE_LABEL_COLOR fe_rgba(245, 247, 255, 255)
#define FE_VALUE_COLOR fe_rgba(205, 222, 255, 255)
#define FE_HELP_COLOR fe_rgba(160, 168, 190, 255)

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
    return fe.n_vis < FE_MAX_ROWS ? fe.n_vis : FE_MAX_ROWS;
}

static void fe_draw_panels(HSD_GObj* gobj, int pass)
{
    int slot;
    float pulse;
    (void) gobj;
    if (pass != 0 || fe.screen == NULL) {
        return;
    }
    hsd_80391A04(1.0F, 1.0F, 1);

    /* backdrop, header band and its accent rule, footer band */
    fe_rect(0, 0, FE_W, FE_H, fe_rgba(20, 26, 46, 255), fe_rgba(5, 6, 12, 255));
    fe_rect(0, 0, FE_W, 92, fe_rgba(34, 44, 80, 255), fe_rgba(20, 26, 46, 255));
    fe_solid(0, 92, FE_W, 2, fe_rgba(96, 150, 255, 255));
    fe_rect(0, 94, FE_W, 10, fe_rgba(96, 150, 255, 60), fe_rgba(96, 150, 255, 0));
    fe_rect(0, 436, FE_W, 44, fe_rgba(0, 0, 0, 120), fe_rgba(0, 0, 0, 200));

    pulse = (float) (fe.frames % 60) / 60.0F;
    pulse = pulse < 0.5F ? pulse * 2.0F : (1.0F - pulse) * 2.0F;

    /* the highlight, drawn where it currently is on its way to the cursor */
    if (fe.n_vis > 0) {
        float y = FE_ROW_Y + FE_ROW_STEP * fe.hl_y;
        u8 glow = (u8) (40 + 50 * pulse);
        fe_solid(FE_ROW_X - 6, y - 6, FE_ROW_W + 12, FE_ROW_H + 12, fe_rgba(96, 150, 255, glow));
    }

    for (slot = 0; slot < fe_rows_shown(); slot++) {
        int idx = fe.vis[fe.scroll + slot];
        const FrontendItem* it = &fe.screen->items[idx];
        float x = FE_ROW_X + fe_row_offset(slot);
        float y = fe_row_y(slot);
        int selected = (fe.scroll + slot) == fe.cursor;

        if (selected) {
            fe_rect(x, y, FE_ROW_W, FE_ROW_H, fe_rgba(84, 132, 240, 245),
                    fe_rgba(52, 88, 190, 245));
            fe_rect(x, y, 5, FE_ROW_H, fe_rgba(255, 255, 255, 255), fe_rgba(200, 220, 255, 255));
        } else if (it->kind == FE_ACTION) {
            fe_rect(x, y, FE_ROW_W, FE_ROW_H, fe_rgba(96, 150, 255, 40),
                    fe_rgba(96, 150, 255, 20));
        } else {
            fe_rect(x, y, FE_ROW_W, FE_ROW_H, fe_rgba(255, 255, 255, 22),
                    fe_rgba(255, 255, 255, 10));
        }

        /* change arrows on the selected value */
        if (selected && (it->kind == FE_CHOICE || it->kind == FE_SLIDER)) {
            GXColor ac = fe_rgba(255, 255, 255, 230);
            float cy = y + FE_ROW_H * 0.5F;
            fe_arrow(x + FE_ROW_W - 14, cy, 10, +1, ac);
            fe_arrow(it->kind == FE_SLIDER ? x + 236 : x + 330, cy, 10, -1, ac);
        }

        /* the value's own furniture: a slider track, a toggle pill */
        if (it->kind == FE_SLIDER && it->get != NULL && it->max > it->min) {
            float frac = (float) (it->get() - it->min) / (float) (it->max - it->min);
            float tx = x + 250, tw = 150, ty = y + FE_ROW_H * 0.5F - 2;
            fe_solid(tx, ty, tw, 4, fe_rgba(255, 255, 255, 50));
            fe_solid(tx, ty, tw * frac, 4,
                     selected ? fe_rgba(255, 255, 255, 255) : fe_rgba(120, 170, 255, 255));
            fe_solid(tx + tw * frac - 3, ty - 5, 6, 14, fe_rgba(255, 255, 255, 255));
        } else if (it->kind == FE_TOGGLE && it->get != NULL) {
            int on = it->get() != 0;
            float px = x + 250, py = y + 8;
            fe_solid(px, py, 40, 18, on ? fe_rgba(90, 220, 150, 255) : fe_rgba(255, 255, 255, 50));
            fe_solid(on ? px + 23 : px + 3, py + 3, 14, 12, fe_rgba(255, 255, 255, 255));
        }
    }

    /* scroll marks, when the list runs past the rows shown */
    if (fe.scroll > 0) {
        fe_solid(FE_W * 0.5F - 12, FE_ROW_Y - 14, 24, 3, fe_rgba(150, 185, 255, 200));
    }
    if (fe.scroll + fe_rows_shown() < fe.n_vis) {
        fe_solid(FE_W * 0.5F - 12, fe_row_y(FE_MAX_ROWS) - 2, 24, 3,
                 fe_rgba(150, 185, 255, 200));
    }
}

static void fe_draw_fade(HSD_GObj* gobj, int pass)
{
    (void) gobj;
    if (pass != 2 || fe.fade <= 0) {
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
            l->pos_x = FE_ROW_X + 20 + dx;
            l->pos_y = fe_row_y(slot) + 7;
            v->hidden = buf[0] == '\0';
            v->pos_x = FE_VALUE_X + dx;
            v->pos_y = fe_row_y(slot) + 7;
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
    int keep = fe.n_vis > 0 ? fe.vis[fe.cursor] : 0;
    int i;
    fe.n_vis = 0;
    for (i = 0; i < fe.screen->n_items && fe.n_vis < FE_MAX_ITEMS; i++) {
        const FrontendItem* it = &fe.screen->items[i];
        if (it->visible == NULL || it->visible()) {
            fe.vis[fe.n_vis++] = i;
        }
    }
    fe.cursor = 0;
    for (i = 0; i < fe.n_vis; i++) {
        if (fe.vis[i] <= keep) {
            fe.cursor = i;
        }
    }
    if (fe.cursor < fe.scroll) {
        fe.scroll = fe.cursor;
    }
    if (fe.cursor >= fe.scroll + FE_MAX_ROWS) {
        fe.scroll = fe.cursor - FE_MAX_ROWS + 1;
    }
    if (fe.scroll > fe.n_vis - fe_rows_shown()) {
        fe.scroll = fe.n_vis - fe_rows_shown();
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
    if (fe.screen == NULL) {
        return;
    }
    fe_rebuild_visible();

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

    fe.title = fe_text(56, 24, 1.1F, 0, fe_rgba(255, 255, 255, 255), fe.screen->title);
    fe.subtitle = fe_text(58, 62, 0.55F, 0, fe_rgba(150, 185, 255, 255), fe.screen->subtitle);
    for (slot = 0; slot < FE_MAX_ROWS; slot++) {
        fe.label[slot] =
            fe_text(FE_ROW_X + 20, fe_row_y(slot) + 7, 0.62F, 0, FE_LABEL_COLOR, " ");
        fe.value[slot] = fe_text(FE_VALUE_X, fe_row_y(slot) + 7, 0.62F, 2, FE_VALUE_COLOR, " ");
        fe.label_str[slot][0] = ' ';
        fe.label_str[slot][1] = '\0';
        fe.value_str[slot][0] = ' ';
        fe.value_str[slot][1] = '\0';
    }
    fe.help = fe_text(56, 398, 0.5F, 0, FE_HELP_COLOR, " ");
    fe.help_str[0] = ' ';
    fe.help_str[1] = '\0';
    /* the SIS font has no slash */
    fe.hints = fe_text(56, 442, 0.5F, 0, fe_rgba(200, 206, 224, 255),
                       "A  Select        Left  Right  Change        B  Back");
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

void gm_Scene_Frontend_OnFrame(void)
{
    u32 in;
    const FrontendItem* it;

    fe.frames++;
    if (fe.screen == NULL) {
        gm_ChangeGameModeAfterCurrentScene(fe.continue_to);
        gm_801A4B60();
        return;
    }

    /* fades: in on arrival, out before leaving */
    if (fe.leaving != 0) {
        if (++fe.fade >= FE_FADE_FRAMES) {
            gm_ChangeGameModeAfterCurrentScene(fe.leaving == 1 ? fe.continue_to : fe.back_to);
            gm_801A4B60();
        }
        fe_refresh_rows();
        return;
    }
    if (fe.fade > 0) {
        fe.fade--;
    }

    fe.hl_y += ((float) (fe.cursor - fe.scroll) - fe.hl_y) * 0.35F;

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
        if (fe.cursor < fe.scroll) {
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
    fe.title = fe.subtitle = fe.help = fe.hints = NULL;
}

static u8 fe_enter_data[4];

GameModeState gm_Mode_Frontend_States[] = {
    {
        0,
        lbDvdPreload_2,
        0,
        NULL,
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
