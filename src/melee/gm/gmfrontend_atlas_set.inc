/* gmfrontend_atlas_set.inc - the Atlas drawing of the SETTINGS pages (Atlas step 5; included by gmfrontend.c after gmfrontend_settings.inc).
 *
 * A DRAWING AND INPUT LAYER over the tables. The pages (gmfrontend_settings.inc, gmfrontend_controls.inc) stay FrontendItem arrays; fe.screen stays the current
 * page (fe_screen_settings[page], fe_screen_remap, the how-to page, the erase screen), so fe_is_settings, the notice rule and the Controls heartbeat's screen test
 * keep working. Each frame this file walks the table (gmfrontend_atlas_table.h: get, visible, enabled and format are evaluated HERE) and hands the host scalars and
 * strings (pc/platform/gw_script_ui_set.inc); the host's events (CHANGE, ACCEPT, BACK, TAB, FOCUS, ALT) come back and the walker applies them with the legacy rules.
 * Every change applies at once and persists, as before; environment variables still win over saved values.
 *
 * ONE SCREEN, SIX TABS. VIDEO, AUDIO, CONTROLS, ONLINE, GAME and MODS are tabs of one host screen; the pages are the FSP_* tables (GAME is FSP_GAMEPLAY, and MODS comes
 * last here though FSP_MODS is the enum's fifth). The tab the host shows changes only when this file says so, after it has submitted the new page's rows. The focus is
 * remembered per tab (a table index) and set again by id. The how-to page, the erase screen and the remap editor are screens of their own under the same host.
 *
 * THE REMAP EDITOR is three tabs over the SAME table fe_items_remap and the SAME capture state (fcr_target, fcr_also, fcr_port, Controls_*): INPUTS (the controller
 * rows and one row per game input), PROFILE and OPTIONS. Its decisions (the heartbeat, the freeze while capturing and until the input is released, START+B, the
 * messages, the binding cache) are gmfrontend_atlas_remap.h, tested against a fake Controls_* API.
 *
 * ORDER IN THE FRAME. gm_Scene_Frontend_OnFrame calls Controls_Menu first (a 100 ms heartbeat: the remap editor's capture is cancelled by the runtime when it lapses),
 * then fss_frame(), and fe.screen stays &fe_screen_remap while the editor is open (tools/port/test_menu_heartbeat.py pins both).
 *
 * THE PAD. Menus read the pad pre-remap only in menu modes (the frontend scene is one); the settings read it through mn_80229624(4), the game's own menu input of every
 * port merged, and hand it to the host as intents. The keyboard and the mouse are the host's. A text row being typed (Player Name, Server) and a controller capture
 * freeze the screen: the host drops every event meanwhile, and primes from what is held when it ends.
 *
 * MELEE_ATLAS=0 or missing Atlas fonts: fss_open fails, nothing here runs, and the legacy kit draws the pages exactly as before. */

#include "gmfrontend_atlas_table.h"
#include "gmfrontend_atlas_remap.h"
#include <melee/mn/mndatadel.h>
#include <platform/gw_ui_erase.h> /* the labels and the category count; the calls are mnDataDel_Erase's */

extern int Ui_SetCanOpen(void);
extern int Ui_SetOpen(int kind, int active_tab, const char* t0, const char* t1, const char* title);
extern void Ui_SetClose(int h);
extern void Ui_SetTabs(int h, int n, const char* names, int active);
extern void Ui_SetFocus(int h, const char* item_id);
extern int Ui_SetFocused(int h);
extern void Ui_SetExplain(int h, const char* kicker, const char* title, const char* what, const char* now, const char* from);
extern void Ui_SetKeys(int h, const char* spec);
extern void Ui_SetNote(int h, const char* text, int kind);
extern void Ui_SetDialog(int h, const char* title, const char* text, const char* a_label, const char* b_label, int default_button);
extern int Ui_SetDialogAnswer(int h);
extern void Ui_Freeze(int h, int on);
extern void Ui_SetIntent(int h, int type, int a);
extern int Ui_SetPoll(int h, int* slot, int* arg);

#define FSS_TABS 6
#define FSS_REMAP_TABS 3
#define FSS_SYNTH 100 /* a row id at or above this is not a table row: a remap input ("t0".."t21") */

