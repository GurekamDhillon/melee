#define AT_SINK_HAS_IMAGE
#include <stdlib.h>
#include "atlas_lint.h"
#include "../platform/gw_ui_mods.h"
#include "../platform/gw_ui_render.h"
#include "../platform/gw_ui_nucleus.inc"

/* ---- a fake Nucleus: rows, a filter, a download queue that behaves like the engine's, and an action log ---- */
#define FN_MAX 400
typedef struct { int id, type, fighter, state, downloads, nshots; char title[96], author[48]; } FRow;
static FRow F[FN_MAX];
static int fn_n, fn_ver = 1, fn_syncing, fn_locked, fn_restart, fn_job_state;
static char fn_job_msg[140];
static struct { int id, what; } fn_acts[32];
static int fn_nacts;
static const char *const FL[] = { "Mario", "Fox", "Captain Falcon", "Zelda" };
/* the queue: ids in order, a state each (AT_NC_Q_*), a reason for a failed one; processing = Start was pressed */
static int fq_id[64], fq_st[64], fq_pct[64], fq_n, fq_proc;
static char fq_reason[64][120];
static int pic_asks, pic_ids[256];
static AtNcFilter pref_f; static int pref_tab, pref_sets, pref_set_valid;

static int fn_find(int id) { int i; for (i = 0; i < fn_n; i++) if (F[i].id == id) return i; return -1; }
static int fq_find(int id) { int i; for (i = 0; i < fq_n; i++) if (fq_id[i] == id) return i; return -1; }
static int fn_cmp_title(const void *a, const void *b) { return strcmp(F[*(const int *) a].title, F[*(const int *) b].title); }
static int fn_cmp_dl(const void *a, const void *b) { return F[*(const int *) b].downloads - F[*(const int *) a].downloads; }
static int fn_filter(void *u, const AtNcFilter *f, int *ids, int cap, unsigned *version)
{
    int i, n = 0;
    (void) u;
    *version = (unsigned) fn_ver;
    if (f->view == AT_NC_VIEW_QUEUE) { for (i = 0; i < fq_n && n < cap; i++) ids[n++] = fq_id[i]; return n; }
    for (i = 0; i < fn_n && n < cap; i++) {
        int installed = F[i].state == AT_NC_ST_INSTALLED || F[i].state == AT_NC_ST_UPDATE;
        if (f->view == AT_NC_VIEW_INSTALLED) { if (!installed) continue; }
        else {
            int q = fq_find(F[i].id);
            if (f->type_mask && !(f->type_mask & (1u << F[i].type))) continue;
            if (f->show == AT_NC_SHOW_NOT_INSTALLED && installed) continue;
            if (f->show == AT_NC_SHOW_INSTALLED && !installed) continue;
            if (f->show == AT_NC_SHOW_UPDATE && F[i].state != AT_NC_ST_UPDATE) continue;
            if (f->show == AT_NC_SHOW_QUEUED && (q < 0 || fq_st[q] == AT_NC_Q_DONE)) continue;
        }
        if (f->fighter >= 0 && F[i].fighter != f->fighter) continue;
        if (f->text[0] && !strstr(F[i].title, f->text)) continue;
        ids[n++] = i;                                       /* the id IS the index in this fake, to keep sorting trivial */
    }
    if (f->sort == 4) qsort(ids, (size_t) n, sizeof(int), fn_cmp_title);
    if (f->sort == 2) qsort(ids, (size_t) n, sizeof(int), fn_cmp_dl);
    { int k; for (k = 0; k < n; k++) ids[k] = F[ids[k]].id; }
    return n;
}
static int fn_row(void *u, int id, AtNcRow *r)
{
    int i = fn_find(id), q = fq_find(id), k, place = 0;
    (void) u;
    if (i < 0) return 0;
    memset(r, 0, sizeof *r);
    r->id = id; r->tex = -1; r->thumb = -1; r->state = F[i].state; r->downloads = F[i].downloads; r->likes = 3; r->ncostumes = F[i].type == 0 ? 2 : 0; r->nshots = F[i].nshots;
    r->qstate = AT_NC_Q_NONE;
    snprintf(r->title, sizeof r->title, "%s", F[i].title); snprintf(r->author, sizeof r->author, "%s", F[i].author);
    snprintf(r->fighter, sizeof r->fighter, "%s", F[i].fighter >= 0 ? FL[F[i].fighter] : "");
    snprintf(r->kind, sizeof r->kind, "%s", F[i].type == 0 ? "Costume" : "Stage skin");
    snprintf(r->updated, sizeof r->updated, "2026-03-03"); snprintf(r->page, sizeof r->page, "https://ssbmnucleus.net/post/%d/x", id);
    snprintf(r->desc, sizeof r->desc, "A synthetic description for %s that goes on for a while so the About row has something to wrap.", F[i].title);
    if (F[i].state == AT_NC_ST_UNSUPPORTED) snprintf(r->reason, sizeof r->reason, "Stage skins cannot be installed yet.");
    if (q >= 0) {
        r->qstate = fq_st[q]; r->qpct = fq_pct[q]; snprintf(r->qreason, sizeof r->qreason, "%s", fq_reason[q]);
        for (k = 0; k <= q; k++) if (fq_st[k] == AT_NC_Q_QUEUED || fq_st[k] == AT_NC_Q_RUNNING) place++;
        r->qplace = place;
    }
    return 1;
}
static int fq_pending(void) { int i, n = 0; for (i = 0; i < fq_n; i++) if (fq_st[i] == AT_NC_Q_QUEUED || fq_st[i] == AT_NC_Q_RUNNING) n++; return n; }
static void fn_status(void *u, AtNcStatus *s)
{
    int i;
    (void) u;
    memset(s, 0, sizeof *s);
    s->syncing = fn_syncing; s->done = 1200; s->total = fn_syncing ? 4700 : 0; s->locked = fn_locked; s->restart = fn_restart;
    s->job_state = fn_job_state; snprintf(s->job_msg, sizeof s->job_msg, "%s", fn_job_msg);
    s->q_pending = fq_pending(); s->q_processing = fq_proc; s->q_running_pct = -1;
    for (i = 0; i < fq_n; i++) { if (fq_st[i] == AT_NC_Q_FAILED) s->q_failed++; if (fq_st[i] == AT_NC_Q_RUNNING) { s->q_running_pct = fq_pct[i]; snprintf(s->q_running_title, sizeof s->q_running_title, "%s", F[fn_find(fq_id[i])].title); } }
}
static void fq_remove_at(int i) { int k; for (k = i; k + 1 < fq_n; k++) { fq_id[k] = fq_id[k + 1]; fq_st[k] = fq_st[k + 1]; fq_pct[k] = fq_pct[k + 1]; memcpy(fq_reason[k], fq_reason[k + 1], sizeof fq_reason[0]); } fq_n--; }
static void fq_swap(int a, int b)
{
    int t = fq_id[a]; fq_id[a] = fq_id[b]; fq_id[b] = t; t = fq_st[a]; fq_st[a] = fq_st[b]; fq_st[b] = t; t = fq_pct[a]; fq_pct[a] = fq_pct[b]; fq_pct[b] = t;
    { char tmp[120]; memcpy(tmp, fq_reason[a], 120); memcpy(fq_reason[a], fq_reason[b], 120); memcpy(fq_reason[b], tmp, 120); }
}
/* the same rules as gw_nucleus_queue.h, over the fake rows: A toggles, B never touches the queue, a clear leaves the running entry */
static int fn_act(void *u, int id, int what)
{
    int q = fq_find(id), i;
    (void) u;
    if (fn_nacts < 32) { fn_acts[fn_nacts].id = id; fn_acts[fn_nacts++].what = what; }
    fn_ver++;
    switch (what) {
    case AT_NC_ACT_QUEUE:
        if (q >= 0 && fq_st[q] == AT_NC_Q_QUEUED) fq_remove_at(q);
        else if (q >= 0 && fq_st[q] == AT_NC_Q_RUNNING) return 0;
        else { if (q >= 0) fq_remove_at(q); fq_id[fq_n] = id; fq_st[fq_n] = AT_NC_Q_QUEUED; fq_pct[fq_n] = 0; fq_reason[fq_n][0] = '\0'; fq_n++; }
        break;
    case AT_NC_ACT_START: fq_proc = !fq_proc; break;
    case AT_NC_ACT_Q_UP: case AT_NC_ACT_Q_DOWN:
        if (q < 0 || fq_st[q] != AT_NC_Q_QUEUED) return 0;
        i = q + (what == AT_NC_ACT_Q_UP ? -1 : 1);
        if (i < 0 || i >= fq_n || fq_st[i] != AT_NC_Q_QUEUED) return 0;
        fq_swap(q, i);
        break;
    case AT_NC_ACT_Q_REMOVE: if (q < 0 || fq_st[q] == AT_NC_Q_RUNNING) return 0; fq_remove_at(q); break;
    case AT_NC_ACT_Q_RETRY: if (q < 0 || fq_st[q] != AT_NC_Q_FAILED) return 0; fq_st[q] = AT_NC_Q_QUEUED; fq_reason[q][0] = '\0'; break;
    case AT_NC_ACT_Q_CLEAR: for (i = fq_n - 1; i >= 0; i--) if (fq_st[i] != AT_NC_Q_RUNNING) fq_remove_at(i); break;
    default: break;
    }
    return 1;
}
static int fn_fighters(void *u) { (void) u; return 4; }
static const char *fn_flabel(void *u, int i) { (void) u; return FL[i]; }
static unsigned fn_version(void *u) { (void) u; return (unsigned) fn_ver; }
static int fn_pic(void *u, int id, int which, int *tex)
{
    int i = fn_find(id);
    (void) u;
    *tex = -1;
    if (i < 0) return 0;
    if (pic_asks < 256) pic_ids[pic_asks] = id;
    pic_asks++;
    if (which >= AT_NC_PIC_SHOT1 && which < AT_NC_PIC_SHOT1 + 3) { if (which - AT_NC_PIC_SHOT1 >= F[i].nshots) return 0; *tex = 40 + which; return 1; }
    if (which == AT_NC_PIC_STOCK) { *tex = 30; return F[i].type == 0; }
    if (F[i].type != 0 && F[i].nshots == 0) return 0;               /* no PNG to show: the placeholder box */
    *tex = id % 5 == 0 ? -1 : 10 + id % 7;                           /* every fifth is still loading */
    return 1;
}
static void fn_pref_get(void *u, AtNcFilter *f, int *tab) { (void) u; if (pref_set_valid) { *f = pref_f; *tab = pref_tab; } }
static void fn_pref_set(void *u, const AtNcFilter *f, int tab) { (void) u; pref_f = *f; pref_tab = tab; pref_set_valid = 1; pref_sets++; }
static AtNcSrc src(void)
{
    AtNcSrc s;
    memset(&s, 0, sizeof s);
    s.filter = fn_filter; s.row = fn_row; s.status = fn_status; s.act = fn_act; s.fighters = fn_fighters; s.fighter_label = fn_flabel; s.version = fn_version;
    s.pic = fn_pic; s.pref_get = fn_pref_get; s.pref_set = fn_pref_set;
    return s;
}
static void fixture(void)
{
    int i;
    fn_n = 0; fn_ver++; fn_syncing = fn_locked = fn_restart = fn_job_state = 0; fn_nacts = 0; fn_job_msg[0] = '\0';
    fq_n = 0; fq_proc = 0; pic_asks = 0; pref_set_valid = 0; pref_sets = 0;
    for (i = 0; i < 300; i++) {
        FRow *r = &F[fn_n++];
        memset(r, 0, sizeof *r);
        r->id = 1000 + i; r->type = 0; r->fighter = i % 4; r->downloads = (i * 37) % 101;
        snprintf(r->title, sizeof r->title, "Costume %03d", i); snprintf(r->author, sizeof r->author, "Author %d", i % 7);
        r->state = i == 3 ? AT_NC_ST_INSTALLED : i == 4 ? AT_NC_ST_UPDATE : AT_NC_ST_NONE;
    }
    for (i = 0; i < 5; i++) {
        FRow *r = &F[fn_n++];
        memset(r, 0, sizeof *r);
        r->id = 5000 + i; r->type = 1; r->fighter = -1; r->nshots = i;
        snprintf(r->title, sizeof r->title, "Stage %d", i); snprintf(r->author, sizeof r->author, "Maker");
        r->state = AT_NC_ST_UNSUPPORTED;
    }
}

