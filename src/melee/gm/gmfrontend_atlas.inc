/* gmfrontend_atlas.inc - the Atlas adapter for the menu tree (spec 6.4, 6.7; plan 2026-10-06-atlas-step2).
 * The tree, the cursor, confirm, back and every pending action stay the legacy ones; this file only describes the current
 * FeMenu to the host each frame (the Ui_* shims, pc/platform/gw_script_ui.inc) and turns the host's events into fm_go,
 * fm_confirm and fm_back. MELEE_ATLAS=0 (Ui_Ready() == 0) leaves the legacy menus untouched.
 *
 * THE CURSOR IS THE LEGACY CURSOR. For the menu's own items the adapter never keeps a position: a focus event from the host
 * becomes fm_go(index), and the focus the host is told is fm.cursor. So fm_do_pending, fm_mirror_flow, fm_position_for, the
 * FMF_PORT port and the native back-out behave exactly as they do without Atlas. Only the two places the legacy menu has
 * no cursor for are kept here: a registry entry (a mod's tile after the menu's own items) and the More strip.
 *
 * INPUT. Under Atlas the legacy pad block is skipped by construction (fa_frame returns true), and so is the legacy mouse block:
 * the host reads the mouse and the keyboard for the top screen itself, and the game's menu input (every port's, merged by
 * mn_80229624) comes in through Ui_Intent. fms_menu_bits() is mouse-only (a right click, a clicked hint glyph), so it is
 * NOT fed in: the host's own mouse path already makes those events, and feeding both would act twice. */

/* The scene policy's own table (pc/platform/gw_ui_policy.c) names scene kinds by number: the title is 0. */
_Static_assert(GS_TITLE == 0, "gw_ui_policy.c AT_SCENE_TITLE must be GS_TITLE");

/* REPLACE has no user in step 2 (the title is an OVERLAY: retail's own exit logic, attract loop and debug path keep running).
 * This is the extension point: a scene kind that Atlas stands in for returns its own on_enter/on_frame pair here, step 8 first. */
GameScene* gmFrontend_AtlasStandIn(u8 kind)
{
    (void) kind;
    return NULL;
}

enum { FA_PRIMARY_TILES = 3 };                                           /* AT_PRIMARY_TILES (gw_ui_screen.h) */
enum { FA_EV_MOVE = 1, FA_EV_FOCUS = 2, FA_EV_ACCEPT = 3, FA_EV_BACK = 4, FA_EV_ALT = 5 }; /* AT_EV_* (gw_ui_input.h) */
enum { FA_DIR_LEFT = 1, FA_DIR_RIGHT = 2, FA_DIR_UP = 3, FA_DIR_DOWN = 4 };               /* AT_DIR_* (gw_ui_focus.h) */
enum { FA_AT_NATIVE, FA_AT_ENTRY, FA_AT_MORE };                          /* where the focus is: the menu's items, a registry tile, More */

static struct {
    int where;          ///< FA_AT_*
    int idx;            ///< FA_AT_ENTRY: the entry; FA_AT_MORE: the More item (the native items use fm.cursor)
    const char* shown;  ///< the atlas_id last submitted (closed when the menu changes)
    bool handled;       ///< fa_frame ran this frame (so the host's queue was read)
} fat;

static bool fa_on(const FeMenu* m)
{
    return m != NULL && m->atlas_id != NULL && Ui_Ready() != 0;
}

/* a new menu opened: the focus starts on its own cursor; coming back from Collection or Data lands on that More item */
static void fa_open_reset(const FeMenu* m, int sel)
{
    fat.where = FA_AT_NATIVE;
    fat.idx = 0;
    if (m->kind == MENU_KIND_MAIN && Ui_Ready()) {
        if (sel == SEL_MAIN_TOY) {
            fat.where = FA_AT_MORE;
            fat.idx = 0;
        } else if (sel == SEL_MAIN_DATA) {
            fat.where = FA_AT_MORE;
            fat.idx = 1;
        }
    }
}

static int fa_chapter_of(const FeMenu* m)
{
    switch (m->kind) {
    case MENU_KIND_1P:
    case MENU_KIND_REG:
    case MENU_KIND_STADIUM:
    case MENU_KIND_MULTI_VS:
    case MENU_KIND_EVENT:
        return 1;
    case MENU_KIND_VS:
    case MENU_KIND_SPECIAL:
        return 2;
    case MENU_KIND_SETTINGS:
        return 5;
    default:
        return 0; /* the main menu and the More side (Collection, Data, Records) */
    }
}