static const char fss_tab_names[] = "VIDEO,AUDIO,CONTROLS,ONLINE,GAME,MODS";
static const char fss_remap_tab_names[] = "INPUTS,PROFILE,OPTIONS";
static const int fss_tab_page[FSS_TABS] = { FSP_VIDEO, FSP_AUDIO, FSP_CONTROLS, FSP_ONLINE, FSP_GAMEPLAY, FSP_MODS };
/* the remap editor's tabs: which rows of fe_items_remap each shows (INPUTS adds one row per game input after its two controller rows) */
static const int fss_rt_inputs[] = { 0, 1 };                    /* Controller, Pick Controller */
static const int fss_rt_profile[] = { 6, 7, 8, 9, 10, 17, 18 }; /* Profile, New, Rename, Delete, Preset, Assign to Port, Reset to Default */
static const int fss_rt_options[] = { 11, 12, 13, 14, 15, 16 }; /* Swap Sticks, Trigger Analog Off, Digital Shield, the two dead zones, Rumble (this profile) */

static struct {
    int h;                 ///< the host's handle, -1 when no Atlas settings screen is up
    int kind;              ///< FSS_K_*
    int tab;               ///< the tab the host shows
    int shown_tab;         ///< the tab whose rows were submitted last
    int tab_focus[FSS_TABS]; ///< the row focused in each tab (a table index, or FSS_SYNTH + a remap input), -1 none
    FssVis vis;            ///< the rows as last submitted: slot -> table index (or FSS_SYNTH + input)
    int erase_cat;         ///< the erase operation awaiting the dialog's answer, -1 none
    const char* want_label; ///< a row to focus after the next submit (the return from the Language screen), by label
    bool refocus;          ///< set the focus by id after the next submit (a tab switch)
    int last_result;       ///< the remap result last told as a note
    int hint_sig;          ///< the key hint set last sent (a function of the focused row's kind); -1 none, so the set is sent when it changes, not every frame
    bool note_dialog;      ///< ONLINE PLAY: a long failure notice is up as a dialog, awaiting its OK
} fss = { -1, 0, 0, -1, { -1, -1, -1, -1, -1, -1 }, { 0 }, -1, NULL, false, 0, -1 };
static FssRemap fssr;

static int fss_page_of(const FrontendScreen* s)
{
    if (s >= &fe_screen_settings[0] && s < &fe_screen_settings[FSP_COUNT]) {
        return (int) (s - &fe_screen_settings[0]);
    }
    return -1;
}

static int fss_tab_of_page(int page)
{
    int t;
    for (t = 0; t < FSS_TABS; t++) {
        if (fss_tab_page[t] == page) {
            return t;
        }
    }
    return 0;
}

static int fss_kind_of(const FrontendScreen* s)
{
    if (fss_page_of(s) >= 0) {
        return FSS_K_TABS;
    }
    if (s == &fe_screen_howto_online) {
        return FSS_K_HOWTO;
    }
    if (s == &fe_screen_erase) {
        return FSS_K_ERASE;
    }
    if (s == &fe_screen_remap) {
        return FSS_K_REMAP;
    }
    if (s == &fe_screen_rules) {
        return FSS_K_RULES;
    }
    if (s == &fe_screen_morerules) {
        return FSS_K_MORERULES;
    }
    if (s == &fe_screen_vs_setup) {
        return FSS_K_MATCH;
    }
    if (s == &fe_screen_online) {
        return FSS_K_ONLINE; /* ONLINE PLAY: Host, Join, Random Opponent and the rules (step 6) */
    }
    return -1;
}

/* the number of tabs the open screen has, and their names */
static int fss_ntabs(void) { return fss.kind == FSS_K_TABS ? FSS_TABS : fss.kind == FSS_K_REMAP ? FSS_REMAP_TABS : 0; }
static const char* fss_names(void) { return fss.kind == FSS_K_REMAP ? fss_remap_tab_names : fss_tab_names; }

/* a screen the Atlas settings draw, when Atlas is on */
static bool fss_wants(const FrontendScreen* s)
{
    return s != NULL && Ui_Ready() != 0 && fss_kind_of(s) >= 0;
}

