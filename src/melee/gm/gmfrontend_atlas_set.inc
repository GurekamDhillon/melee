/* gmfrontend_atlas_set.inc - the Atlas drawing of the SETTINGS pages (Atlas step 5; included by gmfrontend.c after gmfrontend_settings.inc).
 *
 * A DRAWING AND INPUT LAYER over the tables. The pages (gmfrontend_settings.inc, gmfrontend_controls.inc) stay FrontendItem arrays; fe.screen stays the current
 * page (fe_screen_settings[page], fe_screen_remap, the how-to page, the erase screen), so fe_is_settings, the notice rule and the Controls heartbeat's screen test
 * keep working. Each frame this file walks the table (gmfrontend_atlas_table.h: get, visible, enabled and format are evaluated HERE) and hands the host scalars and
 * strings (pc/platform/gw_script_ui_set.inc); the host's events (CHANGE, ACCEPT, BACK, TAB, FOCUS) come back and the walker applies them with the legacy rules.
 * Every change applies at once and persists, as before; environment variables still win over saved values.
 *
 * ONE SCREEN, SIX TABS. VIDEO, AUDIO, CONTROLS, ONLINE, GAME and MODS are tabs of one host screen; the pages are the FSP_* tables (GAME is FSP_GAMEPLAY, and MODS comes
 * last here though FSP_MODS is the enum's fifth). The tab the host shows changes only when this file says so, after it has submitted the new page's rows. The focus is
 * remembered per tab (a table index) and set again by id. The how-to page, the erase screen and the remap editor are screens of their own under the same host.
 *
 * ORDER IN THE FRAME. gm_Scene_Frontend_OnFrame calls Controls_Menu first (a 100 ms heartbeat: the remap editor's capture is cancelled by the runtime when it lapses),
 * then fss_frame(), and fe.screen stays &fe_screen_remap while the editor is open (tools/port/test_menu_heartbeat.py pins both).
 *
 * THE PAD. Menus read the pad pre-remap only in menu modes (the frontend scene is one); the settings read it through mn_80229624(4), the game's own menu input of every
 * port merged, and hand it to the host as intents. The keyboard and the mouse are the host's. A text row being typed (Player Name, Server) freezes the screen.
 *
 * MELEE_ATLAS=0 or missing Atlas fonts: fss_open fails, nothing here runs, and the legacy kit draws the pages exactly as before. */

#include "gmfrontend_atlas_table.h"
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

enum { FSS_K_TABS, FSS_K_REMAP, FSS_K_HOWTO, FSS_K_ERASE };   /* the host's screen kinds (gw_script_ui_set.inc GS_SET_*) */
#define FSS_TABS 6
#define FSS_HAS_REMAP 0 /* the remap editor's Atlas layer: Task 8 */

static const char fss_tab_names[] = "VIDEO,AUDIO,CONTROLS,ONLINE,GAME,MODS";
static const int fss_tab_page[FSS_TABS] = { FSP_VIDEO, FSP_AUDIO, FSP_CONTROLS, FSP_ONLINE, FSP_GAMEPLAY, FSP_MODS };

static struct {
    int h;                 ///< the host's handle, -1 when no Atlas settings screen is up
    int kind;              ///< FSS_K_*
    int tab;               ///< the tab the host shows (kind TABS)
    int shown_tab;         ///< the tab whose rows were submitted last
    int tab_focus[FSS_TABS]; ///< the table index focused in each tab, -1 none
    FssVis vis;            ///< the visible rows of fe.screen as last submitted
    int erase_cat;         ///< the erase operation awaiting the dialog's answer, -1 none
    const char* want_label; ///< a row to focus after the next submit (the return from the Language screen), by label
    bool refocus;          ///< set the focus by id after the next submit (a tab switch)
} fss = { -1, 0, 0, -1, { -1, -1, -1, -1, -1, -1 }, { 0 }, -1, NULL, false };

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
    if (FSS_HAS_REMAP && s == &fe_screen_remap) {
        return FSS_K_REMAP;
    }
    return -1;
}

/* a screen the Atlas settings draw, when Atlas is on */
static bool fss_wants(const FrontendScreen* s)
{
    return s != NULL && Ui_Ready() != 0 && fss_kind_of(s) >= 0;
}

static bool fss_active(void) { return fss.h >= 0; }

