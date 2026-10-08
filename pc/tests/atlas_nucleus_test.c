#include <stdlib.h>
#include "atlas_lint.h"
#include "../platform/gw_ui_mods.h"
#include "../platform/gw_ui_render.h"
#include "../platform/gw_ui_nucleus.inc"

/* ---- a fake Nucleus: rows, a filter, an action log ---- */
#define FN_MAX 400
typedef struct { int id, type, fighter, state, downloads; char title[80], author[40]; } FRow;
static FRow F[FN_MAX];
static int fn_n, fn_ver = 1, fn_syncing, fn_locked, fn_restart, fn_job_state;
static char fn_job_msg[140];
static struct { int id, what; } fn_acts[16];
static int fn_nacts;
static const char *const FL[] = { "Mario", "Fox", "Captain Falcon", "Zelda" };

static int fn_find(int id) { int i; for (i = 0; i < fn_n; i++) if (F[i].id == id) return i; return -1; }
static int fn_cmp_title(const void *a, const void *b) { return strcmp(F[*(const int *) a].title, F[*(const int *) b].title); }
static int fn_cmp_dl(const void *a, const void *b) { return F[*(const int *) b].downloads - F[*(const int *) a].downloads; }
static int fn_filter(void *u, const AtNcFilter *f, int *ids, int cap, unsigned *version)
{
    int i, n = 0;
    (void) u;
    *version = (unsigned) fn_ver;
    for (i = 0; i < fn_n && n < cap; i++) {
        if (f->view == AT_NC_VIEW_INSTALLED) { if (F[i].state != AT_NC_ST_INSTALLED && F[i].state != AT_NC_ST_UPDATE) continue; }
        else if (f->type_mask && !(f->type_mask & (1u << F[i].type))) continue;
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
    int i = fn_find(id);
    (void) u;
    if (i < 0) return 0;
    memset(r, 0, sizeof *r);
    r->id = id; r->tex = -1; r->state = F[i].state; r->downloads = F[i].downloads; r->likes = 3; r->ncostumes = 2;
    snprintf(r->title, sizeof r->title, "%s", F[i].title); snprintf(r->author, sizeof r->author, "%s", F[i].author);
    snprintf(r->fighter, sizeof r->fighter, "%s", F[i].fighter >= 0 ? FL[F[i].fighter] : "");
    snprintf(r->kind, sizeof r->kind, "%s", F[i].type == 0 ? "Costume" : "Stage skin");
    snprintf(r->updated, sizeof r->updated, "2026-03-03"); snprintf(r->page, sizeof r->page, "https://ssbmnucleus.net/post/%d/x", id);
    snprintf(r->desc, sizeof r->desc, "A synthetic description for %s.", F[i].title);
    if (F[i].state == AT_NC_ST_UNSUPPORTED) snprintf(r->reason, sizeof r->reason, "Stage skins cannot be installed yet.");
    return 1;
}
static void fn_status(void *u, AtNcStatus *s)
{
    (void) u;
    memset(s, 0, sizeof *s);
    s->syncing = fn_syncing; s->done = 1200; s->total = fn_syncing ? 4700 : 0; s->locked = fn_locked; s->restart = fn_restart;
    s->job_state = fn_job_state; snprintf(s->job_msg, sizeof s->job_msg, "%s", fn_job_msg);
}
static int fn_act(void *u, int id, int what) { (void) u; if (fn_nacts < 16) { fn_acts[fn_nacts].id = id; fn_acts[fn_nacts++].what = what; } return 1; }
static int fn_fighters(void *u) { (void) u; return 4; }
static const char *fn_flabel(void *u, int i) { (void) u; return FL[i]; }
static unsigned fn_version(void *u) { (void) u; return (unsigned) fn_ver; }
static AtNcSrc src(void)
{
    AtNcSrc s;
    memset(&s, 0, sizeof s);
    s.filter = fn_filter; s.row = fn_row; s.status = fn_status; s.act = fn_act; s.fighters = fn_fighters; s.fighter_label = fn_flabel; s.version = fn_version;
    return s;
}
static void fixture(void)
{
    int i;
    fn_n = 0; fn_ver++; fn_syncing = fn_locked = fn_restart = fn_job_state = 0; fn_nacts = 0; fn_job_msg[0] = '\0';
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
        r->id = 5000 + i; r->type = 1; r->fighter = -1;
        snprintf(r->title, sizeof r->title, "Stage %d", i); snprintf(r->author, sizeof r->author, "Maker");
        r->state = AT_NC_ST_UNSUPPORTED;
    }
}