static bool fss_active(void) { return fss.h >= 0; }

/* "i7" for a table row, "t3" for a remap input: the host's row ids, stable while rows show and hide */
static void fss_row_id(int idx, char* id)
{
    if (idx >= FSS_SYNTH) {
        fss_id(idx - FSS_SYNTH, id);
        id[0] = 't';
    } else {
        fss_id(idx, id);
    }
}

static void fss_close(void)
{
    if (fss.h >= 0) {
        if (fss.kind == FSS_K_REMAP) {
            fssr_close(&fssr); /* a half-capture does not stay bound */
        }
        Ui_SetClose(fss.h);
    }
    fss.h = -1;
    fss.shown_tab = -1;
    fss.erase_cat = -1;
}

/* the host's screen for fe.screen; -1 when the Atlas screen cannot open (the legacy kit draws the page) */
static int fss_open(const FrontendScreen* s)
{
    int kind = fss_kind_of(s), tab = 0, h, i;
    const char* t0 = "MAIN MENU";
    const char* t1 = "";
    const char* title = "SETTINGS";
    if (kind == FSS_K_TABS) {
        tab = fss_tab_of_page(fss_page_of(s));
    } else if (kind == FSS_K_HOWTO) {
        t0 = "SETTINGS"; t1 = "CONTROLS"; title = "HOW TO PLAY ONLINE";
    } else if (kind == FSS_K_ERASE) {
        t0 = "SETTINGS"; t1 = "GAME"; title = "ERASE DATA";
    } else if (kind == FSS_K_RULES) {
        t0 = "VERSUS"; title = "RULES";
    } else if (kind == FSS_K_MORERULES) {
        t0 = "VERSUS"; t1 = "RULES"; title = "MORE RULES";
    } else if (kind == FSS_K_MATCH) {
        t0 = "VERSUS"; t1 = "MELEE"; title = "MATCH SETUP";
    } else if (kind == FSS_K_ONLINE) {
        t0 = "VERSUS"; title = "ONLINE PLAY";
    } else {
        t0 = "SETTINGS"; t1 = "CONTROLS"; title = "REMAP";
    }
    h = Ui_SetOpen(kind, tab, t0, t1, title);
    if (h < 0) {
        return -1;
    }
    OSReport("frontend: Atlas settings screen open (kind %d, tab %d)\n", kind, tab);
    fss.h = h;
    fss.kind = kind;
    fss.tab = tab;
    fss.shown_tab = -1;
    fss.erase_cat = -1;
    fss.hint_sig = -1;
    fss.note_dialog = false;
    for (i = 0; i < FSS_TABS; i++) {
        fss.tab_focus[i] = -1;
    }
    if (kind == FSS_K_REMAP) {
        fssr_open(&fssr, fcr_port); /* the editor's own state (fcr_open already chose the port and started the lease) */
        fss.last_result = fssr.result;
    }
    return h;
}

/* fe_switch_screen under Atlas: only fe.screen changes here; fss_frame reconciles the host (a tab, or another screen kind) on the same frame */
static bool fss_intercepts(const FrontendScreen* s) { return fss.h >= 0 && fss_wants(s); }

static void fss_switch(const FrontendScreen* s)
{
    int page = fss_page_of(s);
    if (page == FSP_MODS) {
        fsm_fill(); /* the mods folder as it is now */
    }
    fe_ol_note[0] = '\0'; /* a notice from the last page stays behind */
    fe.screen = s;
    fe.cursor = 0;
    fe.scroll = 0;
    fe.n_vis = 0;
}

/* the focused row of the screen as a stable index, for remembering it per tab */
static int fss_focused_index(void)
{
    int slot = Ui_SetFocused(fss.h);
    return (slot >= 0 && slot < fss.vis.n) ? fss.vis.idx[slot] : -1;
}

/* a tab asked for (L, R, a click): the page's rows are submitted first, the host shows the tab on the next frame */
static void fss_set_tab(int t)
{
    if (t < 0 || t >= fss_ntabs() || t == fss.tab) {
        return;
    }
    fss.tab_focus[fss.tab] = fss_focused_index(); /* where this tab was left */
    sfxMove();
    if (fss.kind == FSS_K_TABS) {
        fss_switch(&fe_screen_settings[fss_tab_page[t]]);
    } else {
        fss.tab = t; /* the remap editor's tabs are views of one screen */
    }
    fss.refocus = true;
}