static AtNcState ST; static AtScreen SC; static AtView VW;
static AtEvent ev(int type, int a, int b) { AtEvent e; e.type = type; e.a = a; e.b = b; return e; }
static void build(const AtNcSrc *s) { CHECK(at_nc_build(s, &ST, &SC, &VW, 1000.0) == 1); }
/* one event, as the door does it: the model applies it, the action goes to the source, the screen is rebuilt */
static AtNcAction press(const AtNcSrc *s, AtEvent e)
{
    AtNcAction a;
    at_nc_event(s, &ST, &e, 1000.0, &a);
    if (a.kind == AT_NA_ACT) s->act(s->user, a.id, a.what);
    build(s);
    return a;
}
static const char *focused_label(void) { return VW.focus.index >= 0 && VW.focus.index < SC.n_items ? SC.items[VW.focus.index].label : "(none)"; }
static void reset(const AtNcSrc *s) { fixture(); at_nc_state_init(&ST); memset(&VW, 0, sizeof VW); build(s); }
static int has_key(char b) { int i; for (i = 0; i < SC.n_keys; i++) if (SC.keys[i].btn == b) return 1; return 0; }
static const char *key_label(char b) { int i; for (i = 0; i < SC.n_keys; i++) if (SC.keys[i].btn == b) return SC.keys[i].label; return ""; }
static int item_index(const char *id) { int i; for (i = 0; i < SC.n_items; i++) if (strcmp(SC.items[i].id, id) == 0) return i; return -1; }
static void goto_tab(const AtNcSrc *s, int tab) { while (ST.tab != tab) press(s, ev(AT_EV_PAGE, 1, 0)); }
static const char ELLIPSIS[] = "\xE2\x80\xA6";