static void fss_close(void)
{
    if (fss.h >= 0) {
        Ui_SetClose(fss.h);
    }
    fss.h = -1;
    fss.shown_tab = -1;
    fss.erase_cat = -1;
}

/* the host's screen for fe.screen; -1 when the Atlas screen cannot open (the legacy kit draws the page) */
static int fss_open(const FrontendScreen* s)
{
    int kind = fss_kind_of(s), tab = 0, h;
    const char* t0 = "MAIN MENU";
    const char* t1 = "";
    const char* title = "SETTINGS";
    if (kind == FSS_K_TABS) {
        tab = fss_tab_of_page(fss_page_of(s));
    } else if (kind == FSS_K_HOWTO) {
        t0 = "SETTINGS"; t1 = "CONTROLS"; title = "HOW TO PLAY ONLINE";
    } else if (kind == FSS_K_ERASE) {
        t0 = "SETTINGS"; t1 = "GAME"; title = "ERASE DATA";
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
    {
        int i;
        for (i = 0; i < FSS_TABS; i++) {
            fss.tab_focus[i] = -1;
        }
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

/* a tab asked for (L, R, a click): the page's rows are submitted first, the host shows the tab on the next frame */
static void fss_set_tab(int t)
{
    int slot;
    if (fss.kind != FSS_K_TABS || t < 0 || t >= FSS_TABS || t == fss.tab) {
        return;
    }
    slot = Ui_SetFocused(fss.h);
    if (slot >= 0 && slot < fss.vis.n) {
        fss.tab_focus[fss.tab] = fss.vis.idx[slot]; /* where this tab was left */
    }
    sfxMove();
    fss_switch(&fe_screen_settings[fss_tab_page[t]]);
    fss.refocus = true;
}

/* B: the tabs go back to the menus, the how-to page to CONTROLS, the erase screen to GAME, the remap editor through fcr_back */
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
        fss_play(fss_table_event(fe.screen, &fss.vis, slot, ev, arg));
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
        Ui_SetNote(fss.h, fe_ol_note, fe_ol_note_bad ? 1 : 3); /* AT_NOTE_WARN, AT_NOTE_INFO (gw_ui_screen.h) */
        fe_ol_note[0] = '\0';
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
        fss_close(); /* another kind of screen (the how-to page, the erase screen): the host's screen is reopened */
    }
    if (fss.h < 0 && fss_open(s) < 0) {
        return false;
    }
    if (fss.kind == FSS_K_TABS) {
        fss.tab = fss_tab_of_page(fss_page_of(s));
    }
    fss_erase_answer();
    frozen = fte.on;
    Ui_Freeze(fss.h, frozen ? 1 : 0);
    if (frozen) {
        fte_frame(); /* the keys are text until it ends; the host swallows every event meanwhile */
    }
    fss_visible(s, &fss.vis);
    fss_table_submit(fss.h, s, &fss.vis);
    if (fss.kind == FSS_K_TABS) {
        Ui_SetTabs(fss.h, FSS_TABS, fss_tab_names, fss.tab);
    }
    if (fss.refocus || fss.shown_tab != fss.tab) {
        char id[12];
        int want = fss.kind == FSS_K_TABS ? fss.tab_focus[fss.tab] : -1;
        id[0] = '\0';
        if (want >= 0) {
            fss_id(want, id);
        }
        Ui_SetFocus(fss.h, id); /* a tab starts on its first row, and returns to where it was left */
        fss.refocus = false;
        fss.shown_tab = fss.tab;
    }
    if (fss.want_label != NULL) {
        int i;
        for (i = 0; i < fss.vis.n; i++) {
            const char* l = s->items[fss.vis.idx[i]].label;
            if (l != NULL && strcmp(l, fss.want_label) == 0) {
                char id[12];
                fss_id(fss.vis.idx[i], id);
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
    Ui_SetExplain(fss.h, 0, 0, 0, 0, "");
    Ui_SetKeys(fss.h, "A:Select,L:Page,B:Back");
    fss_forward_note();
    return true;
}

/* the scene ends: the host closes the screen with it (Ui_SceneExit); this only forgets the handle */
static void fss_exit(void)
{
    fss.h = -1;
    fss.shown_tab = -1;
    fss.erase_cat = -1;
    fss.want_label = NULL;
}