/* B: the tabs, the Rules screen and MATCH SETUP go back to the menus, the how-to page to CONTROLS, the erase screen to GAME, More Rules to Rules, the remap editor through fcr_back */
static void fss_back(void)
{
    sfxBack();
    switch (fss.kind) {
    case FSS_K_TABS:
        fe.leaving = 2; /* the menus, on the item fe_settings_from_menus recorded */
        break;
    case FSS_K_HOWTO:
        fss_switch(&fe_screen_settings[FSP_CONTROLS]);
        break;
    case FSS_K_ERASE:
        fss_switch(&fe_screen_settings[FSP_GAMEPLAY]);
        fss.want_label = "Erase Data..."; /* back on the row that opened it */
        break;
    case FSS_K_REMAP:
        fcr_back(); /* Controls_Cancel, Controls_Menu(-1), and back to CONTROLS through fe_switch_screen */
        break;
    case FSS_K_RULES:
    case FSS_K_MATCH:
    case FSS_K_ONLINE:
        fe.leaving = 2; /* the menus: VERSUS on RULES (fe_rules_set_screen), or where MATCH SETUP came from */
        break;
    case FSS_K_MORERULES:
        fss_switch(&fe_screen_rules);
        break;
    default:
        break;
    }
}

/* Erase Data, step one: the dialog. It opens on Cancel; A answers the focused button and B always cancels (the host's rule). */
static void fss_erase_ask(int category)
{
    char title[48];
    const char* label = at_erase_label(category);
    int i = 0, k;
    if (fss.h < 0 || category < 0 || category >= at_erase_categories()) {
        return;
    }
    title[i++] = 'E'; title[i++] = 'R'; title[i++] = 'A'; title[i++] = 'S'; title[i++] = 'E'; title[i++] = ' ';
    for (k = 0; label[k] != '\0' && i < (int) sizeof title - 1; k++) {
        char c = label[k];
        title[i++] = (c >= 'a' && c <= 'z') ? (char) (c - 32) : c;
    }
    title[i] = '\0';
    fss.erase_cat = category;
    Ui_SetDialog(fss.h, title, "This cannot be undone.", "Erase", "Cancel", 2);
}

/* Erase Data, step two: the answer. Only a deliberate Erase runs anything; Cancel and B leave the save as it was. */
static void fss_erase_answer(void)
{
    int a;
    if (fss.erase_cat < 0) {
        return;
    }
    a = Ui_SetDialogAnswer(fss.h);
    if (a == 0) {
        return;
    }
    if (a == 1) {
        OSReport("frontend: Erase Data: operation %d confirmed\n", fss.erase_cat);
        mnDataDel_Erase(fss.erase_cat);
        sfxForward();
        Ui_SetNote(fss.h, "Erased.", 0);
    } else {
        sfxBack();
    }
    fss.erase_cat = -1;
}

static void fss_play(unsigned fx)
{
    if (fx & FSS_FX_MOVE) {
        sfxMove();
    }
    if (fx & FSS_FX_FORWARD) {
        sfxForward();
    }
    if (fx & FSS_FX_BACK) {
        sfxBack();
    }
    if (fx & FSS_FX_CONTINUE) {
        fe.leaving = 1;
    } else if (fx & FSS_FX_LEAVE) {
        fe.leaving = 2;
    }
}

/* ---- the remap editor's rows --------------------------------------------------------------------------------------------------------------------------- */