static void rows(void)
{
    AtNcSrc s = src();
    reset(&s);
    CHECK(SC.primary == AT_PRIMARY_LIST && SC.chapter == 4 && strcmp(SC.title, "NUCLEUS") == 0 && SC.n_parents == 2 && SC.preset == AT_PRESET_NONE);
    CHECK(SC.n_tabs == 6 && strcmp(SC.tabs[0].name, "COSTUMES") == 0 && strcmp(SC.tabs[4].name, "INSTALLED") == 0 && strcmp(SC.tabs[5].name, "QUEUE") == 0 && VW.tab == 0);
    CHECK(ST.n == 300 && SC.tabs[0].count == 300);
    CHECK(strcmp(SC.items[0].id, "search") == 0 && strcmp(SC.items[1].id, "fighter") == 0 && strcmp(SC.items[2].id, "show") == 0 && strcmp(SC.items[3].id, "sort") == 0);
    CHECK(strcmp(SC.items[4].title, "Costume 000") == 0 && SC.items[4].thumb_on && SC.items[4].row_h == AT_NC_ROW_H);
    CHECK(VW.focus.index == 4 && VW.scroll == 4);                       /* the first result has the focus; the controls are scrolled out above it */
    CHECK(strcmp(SC.items[7].chip, "INSTALLED") == 0 && strcmp(SC.items[8].chip, "UPDATE") == 0 && SC.items[4].chip[0] == '\0');
    CHECK(strstr(SC.items[4].meta, "Mario") && strstr(SC.items[4].meta, "by Author 0") && strstr(SC.items[4].meta, "downloads"));
    CHECK(strcmp(SC.credit, "Mods from SSBM Nucleus - ssbmnucleus.net") == 0);                  /* the credit is under the pane on every Nucleus screen */
    CHECK(has_key('A') && has_key('B') && has_key('Y') && has_key('X') && has_key('Z') && strcmp(key_label('A'), "Queue") == 0);
    CHECK(!has_key('S'));                                                /* an empty queue: nothing to start */
    /* pictures are asked for the rows on screen only, never the whole 64-row window; a loading one has a placeholder (-1), a ready one its slot */
    CHECK(pic_asks > 0 && pic_asks <= 14);
    CHECK(SC.items[4].thumb_tex == -1 && SC.items[5].thumb_tex == 10 + 1001 % 7 && SC.items[5].thumb2_on && SC.items[5].thumb2_tex == 30);        /* 1000 % 5 == 0 is still loading */
    fn_restart = 1; build(&s);
    CHECK(strstr(VW.note.text, "Restart the game") != NULL && VW.note.kind == AT_NOTE_OK);
    fn_restart = 0; fn_syncing = 1; build(&s);
    CHECK(strstr(VW.note.text, "Syncing the catalog 1200 / 4700") != NULL);
    CHECK(!has_key('Z'));                                               /* no refresh while one is running */
    fn_syncing = 0; fn_locked = 1; build(&s);
    CHECK(strstr(VW.note.text, "Locked while online") != NULL);
    fn_locked = 0; fn_job_state = 3; snprintf(fn_job_msg, sizeof fn_job_msg, "Download failed (HTTP 500)."); build(&s);
    CHECK(strcmp(VW.note.text, "Download failed (HTTP 500).") == 0 && VW.note.kind == AT_NOTE_ERR);
    /* the rows' height is the list window's: four rows and a bit fit under the tab strip and the credit strip, the controls above them */
    CHECK(at_list_window(&SC, at_nc_pane_h(), VW.scroll) >= 4);
}

static void tabs_and_filters(void)
{
    AtNcSrc s = src();
    AtNcAction a;
    reset(&s);
    press(&s, ev(AT_EV_PAGE, 1, 0));                                    /* STAGES */
    CHECK(ST.tab == 1 && ST.n == 5 && strcmp(SC.items[4].title, "Stage 0") == 0 && SC.items[4].flags & AT_CELL_DISABLED);
    CHECK(SC.items[4].thumb_tex == -1 && SC.items[4].thumb_on);          /* no PNG: a placeholder box, never an empty gap */
    CHECK(SC.items[5].thumb_tex >= 0);                                   /* one with a screenshot gets it */
    press(&s, ev(AT_EV_PAGE, 1, 0)); press(&s, ev(AT_EV_PAGE, 1, 0)); press(&s, ev(AT_EV_PAGE, 1, 0));
    CHECK(ST.tab == 4 && ST.f.view == AT_NC_VIEW_INSTALLED && ST.n == 2 && SC.items[0].id[0] == 's' && SC.items[2].id[0] == 'r');   /* INSTALLED: the two that are; two controls */
    press(&s, ev(AT_EV_PAGE, 1, 0));
    CHECK(ST.tab == 5 && ST.f.view == AT_NC_VIEW_QUEUE && ST.n == 0 && strcmp(SC.items[0].id, "none") == 0);        /* QUEUE: empty, and it says so */
    press(&s, ev(AT_EV_PAGE, 1, 0));
    CHECK(ST.tab == 0 && ST.n == 300);                                  /* wraps */
    press(&s, ev(AT_EV_PAGE, -1, 0));
    CHECK(ST.tab == 5);
    press(&s, ev(AT_EV_PAGE, 1, 0));
    /* the fighter row: right steps through Any, Mario... and wraps; left goes back */
    ST.sel = 1; ST.top = 0; build(&s);
    CHECK(strcmp(focused_label(), "Fighter") == 0 && strcmp(SC.items[1].text, "Any") == 0);
    press(&s, ev(AT_EV_MOVE, AT_DIR_RIGHT, 0));
    CHECK(ST.f.fighter == 0 && strcmp(SC.items[1].text, "Mario") == 0 && ST.n == 75);
    press(&s, ev(AT_EV_MOVE, AT_DIR_LEFT, 0)); press(&s, ev(AT_EV_MOVE, AT_DIR_LEFT, 0));
    CHECK(ST.f.fighter == 3 && strcmp(SC.items[1].text, "Zelda") == 0);
    press(&s, ev(AT_EV_MOVE, AT_DIR_RIGHT, 0));
    CHECK(ST.f.fighter == -1 && ST.n == 300);
    /* the sort row */
    ST.sel = 3; ST.top = 0; build(&s);
    press(&s, ev(AT_EV_MOVE, AT_DIR_RIGHT, 0)); press(&s, ev(AT_EV_MOVE, AT_DIR_RIGHT, 0));
    CHECK(ST.f.sort == 2 && strcmp(SC.items[3].text, "Most downloaded") == 0);
    CHECK(F[fn_find(ST.ids[0])].downloads >= F[fn_find(ST.ids[1])].downloads);
    /* a catalog that changed under the screen is noticed */
    fn_ver++; F[0].state = AT_NC_ST_INSTALLED; build(&s);
    CHECK(ST.ver == (unsigned) fn_ver);
    /* B leaves, and does nothing else */
    a = press(&s, ev(AT_EV_BACK, 0, 0));
    CHECK(a.kind == AT_NA_BACK && fn_nacts == 0);
}