static AtNcState ST; static AtScreen SC; static AtView VW;
static AtEvent ev(int type, int a, int b) { AtEvent e; e.type = type; e.a = a; e.b = b; return e; }
static void build(const AtNcSrc *s) { CHECK(at_nc_build(s, &ST, &SC, &VW, 1000.0) == 1); }
static AtNcAction press(const AtNcSrc *s, AtEvent e) { AtNcAction a; at_nc_event(s, &ST, &e, 1000.0, &a); build(s); return a; }
static const char *focused_label(void) { return VW.focus.index >= 0 && VW.focus.index < SC.n_items ? SC.items[VW.focus.index].label : "(none)"; }
static void reset(const AtNcSrc *s) { fixture(); at_nc_state_init(&ST); memset(&VW, 0, sizeof VW); build(s); }
static int has_key(char b) { int i; for (i = 0; i < SC.n_keys; i++) if (SC.keys[i].btn == b) return 1; return 0; }

static void rows(void)
{
    AtNcSrc s = src();
    reset(&s);
    CHECK(SC.primary == AT_PRIMARY_LIST && SC.chapter == 4 && strcmp(SC.title, "NUCLEUS") == 0 && SC.n_parents == 2);
    CHECK(SC.n_tabs == 5 && strcmp(SC.tabs[0].name, "COSTUMES") == 0 && strcmp(SC.tabs[4].name, "INSTALLED") == 0 && VW.tab == 0);
    CHECK(ST.n == 300 && SC.tabs[0].count == 300);
    CHECK(strcmp(SC.items[0].id, "search") == 0 && strcmp(SC.items[1].id, "fighter") == 0 && strcmp(SC.items[2].id, "sort") == 0);
    CHECK(strcmp(SC.items[3].label, "Costume 000") == 0 && strcmp(SC.items[3].group, "RESULTS") == 0 && strcmp(SC.items[0].group, "FILTERS") == 0);
    CHECK(VW.focus.index == 3);                                         /* the first result has the focus, not a control */
    CHECK(strcmp(SC.items[6].text, "INSTALLED") == 0 && strcmp(SC.items[7].text, "UPDATE") == 0 && strcmp(SC.items[3].text, "") == 0);
    /* the credit and the author are always on screen, and the restart rule when something was installed */
    CHECK(VW.ex.has && strcmp(VW.ex.from_text, "Mods from SSBM Nucleus - ssbmnucleus.net") == 0 && strstr(VW.ex.what, "Author 0") != NULL);
    CHECK(strstr(VW.ex.kicker, "MARIO") != NULL);
    CHECK(has_key('A') && has_key('B') && has_key('Y') && has_key('X') && has_key('Z'));
    fn_restart = 1; build(&s);
    CHECK(strstr(VW.note.text, "Restart the game") != NULL && VW.note.kind == AT_NOTE_OK);
    fn_restart = 0; fn_syncing = 1; build(&s);
    CHECK(strstr(VW.note.text, "Syncing the catalog 1200 / 4700") != NULL);
    CHECK(!has_key('Z'));                                               /* no refresh while one is running */
    fn_syncing = 0; fn_locked = 1; build(&s);
    CHECK(strstr(VW.note.text, "Locked while online") != NULL);
    fn_locked = 0; fn_job_state = 3; snprintf(fn_job_msg, sizeof fn_job_msg, "Download failed (HTTP 500)."); build(&s);
    CHECK(strcmp(VW.note.text, "Download failed (HTTP 500).") == 0 && VW.note.kind == AT_NOTE_ERR);
}