/* the rows of the remap editor's current tab: table rows by index, and on INPUTS one row per game input after the two controller rows */
static void fss_remap_vis(void)
{
    int i, n = 0, count;
    const int* list;
    if (fss.tab == 1) {
        list = fss_rt_profile; count = (int) (sizeof fss_rt_profile / sizeof fss_rt_profile[0]);
    } else if (fss.tab == 2) {
        list = fss_rt_options; count = (int) (sizeof fss_rt_options / sizeof fss_rt_options[0]);
    } else {
        list = fss_rt_inputs; count = (int) (sizeof fss_rt_inputs / sizeof fss_rt_inputs[0]);
    }
    for (i = 0; i < count && n < FSS_MAX; i++) {
        fss.vis.idx[n++] = list[i];
    }
    if (fss.tab == 0) {
        for (i = 0; i < FSSR_TARGETS && n < FSS_MAX; i++) {
            fss.vis.idx[n++] = FSS_SYNTH + i;
        }
    }
    fss.vis.n = n;
}

static void fss_remap_submit(const FrontendScreen* s)
{
    int slot;
    int ready = fcr_ready();
    Ui_SetRows(fss.h, fss.vis.n);
    for (slot = 0; slot < fss.vis.n; slot++) {
        int idx = fss.vis.idx[slot];
        if (idx < FSS_SYNTH) {
            fss_submit_row(fss.h, slot, &s->items[idx], idx);
        } else {
            int t = idx - FSS_SYNTH;
            char id[12], help[128];
            fss_row_id(idx, id);
            sprintf(help, "Press A, then the controller input that should act as %s. START+B cancels. X switches between Swap and Also.", fcr_targets[t]);
            Ui_SetRow(fss.h, slot, id, fcr_targets[t], help, FSS_VK_TEXT);
            Ui_SetRowVal(fss.h, slot, 0, 0, 0, 0, ready ? 0 : FSS_IF_DISABLED);
            Ui_SetRowText(fss.h, slot, fssr.bind[t], ready ? "" : "Connect a controller to this port first.", "GAME INPUTS");
        }
    }
}

/* a remap event the table does not decide: A on an input row starts a capture for it, X switches between Swap and Also */
static bool fss_remap_event(int ev, int slot, int arg)
{
    if (fss.kind != FSS_K_REMAP || fss.tab != 0) {
        return false;
    }
    if (ev == FSS_EV_ACCEPT && slot >= 0 && slot < fss.vis.n && fss.vis.idx[slot] >= FSS_SYNTH) {
        int t = fss.vis.idx[slot] - FSS_SYNTH;
        if (fssr_bind(&fssr, t, fcr_also)) {
            fcr_target = t; /* what the legacy Game Input row held */
            sfxForward();
        } else {
            sfxBack(); /* greyed out: the reason is on the row */
        }
        return true;
    }
    if (ev == FSS_EV_ALT && arg == 'X') {
        fcr_also = !fcr_also;
        sfxMove();
        Ui_SetNote(fss.h, fcr_also ? "Also: the new input is added; the old one stays." : "Swap: the new input takes the place of the old one.", 3);
        return true;
    }
    return false;
}

static void fss_event(int ev, int slot, int arg)
{
    switch (ev) {
    case FSS_EV_FOCUS:
        sfxMove();
        break;
    case FSS_EV_TAB:
        fss_set_tab(arg);
        break;
    case FSS_EV_BACK:
        fss_back();
        break;
    case FSS_EV_CHANGE:
    case FSS_EV_ACCEPT:
    case FSS_EV_ALT:
        if (fss_remap_event(ev, slot, arg)) {
            break;
        }
        if (ev != FSS_EV_ALT) {
            fss_play(fss_table_event(fe.screen, &fss.vis, slot, ev, arg));
        }
        break;
    default:
        break;
    }
}

/* the game's menu input, every port merged, as the host's intents (the legacy block read the same bits one action per frame) */
static void fss_feed_pad(void)
{
    u32 in = mn_80229624(4);
    if (in & MenuInput_Up) {
        Ui_SetIntent(fss.h, FSS_IN_MOVE, FSS_DIR_UP);
    }
    if (in & MenuInput_Down) {
        Ui_SetIntent(fss.h, FSS_IN_MOVE, FSS_DIR_DOWN);
    }
    if (in & MenuInput_Left) {
        Ui_SetIntent(fss.h, FSS_IN_MOVE, FSS_DIR_LEFT);
    }
    if (in & MenuInput_Right) {
        Ui_SetIntent(fss.h, FSS_IN_MOVE, FSS_DIR_RIGHT);
    }
    if (in & MenuInput_Confirm) {
        Ui_SetIntent(fss.h, FSS_IN_ACCEPT, 0);
    }
    if (in & MenuInput_Back) {
        Ui_SetIntent(fss.h, FSS_IN_BACK, 0);
    }
    if (in & MenuInput_LTrigger) {
        Ui_SetIntent(fss.h, FSS_IN_PAGE, -1);
    }
    if (in & MenuInput_RTrigger) {
        Ui_SetIntent(fss.h, FSS_IN_PAGE, 1);
    }
    if (in & MenuInput_XButton) {
        Ui_SetIntent(fss.h, FSS_IN_ALT, 'X');
    }
    if (in & MenuInput_YButton) {
        Ui_SetIntent(fss.h, FSS_IN_ALT, 'Y');
    }
}