/* the item's id: its label in lower case with every other run of characters a dash ("home-run-contest"); unique per menu */
static const char* fa_item_id(const FeMenuItem* it)
{
    static char id[24];
    int n = 0, i;
    bool dash = false;
    for (i = 0; it->label[i] != '\0' && n < 23; i++) {
        char c = it->label[i];
        if (c >= 'A' && c <= 'Z') {
            c = (char) (c + 32);
        }
        if ((c >= 'a' && c <= 'z') || (c >= '0' && c <= '9')) {
            id[n++] = c;
            dash = false;
        } else if (!dash && n > 0) {
            id[n++] = '-';
            dash = true;
        }
    }
    if (n > 0 && id[n - 1] == '-') {
        n--;
    }
    id[n] = '\0';
    return id;
}

/* A built-in entry owned by its mod: the LAB (the Geno Lab script mod) is tagged MOD like a registry entry. The spec's "moved to the
 * entry's on_entry" is not done: a Lua handler cannot enter GM_LAB without a new gameplay API, and the native action does exactly that. */
static const char* fa_item_tag(const FeMenu* m, const FeMenuItem* it)
{
    return (m->kind == MENU_KIND_1P && it->sel == SEL_1P_LAB) ? "MOD" : "";
}

static const char* fa_numeral(int i)
{
    static const char* const n[] = { "I", "II", "III", "IV", "V", "VI" };
    return i >= 0 && i < 6 ? n[i] : "";
}

static const char* fa_kicker(const FeMenu* m, int i)
{
    static const char* const k[] = { "CHAPTER I", "CHAPTER II", "CHAPTER III", "CHAPTER IV", "CHAPTER V", "CHAPTER VI" };
    return (m->kind == MENU_KIND_MAIN && i >= 0 && i < 6) ? k[i] : m->title;
}

/* the parents of a menu, nearest three, root first: MULTI-MAN MELEE is under MAIN MENU > SOLO > STADIUM */
static void fa_trail(const FeMenu* m)
{
    const char* t[8];
    int n = 0, i;
    const FeMenu* p = m;
    while (p->back_kind != 0xFF && n < 8) {
        p = fm_find(p->back_kind);
        if (p == NULL) {
            break;
        }
        t[n++] = p->title;
    }
    for (i = n > 3 ? 2 : n - 1; i >= 0; i--) {
        Ui_Trail(t[i]);
    }
}

/* The More strip is the main menu's: Collection and Data open their hubs (as the legacy main items do), Credits is the host's own
 * screen. Indices: 0 COLLECTION, 1 DATA, 2 CREDITS. */
static void fa_more_submit(const FeMenu* m)
{
    if (m->kind == MENU_KIND_MAIN) {
        Ui_More("collection", "COLLECTION");
        Ui_More("data", "DATA");
        Ui_More("credits", "CREDITS");
    }
}

static void fa_more(int index)
{
    sfxForward();
    if (index == 0 || index == 1) {
        /* the legacy FA_SUB path: vanilla enters every submenu on its first item */
        fm.pend_kind = 1;
        fm.pend_a = index == 0 ? MENU_KIND_TOY : MENU_KIND_DATA;
        fm.pend_b = 0;
        fm.leave_at = fp.frame + 1;
    } else {
        Ui_OpenCredits();
    }
}