static void tabs_and_filters(void)
{
    AtNcSrc s = src();
    AtNcAction a;
    reset(&s);
    press(&s, ev(AT_EV_PAGE, 1, 0));                                    /* STAGES */
    CHECK(ST.tab == 1 && ST.n == 5 && strcmp(SC.items[3].label, "Stage 0") == 0 && SC.items[3].flags & AT_CELL_DISABLED);
    press(&s, ev(AT_EV_PAGE, 1, 0)); press(&s, ev(AT_EV_PAGE, 1, 0)); press(&s, ev(AT_EV_PAGE, 1, 0));
    CHECK(ST.tab == 4 && ST.f.view == AT_NC_VIEW_INSTALLED && ST.n == 2);          /* INSTALLED: the two that are */
    press(&s, ev(AT_EV_PAGE, 1, 0));
    CHECK(ST.tab == 0 && ST.n == 300);                                  /* wraps */
    press(&s, ev(AT_EV_PAGE, -1, 0));
    CHECK(ST.tab == 4);
    press(&s, ev(AT_EV_PAGE, 1, 0));
    /* the fighter row: right steps through Any, Mario... and wraps; left goes back */
    ST.sel = 1; build(&s);
    CHECK(strcmp(focused_label(), "Fighter") == 0 && strcmp(SC.items[1].text, "Any") == 0);
    press(&s, ev(AT_EV_MOVE, AT_DIR_RIGHT, 0));
    CHECK(ST.f.fighter == 0 && strcmp(SC.items[1].text, "Mario") == 0 && ST.n == 75);
    press(&s, ev(AT_EV_MOVE, AT_DIR_LEFT, 0)); press(&s, ev(AT_EV_MOVE, AT_DIR_LEFT, 0));
    CHECK(ST.f.fighter == 3 && strcmp(SC.items[1].text, "Zelda") == 0);
    press(&s, ev(AT_EV_MOVE, AT_DIR_RIGHT, 0));
    CHECK(ST.f.fighter == -1 && ST.n == 300);
    /* the sort row */
    ST.sel = 2; build(&s);
    press(&s, ev(AT_EV_MOVE, AT_DIR_RIGHT, 0)); press(&s, ev(AT_EV_MOVE, AT_DIR_RIGHT, 0));
    CHECK(ST.f.sort == 2 && strcmp(SC.items[2].text, "Most downloaded") == 0);
    CHECK(F[fn_find(ST.ids[0])].downloads >= F[fn_find(ST.ids[1])].downloads);
    /* a catalog that changed under the screen is noticed */
    fn_ver++; F[0].state = AT_NC_ST_INSTALLED; build(&s);
    CHECK(ST.ver == (unsigned) fn_ver);
    /* B leaves */
    a = press(&s, ev(AT_EV_BACK, 0, 0));
    CHECK(a.kind == AT_NA_BACK);
}

static void search(void)
{
    AtNcSrc s = src();
    AtNcAction a;
    reset(&s);
    ST.sel = 0; build(&s);
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
    CHECK(strcmp(SC.items[0].text, "Costume 07") == 0);
    /* nothing matches: say so, and the keys stay sane */
    ST.sel = 0; press(&s, ev(AT_EV_ACCEPT, 0, 0));
    for (a.kind = 0; ST.edit_buf[0]; ) at_nc_type(&ST, 8);
    { const char *w = "ZZZ"; int i; for (i = 0; w[i]; i++) at_nc_type(&ST, w[i]); }
    press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(ST.n == 0 && SC.n_items == 3 && strcmp(VW.ex.title, "Nothing matches") == 0 && has_key('B') && !has_key('Y'));
    /* a long query is cut, never overruns */
    { int i; ST.sel = 0; press(&s, ev(AT_EV_ACCEPT, 0, 0)); for (i = 0; i < 80; i++) at_nc_type(&ST, 'Q'); CHECK(strlen(ST.edit_buf) < sizeof ST.edit_buf); }
}

