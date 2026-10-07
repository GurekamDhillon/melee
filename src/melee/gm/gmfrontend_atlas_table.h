/* gmfrontend_atlas_table.h - the Atlas TABLE WALKER: one FrontendScreen (a settings page, MATCH SETUP, the remap profile and option rows) becomes the rows of one
 * tab of the Atlas screen, and the host's events become the legacy rules.
 *
 * The pages are tables of function pointers that live on the game side, so the walker evaluates them HERE (get, visible, enabled, format) and hands the host
 * scalars and strings through the Ui_* shims (the game calls the shims unprefixed; the host defines gw_Ui_*): Ui_SetRows, Ui_SetRow, Ui_SetRowVal,
 * Ui_SetRowText, Ui_SetRowOpt (no call takes more than eight arguments, and none takes a pointer the host would have to write: Ui_ItemStep returns the new
 * value). The tables stay the source of truth; nothing here is a second copy of a page.
 *
 * The events are the legacy ones, moved: fss_table_event is fe_change (gmfrontend.c) and the Confirm and Back branches of gm_Scene_Frontend_OnFrame. It returns
 * FSS_FX_* bits for the caller to play (sounds, rebuild the visible rows, leave the screen), exactly what the legacy code did inline.
 *
 * No libc and no game headers, so the native test (pc/tests/atlas_walker_test.c) runs it with stand-in shims and the PowerPC check passes with -nostdinc.
 * Include after gmfrontend_items.h. Row ids are TABLE indices ("i7"), so a row that hides and shows keeps its id and the host refocuses by id. */
#ifndef GMFRONTEND_ATLAS_TABLE_H
#define GMFRONTEND_ATLAS_TABLE_H

#define FSS_MAX 64          /* the visible rows of a page: AT_MAX_ITEMS (the MODS page has 41) */
#define FSS_OPTS_MAX 8      /* AT_MAX_OPTS: a choice with more options is shown as text */
#define FSS_TEXT 80         /* FE_STR: the longest value text a format callback may write */

typedef struct { int n, idx[FSS_MAX]; } FssVis;                 /* the visible rows: slot -> table index */
enum { FSS_EV_CHANGE = 1, FSS_EV_ACCEPT = 2, FSS_EV_BACK = 3, FSS_EV_TAB = 4, FSS_EV_FOCUS = 5, FSS_EV_ALT = 6 };
enum { FSS_FX_MOVE = 1, FSS_FX_BACK = 2, FSS_FX_FORWARD = 4, FSS_FX_REBUILD = 8, FSS_FX_CONTINUE = 16, FSS_FX_LEAVE = 32, FSS_FX_BUMP = 64 };
/* the value kinds and row flags the host knows (AT_VAL_* and AT_ITEM_*, gw_ui_screen.h: gw_script_ui_set.inc asserts the numbers) */
enum { FSS_VK_NONE = 0, FSS_VK_TOGGLE = 1, FSS_VK_CHOICE = 2, FSS_VK_SLIDER = 3, FSS_VK_TEXT = 4 };
enum { FSS_IF_A_STEPS = 1, FSS_IF_RO = 2, FSS_IF_DISABLED = 8 };
/* what the adapter feeds the host's intents with (AT_EV_* and AT_DIR_*: the host test asserts the numbers) */
enum { FSS_IN_MOVE = 1, FSS_IN_ACCEPT = 3, FSS_IN_BACK = 4, FSS_IN_ALT = 5, FSS_IN_PAGE = 6 };
enum { FSS_DIR_LEFT = 1, FSS_DIR_RIGHT = 2, FSS_DIR_UP = 3, FSS_DIR_DOWN = 4 };
enum { FSS_K_TABS, FSS_K_REMAP, FSS_K_HOWTO, FSS_K_ERASE, FSS_K_RULES, FSS_K_MORERULES, FSS_K_MATCH, FSS_K_MODS };   /* the host's screen kinds (gw_script_ui_set.inc GS_SET_*: the host test asserts the numbers) */

/* the host's shims */
extern void Ui_SetRows(int h, int n);
extern void Ui_SetRow(int h, int slot, const char* id, const char* label, const char* help, int vkind);
extern void Ui_SetRowVal(int h, int slot, int vmin, int vmax, int vstep, int value, unsigned flags);
extern void Ui_SetRowText(int h, int slot, const char* text, const char* reason, const char* group);
extern void Ui_SetRowOpt(int h, int slot, int k, const char* text);
extern int Ui_ItemStep(int vkind, int vmin, int vmax, int vstep, int cur, int dir);   /* the new value; cur when nothing changed (= at_item_apply) */