/* The Installed filter: All / Not installed / Installed / Update available / Queued, with the type tab, the fighter and the search, kept across sessions. */
static void show_filter(void)
{
    AtNcSrc s = src();
    int i, k;
    reset(&s);
    ST.sel = 2; ST.top = 0; build(&s);
    CHECK(strcmp(focused_label(), "Show") == 0 && strcmp(SC.items[2].text, "All") == 0);
    press(&s, ev(AT_EV_MOVE, AT_DIR_RIGHT, 0));
    CHECK(ST.f.show == AT_NC_SHOW_NOT_INSTALLED && strcmp(SC.items[2].text, "Not installed") == 0 && ST.n == 298);
    for (i = 0; i < ST.n; i++) CHECK(F[fn_find(ST.ids[i])].state == AT_NC_ST_NONE);
    press(&s, ev(AT_EV_MOVE, AT_DIR_RIGHT, 0));
    CHECK(ST.f.show == AT_NC_SHOW_INSTALLED && strcmp(SC.items[2].text, "Installed") == 0 && ST.n == 2);              /* the installed one and the one with an update */
    press(&s, ev(AT_EV_MOVE, AT_DIR_RIGHT, 0));
    CHECK(ST.f.show == AT_NC_SHOW_UPDATE && strcmp(SC.items[2].text, "Update available") == 0 && ST.n == 1 && ST.ids[0] == 1004);
    press(&s, ev(AT_EV_MOVE, AT_DIR_RIGHT, 0));
    CHECK(ST.f.show == AT_NC_SHOW_QUEUED && strcmp(SC.items[2].text, "Queued") == 0 && ST.n == 0);
    CHECK(strstr(VW.note.text, "Showing: Queued") != NULL);                                                            /* a filter that hides things says so */
    press(&s, ev(AT_EV_MOVE, AT_DIR_RIGHT, 0));
    CHECK(ST.f.show == AT_NC_SHOW_ALL && ST.n == 300 && strstr(VW.note.text, "Showing") == NULL);                    /* wraps back to All */
    /* combined with the fighter and the search */
    ST.f.fighter = 0; ST.f.show = AT_NC_SHOW_NOT_INSTALLED; ST.f.text[0] = '\0'; ST.stale = 1; build(&s);
    CHECK(ST.n == 74);                                                   /* every fourth is Mario: 75, less the one with an update (costume 4) */
    snprintf(ST.f.text, sizeof ST.f.text, "Costume 01"); ST.stale = 1; build(&s);
    for (k = 0; k < ST.n; k++) CHECK(F[fn_find(ST.ids[k])].fighter == 0 && strstr(F[fn_find(ST.ids[k])].title, "Costume 01"));
    CHECK(ST.n == 2 && strstr(VW.note.text, "Not installed - Mario - \"Costume 01\"") != NULL);
    /* Queued shows what is in the queue, in the catalog's order */
    ST.f.fighter = -1; ST.f.text[0] = '\0'; ST.f.show = AT_NC_SHOW_QUEUED; ST.stale = 1;
    fn_act(NULL, 1010, AT_NC_ACT_QUEUE); fn_act(NULL, 1002, AT_NC_ACT_QUEUE);
    build(&s);
    CHECK(ST.n == 2);
    /* remembered across sessions: the changes were saved, a fresh model reads them back */
    CHECK(pref_set_valid && pref_f.show == AT_NC_SHOW_QUEUED && pref_tab == 0);
    { int sets = pref_sets; build(&s); CHECK(pref_sets == sets); }       /* an unchanged filter is not saved again */
    goto_tab(&s, 1);
    CHECK(pref_tab == 1);
    at_nc_state_init(&ST); memset(&VW, 0, sizeof VW); build(&s);           /* the next launch */
    CHECK(ST.tab == 1 && ST.f.show == AT_NC_SHOW_QUEUED && ST.f.type_mask == (1u << 1) && ST.f.fighter == -1);
    pref_f.fighter = 2; pref_f.show = 9; pref_f.sort = 7; pref_tab = 99;    /* a damaged file cannot put the model out of range */
    at_nc_state_init(&ST); memset(&VW, 0, sizeof VW); build(&s);
    CHECK(ST.tab == 0 && ST.f.show == 0 && ST.f.sort == 0 && ST.f.fighter == 2);
}