static void actions(void)
{
    AtNcSrc s = src();
    AtNcAction a;
    reset(&s);
    /* A on an uninstalled costume asks the host to install it */
    a = press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(a.kind == AT_NA_ACT && a.what == AT_NC_ACT_INSTALL && a.id == 1000);
    /* A on an update installs again */
    ST.sel = 3 + 4; build(&s);
    a = press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(a.kind == AT_NA_ACT && a.what == AT_NC_ACT_INSTALL && a.id == 1004);
    /* A on an installed one opens its detail instead (where it can be removed) */
    ST.sel = 3 + 3; build(&s);
    a = press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(a.kind == AT_NA_NONE && ST.mode == 1 && ST.detail_id == 1003);
    /* a stage skin says why not */
    reset(&s);
    press(&s, ev(AT_EV_PAGE, 1, 0));
    a = press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(a.kind == AT_NA_NONE && strstr(ST.note, "cannot be installed") != NULL);
    /* Z refreshes */
    reset(&s);
    a = press(&s, ev(AT_EV_ALT, 'Z', 0));
    CHECK(a.kind == AT_NA_ACT && a.what == AT_NC_ACT_REFRESH);
    /* Y opens a detail; on a control row Y does nothing */
    ST.sel = 1; a = press(&s, ev(AT_EV_ALT, 'Y', 0)); CHECK(ST.mode == 0);
    ST.sel = 5; build(&s); a = press(&s, ev(AT_EV_ALT, 'Y', 0)); CHECK(ST.mode == 1 && ST.detail_id == 1002);
}

static void detail(void)
{
    AtNcSrc s = src();
    AtNcAction a;
    int i, k;
    reset(&s);
    ST.sel = 3 + 4; build(&s);                                          /* the update one */
    press(&s, ev(AT_EV_ALT, 'Y', 0));
    CHECK(ST.mode == 1);
    CHECK(at_nc_detail_build(&s, &ST, &SC, &VW, 1000.0) == 1);
    CHECK(strcmp(SC.id, "mods.nucleus.detail") == 0 && strcmp(SC.items[0].id, "act") == 0 && strcmp(SC.items[0].label, "Update") == 0);
    CHECK(strcmp(SC.items[1].id, "rm") == 0 && strcmp(SC.items[2].id, "page") == 0);
    {
        int credit = 0, author = 0;
        for (i = 0; i < SC.n_items; i++) { if (strstr(SC.items[i].sub, "Mods from SSBM Nucleus - ssbmnucleus.net")) credit = 1; if (strstr(SC.items[i].sub, "Author 4")) author = 1; }
        CHECK(credit && author && strcmp(VW.ex.from_text, AT_NC_CREDIT) == 0);
    }
    /* A on the first row installs (an update); on Uninstall removes; on the page row opens it */
    ST.detail_sel = 0; at_nc_detail_event(&s, &ST, &(AtEvent){ AT_EV_ACCEPT, 0, 0 }, 1000.0, &a);
    CHECK(a.kind == AT_NA_ACT && a.what == AT_NC_ACT_INSTALL && a.id == 1004);
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
        CHECK(VW.focus.index >= VW.scroll && VW.focus.index < VW.scroll + AT_MODS_DETAIL_VISIBLE);
        at_render_ex(&SC, &VW, 640.0f, 1000.0, 0, &FAKE, &sk, &hits, &info);
        for (j = 0; j < hits.n; j++) if (hits.h[j].kind == AT_HIT_CELL && hits.h[j].b == VW.focus.index) on = 1;
        CHECK(on);
        at_nc_detail_event(&s, &ST, &(AtEvent){ AT_EV_MOVE, AT_DIR_DOWN, 0 }, 1000.0, &a);
    }
    at_nc_detail_event(&s, &ST, &(AtEvent){ AT_EV_BACK, 0, 0 }, 1000.0, &a);
    CHECK(ST.mode == 0);
    /* a not-installed costume has no Uninstall row; an unsupported one says why on its first row */
    reset(&s); ST.detail_id = 1000; ST.mode = 1;
    CHECK(at_nc_detail_build(&s, &ST, &SC, &VW, 1000.0) == 1 && strcmp(SC.items[0].label, "Install") == 0 && strcmp(SC.items[1].id, "page") == 0);
    ST.detail_id = 5001;
    CHECK(at_nc_detail_build(&s, &ST, &SC, &VW, 1000.0) == 1 && strcmp(SC.items[0].label, "Not yet") == 0 && strstr(SC.items[0].sub, "cannot be installed") != NULL);
    ST.detail_id = 999999;
    CHECK(at_nc_detail_build(&s, &ST, &SC, &VW, 1000.0) == 0);          /* a mod that vanished: the caller goes back to the list */
}