/* the legacy notice line ("Saved.", "Controllers recalibrated...") becomes the host's note, once */
static void fss_forward_note(void)
{
    if (fe_ol_note[0] != '\0') {
        if (fss.kind == FSS_K_ONLINE && fe_ol_note_bad && strlen(fe_ol_note) > 60) {
            /* a server sentence or a room closing is too long for the corner note: a dialog says it whole, and OK dismisses it */
            Ui_SetDialog(fss.h, "CONNECTION PROBLEM", fe_ol_note, "OK", "", 1);
            fss.note_dialog = true;
        } else {
            /* AT_NOTE_ERR for a failed connection, AT_NOTE_WARN for another warning, AT_NOTE_INFO (gw_ui_parts.h) */
            Ui_SetNote(fss.h, fe_ol_note, fe_ol_note_bad ? (fss.kind == FSS_K_ONLINE ? 2 : 1) : 3);
        }
        fe_ol_note[0] = '\0';
    }
}

/* the one place the remap layer's frame runs: fcr_frame, moved (gmfrontend_atlas_remap.h). Called from fss_frame, which gm_Scene_Frontend_OnFrame calls AFTER its
 * Controls_Menu heartbeat. Returns true while input is frozen (a capture, or the input that ended it still held). */
static bool fss_remap_frame(void)
{
    return fssr_frame(&fssr) != 0;
}

/* the editor's note when a result changes, and its explainer: the live tester ("what the game receives") and, while capturing, the prompt */
static void fss_remap_view(void)
{
    char tester[FE_STR];
    if (fssr.result != fss.last_result) {
        fss.last_result = fssr.result;
        if (fssr.result > 1) {
            Ui_SetNote(fss.h, fssr_message_for(fssr.result), fssr_message_kind(fssr.result));
        }
    }
    Controls_Tester(tester, sizeof tester);
    Ui_SetExplain(fss.h, 0, 0, Controls_Capturing() ? fssr_message_for(1) : 0, tester, "");
}

/* The key hints follow the focused row's kind (fss_hints_for): the logical glyphs, and only what the row can do. The remap editor's INPUTS tab names its X mode. Sent when
 * the set changes. An action that shows a status is an action; a slider with no set is a readout. */
static void fss_hints(const FrontendScreen* s)
{
    char spec[64];
    int slot = Ui_SetFocused(fss.h), vk = FSS_VK_NONE, disabled = 0, tabs = fss_ntabs() > 0 ? 1 : 0, sig;
    if (fss.kind == FSS_K_REMAP && fss.tab == 0 && slot >= 0 && slot < fss.vis.n && fss.vis.idx[slot] >= FSS_SYNTH) {   /* an input row; the Controller choice rows get the ordinary hints */
        sig = 1000 + (fcr_also ? 1 : 0);
        if (sig != fss.hint_sig) {
            Ui_SetKeys(fss.h, fcr_also ? "A:Rebind,X:Also,L:Page,B:Back" : "A:Rebind,X:Swap,L:Page,B:Back");
            fss.hint_sig = sig;
        }
        return;
    }
    if (slot >= 0 && slot < fss.vis.n && fss.vis.idx[slot] < FSS_SYNTH) {
        const FrontendItem* it = &s->items[fss.vis.idx[slot]];
        vk = it->kind == FE_ACTION ? FSS_VK_NONE : (it->kind == FE_SLIDER && it->set == 0) ? FSS_VK_TEXT : fss_vkind(it);
        disabled = (it->enabled != 0 && !it->enabled()) ? 1 : 0;
    }
    sig = vk * 4 + disabled * 2 + tabs;
    if (sig != fss.hint_sig) {
        fss_hints_for(vk, disabled, tabs, spec, sizeof spec);
        Ui_SetKeys(fss.h, spec);
        fss.hint_sig = sig;
    }
}

