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
#include <sysdolphin/baselib/dobj.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/jobj.h>
#include <sysdolphin/baselib/mobj.h>
#include <sysdolphin/baselib/tobj.h>
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
 * Everything renders through aurora at the window's resolution.
 *
 * THE MENU TREE (gmfrontend_menus.inc, MELEE_FRONTEND_MENUS, default on). GM_MENU's list screens
 * - Main, Solo (1P), Regular Match, Stadium, Multi-Man, Versus, Special Melee, Collection
 * (trophies), Options, Data, Records - are frontend screens: one GS_FRONTEND scene holds the
 * whole tree, and a screen is (MenuKind, selection) exactly as vanilla positions it.
 *   routing   gmFrontend_Route turns every mode change INTO GM_MENU into GM_FRONTEND, opening
 *             where gmmenumode.c would have (previous mode -> (kind, selection); force_main_menu;
 *             the language change) - unless that is a screen not drawn here (the Event list).
 *   items     each item does what its mnmain.c think does: the same GM_* mode, gm_801677E8(port)
 *             where vanilla calls it, sfxForward/sfxBack/sfxMove, B to the same parent item;
 *             locks are mn_80229938 and hide the item as vanilla does. VS > Melee goes through
 *             MATCH SETUP (a toolkit screen, its own GS_FRONTEND scene) to GM_VS.
 *   native    screens not replaced yet run natively in GM_MENU: the frontend leaves with a
 *             request (gmFrontend_NativeRequest); gmmenumode.c positions the menu and mnmain.c
 *             opens the screen as the parent's think would (mn_PcOpenNative). Their back-out
 *             (mn_80229894) returns here through gmFrontend_NativeReturn.
 *   arrival   as mnMain_Scene_OnEnter: the menu music and lbCardGame_SaveChanges(); the state's
 *             on_enter does gmmenumode's card work area and preload-cache bookkeeping.
 *   trace     MenuFlow mirrors the frontend's (kind, selection), so the scene trace's
 *             "scene: cursor menu" lines cover it; the log names every screen and action.
 *
 * THE PLAYER (gmfrontend_player.inc) plays the art pipeline's *_layout.json / *_motion.json at
 * runtime (JSON read from ui/ beside the exe via gw_UiFile_Read; png2gx.py --layout converts
 * the textures and copies the JSON). THE KIT (gmfrontend_kit.inc) is brief section 1: atlas
 * text from font_manifest.json, section palettes from kit.json, chrome_layout / list_layout,
 * kit_motion's row events. MELEE_FE_HUBDEMO=<frame> shows out_hub's own hub frozen at a frame
 * of the pipeline's preview script, for comparison with its preview sheets.
 * Design notes and the art still expected: _research/frontend-menus.md. */

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
    FE_DO_CALL,     ///< run the item's `call` (the screen stays)
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
    void (*call)(void);                 ///< FE_DO_CALL
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

static void fe_open_online(void);