static void window_walk(void)
{
    AtNcSrc s = src();
    int k;
    reset(&s);
    for (k = 1; k <= 300; k++) {
        press(&s, ev(AT_EV_MOVE, AT_DIR_DOWN, 0));
        CHECK(SC.n_items <= AT_MAX_ITEMS && VW.focus.index >= 0 && VW.focus.index < SC.n_items);
        CHECK(VW.focus.index >= VW.scroll && VW.focus.index < VW.scroll + AT_NC_VISIBLE);
    }
    CHECK(ST.sel == 0);                                                /* 300 steps down from the first result: wrapped to the top (the Search row) */
}

static void render_fit(void)
{
    AtNcSrc s = src();
    static const float widths[3] = { 640.0f, 960.0f, 1280.0f };
    int w, i, j;
    reset(&s);
    memset(F[0].title, 'W', 79); F[0].title[79] = '\0';
    memset(F[0].author, 'a', 39); F[0].author[39] = '\0';
    for (w = 0; w < 3; w++) {
        static AtHits hits; AtRenderInfo info; AtSink sk;
        at_nc_state_init(&ST); memset(&VW, 0, sizeof VW); build(&s);
        sk = rec_sink();
        at_render_ex(&SC, &VW, widths[w], 1000.0, 0, &FAKE, &sk, &hits, &info);
        CHECK(info.entries < 1500 && !info.capped);
        for (i = 0; i < hits.n; i++) {
            if (hits.h[i].kind != AT_HIT_CELL) continue;
            for (j = 0; j < REC.nt; j++) {
                float x0, x1, y0, y1;
                lint_text_box(&REC.t[j], &x0, &x1, &y0, &y1);
                if (REC.t[j].base < hits.h[i].r.y || REC.t[j].base > hits.h[i].r.y + hits.h[i].r.h) continue;
                if (x0 < hits.h[i].r.x - 0.5f || x0 > hits.h[i].r.x + hits.h[i].r.w) continue;
                CHECK(x1 <= hits.h[i].r.x + hits.h[i].r.w + 0.5f && at_role_size(REC.t[j].role) >= 12);
            }
        }
        /* the five tabs fit and are all hit-testable */
        { int tabs = 0; for (i = 0; i < hits.n; i++) if (hits.h[i].kind == AT_HIT_TAB) tabs++; CHECK(tabs == 5); }
        CHECK(lint_text_overlaps() == 0);
    }
}

int main(void)
{
    rows();
    tabs_and_filters();
    search();
    actions();
    detail();
    window_walk();
    render_fit();
    ATLAS_DONE("atlas nucleus");
}