/* One frame of the Atlas settings. True when it ran (the caller returns: nothing of the legacy rows is built or read). */
static bool fss_frame(void)
{
    const FrontendScreen* s = fe.screen;
    bool frozen;
    int slot = 0, arg = 0, ev;
    if (!fss_wants(s)) {
        if (fss.h >= 0) {
            fss_close();
        }
        return false;
    }
    if (fss.h >= 0 && fss.kind != fss_kind_of(s)) {
        fss_close(); /* another kind of screen (the how-to page, the erase screen, the remap editor): the host's screen is reopened */
    }
    if (fss.h < 0 && fss_open(s) < 0) {
        return false;
    }
    if (fss.kind == FSS_K_TABS) {
        fss.tab = fss_tab_of_page(fss_page_of(s));
    }
    fss_erase_answer();
    if (fss.note_dialog && Ui_SetDialogAnswer(fss.h) != 0) {
        fss.note_dialog = false;
    }
    if (fss.kind == FSS_K_REMAP) {
        frozen = fss_remap_frame(); /* the heartbeat, START+B, the result, the release wait */
        fssr_bindings(&fssr);
    } else {
        frozen = fte.on;
        if (frozen) {
            fte_frame(); /* the keys are text until it ends; the host swallows every event meanwhile */
        }
    }
    Ui_Freeze(fss.h, frozen ? 1 : 0);
    if (fss.kind == FSS_K_REMAP) {
        fss_remap_vis();
        fss_remap_submit(s);
    } else {
        fss_visible(s, &fss.vis);
        fss_table_submit(fss.h, s, &fss.vis);
    }
    if (fss_ntabs() > 0) {
        Ui_SetTabs(fss.h, fss_ntabs(), fss_names(), fss.tab);
    }
    if (fss.refocus || fss.shown_tab != fss.tab) {
        char id[12];
        int want = fss_ntabs() > 0 ? fss.tab_focus[fss.tab] : -1;
        id[0] = '\0';
        if (want >= 0) {
            fss_row_id(want, id);
        }
        Ui_SetFocus(fss.h, id); /* a tab starts on its first row, and returns to where it was left */
        fss.refocus = false;
        fss.shown_tab = fss.tab;
    }
    if (fss.want_label != NULL) {
        int i;
        for (i = 0; i < fss.vis.n; i++) {
            const char* l = fss.vis.idx[i] < FSS_SYNTH ? s->items[fss.vis.idx[i]].label : NULL;
            if (l != NULL && strcmp(l, fss.want_label) == 0) {
                char id[12];
                fss_row_id(fss.vis.idx[i], id);
                Ui_SetFocus(fss.h, id);
                break;
            }
        }
        fss.want_label = NULL;
    }
    if (!frozen && fe.frames >= 4) { /* let the button that brought us here go */
        fss_feed_pad();
    }
    while ((ev = Ui_SetPoll(fss.h, &slot, &arg)) != 0) {
        fss_event(ev, slot, arg);
        if (fe.screen != s) {
            break; /* the screen changed: what is left in the queue was for the old one */
        }
    }
    if (fss.h >= 0 && fe.screen == s) {
        if (fss.kind == FSS_K_REMAP) {
            fss_remap_view();
        } else {
            Ui_SetExplain(fss.h, 0, 0, 0, 0, "");
        }
        fss_hints(s);
        fss_forward_note();
    }
    return true;
}

/* the scene ends: the host closes the screen with it (Ui_SceneExit); a capture in progress ends too */
static void fss_exit(void)
{
    if (fss.h >= 0 && fss.kind == FSS_K_REMAP) {
        fssr_close(&fssr);
    }
    fss.h = -1;
    fss.shown_tab = -1;
    fss.erase_cat = -1;
    fss.want_label = NULL;
}