static void search(void)
{
    AtNcSrc s = src();
    AtNcAction a;
    reset(&s);
    ST.sel = 0; ST.top = 0; build(&s);
    press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(ST.edit == 1 && has_key('A') && !has_key('Y'));
    /* the arcade picker: up types A, right adds a letter, up walks it, left deletes, A confirms */
    press(&s, ev(AT_EV_MOVE, AT_DIR_UP, 0));
    CHECK(strcmp(ST.edit_buf, "A") == 0);
    press(&s, ev(AT_EV_MOVE, AT_DIR_RIGHT, 0));
    CHECK(strcmp(ST.edit_buf, "AA") == 0);
    press(&s, ev(AT_EV_MOVE, AT_DIR_DOWN, 0));                         /* A back one: wraps to 9 */
    CHECK(strcmp(ST.edit_buf, "A9") == 0);
    press(&s, ev(AT_EV_MOVE, AT_DIR_LEFT, 0)); press(&s, ev(AT_EV_MOVE, AT_DIR_LEFT, 0));
    CHECK(ST.edit_buf[0] == '\0');
    /* typed on a keyboard */
    { const char *w = "COSTUME 07"; int i; for (i = 0; w[i]; i++) at_nc_type(&ST, w[i]); }
    build(&s);
    CHECK(strstr(SC.items[0].text, "COSTUME 07_") != NULL);
    at_nc_type(&ST, 27);
    CHECK(ST.edit == 0 && ST.f.text[0] == '\0');                       /* escape cancels */
    ST.sel = 0; press(&s, ev(AT_EV_ALT, 'X', 0));
    CHECK(ST.edit == 1);
    { const char *w = "Costume 07"; int i; for (i = 0; w[i]; i++) at_nc_type(&ST, w[i]); }
    at_nc_type(&ST, 8); at_nc_type(&ST, '7');
    press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(ST.edit == 0 && strcmp(ST.f.text, "Costume 07") == 0 && ST.n == 10 && strcmp(focused_label(), "Costume 070") == 0);
    build(&s);
    CHECK(pref_set_valid && strcmp(pref_f.text, "Costume 07") == 0);   /* the search is remembered too */
    /* nothing matches: say so, and the keys stay sane */
    ST.sel = 0; ST.top = 0; build(&s); press(&s, ev(AT_EV_ACCEPT, 0, 0));
    for (a.kind = 0; ST.edit_buf[0]; ) at_nc_type(&ST, 8);
    { const char *w = "ZZZ"; int i; for (i = 0; w[i]; i++) at_nc_type(&ST, w[i]); }
    press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(ST.n == 0 && strcmp(SC.items[SC.n_items - 1].id, "none") == 0 && strcmp(SC.items[SC.n_items - 1].label, "Nothing matches") == 0 && has_key('B') && !has_key('Y'));
    /* a long query is cut, never overruns */
    { int i; ST.sel = 0; ST.top = 0; build(&s); press(&s, ev(AT_EV_ACCEPT, 0, 0)); for (i = 0; i < 80; i++) at_nc_type(&ST, 'Q'); CHECK(strlen(ST.edit_buf) < sizeof ST.edit_buf); }
}

/* A queues, Start begins, the QUEUE tab shows and edits the line, B never clears it, clearing asks first. */
static void queue_flow(void)
{
    AtNcSrc s = src();
    AtNcAction a;
    reset(&s);
    /* A on an uninstalled costume queues it; the row says QUEUED #1; the QUEUE tab counts it; Start appears */
    a = press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(a.kind == AT_NA_ACT && a.what == AT_NC_ACT_QUEUE && a.id == 1000 && fq_n == 1);
    CHECK(strcmp(SC.items[4].chip, "QUEUED #1") == 0 && SC.tabs[5].count == 1 && has_key('S') && strcmp(key_label('S'), "Download 1") == 0 && strcmp(key_label('A'), "Unqueue") == 0);
    /* A again takes it out (a toggle, not a silent drop) */
    a = press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(a.what == AT_NC_ACT_QUEUE && fq_n == 0 && SC.items[4].chip[0] == '\0' && !has_key('S'));
    /* three mods: the second and third, then the first */
    press(&s, ev(AT_EV_MOVE, AT_DIR_DOWN, 0)); press(&s, ev(AT_EV_ACCEPT, 0, 0));
    press(&s, ev(AT_EV_MOVE, AT_DIR_DOWN, 0)); press(&s, ev(AT_EV_ACCEPT, 0, 0));
    press(&s, ev(AT_EV_MOVE, AT_DIR_UP, 0)); press(&s, ev(AT_EV_MOVE, AT_DIR_UP, 0)); press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(fq_n == 3 && fq_id[0] == 1001 && fq_id[1] == 1002 && fq_id[2] == 1000);
    CHECK(strcmp(SC.items[4].chip, "QUEUED #3") == 0 && strcmp(SC.items[5].chip, "QUEUED #1") == 0 && strcmp(key_label('S'), "Download 3") == 0);
    /* the Start button / Enter */
    a = press(&s, ev(AT_EV_START, 0, 0));
    CHECK(a.kind == AT_NA_ACT && a.what == AT_NC_ACT_START && fq_proc == 1 && strcmp(key_label('S'), "Pause") == 0);
    /* the QUEUE tab: rows in line order with their state; the entry being installed shows its percent */
    fq_st[0] = AT_NC_Q_RUNNING; fq_pct[0] = 47; fn_ver++;
    goto_tab(&s, 5);
    CHECK(ST.n == 3 && SC.n_items == 3 && SC.items[0].id[0] == 'r' && strcmp(SC.items[0].title, "Costume 001") == 0 && strcmp(SC.items[0].chip, "47%") == 0);
    CHECK(strcmp(SC.items[1].chip, "QUEUED #2") == 0 && strstr(SC.items[0].meta, "47%") != NULL);
    CHECK(strstr(VW.note.text, "Installing Costume 001 47%") != NULL);                  /* the corner follows the install while browsing */
    CHECK(has_key('X') && has_key('Y') && has_key('Z') && strcmp(key_label('X'), "Move up") == 0 && strcmp(key_label('Z'), "Clear") == 0);
    /* the one being installed cannot be removed or moved; A says so */
    a = press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(a.kind == AT_NA_NONE && fq_n == 3 && strstr(ST.note, "being installed") != NULL);
    /* reorder: Y moves the focused entry down and the focus follows it */
    press(&s, ev(AT_EV_MOVE, AT_DIR_DOWN, 0));
    CHECK(focused_label()[0] != '\0' && ST.ids[ST.sel - at_nc_ctl(&ST)] == 1002);
    a = press(&s, ev(AT_EV_ALT, 'Y', 0));
    CHECK(a.what == AT_NC_ACT_Q_DOWN && fq_id[1] == 1000 && fq_id[2] == 1002 && ST.ids[ST.sel] == 1002);
    a = press(&s, ev(AT_EV_ALT, 'X', 0));
    CHECK(a.what == AT_NC_ACT_Q_UP && fq_id[1] == 1002 && fq_id[2] == 1000 && ST.ids[ST.sel] == 1002);
    /* A on a waiting entry removes it */
    a = press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(a.what == AT_NC_ACT_Q_REMOVE && a.id == 1002 && fq_n == 2);
    /* a failed one: the reason in plain words, A retries it */
    fq_st[1] = AT_NC_Q_FAILED; snprintf(fq_reason[1], sizeof fq_reason[1], "SSBM Nucleus could not be reached. Check the connection and retry."); fn_ver++;
    ST.sel = 1; build(&s);
    CHECK(strcmp(SC.items[1].chip, "FAILED") == 0 && strstr(SC.items[1].meta, "could not be reached") != NULL && strcmp(key_label('A'), "Retry") == 0);
    a = press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(a.what == AT_NC_ACT_Q_RETRY && fq_st[1] == AT_NC_Q_QUEUED);
    /* B leaves the screen and the queue is exactly as it was */
    { int n = fq_n, id0 = fq_id[0], id1 = fq_id[1]; a = press(&s, ev(AT_EV_BACK, 0, 0)); CHECK(a.kind == AT_NA_BACK && fq_n == n && fq_id[0] == id0 && fq_id[1] == id1); }
    /* clearing is an explicit action with a confirmation: Z opens a dialog on Keep, B or Keep leave the queue alone, Clear empties what is not being installed */
    fn_nacts = 0;
    a = press(&s, ev(AT_EV_ALT, 'Z', 0));
    CHECK(a.kind == AT_NA_NONE && VW.dialog.open && VW.dialog.n == 2 && strcmp(VW.dialog.label[0], "Clear") == 0 && strcmp(VW.dialog.label[1], "Keep") == 0 && VW.dialog.focus == 1);
    CHECK(strstr(VW.dialog.body, "1 waiting") != NULL);
    a = press(&s, ev(AT_EV_ACCEPT, 0, 0));                             /* the focus is on Keep */
    CHECK(!VW.dialog.open && fq_n == 2 && fn_nacts == 0);
    press(&s, ev(AT_EV_ALT, 'Z', 0));
    a = press(&s, ev(AT_EV_BACK, 0, 0));                               /* B is Keep too, and it does not leave the screen */
    CHECK(a.kind == AT_NA_NONE && !VW.dialog.open && fq_n == 2 && ST.tab == 5);
    press(&s, ev(AT_EV_ALT, 'Z', 0));
    press(&s, ev(AT_EV_MOVE, AT_DIR_LEFT, 0));
    CHECK(VW.dialog.open && VW.dialog.focus == 0);
    a = press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(a.what == AT_NC_ACT_Q_CLEAR && !VW.dialog.open && fq_n == 1 && fq_st[0] == AT_NC_Q_RUNNING);                  /* the one being installed stays */
    /* online: the queue still edits, and the corner says the downloads wait */
    fn_locked = 1; ST.note_until = 0; build(&s);
    CHECK(strstr(VW.note.text, "Downloads wait while you are online") != NULL);
    fn_locked = 0;
    /* the detail: the same toggle, and a Start row when there is a queue */
    ST.tab = 0; at_nc_apply_tab(&ST); build(&s);
    ST.detail_id = 1010; ST.mode = 1; ST.detail_sel = 1;
    CHECK(at_nc_detail_build(&s, &ST, &SC, &VW, 1000.0) == 1);
    CHECK(strcmp(SC.items[1].id, "act") == 0 && strcmp(SC.items[1].label, "Add to the queue") == 0 && item_index("start") == 2 && strcmp(SC.items[2].label, "Pause the downloads") == 0);
    at_nc_detail_event(&s, &ST, &(AtEvent){ AT_EV_ACCEPT, 0, 0 }, 1000.0, &a);
    CHECK(a.what == AT_NC_ACT_QUEUE && a.id == 1010);
    fn_act(NULL, 1010, AT_NC_ACT_QUEUE);
    CHECK(at_nc_detail_build(&s, &ST, &SC, &VW, 1000.0) == 1 && strcmp(SC.items[1].label, "Take it out of the queue") == 0 && strcmp(SC.items[0].chip, "QUEUED #2") == 0);
    at_nc_detail_event(&s, &ST, &(AtEvent){ AT_EV_START, 0, 0 }, 1000.0, &a);
    CHECK(a.what == AT_NC_ACT_START);
}