/* the whole screen, every frame: the host replaces its record only when something differs */
static void fa_submit(const FeMenu* m)
{
    int i, ne = Ui_EntryCount(m->atlas_id), fb, fi, shown = fm.nvis;
    const FeMenuItem* f = fm.nvis > 0 ? &m->items[fm.vis[fm.cursor]] : NULL;
    if (!Ui_Begin(m->atlas_id, FA_PRIMARY_TILES, fa_chapter_of(m), m->title)) {
        return;
    }
    Ui_Cols(m->kind == MENU_KIND_MAIN ? 1 : 0);
    fa_trail(m);
    for (i = 0; i < fm.nvis; i++) {
        const FeMenuItem* it = &m->items[fm.vis[i]];
        Ui_Tile(fa_item_id(it), it->label, fa_item_tag(m, it), m->kind == MENU_KIND_MAIN ? fa_numeral(i) : "", 0);
    }
    for (i = 0; i < ne; i++) {
        Ui_Tile(Ui_EntryField(m->atlas_id, i, "id"), Ui_EntryField(m->atlas_id, i, "label"), "MOD", "", 0);
    }
    fa_more_submit(m);
    if (fat.where == FA_AT_NATIVE && f != NULL) {
        Ui_Explain(fa_kicker(m, fm.cursor), f->label, f->desc);
    } else if (fat.where == FA_AT_ENTRY) {
        Ui_Explain("MOD", Ui_EntryField(m->atlas_id, fat.idx, "label"), Ui_EntryField(m->atlas_id, fat.idx, "blurb"));
    }
    Ui_Key('A', "Open");
    Ui_Key('B', m->kind == MENU_KIND_MAIN ? "Title" : "Back");
    if (fat.where == FA_AT_ENTRY) {
        fb = 0;
        fi = shown + (fat.idx < ne ? fat.idx : 0);
    } else if (fat.where == FA_AT_MORE) {
        fb = 1;
        fi = fat.idx;
    } else {
        fb = 0;
        fi = fm.cursor; /* the legacy cursor is the truth */
    }
    Ui_Commit(fb, fi);
}

/* A host event for the screen: the legacy cursor, confirm and back do the work. */
static void fa_event(int type, int block, int index)
{
    const FeMenu* m = fm.menu;
    if (type == FA_EV_FOCUS) {
        if (block == 1) {
            fat.where = FA_AT_MORE;
            fat.idx = index;
        } else if (index < fm.nvis) {
            fat.where = FA_AT_NATIVE;
            fm_go(index); /* the legacy cursor setter: sound, scroll, the kit's own events */
        } else {
            fat.where = FA_AT_ENTRY;
            fat.idx = index - fm.nvis;
        }
    } else if (type == FA_EV_ACCEPT) {
        if (block == 1) {
            fa_more(index);
        } else if (index < fm.nvis) {
            fm_go(index);
            fm_confirm();
        } else {
            sfxForward();
            Ui_EntryActivate(m->atlas_id, index - fm.nvis);
        }
    } else if (type == FA_EV_BACK) {
        fm_back();
    }
}

/* Returns true when Atlas handled this frame's input (the legacy pad and mouse blocks are skipped). `pad` is the game's menu
 * input of every port, merged (mn_80229624). One action per frame, as the legacy path. */
static bool fa_frame(u32 pad)
{
    int type, block, index;
    if (!fa_on(fm.menu)) {
        return false;
    }
    fat.handled = true;
    if (pad & MenuInput_Up) {
        Ui_Intent(FA_EV_MOVE, FA_DIR_UP);
    }
    if (pad & MenuInput_Down) {
        Ui_Intent(FA_EV_MOVE, FA_DIR_DOWN);
    }
    if (pad & MenuInput_Left) {
        Ui_Intent(FA_EV_MOVE, FA_DIR_LEFT);
    }
    if (pad & MenuInput_Right) {
        Ui_Intent(FA_EV_MOVE, FA_DIR_RIGHT);
    }
    if (pad & MenuInput_Confirm) {
        Ui_Intent(FA_EV_ACCEPT, 0);
    }
    if (pad & MenuInput_Back) {
        Ui_Intent(FA_EV_BACK, 0);
    }
    while (Ui_PollEvent(&type, &block, &index)) {
        fa_event(type, block, index);
        if (type == FA_EV_ACCEPT || type == FA_EV_BACK) {
            break; /* one action per frame; what is left in the queue is stale */
        }
    }
    return true;
}

/* every frame, after the legacy frame (a pending action may have changed the menu): show this menu, close the last one */
static void fa_sync(void)
{
    int type, block, index;
    if (!fa_on(fm.menu)) {
        if (fat.shown != NULL) {
            Ui_Close(fat.shown);
            fat.shown = NULL;
        }
        return;
    }
    if (fat.shown != NULL && strcmp(fat.shown, fm.menu->atlas_id) != 0) {
        Ui_Close(fat.shown);
    }
    if (!fat.handled) {
        while (Ui_PollEvent(&type, &block, &index)) {
            /* the menu is not taking input this frame (fading, leaving, first frames): the host's events are not for it */
        }
    }
    fat.handled = false;
    fa_submit(fm.menu);
    fat.shown = fm.menu->atlas_id;
}

static void fa_scene_exit(void)
{
    fat.shown = NULL; /* the host closes the screens with the scene (Ui_SceneExit) */
    fat.handled = false;
}