static void fss_visible(const FrontendScreen* s, FssVis* v)
{
    int i;
    v->n = 0;
    for (i = 0; i < s->n_items && v->n < FSS_MAX; i++) {
        if (s->items[i].visible == 0 || s->items[i].visible()) {
            v->idx[v->n++] = i;
        }
    }
}

static int fss_vkind(const FrontendItem* it)
{
    switch (it->kind) {
    case FE_TOGGLE:
        return FSS_VK_TOGGLE;
    case FE_CHOICE:
        return FSS_VK_CHOICE;
    case FE_SLIDER:
        return it->set != 0 ? FSS_VK_SLIDER : FSS_VK_TEXT; /* a slider with no set is a readout: text (the kit's FKW_READOUT rule) */
    default:
        return it->format != 0 ? FSS_VK_TEXT : FSS_VK_NONE; /* an action with a status shows it; else no value */
    }
}

/* "i" and the decimal table index */
static void fss_id(int n, char* id)
{
    char t[8];
    int d = 0, k;
    id[0] = 'i';
    if (n <= 0) {
        t[d++] = '0';
    }
    while (n > 0 && d < 7) {
        t[d++] = (char) ('0' + n % 10);
        n /= 10;
    }
    for (k = 0; k < d; k++) {
        id[1 + k] = t[d - 1 - k];
    }
    id[1 + d] = 0;
}

/* a choice's options when they fit the host's record (otherwise the walker shows the text) */
static int fss_opts(const FrontendItem* it)
{
    int n = it->max - it->min + 1;
    return (it->options != 0 && n > 0 && n <= FSS_OPTS_MAX) ? n : 0;
}

/* One row of a table into the host's slot: values, text, flags, options. `table_index` names the row ("i7"). The remap editor submits some rows through this and
 * builds its own for the rest. */
static void fss_submit_row(int h, int slot, const FrontendItem* it, int table_index)
{
    int k;
    char id[12], text[FSS_TEXT];
    unsigned flags = 0;
    int vk = fss_vkind(it), value = it->get != 0 ? it->get() : 0, nopts = fss_opts(it);
    const char* reason = "";
    fss_id(table_index, id);
    text[0] = 0;
    if (it->format != 0 && (vk == FSS_VK_SLIDER || vk == FSS_VK_TEXT || (vk == FSS_VK_CHOICE && it->options == 0))) {
        it->format(value, text);
    } else if (vk == FSS_VK_CHOICE && it->options != 0 && nopts == 0) {
        int idx = value - it->min, n = it->max - it->min + 1; /* more options than the host holds: the text is ours */
        const char* o = (idx >= 0 && idx < n) ? it->options[idx] : 0;
        for (k = 0; o != 0 && o[k] != 0 && k < FSS_TEXT - 1; k++) {
            text[k] = o[k];
        }
        text[k < FSS_TEXT ? k : FSS_TEXT - 1] = 0;
    }
    if (it->kind == FE_SLIDER && it->set != 0) {
        flags |= FSS_IF_A_STEPS; /* A steps a slider (+1 step): the legacy Confirm branch */
    }
    if (it->kind == FE_SLIDER && it->set == 0) {
        flags |= FSS_IF_RO; /* a readout */
    }
    if (it->enabled != 0 && !it->enabled()) {
        flags |= FSS_IF_DISABLED;
        if (it->off_reason != 0) {
            reason = it->off_reason;
        }
    }
    Ui_SetRow(h, slot, id, it->label != 0 ? it->label : "", it->help != 0 ? it->help : "", vk);
    Ui_SetRowVal(h, slot, it->min, it->max, it->step, vk == FSS_VK_TOGGLE ? (value != 0) : value, flags);
    Ui_SetRowText(h, slot, text, reason, it->group != 0 ? it->group : "");
    if (vk == FSS_VK_CHOICE && nopts > 0) {
        for (k = 0; k < nopts; k++) {
            Ui_SetRowOpt(h, slot, k, it->options[k] != 0 ? it->options[k] : "");
        }
    }
}