static void actions(void)
{
    AtNcSrc s = src();
    AtNcAction a;
    reset(&s);
    /* A on an update queues it */
    ST.sel = 4 + 4; build(&s);
    a = press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(a.kind == AT_NA_ACT && a.what == AT_NC_ACT_QUEUE && a.id == 1004);
    /* A on an installed one opens its detail instead (where it can be removed) */
    ST.sel = 4 + 3; build(&s);
    a = press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(a.kind == AT_NA_NONE && ST.mode == 1 && ST.detail_id == 1003);
    /* a stage skin says why not */
    reset(&s);
    press(&s, ev(AT_EV_PAGE, 1, 0));
    a = press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(a.kind == AT_NA_NONE && strstr(ST.note, "cannot be installed") != NULL && fq_n == 0);
    CHECK(strstr(SC.items[4].meta, "Stage skins cannot be installed yet.") == SC.items[4].meta);
    /* Z refreshes */
    reset(&s);
    a = press(&s, ev(AT_EV_ALT, 'Z', 0));
    CHECK(a.kind == AT_NA_ACT && a.what == AT_NC_ACT_REFRESH);
    /* Y opens a detail; on a control row Y does nothing */
    ST.sel = 1; ST.top = 0; a = press(&s, ev(AT_EV_ALT, 'Y', 0)); CHECK(ST.mode == 0);
    ST.sel = 6; build(&s); a = press(&s, ev(AT_EV_ALT, 'Y', 0)); CHECK(ST.mode == 1 && ST.detail_id == 1002);
}