static const FrontendItem fe_items_vs_setup[] = {
    { FE_ACTION, FE_DO_CONTINUE, "Continue to Character Select",
      "Pick fighters with these rules." },
    { FE_ACTION, FE_DO_CALL, "Online Play", "Play a friend over the internet.", NULL, NULL, 0, 0,
      0, NULL, NULL, NULL, fe_open_online },
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

/* ---- ONLINE PLAY: host or join a match over the internet -------------------------------------
 * The connection itself is pc/platform/gw_netplay.c (its header explains codes, UPnP and hole
 * punching). This screen only collects the choices and shows progress: Host Match copies this
 * machine's code to the clipboard for the friend; the friend pastes it and presses Connect. Once
 * the two are connected both go straight into the agreed match. */
int Netplay_MenuBegin(int host, int ck, int color, int stage_ext, int stocks, int minutes,
                      int delay);
int Netplay_MenuPaste(int as_host);
int Netplay_MenuPoll(void);
void Netplay_MenuCancel(void);
void Netplay_MenuLaunch(void);
void Netplay_MenuStatus(char* out, int cap);
void Netplay_MenuCode(char* out, int cap);
void Netplay_MenuPeer(char* out, int cap);
int Netplay_MenuHasServer(void);
int Netplay_RematchPending(void);
void Netplay_RematchTaken(void);
int Netplay_CodeSlot(void);
void Netplay_CodeStep(int dir);
void Netplay_CodeNext(void);
int Netplay_CodeComplete(void);
void Netplay_CodeText(char* out, int cap);
int Netplay_CodeAutofill(void);
int Netplay_CodeKeys(void);
int Netplay_LobbyPhase(void);
int Netplay_LobbySeq(void);
int Netplay_LobbyMe(void);
int Netplay_LobbyInfo(int what);
int Netplay_LobbyStage(int i);
void Netplay_LobbyStageName(int i, char* out, int cap);
int Netplay_LobbyPlayer(int who, int what);
void Netplay_LobbyCode(char* out, int cap);
void Netplay_LobbyChar(int ck, int color);
void Netplay_LobbyStageAct(int i);
void Netplay_LobbyReady(int on);
int Netplay_LobbyActive(void);
static int fe_np_stage_id(void);
int Netplay_MenuGetLetter(int i);
void Netplay_MenuSetLetter(int i, int v);

enum { FE_NP_IDLE, FE_NP_WORKING, FE_NP_CONNECTED, FE_NP_FAILED, FE_NP_RUNNING, FE_NP_LOBBY };

/* CharacterKind order */
static const char* const fe_np_chars[] = {
    "Captain Falcon", "Donkey Kong", "Fox", "Mr. Game & Watch", "Kirby", "Bowser", "Link",
    "Luigi", "Mario", "Marth", "Mewtwo", "Ness", "Peach", "Pikachu", "Ice Climbers",
    "Jigglypuff", "Samus", "Yoshi", "Zelda", "Sheik", "Falco", "Young Link", "Dr. Mario", "Roy",
    "Pichu", "Ganondorf",
};
static void fe_np_char_name(int ck, char* out)
{
    if (ck >= 0 && ck <= 25) {
        sprintf(out, "%s", fe_np_chars[ck]);
    } else {
        sprintf(out, "Fighter %d", ck); /* an m-ex fighter: its icon says who */
    }
}
static const char* const fe_np_stages[] = { "Battlefield", "Final Destination", "Dream Land",
                                            "Yoshi's Story", "Fountain of Dreams",
                                            "Pokemon Stadium" };
static const int fe_np_stage_ext[] = { 31, 32, 28, 8, 2, 3 };
static const char* const fe_np_roles[] = { "Host", "Join" };

static int fe_np_role, fe_np_ck = 2, fe_np_color, fe_np_stage, fe_np_stocks = 4,
                       fe_np_minutes = 8, fe_np_delay = 2;
static int fe_np_phase; ///< FE_NP_*, from Netplay_MenuPoll

static int fe_np_get_role(void) { return fe_np_role; }
static void fe_np_set_role(int v)
{
    if (fe_np_phase == FE_NP_WORKING) {
        Netplay_MenuCancel();
        fe_np_phase = FE_NP_IDLE;
    }
    fe_np_role = v;
    if (v == 1 && Netplay_MenuHasServer()) {
        Netplay_CodeAutofill(); /* a code your friend sent, already copied? */
    }
}
static int fe_np_get_ck(void) { return fe_np_ck; }
static void fe_np_set_ck(int v) { fe_np_ck = v; }
static int fe_np_get_color(void) { return fe_np_color; }
static void fe_np_set_color(int v) { fe_np_color = v; }
static void fe_np_fmt_color(int v, char* out) { sprintf(out, "Color %d", v + 1); }
static int fe_np_get_stage(void) { return fe_np_stage; }
static void fe_np_set_stage(int v) { fe_np_stage = v; }
static int fe_np_get_stocks(void) { return fe_np_stocks; }
static void fe_np_set_stocks(int v) { fe_np_stocks = v; }
static int fe_np_get_minutes(void) { return fe_np_minutes; }
static void fe_np_set_minutes(int v) { fe_np_minutes = v; }
static void fe_np_fmt_minutes(int v, char* out) { sprintf(out, "%d min", v); }
static int fe_np_get_delay(void) { return fe_np_delay; }
static void fe_np_set_delay(int v) { fe_np_delay = v; }
static void fe_np_fmt_delay(int v, char* out) { sprintf(out, "%d frames", v); }
static int fe_np_is_host(void) { return fe_np_role == 0; }
static int fe_np_is_join(void) { return fe_np_role == 1; }
static int fe_np_zero(void) { return 0; }
static void fe_np_fmt_code(int v, char* out)
{
    (void) v;
    Netplay_MenuCode(out, 64);
}
static void fe_np_fmt_peer(int v, char* out)
{
    (void) v;
    Netplay_MenuPeer(out, 64);
}
static void fe_np_start(void)
{
    if (Netplay_MenuBegin(fe_np_role == 0, fe_np_ck, fe_np_color, fe_np_stage_id(),
                          fe_np_stocks, fe_np_minutes, fe_np_delay) == 0)
    {
        fe_np_phase = FE_NP_WORKING;
    } else {
        fe_np_phase = FE_NP_FAILED;
    }
}
static void fe_np_paste_host(void) { Netplay_MenuPaste(0); }

/* ---- picking on Melee's own screens ---------------------------------------------------------
 * Character opens the real character select screen (VS mode's CSS, seeded with the current pick)
 * and Stage the real stage select screen. Confirming there returns HERE instead of going on:
 * gmvsmelee.c's CSS/SSS exit handlers ask Frontend_OnlinePick() and hand the result back through
 * Frontend_OnlinePicked. At the moment of confirming, the screen copies the chosen icon's texture
 * (Frontend_CaptureIcon) so this screen can show it after the CSS/SSS art is freed. m-ex fighters
 * and stages come for free: they are whatever the disc's own screens offer. */
void SceneLaunch_SetText(const char* text);

static int fe_np_pick; ///< 0 none, 1 picking a character, 2 picking a stage
static int fe_np_stage_pick = -1; ///< the external stage id picked on the SSS, -1 = the list's

typedef struct FeNpIcon {
    bool ok;
    u16 w, h;
    int fmt, tlut_fmt, tlut_n;
    u8 data[64 * 1024] ATTRIBUTE_ALIGN(32);
    u8 lut[512] ATTRIBUTE_ALIGN(32);
    GXTexObj obj;
    GXTlutObj tlut;
} FeNpIcon;
static FeNpIcon fe_np_icon[2]; ///< 0 the character, 1 the stage

/* Every fighter's CSS icon, captured on each CSS visit - the lobby shows the opponent's as well. */
typedef struct FeNpCharIcon {
    bool ok;
    u16 w, h;
    int fmt, tlut_fmt, tlut_n;
    u8 data[4096] ATTRIBUTE_ALIGN(32);
    u8 lut[512] ATTRIBUTE_ALIGN(32);
} FeNpCharIcon;
#define FE_NP_MAX_CK 0x48
static FeNpCharIcon fe_np_char_icon[FE_NP_MAX_CK];

int Frontend_OnlinePick(void) { return fe_np_pick; }

/* Copy the biggest texture under `root` - the icon the player just confirmed. */
static void fe_np_find_tex(HSD_JObj* j, HSD_TObj** best, int* area, int depth)
{
    /* the chosen icon and its own subtree - not its siblings, which are the other icons */
    for (; j != NULL && depth < 12; j = depth == 0 ? NULL : HSD_JObjGetNext(j)) {
        HSD_DObj* d = HSD_JObjGetDObj(j);
        for (; d != NULL; d = d->next) {
            HSD_TObj* t = d->mobj != NULL ? d->mobj->tobj : NULL;
            for (; t != NULL; t = t->next) {
                if (t->imagedesc != NULL && t->imagedesc->image_ptr != NULL) {
                    /* colour beats intensity: an icon's plate/outline is an I4/IA4 mask, the
                       portrait a colour (usually CI8) texture of the same size */
                    int fmt = t->imagedesc->format;
                    int a = t->imagedesc->width * t->imagedesc->height * (fmt >= 4 ? 4 : 1);
                    if (a > *area) {
                        *area = a;
                        *best = t;
                    }
                }
            }
        }
        fe_np_find_tex(HSD_JObjGetChild(j), best, area, depth + 1);
    }
}

void Frontend_CaptureCharIcon(int ck, HSD_JObj* root)
{
    FeNpCharIcon* ic;
    HSD_TObj* t = NULL;
    int area = 0;
    u32 size;
    if (ck < 0 || ck >= FE_NP_MAX_CK || root == NULL) {
        return;
    }
    ic = &fe_np_char_icon[ck];
    fe_np_find_tex(root, &t, &area, 0);
    if (t == NULL) {
        return;
    }
    size = GXGetTexBufferSize(t->imagedesc->width, t->imagedesc->height, t->imagedesc->format,
                              GX_FALSE, 0);
    if (size == 0 || size > sizeof ic->data) {
        return;
    }
    memcpy(ic->data, t->imagedesc->image_ptr, size);
    ic->w = t->imagedesc->width;
    ic->h = t->imagedesc->height;
    ic->fmt = t->imagedesc->format;
    ic->tlut_n = 0;
    if (t->tlut != NULL && t->tlut->lut != NULL && t->tlut->n_entries * 2 <= (int) sizeof ic->lut) {
        ic->tlut_n = t->tlut->n_entries;
        ic->tlut_fmt = t->tlut->fmt;
        memcpy(ic->lut, t->tlut->lut, (size_t) ic->tlut_n * 2);
    }
    ic->ok = true;
}

void Frontend_CaptureIcon(int which, HSD_JObj* root)
{
    FeNpIcon* ic;
    HSD_TObj* t = NULL;
    int area = 0;
    u32 size;
    if (which < 0 || which > 1 || root == NULL) {
        return;
    }
    ic = &fe_np_icon[which];
    fe_np_find_tex(root, &t, &area, 0);
    if (t == NULL) {
        OSReport("frontend: online pick - no texture under the chosen icon\n");
        return;
    }
    size = GXGetTexBufferSize(t->imagedesc->width, t->imagedesc->height, t->imagedesc->format,
                              GX_FALSE, 0);
    if (size == 0 || size > sizeof ic->data) {
        return;
    }
    memcpy(ic->data, t->imagedesc->image_ptr, size);
    ic->w = t->imagedesc->width;
    ic->h = t->imagedesc->height;
    ic->fmt = t->imagedesc->format;
    ic->tlut_n = 0;
    if (t->tlut != NULL && t->tlut->lut != NULL && t->tlut->n_entries * 2 <= (int) sizeof ic->lut) {
        ic->tlut_n = t->tlut->n_entries;
        ic->tlut_fmt = t->tlut->fmt;
        memcpy(ic->lut, t->tlut->lut, (size_t) ic->tlut_n * 2);
    }
    ic->ok = true;
    OSReport("frontend: online pick - captured the %s icon, %dx%d format %d%s\n",
             which == 0 ? "character" : "stage", ic->w, ic->h, ic->fmt,
             ic->tlut_n != 0 ? " (palette)" : "");
}

static void fe_np_open(int which, const char* scene);
static void fe_switch_screen(const FrontendScreen* s);

static char fe_np_scene[160];
static void fe_np_pick_char(void)
{
    /* a CPU in port 2: the CSS shows "Ready to fight" only with two fighters */
    sprintf(fe_np_scene, "mode=vs;at=css;p1=ck:%d/c%d/hu;p2=ck:%d/c%d/cpu", fe_np_ck, fe_np_color,
            fe_np_ck == 2 ? 20 : 2, 1);
    fe_np_open(1, fe_np_scene);
}
static void fe_np_pick_stage(void)
{
    sprintf(fe_np_scene, "mode=vs;at=sss;p1=ck:%d/c%d/hu;p2=ck:%d/c%d/hu", fe_np_ck, fe_np_color,
            fe_np_ck, (fe_np_color + 1) % 4);
    fe_np_open(2, fe_np_scene);
}
static int fe_np_stage_id(void)
{
    return fe_np_stage_pick >= 0 ? fe_np_stage_pick : fe_np_stage_ext[fe_np_stage];
}

static void fe_np_draw_icon(int which, int item_index, float row_top, float pitch, float row_h,
                            float row_x1, int scroll, int nrows);
static void fe_np_draw_char_icon(int ck, int item_index, float row_top, float pitch, float row_h,
                                 float row_x1, int scroll, int nrows);

/* ROOM CODES (a matchmaking server is configured): the join code is four letters, each picked with
 * left/right - no typing needed with a controller - or pasted. */
static const char* const fe_np_letters[] = {
    "A", "B", "C", "D", "E", "F", "G", "H", "J", "K", "L", "M", "N", "P", "Q", "R",
    "S", "T", "U", "V", "W", "X", "Y", "Z", "2", "3", "4", "5", "6", "7", "8", "9",
};
static int fe_np_join_room(void) { return fe_np_role == 1 && Netplay_MenuHasServer(); }
/* the code field: Left/Right changes the active slot's letter, A moves on, typing fills it */
static int fe_np_code_v;
static int fe_np_code_get(void) { return fe_np_code_v; }
static void fe_np_code_set(int v)
{
    Netplay_CodeStep(v > fe_np_code_v ? 1 : -1);
    fe_np_code_v = 0;
}
static void fe_np_code_fmt(int v, char* out)
{
    (void) v;
    Netplay_CodeText(out, 40);
}
static bool fe_np_code_open; ///< the code entry popup is up: keys are text, the menu waits
static void fe_np_code_next(void) { fe_np_code_open = true; }
static int fe_np_join_addr(void) { return fe_np_role == 1 && !Netplay_MenuHasServer(); }
static int fe_np_l0(void) { return Netplay_MenuGetLetter(0); }
static int fe_np_l1(void) { return Netplay_MenuGetLetter(1); }
static int fe_np_l2(void) { return Netplay_MenuGetLetter(2); }
static int fe_np_l3(void) { return Netplay_MenuGetLetter(3); }
static void fe_np_s0(int v) { Netplay_MenuSetLetter(0, v); }
static void fe_np_s1(int v) { Netplay_MenuSetLetter(1, v); }
static void fe_np_s2(int v) { Netplay_MenuSetLetter(2, v); }
static void fe_np_s3(int v) { Netplay_MenuSetLetter(3, v); }
static void fe_np_paste_friend(void) { Netplay_MenuPaste(1); }

static const FrontendItem fe_items_online[] = {
    { FE_CHOICE, 0, "Play As", "Host a match, or join a friend's.", fe_np_get_role,
      fe_np_set_role, 0, 1, 1, fe_np_roles },
    { FE_ACTION, FE_DO_CALL, "Character",
      "Pick your fighter and costume on the character select screen.", NULL, NULL, 0, 0, 0, NULL,
      NULL, NULL, fe_np_pick_char },
    { FE_ACTION, FE_DO_CALL, "Stage", "Pick the stage on the stage select screen.", NULL, NULL, 0,
      0, 0, NULL, NULL, fe_np_is_host, fe_np_pick_stage },
    { FE_SLIDER, 0, "Stocks", "Lives each player starts with.", fe_np_get_stocks,
      fe_np_set_stocks, 1, 9, 1, NULL, NULL, fe_np_is_host },
    { FE_SLIDER, 0, "Time Limit", "The match clock.", fe_np_get_minutes, fe_np_set_minutes, 1,
      20, 1, NULL, fe_np_fmt_minutes, fe_np_is_host },
    { FE_SLIDER, 0, "Input Delay", "2 suits most connections; raise it if it stutters.",
      fe_np_get_delay, fe_np_set_delay, 0, 8, 1, NULL, fe_np_fmt_delay, fe_np_is_host },
    { FE_ACTION, FE_DO_CALL, "Paste Host Code", "Copy your friend's code, then press A here.",
      NULL, NULL, 0, 0, 0, NULL, NULL, fe_np_is_join, fe_np_paste_host },
    { FE_CHOICE, 0, "Room Code",
      "A to enter the code: type or paste (Ctrl+V), or Left/Right + A per letter.",
      fe_np_code_get, fe_np_code_set, -1000, 1000, 1, NULL, fe_np_code_fmt, fe_np_join_room,
      fe_np_code_next },
    { FE_SLIDER, 0, "Host Code", "The code you pasted.", fe_np_zero, NULL, 0, 0, 0, NULL,
      fe_np_fmt_peer, fe_np_join_addr },
    { FE_ACTION, FE_DO_CALL, "Host Match", "Start hosting; your code is copied for your friend.",
      NULL, NULL, 0, 0, 0, NULL, NULL, fe_np_is_host, fe_np_start },
    { FE_ACTION, FE_DO_CALL, "Connect", "Join the host whose code you pasted.", NULL, NULL, 0, 0,
      0, NULL, NULL, fe_np_is_join, fe_np_start },
    { FE_SLIDER, 0, "Your Code", "Your address; it is copied to the clipboard.", fe_np_zero,
      NULL, 0, 0, 0, NULL, fe_np_fmt_code },
    { FE_ACTION, FE_DO_CALL, "Paste Friend's Code",
      "Only if they can't connect: paste their code here.", NULL, NULL, 0, 0, 0, NULL, NULL,
      fe_np_is_host, fe_np_paste_friend },
};

static const FrontendScreen fe_screen_online = {
    "ONLINE",
    "ONLINE PLAY",
    fe_items_online,
    sizeof fe_items_online / sizeof fe_items_online[0],
};

/* ---- THE WAITING ROOM and THE LOBBY (online) ------------------------------------------------
 * Host Match / Connect lead to the WAITING ROOM (the room code, big, and the status); once the
 * other player is in, both go to the LOBBY: competitive pick/ban run by gw_netplay.c's rules -
 * Game 1 characters double-blind on the real CSS, then stage striking 1-2-2 after a coin flip;
 * Game 2+ the winner bans 2 stages, the loser picks, then winner and loser pick characters. Then
 * both press Ready and the match starts. Art for these screens: menu/out_lobby (LOBBY.md). */
enum { LBP_OFF, LBP_CHAR_BLIND, LBP_STRIKE, LBP_BAN, LBP_PICK, LBP_CHAR_WINNER, LBP_CHAR_LOSER,
       LBP_READY, LBP_GO };
enum { LBS_FREE, LBS_P1, LBS_P2, LBS_BANNED, LBS_PICKED };

static void fe_wr_code(int v, char* out)
{
    (void) v;
    Netplay_LobbyCode(out, 40);
}
static void fe_wr_status(int v, char* out)
{
    (void) v;
    Netplay_MenuStatus(out, 60);
}
static void fe_wr_leave(void)
{
    Netplay_MenuCancel();
    fe_np_phase = FE_NP_IDLE;
    fe_switch_screen(&fe_screen_online);
}
static const FrontendItem fe_items_wait[] = {
    { FE_SLIDER, 0, "Room Code", "Send this code to your friend - it is on your clipboard.",
      fe_np_zero, NULL, 0, 0, 0, NULL, fe_wr_code },
    { FE_SLIDER, 0, "Status", "", fe_np_zero, NULL, 0, 0, 0, NULL, fe_wr_status },
    { FE_ACTION, FE_DO_CALL, "Leave Room", "Close the room and go back.", NULL, NULL, 0, 0, 0,
      NULL, NULL, NULL, fe_wr_leave },
};
static const FrontendScreen fe_screen_wait = {
    "ONLINE",
    "WAITING ROOM",
    fe_items_wait,
    sizeof fe_items_wait / sizeof fe_items_wait[0],
};

static int fe_lb_me(void) { return Netplay_LobbyMe(); }
static int fe_lb_phase(void) { return Netplay_LobbyPhase(); }
static bool fe_lb_revealed(void)
{
    int p = fe_lb_phase();
    return p != LBP_CHAR_BLIND; /* blind picks stay hidden until both are locked */
}
static void fe_lb_set(int v, char* out)
{
    int me = fe_lb_me();
    (void) v;
    sprintf(out, "Game %d    You %d - %d Opponent", Netplay_LobbyInfo(0), Netplay_LobbyInfo(2 + me),
            Netplay_LobbyInfo(2 + (1 - me)));
}
static void fe_lb_player(int who, char* out)
{
    char name[32];
    int locked = Netplay_LobbyPlayer(who, 2), ready = Netplay_LobbyPlayer(who, 3);
    int mine = who == fe_lb_me();
    const char* st;
    fe_np_char_name(Netplay_LobbyPlayer(who, 0), name);
    st = ready ? "READY" : locked ? "LOCKED IN" : "PICKING";
    if (!mine && !fe_lb_revealed()) {
        sprintf(out, "%s", locked ? "LOCKED IN" : "PICKING...");
    } else if (fe_lb_phase() >= LBP_READY) {
        sprintf(out, "%s - %s", name, ready ? "READY" : "not ready");
    } else {
        sprintf(out, "%s - %s", name, st);
    }
}
static void fe_lb_you(int v, char* out)
{
    (void) v;
    fe_lb_player(fe_lb_me(), out);
}
static void fe_lb_opp(int v, char* out)
{
    (void) v;
    fe_lb_player(1 - fe_lb_me(), out);
}
static int fe_lb_can_pick_char(void)
{
    int p = fe_lb_phase(), me = fe_lb_me();
    if (p == LBP_CHAR_BLIND) {
        return !Netplay_LobbyPlayer(me, 2);
    }
    return (p == LBP_CHAR_WINNER || p == LBP_CHAR_LOSER) && Netplay_LobbyInfo(4) == me;
}
static void fe_lb_pick_char(void)
{
    if (fe_lb_can_pick_char()) {
        fe_np_pick_char(); /* the real CSS; the pick comes back through Frontend_OnlinePicked */
    }
}
static int fe_lb_stage_phase(void)
{
    int p = fe_lb_phase();
    return p == LBP_STRIKE || p == LBP_BAN || p == LBP_PICK;
}
static void fe_lb_stage_fmt(int i, char* out)
{
    static const char* const st[] = { "", "struck by P1", "struck by P2", "BANNED", "PICKED" };
    int s = Netplay_LobbyStage(i);
    sprintf(out, "%s", s >= 0 && s <= 4 ? st[s] : "");
}
#define FE_LB_STAGE(i)                                                                            \
    static void fe_lb_sf##i(int v, char* out) { (void) v; fe_lb_stage_fmt(i, out); }             \
    static void fe_lb_sa##i(void)                                                                 \
    {                                                                                             \
        if (fe_lb_stage_phase() && Netplay_LobbyInfo(4) == fe_lb_me() &&                          \
            Netplay_LobbyStage(i) == LBS_FREE)                                                    \
            Netplay_LobbyStageAct(i);                                                             \
    }
FE_LB_STAGE(0)
FE_LB_STAGE(1)
FE_LB_STAGE(2)
FE_LB_STAGE(3)
FE_LB_STAGE(4)
FE_LB_STAGE(5)
static int fe_lb_ready_phase(void) { return fe_lb_phase() == LBP_READY; }
static void fe_lb_ready_fmt(int v, char* out)
{
    int me = fe_lb_me();
    (void) v;
    if (Netplay_LobbyInfo(8) > 0) {
        sprintf(out, "Starting in %d...", Netplay_LobbyInfo(8));
    } else {
        sprintf(out, "%s", Netplay_LobbyPlayer(me, 3) ? "READY - waiting" : "press A");
    }
}
static void fe_lb_ready(void) { Netplay_LobbyReady(!Netplay_LobbyPlayer(fe_lb_me(), 3)); }
static void fe_lb_leave(void)
{
    Netplay_MenuCancel();
    fe_np_phase = FE_NP_IDLE;
    fe_switch_screen(&fe_screen_online);
}
static const FrontendItem fe_items_lobby[] = {
    { FE_SLIDER, 0, "Set", "", fe_np_zero, NULL, 0, 0, 0, NULL, fe_lb_set },
    { FE_SLIDER, 0, "You", "", fe_np_zero, NULL, 0, 0, 0, NULL, fe_lb_you },
    { FE_SLIDER, 0, "Opponent", "", fe_np_zero, NULL, 0, 0, 0, NULL, fe_lb_opp },
    { FE_ACTION, FE_DO_CALL, "Pick Character", "Pick your fighter on the character select screen.",
      NULL, NULL, 0, 0, 0, NULL, NULL, fe_lb_can_pick_char, fe_lb_pick_char },
    { FE_ACTION, FE_DO_CALL, "Battlefield", "", NULL, NULL, 0, 0, 0, NULL, fe_lb_sf0, NULL, fe_lb_sa0 },
    { FE_ACTION, FE_DO_CALL, "Dream Land", "", NULL, NULL, 0, 0, 0, NULL, fe_lb_sf1, NULL, fe_lb_sa1 },
    { FE_ACTION, FE_DO_CALL, "Final Destination", "", NULL, NULL, 0, 0, 0, NULL, fe_lb_sf2, NULL,
      fe_lb_sa2 },
    { FE_ACTION, FE_DO_CALL, "Fountain of Dreams", "", NULL, NULL, 0, 0, 0, NULL, fe_lb_sf3, NULL,
      fe_lb_sa3 },
    { FE_ACTION, FE_DO_CALL, "Pokemon Stadium", "", NULL, NULL, 0, 0, 0, NULL, fe_lb_sf4, NULL,
      fe_lb_sa4 },
    { FE_ACTION, FE_DO_CALL, "Yoshi's Story", "", NULL, NULL, 0, 0, 0, NULL, fe_lb_sf5, NULL,
      fe_lb_sa5 },
    { FE_ACTION, FE_DO_CALL, "Ready", "Both players ready starts the match.", NULL, NULL, 0, 0, 0,
      NULL, fe_lb_ready_fmt, fe_lb_ready_phase, fe_lb_ready },
    { FE_ACTION, FE_DO_CALL, "Leave Room", "Leave the room.", NULL, NULL, 0, 0, 0, NULL, NULL, NULL,
      fe_lb_leave },
};
static const FrontendScreen fe_screen_lobby = {
    "ONLINE",
    "LOBBY",
    fe_items_lobby,
    sizeof fe_items_lobby / sizeof fe_items_lobby[0],
};

/* What the lobby asks of this player right now - the help line. */
static void fe_lb_instruction(char* out)
{
    int p = fe_lb_phase(), me = fe_lb_me(), turn = Netplay_LobbyInfo(4), left = Netplay_LobbyInfo(5);
    bool mine = turn == me;
    switch (p) {
    case LBP_CHAR_BLIND:
        sprintf(out, "%s", Netplay_LobbyPlayer(me, 2) ? "Locked in - waiting for your opponent"
                                                      : "Pick your character (hidden until both lock in)");
        break;
    case LBP_STRIKE:
        sprintf(out, mine ? "Your turn: strike %d stage%s (coin flip: P%d strikes first)"
                          : "Opponent is striking %d stage%s (coin flip: P%d strikes first)",
                left, left == 1 ? "" : "s", Netplay_LobbyInfo(6) + 1);
        break;
    case LBP_BAN:
        sprintf(out, mine ? "You won the last game: ban %d stage%s" : "Opponent (last winner) bans %d stage%s",
                left, left == 1 ? "" : "s");
        break;
    case LBP_PICK:
        sprintf(out, "%s", mine ? "Your counterpick: choose the stage" : "Opponent is picking the stage");
        break;
    case LBP_CHAR_WINNER:
    case LBP_CHAR_LOSER:
        sprintf(out, "%s", mine ? "Your turn: pick your character" : "Opponent is picking their character");
        break;
    case LBP_READY:
        sprintf(out, "%s", "Both press Ready to start");
        break;
    default:
        sprintf(out, "%s", "Starting...");
        break;
    }
}

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
#define FE_STR 80 ///< longest string a row, value or help line shows

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

void Frontend_OnlinePicked(int which, int a, int b)
{
    if (which == 1 && a >= 0) {
        fe_np_ck = a;
        fe_np_color = b;
    } else if (which == 2 && a >= 0) {
        fe_np_stage_pick = a;
    }
    fe_np_pick = 0;
    SceneLaunch_SetText(NULL);
    fe.screen = &fe_screen_online;
    if (Netplay_LobbyActive()) {
        /* picked in the lobby: tell the room, and back to the lobby */
        if (which == 1 && a >= 0) {
            Netplay_LobbyChar(a, b);
        }
        fe.screen = &fe_screen_lobby;
    }
    fe.continue_to = GM_VS;
    fe.back_to = GM_MENU;
    fe.next_menus = false;
}

/* After an online match in a room (persistent rooms): the results screen comes back here. */
void Frontend_BackToOnline(void)
{
    fe_np_pick = 0;
    SceneLaunch_SetText(NULL);
    fe.screen = &fe_screen_online;
    fe.continue_to = GM_VS;
    fe.back_to = GM_MENU;
    fe.next_menus = false;
}

static void fe_np_open(int which, const char* scene)
{
    fe_np_pick = which;
    SceneLaunch_SetText(scene);
    fe.continue_to = GM_VS;
    fe.leaving = 1; /* fade out into VS mode, which opens at the CSS / SSS */
}


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
    if (fe_np_pick != 0 && to == GM_MENU) {
        fe_np_pick = 0; /* left the CSS/SSS for the main menu: the online pick is abandoned */
        SceneLaunch_SetText(NULL);
    }
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

/* A fighter's captured CSS icon beside a row of the lobby. */
static void fe_np_draw_char_icon(int ck, int item_index, float row_top, float pitch, float row_h,
                                 float row_x1, int scroll, int nrows)
{
    FeNpCharIcon* ic;
    int v, r = -1;
    float y, h, w, x;
    if (ck < 0 || ck >= FE_NP_MAX_CK || !fe_np_char_icon[ck].ok) {
        return;
    }
    ic = &fe_np_char_icon[ck];
    for (v = 0; v < fe.n_vis; v++) {
        if (fe.vis[v] == item_index) {
            r = v - scroll;
        }
    }
    if (r < 0 || r >= nrows) {
        return;
    }
    h = row_h + 6.0F;
    w = h * (float) ic->w / (float) ic->h;
    y = row_top + r * pitch - 3.0F;
    x = row_x1 - w - 36.0F + (240.0F - (y + h * 0.5F)) * 0.25F;
    {
        FeTex t;
        memset(&t, 0, sizeof t);
        t.data = ic->data;
        t.w = ic->w;
        t.h = ic->h;
        t.ok = true;
        if (ic->tlut_n != 0) {
            t.lut = ic->lut;
            GXInitTlutObj(&t.tlut, ic->lut, (GXTlutFmt) ic->tlut_fmt, (u16) ic->tlut_n);
            GXInitTexObjCI(&t.obj, ic->data, ic->w, ic->h, (GXTexFmt) ic->fmt, GX_CLAMP, GX_CLAMP,
                           GX_FALSE, GX_TLUT0);
        } else {
            GXInitTexObj(&t.obj, ic->data, ic->w, ic->h, (GXTexFmt) ic->fmt, GX_CLAMP, GX_CLAMP,
                         GX_FALSE);
        }
        GXInitTexObjLOD(&t.obj, GX_LINEAR, GX_LINEAR, 0.0F, 0.0F, 0.0F, GX_FALSE, GX_FALSE,
                        GX_ANISO_1);
        fe_tex_quad(&t, x, y, w, h, (GXColor) { 255, 255, 255, 255 });
    }
}

/* The captured icons, beside their rows (drawn over the kit's rows). */
static void fe_np_draw_icon(int which, int item_index, float row_top, float pitch, float row_h,
                            float row_x1, int scroll, int nrows)
{
    FeNpIcon* ic = &fe_np_icon[which];
    int v, r = -1;
    float y, h, w, x;
    if (!ic->ok) {
        return;
    }
    for (v = 0; v < fe.n_vis; v++) {
        if (fe.vis[v] == item_index) {
            r = v - scroll;
        }
    }
    if (r < 0 || r >= nrows) {
        return;
    }
    h = row_h + 6.0F;
    w = h * (float) ic->w / (float) ic->h;
    y = row_top + r * pitch - 3.0F;
    x = row_x1 - w - 36.0F + (240.0F - (y + h * 0.5F)) * 0.25F; /* follow the rows' lean */
    {
        FeTex t;
        memset(&t, 0, sizeof t);
        t.data = ic->data;
        t.w = ic->w;
        t.h = ic->h;
        t.ok = true;
        if (ic->tlut_n != 0) {
            t.lut = ic->lut;
            GXInitTlutObj(&t.tlut, ic->lut, (GXTlutFmt) ic->tlut_fmt, (u16) ic->tlut_n);
            GXInitTexObjCI(&t.obj, ic->data, ic->w, ic->h, (GXTexFmt) ic->fmt, GX_CLAMP,
                           GX_CLAMP, GX_FALSE, GX_TLUT0);
        } else {
            GXInitTexObj(&t.obj, ic->data, ic->w, ic->h, (GXTexFmt) ic->fmt, GX_CLAMP, GX_CLAMP,
                         GX_FALSE);
        }
        GXInitTexObjLOD(&t.obj, GX_LINEAR, GX_LINEAR, 0.0F, 0.0F, 0.0F, GX_FALSE, GX_FALSE,
                        GX_ANISO_1);
        fe_tex_quad(&t, x, y, w, h, (GXColor) { 255, 255, 255, 255 });
    }
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
#include "gmfrontend_kitlist.inc"

/* The toolkit screens are drawn with the kit when its files are there (fe_kit), with the old
 * art pack and SisLib otherwise. */
static bool fe_kit;

static bool fe_kit_available(void)
{
    kit_load();
    ff_load();
    return ff.state == 1 && kit.state == 1;
}

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
    if (fm.active || fk.on) {
        fp_draw();
        if (fk.on && fe.screen == &fe_screen_lobby) {
            int me = Netplay_LobbyMe();
            fe_np_draw_char_icon(Netplay_LobbyPlayer(me, 0), 1, kit.rows_top, kit.pitch, kit.row_h,
                                 kit.row_x1, fk.scroll, fk.nrows);
            if (Netplay_LobbyPhase() != 1) {
                fe_np_draw_char_icon(Netplay_LobbyPlayer(1 - me, 0), 2, kit.rows_top, kit.pitch,
                                     kit.row_h, kit.row_x1, fk.scroll, fk.nrows);
            }
        }
        if (fk.on && fe.screen == &fe_screen_online) {
            fe_np_draw_icon(0, 1, kit.rows_top, kit.pitch, kit.row_h, kit.row_x1, fk.scroll,
                            fk.nrows);
            fe_np_draw_icon(1, 2, kit.rows_top, kit.pitch, kit.row_h, kit.row_x1, fk.scroll,
                            fk.nrows);
        }
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
        if (it->kind == FE_ACTION && it->format != NULL) {
            it->format(0, out);
        }
        return;
    }
    v = it->get();
    if (it->format != NULL && (it->kind == FE_ACTION || it->kind == FE_CHOICE)) {
        it->format(v, out); /* a status beside an action, or a choice shown its own way */
        return;
    }
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
    char buf[FE_STR];
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
        if ((fe.screen == &fe_screen_online || fe.screen == &fe_screen_wait) &&
            fe_np_phase != FE_NP_IDLE)
        {
            Netplay_MenuStatus(buf, FE_STR);
            h = buf;
        }
        if (fe.screen == &fe_screen_lobby) {
            fe_lb_instruction(buf);
            h = buf;
        }
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
        if (it->kind == FE_ACTION && it->action == FE_DO_CONTINUE && !fe_kit) {
            button = i; /* drawn as the CONTINUE button, not a row (on the kit: the first row) */
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
    fe_kit = !fe.loading && fe_kit_available();
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
    if (fe_kit) {
        fk_build(true);
        return;
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

/* Show another screen in the same scene: the rows, title and subtitle follow. */
static void fe_switch_screen(const FrontendScreen* s)
{
    fe.screen = s;
    fe.cursor = 0;
    fe.scroll = 0;
    fe.hl_y = 0.0F;
    fe.n_vis = 0;
    fe_rebuild_visible();
    if (fe_kit) {
        fk_build(true); /* the same scene, the next screen sliding in */
        return;
    }
    if (fe.title != NULL) {
        HSD_SisLib_803A5CC4(fe.title);
    }
    if (fe.subtitle != NULL) {
        HSD_SisLib_803A5CC4(fe.subtitle);
    }
    fe.title = fe_text(44, 24, 0.9F, 0, FE_BONE, s->title);
    fe.subtitle = fe_text(FE_PANEL_X + 44, FE_PANEL_Y + 7, 0.55F, 0, FE_INK, s->subtitle);
    fe_refresh_rows();
}

static void fe_open_online(void)
{
    fe_np_phase = FE_NP_IDLE;
    fe_switch_screen(&fe_screen_online);
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
    if (fe_kit) {
        fk_note_dir(dir);
        if (it->kind == FE_SLIDER && v == it->get()) {
            fk_bump(dir); /* held at an end: the kit's "can't go further" */
        }
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
        if (fe_kit) {
            fk_frame();
        }
        return;
    }
    if (fe.fade > 0) {
        fe.fade--;
    }

    if (fe.cursor < fe.n_list) {
        fe.hl_y += ((float) (fe.cursor - fe.scroll) - fe.hl_y) * 0.35F;
    }

    if (fe.screen == &fe_screen_online && fe_np_phase == FE_NP_IDLE && Netplay_RematchPending() &&
        fe.frames >= (fe_np_role == 0 ? 30 : 150))
    {
        /* back from a match in a room: reconnect to it - the host re-hosts the same code first,
           the guest follows a couple of seconds later */
        Netplay_RematchTaken();
        fe_np_start();
    }
    if (fe.screen == &fe_screen_online && fe_np_phase == FE_NP_WORKING) {
        fe_switch_screen(&fe_screen_wait); /* Host Match / Connect: into the waiting room */
    }
    if (fe_np_code_open) {
        /* THE CODE ENTRY: keyboard keys are text while it is open (and only then); a controller
           changes the active letter with Left/Right, A moves on (and closes after the last
           slot), B or Enter/Esc closes. The rest of the menu waits. */
        u32 cin = mn_80229624(4);
        int k = fe.screen == &fe_screen_online ? Netplay_CodeKeys() : 2;
        if (cin & MenuInput_Left) {
            Netplay_CodeStep(-1);
            sfxMove();
        } else if (cin & MenuInput_Right) {
            Netplay_CodeStep(+1);
            sfxMove();
        } else if (cin & MenuInput_Confirm) {
            if (Netplay_CodeSlot() == 3 && Netplay_CodeComplete()) {
                k = 2;
            } else {
                Netplay_CodeNext();
            }
            sfxForward();
        } else if (cin & MenuInput_Back) {
            k = 2;
            sfxBack();
        }
        if (k == 2) {
            fe_np_code_open = false;
        }
        fe_refresh_rows();
        if (fe_kit) {
            fk_frame(); /* the field redraws as it is typed */
        }
        return;
    }
    if ((fe.screen == &fe_screen_wait || fe.screen == &fe_screen_lobby) &&
        (fe_np_phase == FE_NP_WORKING || fe_np_phase == FE_NP_LOBBY))
    {
        static int seq = -1;
        fe_np_phase = Netplay_MenuPoll();
        if (fe_np_phase == FE_NP_LOBBY && fe.screen != &fe_screen_lobby) {
            fe_switch_screen(&fe_screen_lobby); /* the other player is in */
        }
        if (fe.screen == &fe_screen_lobby && Netplay_LobbySeq() != seq) {
            seq = Netplay_LobbySeq();
            fe_rebuild_visible(); /* rows come and go with the phase */
        }
        if (fe_np_phase == FE_NP_FAILED) {
            fe_switch_screen(&fe_screen_online);
        }
        if (fe_np_phase == FE_NP_CONNECTED) {
            /* the agreed match is the configured scene: VS mode seeds it and starts at the match */
            Netplay_MenuLaunch();
            fe.continue_to = GM_VS;
            fe.leaving = 1;
            fe_np_phase = FE_NP_IDLE;
        }
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
                if (it->action == FE_DO_CALL) {
                    sfxForward();
                    if (it->call != NULL) {
                        it->call();
                    }
                    fe_rebuild_visible();
                } else if (it->action == FE_DO_CONTINUE) {
                    sfxForward();
                    fe.leaving = 1;
                } else {
                    sfxBack();
                    fe.leaving = 2;
                }
            } else if (it->call != NULL) {
                sfxForward();
                it->call(); /* e.g. the code field: A moves to the next slot */
                fe_rebuild_visible();
            } else {
                fe_change(it, +1);
            }
        } else if (in & MenuInput_Back) {
            sfxBack();
            if (fe.screen == &fe_screen_online) {
                if (fe_np_phase == FE_NP_WORKING) {
                    Netplay_MenuCancel();
                    fe_np_phase = FE_NP_IDLE;
                } else {
                    fe_switch_screen(&fe_screen_vs_setup);
                }
            } else {
                fe.leaving = 2;
            }
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
    if (fe_kit) {
        fk_frame();
    }
}

void gm_Scene_Frontend_OnExit(void* exit_data)
{
    int slot;
    (void) exit_data;
    if (fm.active) {
        fm_scene_exit();
        return;
    }
    fk_exit();
    fe_kit = false;
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