/* the whole page, every frame: values, text, flags. The host replaces its record only when something differs. */
static void fss_table_submit(int h, const FrontendScreen* s, const FssVis* v)
{
    int slot;
    Ui_SetRows(h, v->n);
    for (slot = 0; slot < v->n; slot++) {
        fss_submit_row(h, slot, &s->items[v->idx[slot]], v->idx[slot]);
    }
}

/* fe_change: a toggle flips, a slider adds a step and clamps, a choice wraps. Returns the FX bits (0 when nothing happened). */
static unsigned fss_change(const FrontendItem* it, int dir)
{
    int vk = fss_vkind(it), v0, v1;
    if (it->get == 0 || it->set == 0 || (vk != FSS_VK_TOGGLE && vk != FSS_VK_CHOICE && vk != FSS_VK_SLIDER)) {
        return 0;
    }
    v0 = it->get();
    v1 = Ui_ItemStep(vk, it->min, it->max, it->step, v0, dir);
    if (v1 == v0) {
        return vk == FSS_VK_SLIDER ? FSS_FX_BUMP : 0; /* held at an end: the kit's "can't go further" */
    }
    it->set(v1);
    return FSS_FX_MOVE | FSS_FX_REBUILD; /* a value can show or hide other rows */
}

/* A host event for the row in `slot`: the legacy rules. FSS_EV_BACK is the caller's to interpret (the remap editor and the how-to page have their own Back). */
static unsigned fss_table_event(const FrontendScreen* s, const FssVis* v, int slot, int ev, int arg)
{
    const FrontendItem* it;
    if (ev == FSS_EV_BACK) {
        return FSS_FX_BACK | FSS_FX_LEAVE;
    }
    if (slot < 0 || slot >= v->n || v->idx[slot] < 0 || v->idx[slot] >= s->n_items) {
        return 0;
    }
    it = &s->items[v->idx[slot]];
    if (ev == FSS_EV_CHANGE) {
        return fss_change(it, arg < 0 ? -1 : 1);
    }
    if (ev == FSS_EV_ACCEPT) {
        if (it->enabled != 0 && !it->enabled()) {
            return FSS_FX_BACK | FSS_FX_BUMP; /* greyed out: no call; the reason is on the row */
        }
        if (it->kind == FE_ACTION) {
            if (it->action == FE_DO_CALL) {
                if (it->call != 0) {
                    it->call();
                }
                return FSS_FX_FORWARD | FSS_FX_REBUILD;
            }
            if (it->action == FE_DO_CONTINUE) {
                return FSS_FX_FORWARD | FSS_FX_CONTINUE;
            }
            return FSS_FX_BACK | FSS_FX_LEAVE;
        }
        if (it->call != 0) {
            it->call(); /* e.g. the code field: A moves to the next slot */
            return FSS_FX_FORWARD | FSS_FX_REBUILD;
        }
        return fss_change(it, 1);
    }
    return 0;
}

/* The key hints of the focused row, as the string Ui_SetKeys takes ("A:Change,L:Page,B:Back"): a function of the row's kind only. The glyphs are the LOGICAL buttons
 * (spec section 9): a remap that moves a logical button to another physical one does not change a hint, and no hint names a physical button. A changes a toggle, a choice
 * or a slider (the legacy Confirm rule), selects an action, and is not offered on a readout or a disabled row. L pages the tabs; a screen with no tabs has no L. B is always Back.
 * `vkind` is FSS_VK_*: pass FSS_VK_TEXT for a readout, FSS_VK_NONE for an action (even one that shows a status). */
static void fss_hints_for(int vkind, int disabled, int tabs, char* out, int cap)
{
    const char* a = "";
    const char* parts[3];
    int np = 0, n = 0, i, k;
    if (cap <= 0) {
        return;
    }
    if (!disabled) {
        if (vkind == FSS_VK_TOGGLE || vkind == FSS_VK_CHOICE || vkind == FSS_VK_SLIDER) {
            a = "A:Change";
        } else if (vkind == FSS_VK_NONE) {
            a = "A:Select";
        }
    }
    if (a[0] != 0) {
        parts[np++] = a;
    }
    if (tabs) {
        parts[np++] = "L:Page";
    }
    parts[np++] = "B:Back";
    for (i = 0; i < np; i++) {
        if (i > 0 && n < cap - 1) {
            out[n++] = ',';
        }
        for (k = 0; parts[i][k] != 0 && n < cap - 1; k++) {
            out[n++] = parts[i][k];
        }
    }
    out[n] = 0;
}

#endif