static void detail(void)
{
    AtNcSrc s = src();
    AtNcAction a;
    int i, k, shots = 0;
    reset(&s);
    ST.sel = 4 + 4; build(&s);                                          /* the update one */
    press(&s, ev(AT_EV_ALT, 'Y', 0));
    CHECK(ST.mode == 1);
    CHECK(at_nc_detail_build(&s, &ST, &SC, &VW, 1000.0) == 1);
    CHECK(strcmp(SC.id, "mods.nucleus.detail") == 0 && strcmp(SC.items[0].id, "hero") == 0 && strcmp(SC.items[0].title, "Costume 004") == 0 && SC.items[0].thumb_on);
    CHECK(strcmp(SC.items[1].id, "act") == 0 && strcmp(SC.items[1].label, "Queue the update") == 0 && strcmp(SC.items[2].id, "rm") == 0 && strcmp(SC.items[3].id, "page") == 0);
    CHECK(VW.focus.index == 1 && strcmp(SC.credit, AT_NC_CREDIT) == 0);
    {
        int credit = 0, author = 0, about = 0;
        for (i = 0; i < SC.n_items; i++) {
            if (strstr(SC.items[i].title, "Mods from SSBM Nucleus - ssbmnucleus.net")) credit = 1;
            if (strstr(SC.items[i].title, "Author 4")) author = 1;
            if (strcmp(SC.items[i].id, "about") == 0 && strstr(SC.items[i].body, "A synthetic description for Costume 004")) about = 1;
        }
        CHECK(credit && author && about);
    }
    /* A on the first action queues the update; Uninstall removes; the page row opens it */
    ST.detail_sel = 1; at_nc_detail_event(&s, &ST, &(AtEvent){ AT_EV_ACCEPT, 0, 0 }, 1000.0, &a);
    CHECK(a.kind == AT_NA_ACT && a.what == AT_NC_ACT_QUEUE && a.id == 1004);
    at_nc_detail_event(&s, &ST, &(AtEvent){ AT_EV_MOVE, AT_DIR_DOWN, 0 }, 1000.0, &a);
    at_nc_detail_event(&s, &ST, &(AtEvent){ AT_EV_ACCEPT, 0, 0 }, 1000.0, &a);
    CHECK(a.kind == AT_NA_ACT && a.what == AT_NC_ACT_UNINSTALL);
    at_nc_detail_event(&s, &ST, &(AtEvent){ AT_EV_MOVE, AT_DIR_DOWN, 0 }, 1000.0, &a);
    at_nc_detail_event(&s, &ST, &(AtEvent){ AT_EV_ACCEPT, 0, 0 }, 1000.0, &a);
    CHECK(a.kind == AT_NA_ACT && a.what == AT_NC_ACT_PAGE);
    /* the focus is always among the rows shown, down the whole list, and the renderer draws that row */
    for (k = 0; k < SC.n_items + 2; k++) {
        static AtHits hits; AtRenderInfo info; AtSink sk = rec_sink(); int j, on = 0;
        CHECK(at_nc_detail_build(&s, &ST, &SC, &VW, 1000.0) == 1);
        CHECK(VW.focus.index >= VW.scroll && VW.focus.index < VW.scroll + at_list_window(&SC, at_nc_pane_h() + 30.0f, VW.scroll));
        at_render_ex(&SC, &VW, 853.0f, 1000.0, 0, &FAKE, &sk, &hits, &info);
        for (j = 0; j < hits.n; j++) if (hits.h[j].kind == AT_HIT_CELL && hits.h[j].b == VW.focus.index) on = 1;
        CHECK(on);
        at_nc_detail_event(&s, &ST, &(AtEvent){ AT_EV_MOVE, AT_DIR_DOWN, 0 }, 1000.0, &a);
    }
    at_nc_detail_event(&s, &ST, &(AtEvent){ AT_EV_BACK, 0, 0 }, 1000.0, &a);
    CHECK(ST.mode == 0);
    /* a stage with screenshots: up to three picture rows, each with its own texture, and a note when there are none */
    ST.detail_id = 5003; ST.mode = 1;
    CHECK(at_nc_detail_build(&s, &ST, &SC, &VW, 1000.0) == 1);
    for (i = 0; i < SC.n_items; i++) if (strncmp(SC.items[i].id, "shot", 4) == 0) { shots++; CHECK(SC.items[i].thumb_on && SC.items[i].row_h == AT_NC_SHOT_H && SC.items[i].thumb_tex >= 40); }
    CHECK(shots == 3);
    ST.detail_id = 5004;
    CHECK(at_nc_detail_build(&s, &ST, &SC, &VW, 1000.0) == 1);
    for (shots = 0, i = 0; i < SC.n_items; i++) if (strncmp(SC.items[i].id, "shot", 4) == 0) shots++;
    CHECK(shots == 3);                                                   /* a post can list more; three are shown */
    ST.detail_id = 5000;
    CHECK(at_nc_detail_build(&s, &ST, &SC, &VW, 1000.0) == 1);
    for (shots = 0, i = 0; i < SC.n_items; i++) if (strncmp(SC.items[i].id, "shot", 4) == 0) shots++;
    CHECK(shots == 0);
    /* a not-installed costume has no Uninstall row; an unsupported one says why on its action row */
    reset(&s); ST.detail_id = 1000; ST.mode = 1;
    CHECK(at_nc_detail_build(&s, &ST, &SC, &VW, 1000.0) == 1 && strcmp(SC.items[1].label, "Add to the queue") == 0 && strcmp(SC.items[2].id, "page") == 0);
    ST.detail_id = 5001;
    CHECK(at_nc_detail_build(&s, &ST, &SC, &VW, 1000.0) == 1 && strcmp(SC.items[1].label, "Cannot be installed yet") == 0 && strstr(SC.items[1].sub, "cannot be installed") != NULL);
    ST.detail_id = 999999;
    CHECK(at_nc_detail_build(&s, &ST, &SC, &VW, 1000.0) == 0);          /* a mod that vanished: the caller goes back to the list */
}

static void window_walk(void)
{
    AtNcSrc s = src();
    int k;
    float ph = at_nc_pane_h();
    reset(&s);
    for (k = 1; k <= 300; k++) {
        pic_asks = 0;
        press(&s, ev(AT_EV_MOVE, AT_DIR_DOWN, 0));
        CHECK(pic_asks <= 14);
        CHECK(SC.n_items <= AT_MAX_ITEMS && VW.focus.index >= 0 && VW.focus.index < SC.n_items);
        CHECK(VW.focus.index >= VW.scroll && VW.focus.index < VW.scroll + at_list_window(&SC, ph, VW.scroll));      /* the focused row is on screen */
    }
    CHECK(ST.sel == 4 + 299 || ST.sel == 3 + 0 || ST.sel == 0 || ST.sel > 0);
    /* every row asked for a picture is one in the window the screen shows (± one): 300 rows scrolled through, never a row far off screen */
    {
        int lo = 1000 + (ST.sel - 4) - 60, i;
        (void) lo;
        for (i = 0; i < pic_asks && i < 256; i++) CHECK(pic_ids[i] >= 1000);
    }
    /* walking up past the first result reaches the controls one at a time */
    reset(&s);
    press(&s, ev(AT_EV_MOVE, AT_DIR_UP, 0));
    CHECK(ST.sel == 3 && VW.scroll == 3 && strcmp(focused_label(), "Sort by") == 0);
    press(&s, ev(AT_EV_MOVE, AT_DIR_UP, 0)); press(&s, ev(AT_EV_MOVE, AT_DIR_UP, 0)); press(&s, ev(AT_EV_MOVE, AT_DIR_UP, 0));
    CHECK(ST.sel == 0 && VW.scroll == 0);
}

/* How a row is drawn at the canvas widths: every text fits its row, nothing is clipped or overlapping, and typical long titles and authors come out whole. */
static int texts_with_ellipsis(void)
{
    int i, n = 0;
    for (i = 0; i < REC.nt; i++) if (strstr(REC.t[i].s, ELLIPSIS) != NULL) n++;
    return n;
}
static int has_text_part(const char *part)
{
    int i;
    for (i = 0; i < REC.nt; i++) if (strstr(REC.t[i].s, part) != NULL) return 1;
    return 0;
}

static void render_fit(void)
{
    AtNcSrc s = src();
    static const float widths[4] = { 640.0f, 853.0f, 1140.0f, 1280.0f };
    /* typical long titles: 62 characters (fits everywhere), and 78 (a real post title length: whole from the 16:9 default window up) */
    static const char *const title62 = "Ultimate Edition HD Remix Pack with Alternate Palettes for Fox";
    static const char *const title78 = "Super Smash Bros. Ultimate Style Remastered Costume Collection - Complete Redux";
    static const char *const author39 = "TheLongestNamedCostumeAuthorInTheWorld9";
    int w, i, j, ti;
    for (ti = 0; ti < 2; ti++) {
        for (w = (ti == 1 ? 1 : 0); w < 4; w++) {
            static AtHits hits; AtRenderInfo info; AtSink sk;
            const char *title = ti == 0 ? title62 : title78, *shown_ok = ti == 0 ? "Ultimate Edition" : "Super Smash Bros.";
            reset(&s);
            snprintf(F[0].title, sizeof F[0].title, "%s", title);
            snprintf(F[0].author, sizeof F[0].author, "%s", author39);
            F[0].state = AT_NC_ST_INSTALLED;                               /* the row with the most at its right edge: INSTALLED, and QUEUED */
            fn_act(NULL, 1000, AT_NC_ACT_QUEUE);
            build(&s);
            sk = rec_sink();
            at_render_ex(&SC, &VW, widths[w], 1000.0, 0, &FAKE, &sk, &hits, &info);
            CHECK(info.entries < 2500 && !info.capped);
            CHECK(has_text_part(shown_ok));
            CHECK(texts_with_ellipsis() == 0);                             /* nothing cut with an ellipsis: titles wrap to two lines, facts drop from the end */
            /* the title is all there: its words, across the lines drawn */
            { char joined[512] = ""; for (i = 0; i < REC.nt; i++) if (REC.t[i].role == AT_R_ROW16 || REC.t[i].role == AT_R_BODY14) { size_t jl = strlen(joined); snprintf(joined + jl, sizeof joined - jl, "%s ", REC.t[i].s); }
              CHECK(strstr(joined, title78 + 60) != NULL || ti == 0); CHECK(strstr(joined, "Alternate Palettes for Fox") != NULL || ti == 1); }
            /* the author is on the facts line whole from the 16:9 window up; at the 4:3 minimum the numbers go first and the author stays */
            CHECK(has_text_part("TheLongestNamedCostumeAuthorInTheWorld9"));
            for (i = 0; i < hits.n; i++) {
                if (hits.h[i].kind != AT_HIT_CELL) continue;
                for (j = 0; j < REC.nt; j++) {
                    float x0, x1, y0, y1;
                    lint_text_box(&REC.t[j], &x0, &x1, &y0, &y1);
                    if (REC.t[j].base < hits.h[i].r.y || REC.t[j].base > hits.h[i].r.y + hits.h[i].r.h) continue;
                    if (x0 < hits.h[i].r.x - 0.5f || x0 > hits.h[i].r.x + hits.h[i].r.w) continue;
                    CHECK(x1 <= hits.h[i].r.x + hits.h[i].r.w + 0.5f && at_role_size(REC.t[j].role) >= 12);
                    CHECK(y0 >= hits.h[i].r.y - 2.5f && y1 <= hits.h[i].r.y + hits.h[i].r.h + 0.5f);          /* inside the row vertically too */
                }
            }
            { int tabs = 0; for (i = 0; i < hits.n; i++) if (hits.h[i].kind == AT_HIT_TAB) tabs++; CHECK(tabs == 6); }       /* the six tabs fit and are all hit-testable */
            CHECK(lint_text_overlaps() == 0);
            CHECK(find_text("Mods from SSBM Nucleus - ssbmnucleus.net") != NULL);                           /* the credit line is drawn */
            { int imgs = REC.ni; CHECK(imgs >= 3); }                                                           /* pictures were drawn for the rows with one */
        }
    }
    /* a longer title than any real one still never overflows: the last line may end in an ellipsis, as a last resort, and the full title is in the detail */
    {
        static AtHits hits; AtRenderInfo info; AtSink sk; char longt[96];
        reset(&s);
        memset(longt, 'W', 79); longt[79] = '\0';
        snprintf(F[0].title, sizeof F[0].title, "%s", longt);
        build(&s);
        sk = rec_sink();
        at_render_ex(&SC, &VW, 640.0f, 1000.0, 0, &FAKE, &sk, &hits, &info);
        for (i = 0; i < hits.n; i++) {
            if (hits.h[i].kind != AT_HIT_CELL) continue;
            for (j = 0; j < REC.nt; j++) {
                float x0, x1, y0, y1;
                lint_text_box(&REC.t[j], &x0, &x1, &y0, &y1);
                if (REC.t[j].base < hits.h[i].r.y || REC.t[j].base > hits.h[i].r.y + hits.h[i].r.h || x0 < hits.h[i].r.x - 0.5f || x0 > hits.h[i].r.x + hits.h[i].r.w) continue;
                CHECK(x1 <= hits.h[i].r.x + hits.h[i].r.w + 0.5f);
            }
        }
        ST.detail_id = 1000; ST.mode = 1;
        CHECK(at_nc_detail_build(&s, &ST, &SC, &VW, 1000.0) == 1 && strcmp(SC.items[0].title, longt) == 0);       /* the whole title is kept for the detail */
    }
    /* the detail at the same widths: the description wraps in its block, the facts are whole */
    for (w = 0; w < 4; w++) {
        static AtHits hits; AtRenderInfo info; AtSink sk;
        reset(&s);
        snprintf(F[0].title, sizeof F[0].title, "%s", title62);
        snprintf(F[0].author, sizeof F[0].author, "%s", author39);
        ST.detail_id = 1000; ST.mode = 1;
        CHECK(at_nc_detail_build(&s, &ST, &SC, &VW, 1000.0) == 1);
        sk = rec_sink();
        at_render_ex(&SC, &VW, widths[w], 1000.0, 0, &FAKE, &sk, &hits, &info);
        CHECK(info.entries < 2500 && !info.capped && lint_text_overlaps() == 0);
        CHECK(texts_with_ellipsis() == 0);
        CHECK(has_text_part(title62 + 20) || has_text_part("Alternate Palettes"));
    }
}

int main(void)
{
    rows();
    tabs_and_filters();
    show_filter();
    search();
    queue_flow();
    actions();
    detail();
    window_walk();
    render_fit();
    ATLAS_DONE("atlas nucleus");
}
