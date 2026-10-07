/* gmfrontend_atlas_select.inc - the Atlas drawing of the character select and the stage select (Atlas step 4; included by gmfrontend.c, after
 * gmfrontend_select.inc, TARGET_PC only).
 *
 * THE RULES ARE NOT HERE. They are the host's models (pc/platform/gw_ui_css.c, gw_ui_sss.c), driven through the scalar Ui_Css* and Ui_Sss*
 * shims: game memory is big-endian and the models are native, so this file holds no model and passes no struct. It does four things:
 *   - opens: the legacy fs_css_open / fs_sss_open have already read CSSData / SSSData and the disc's roster (fcs); this file pushes that state into
 *     the models and opens the host's native screen (Ui_SelOpen). Any refusal (MELEE_ATLAS=0, no Atlas fonts, no profile for the mode's match
 *     type, the online lobby) leaves fas.active at 0 and the legacy drawing runs as before: it is a real screen, not a stub;
 *   - reads the pads (the legacy fs_in, the same sources) and the host's mouse events, and steps the models;
 *   - submits the view each frame: the cells only when their signature changes, the cursors, cards, explainer, keys and toast every frame;
 *   - ends the scene through the UNCHANGED legacy writers: the model's ports are copied into fcs.p[] and fs_css_finish writes CSSData exactly as
 *     it always did (a mode's stocks and nametag are never touched), then fs_sss_finish for the stage.
 * The legacy drawing, fs_build_css and fs_build_sss, stays untouched until the owner has looked at the Atlas screens (retirement is Task 14).
 * Disc art: the icons and portraits the legacy roster already borrowed are decoded into the kit's pool at once (Ui_ArtDecode copies the pixels:
 * no pointer into the archive is kept, and nothing is written anywhere). A Geno-defined fighter with no icon of its own submits tex = -1. */

extern int Ui_SelOpen(int kind, int match_type, const char* t0, const char* t1, const char* title);
extern void Ui_SelClose(int h);
extern void Ui_SelTab(int h, int i, const char* name, int count, int active);
extern void Ui_SelBeginCells(int h, int n);
extern void Ui_SelCell(int h, int i, const char* id, const char* name, const char* abbr, int tex, unsigned flags, char origin);
extern void Ui_SelCard(int h, int port, int kind, const char* name, const char* sub, const char* abbr, int tex, int cpu_lv, int team,
                       unsigned flags);
extern void Ui_SelCursor(int h, int port, int active, int cell, int card);
extern void Ui_SelExplain(int h, const char* kicker, const char* title, const char* what, const char* abbr, int tex,
                          const char* stepper_label, const char* stepper_text, const char* from);
extern void Ui_SelKeys(int h, const char* spec);
extern void Ui_SelCounter(int h, const char* text);
extern void Ui_SelNote(int h, const char* text, int kind);
extern int Ui_SelPoll(int h, int* kind, int* a, int* b);
extern int Ui_SelCols(int h);
extern void Ui_SelProgress(int h, int permille);
extern int Settings_Int(const char* key, int dflt);
extern void Ui_ArtDrop(void);
extern int Ui_ArtGeno(int ck, int what, int costume); /* a Geno define's own package art as a kit texture, or -1 (what: 0 icon, 1 portrait) */
extern int Ui_ArtDecode(const char* key, int gx_fmt, const void* img, int size, int w, int h, int tlut_fmt, const void* tlut, int tlut_n);

extern int Ui_CssOpen(int match_type, int online, int teams);
extern void Ui_CssPair(int zelda_ck, int sheik_ck);
extern void Ui_CssSheikOk(int ok);
extern void Ui_CssCostumes(int ck, int n);
extern void Ui_CssRosterBegin(void);
extern void Ui_CssRosterAdd(int ck, int ok, int tab_of);
extern void Ui_CssRosterEnd(void);
extern void Ui_CssPort(int p, int kind, int ck, int costume, int cpu_lv, int team);
extern void Ui_CssEntering(int port);
extern void Ui_CssTraining(int human);
extern void Ui_CssCols(int cols);
extern void Ui_CssSetTab(int tab);
extern void Ui_CssSetPortCk(int p, int ck, int costume);
extern unsigned Ui_CssStep(int port, unsigned trig, unsigned rep, int frame);
extern unsigned Ui_CssMouse(int port, int vis, int card, int moved, int click, int rclick, int wheel);
extern int Ui_CssGet(int what, int i);
extern int Ui_CssBlocker(char* out, int cap);
extern int Ui_CssToast(char* out, int cap);

extern void Ui_SssOpen(int online);
extern void Ui_SssStagesBegin(void);
extern void Ui_SssStageAdd(int ext, int type, int row, int ok, int tab_of);
extern void Ui_SssStagesEnd(void);
extern void Ui_SssCols(int cols);
extern void Ui_SssCursorExt(int ext);
extern void Ui_SssSetTab(int tab);
extern unsigned Ui_SssStep(unsigned trig, unsigned rep, int random_pick);
extern unsigned Ui_SssMouse(int vis, int moved, int click, int wheel);
extern int Ui_SssRandomPool(void);
extern int Ui_SssPickRandom(int r);
extern int Ui_SssGet(int what, int i);
extern int Ui_SssToast(char* out, int cap);

/* the host's numbers (gw_ui_css.h, gw_ui_sss.h, gw_ui_parts.h): game code cannot include a host header, so they are repeated here and the
 * host's tests pin them */
enum { FA_CI_LEFT = 1, FA_CI_RIGHT = 2, FA_CI_UP = 4, FA_CI_DOWN = 8, FA_CI_A = 16, FA_CI_B = 32, FA_CI_X = 64, FA_CI_Y = 128, FA_CI_Z = 256,
       FA_CI_L = 512, FA_CI_R = 1024, FA_CI_START = 2048 };
enum { FA_CE_MOVE = 1, FA_CE_FORWARD = 2, FA_CE_BACK = 4, FA_CE_TOAST = 8, FA_CE_FINISH_GO = 16, FA_CE_FINISH_BACK = 32 };
enum { FA_CELL_LOCKED = 1, FA_CELL_DISABLED = 32 };
enum { FA_CARD_OPEN = 1, FA_CARD_CLOSED = 2, FA_CARD_READY = 4, FA_CARD_FOCUS = 8 };
enum { FA_SEL_CSS = 0, FA_SEL_SSS = 1, FA_SEL_LOAD = 2 };
enum { FA_SELEV_CELL = 1, FA_SELEV_CARD = 2, FA_SELEV_TAB = 3, FA_SELEV_KEY = 4, FA_SELEV_WHEEL = 5 };
enum { FA_NOTE_INFO = 3 };
/* Ui_CssGet / Ui_SssGet selectors (gw_script_ui_sel.inc GS_CSSG_*, GS_SSSG_*) */
enum { FA_CG_KIND = 0, FA_CG_CK, FA_CG_COSTUME, FA_CG_CPU_LV, FA_CG_TEAM, FA_CG_CUR, FA_CG_CARD, FA_CG_TARGET, FA_CG_DONE = 10, FA_CG_N_VIS, FA_CG_TAB,
       FA_CG_COUNT_IN, FA_CG_TRAIN_H, FA_CG_TRAIN_C, FA_CG_N_SLOTS, FA_CG_ONLINE, FA_CG_TEAMS, FA_CG_TAB_OFFERED = 20, FA_CG_TAB_COUNT, FA_CG_VIS_SLOT,
       FA_CG_SLOT_CK, FA_CG_SLOT_OK, FA_CG_SLOT_TAB, FA_CG_VIS_OF_CK, FA_CG_MOUSE_PORT, FA_CG_MAX_HUMANS = 30, FA_CG_CPU_CARDS, FA_CG_MIN_TO_START,
       FA_CG_ENTERING, FA_CG_DUMMY, FA_CG_HAS_SSS };
enum { FA_SG_N = 0, FA_SG_TAB, FA_SG_CUR, FA_SG_DONE, FA_SG_GO, FA_SG_PICK_EXT, FA_SG_N_VIS, FA_SG_VIS_SLOT, FA_SG_EXT, FA_SG_TYPE, FA_SG_OK, FA_SG_ROW,
       FA_SG_TAB_OFFERED, FA_SG_TAB_COUNT, FA_SG_TAB_OF, FA_SG_CUR_TYPE, FA_SG_TAB_FOR_EXT };
#define FA_RETAIL_FIGHTERS 26 /* the disc's own fighters are 0..25; everything above (m-ex and Geno definitions) is "added" */

static struct {
    int active;     ///< an Atlas select owns this scene (the legacy drawing is off)
    int art;        ///< FL_CSS or FL_SSS
    int h;          ///< the host handle
    int finished;   ///< the scene is ending: nothing more is read or submitted
    int sig;        ///< the cells' signature last submitted (-1: none)
    int enter;      ///< a one-player mode's entering port, or -1
    int ntab;       ///< the tabs submitted: position -> model tab id
    int tabid[4];
    int art_ck, art_cos, art_tex;
} fas = { 0, 0, -1, 0, -1, -1 };   /* no handle (a zero would name one), no cells submitted, no entering port */

static bool fas_active(void) { return fas.active != 0; }

/* ---- small helpers -------------------------------------------------------------------------------- */

/* the GX byte size of an image (4x4, 8x4 or 8x8 texel blocks of 32 bytes; RGBA8 64): what Ui_ArtDecode needs to refuse a short buffer */
static int fas_gx_size(int format, int w, int h)
{
    int bw = 4, bh = 4, per = 32;
    switch (format) {
    case 0: /* I4 */
    case 8: /* C4 */
        bw = 8, bh = 8;
        break;
    case 1: /* I8 */
    case 2: /* IA4 */
    case 9: /* C8 */
        bw = 8, bh = 4;
        break;
    case 6: /* RGBA8 */
        per = 64;
        break;
    default: /* IA8, RGB565, RGB5A3, C14X2 */
        break;
    }
    return ((w + bw - 1) / bw) * ((h + bh - 1) / bh) * per;
}

/* an image of the disc's art, decoded at once into the kit's pool: its texture index, or -1 (no image, a format the host cannot read, a full pool) */
static int fas_art(const char* key, FsArt a)
{
    int fmt;
    if (a.img == NULL || a.img->image_ptr == NULL || a.img->width <= 0 || a.img->height <= 0) {
        return -1;
    }
    fmt = (int) a.img->format;
    if ((fmt == 8 || fmt == 9 || fmt == 10) && (a.tl == NULL || a.tl->lut == NULL)) {
        return -1;
    }
    return Ui_ArtDecode(key, fmt, a.img->image_ptr, fas_gx_size(fmt, (int) a.img->width, (int) a.img->height), (int) a.img->width,
                        (int) a.img->height, a.tl != NULL ? (int) a.tl->fmt : 0, a.tl != NULL ? (const void*) a.tl->lut : NULL,
                        a.tl != NULL ? (int) a.tl->n_entries : 0);
}

/* the two letters on a tile with no art: the initials of the first two words ("Captain Falcon" CF, "Mr. Game & Watch" MG), or the first two
 * letters of one word ("Fox" FO); upper case */
static void fas_abbr(const char* name, char* out)
{
    char letters[2], initials[2];
    int nl = 0, ni = 0, i;
    bool start = true;
    for (i = 0; name[i] != '\0'; i++) {
        char c = name[i];
        bool alnum = (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9');
        if (!alnum) {
            start = true;
            continue;
        }
        if (c >= 'a' && c <= 'z') {
            c = (char) (c - 32);
        }
        if (nl < 2) {
            letters[nl++] = c;
        }
        if (start && ni < 2) {
            initials[ni++] = c;
        }
        start = false;
    }
    out[0] = out[1] = out[2] = '\0';
    if (ni >= 2) {
        out[0] = initials[0];
        out[1] = initials[1];
    } else if (nl >= 1) {
        out[0] = letters[0];
        out[1] = nl >= 2 ? letters[1] : '\0';
    } else {
        out[0] = '?';
    }
}

static void fas_fighter_name(int ck, char* out, int cap)
{
    out[0] = '\0';
    if (ck == FS_CK_RANDOM) {
        fp_copy(out, "Random", cap);
    } else if (ck >= 0) {
        if (!Geno_DefineName(ck, out, cap)) {
            Netplay_FighterName(ck, out, cap);
        }
    }
}

/* one log line per fighter and kind of art: the Atlas screens cannot be seen in a screenshot (the host overlay is not in it), so the log is the evidence */
static void fas_geno_art_said(int ck, int what, int tex)
{
    static unsigned char said[128][2];
    if (ck >= 0 && ck < 128 && what >= 0 && what < 2 && !said[ck][what]) {
        said[ck][what] = 1;
        if (what == 0) {
            OSReport("frontend: character select - ck %d icon from its package (kit texture %d)\n", ck, tex);
        } else {
            OSReport("frontend: character select - ck %d portrait from its package (kit texture %d)\n", ck, tex);
        }
    }
}

/* a base "none" define has no donor: its tile and portrait are its package's or nothing (never Mario's) */
static bool fas_ck_is_none(int ck) { return ck >= FA_RETAIL_FIGHTERS && Geno_DefineCostumeCountCK(ck) > 0; }

static int fas_tab_of_ck(int ck) { return ck < FA_RETAIL_FIGHTERS && Geno_DefineBaseCK(ck) < 0 ? 1 : 2; }

/* the model's step input for one port: the same pad sources the legacy fs_in reads */
static void fas_pad(int port, unsigned* trig, unsigned* rep)
{
    FsIn in = fs_in(port);
    unsigned t = 0, r = 0;
    if (in.trig & PAD_BUTTON_A) t |= FA_CI_A;
    if (in.trig & PAD_BUTTON_B) t |= FA_CI_B;
    if (in.trig & PAD_BUTTON_X) t |= FA_CI_X;
    if (in.trig & PAD_BUTTON_Y) t |= FA_CI_Y;
    if (in.trig & PAD_TRIGGER_Z) t |= FA_CI_Z;
    if (in.trig & PAD_TRIGGER_L) t |= FA_CI_L;
    if (in.trig & PAD_TRIGGER_R) t |= FA_CI_R;
    if (in.trig & PAD_BUTTON_START) t |= FA_CI_START;
    if (in.rep & PAD_ANY_LEFT) r |= FA_CI_LEFT;
    if (in.rep & PAD_ANY_RIGHT) r |= FA_CI_RIGHT;
    if (in.rep & PAD_ANY_UP) r |= FA_CI_UP;
    if (in.rep & PAD_ANY_DOWN) r |= FA_CI_DOWN;
    *trig = t;
    *rep = r;
}

/* a key hint the mouse clicked stands for that button (the legacy fms_pad_bits) */
static unsigned fas_key_bits(int btn)
{
    switch (btn) {
    case 'A': return FA_CI_A;
    case 'B': return FA_CI_B;
    case 'X': return FA_CI_X;
    case 'Y': return FA_CI_Y;
    case 'Z': return FA_CI_Z;
    case 'L': return FA_CI_L;
    case 'R': return FA_CI_R;
    case 'S': return FA_CI_START;
    default: return 0;
    }
}

static void fas_sfx(unsigned ev)
{
    if (ev & FA_CE_MOVE) {
        sfxMove();
    }
    if (ev & FA_CE_FORWARD) {
        sfxForward();
    }
    if (ev & FA_CE_BACK) {
        sfxBack();
    }
}

/* the tabs the model offers, as the host's strip: position -> model id; ALL always, RETAIL and ADDED (and TOURNAMENT) only when they hold something */
static const char* const fas_tab_names[4] = { "ALL", "RETAIL", "ADDED", "TOURNAMENT" };

static void fas_submit_tabs(bool sss, int cur_tab)
{
    int t, n = 0;
    for (t = 0; t < 4; t++) {
        int offered = sss ? Ui_SssGet(FA_SG_TAB_OFFERED, t) : (t < 3 ? Ui_CssGet(FA_CG_TAB_OFFERED, t) : 0);
        if (offered) {
            fas.tabid[n] = t;
            Ui_SelTab(fas.h, n, fas_tab_names[t], sss ? Ui_SssGet(FA_SG_TAB_COUNT, t) : Ui_CssGet(FA_CG_TAB_COUNT, t), t == cur_tab);
            n++;
        }
    }
    fas.ntab = n;
}

static void fas_crumbs(int mt, const char** t0, const char** t1)
{
    static const char* const vs[11] = { "MELEE", "CAMERA MODE", "STAMINA", "SUDDEN DEATH", "GIANT", "TINY", "INVISIBLE", "FIXED CAMERA",
                                        "SINGLE BUTTON", "LIGHTNING", "SLO-MO" };
    *t0 = "VERSUS";
    *t1 = "MELEE";
    if (fe_sel_training) {
        *t0 = "SOLO";
        *t1 = "TRAINING";
    } else if (fe_sel_mode != NULL) {
        *t0 = "SOLO";
        *t1 = fe_sel_mode;
    } else if (mt >= 0 && mt <= 0xA) {
        *t1 = vs[mt];
    } else if (mt == 0xB) {
        *t0 = "SOLO";
        *t1 = "CLASSIC";
    } else if (mt == 0xC) {
        *t0 = "SOLO";
        *t1 = "ADVENTURE";
    } else if (mt == 0xD) {
        *t0 = "SOLO";
        *t1 = "ALL-STAR";
    } else if (mt == 0xE) {
        *t0 = "SOLO";
        *t1 = "EVENT MATCH";
    } else if (mt >= 0xF) {
        *t0 = "SOLO";
        *t1 = "STADIUM";
    }
}

/* ---- the character select ------------------------------------------------------------------------- */

/* The art is keyed by fighter or stage, not by disc: the disc is fixed for the life of the process, but the roster is not (a mod toggled changes which fighter or stage
 * a key names). When the list this select shows is not the one the art in the pool was decoded for, the pool is dropped first. kind 0: fighters, 1: stages. */
static void fas_art_identity(int kind, int sig)
{
    static int last[2];
    if (last[kind] != 0 && last[kind] != sig) {
        Ui_ArtDrop();
    }
    last[kind] = sig == 0 ? 1 : sig;
}

static int fas_open_css(void)
{
    CSSData* css = fcs.css;
    const char *t0, *t1;
    int mt, i, p;
    if (css == NULL) {
        return 0;
    }
    if (fcs.online && Settings_Int("atlas_lobby", 0) == 0) {
        return 0; /* the online lobby's pick keeps the legacy screen until a two-client look: MELEE_ATLAS_LOBBY=1 turns the Atlas one on (Task 13 item 9) */
    }
    mt = fcs.online ? 0x40 /* AT_MT_LOBBY */ : fcs.training ? 0x17 : (int) css->match_type;
    if (!Ui_CssOpen(mt, fcs.online ? 1 : 0, fcs.teams)) {
        return 0; /* no profile for this match type: the screen it had stays; never guess */
    }
    {
        int sig = fcs.n * 7919 + Geno_DefineCount() * 104729, k;
        for (k = 0; k < fcs.n; k++) {
            sig = sig * 31 + fcs.ck[k] + 1;
        }
        fas_art_identity(0, sig);
    }
    Ui_CssPair(CKind_Zelda, CKind_Seak);
    Ui_CssCostumes(CKind_Seak, fs_costumes(CKind_Seak));
    Ui_CssSheikOk(fs_sheik_ok() ? 1 : 0);
    Ui_CssRosterBegin();
    for (i = 0; i < fcs.n; i++) {
        Ui_CssCostumes(fcs.ck[i], fs_costumes(fcs.ck[i]));
        Ui_CssRosterAdd(fcs.ck[i], fcs.ok[i] ? 1 : 0, fas_tab_of_ck(fcs.ck[i]));
    }
    Ui_CssRosterEnd();
    for (p = 0; p < 4; p++) {
        Ui_CssPort(p, fcs.p[p].kind, fcs.p[p].ck, fcs.p[p].costume, fcs.p[p].cpu_lv, fcs.p[p].team);
    }
    fas.enter = -1;
    if (fcs.training) {
        Ui_CssTraining(fcs.train_h);
    } else if (Ui_CssGet(FA_CG_ENTERING, 0)) {
        fas.enter = css->unk_0x0 >= 1 && css->unk_0x0 <= 4 ? (int) css->unk_0x0 - 1 : 0;
        Ui_CssEntering(fas.enter);
    }
    fas_crumbs(mt, &t0, &t1);
    if (fcs.online) {
        t0 = "VERSUS";
        t1 = "ONLINE PLAY";
    }
    fas.h = Ui_SelOpen(FA_SEL_CSS, mt, t0, t1, fcs.online ? "PICK A FIGHTER" : "FIGHTERS");
    if (fas.h < 0) {
        fas.enter = -1;
        return 0; /* the legacy screen runs, and its preload writes every port as it always did */
    }
    if (fas.enter >= 0) {
        fs_preload_only = fas.enter; /* Event, Classic, Stadium...: the mode filled the other entries (Sandbag, the Multi-Man, the opponents); set only now the screen is up */
    }
    OSReport("frontend: character select - Atlas, %d fighters, match type %d\n", fcs.n, mt);
    return 1;
}

static const char* fas_css_hint(char* buf)
{
    int lead = 0, p, target, card;
    char why[FE_STR];
    for (p = 0; p < 4; p++) {
        if (Ui_CssGet(FA_CG_KIND, p) == FS_HMN) {
            lead = p;
            break;
        }
    }
    target = Ui_CssGet(FA_CG_TARGET, lead);
    card = Ui_CssGet(FA_CG_CARD, lead);
    if (target != lead) {
        sprintf(buf, "P%d is picking for the P%d CPU: A on a fighter, B to stop.", lead + 1, target + 1);
        return buf;
    }
    if (!Ui_CssBlocker(why, FE_STR)) {
        return Ui_CssGet(FA_CG_HAS_SSS, 0) ? "Ready! Press START to choose the stage." : "Ready! Press START.";
    }
    if (card >= 0) {
        if (card == lead) {
            return "Your card: X/Y costume, A back to the fighters.";
        }
        if (Ui_CssGet(FA_CG_KIND, card) == FS_OFF) {
            return "A: add a CPU here.";
        }
        if (Ui_CssGet(FA_CG_KIND, card) == FS_CPU) {
            return "CPU: A pick its fighter, X/Y level, Z remove.";
        }
        return "Another player's card.";
    }
    return Ui_CssGet(FA_CG_CPU_CARDS, 0) ? "A: pick. X/Y: costume. L/R: tabs. Down: CPUs." : "A: pick. X/Y: costume. L/R: tabs.";
}

static void fas_submit_css(void)
{
    char name[40], abbr[4], buf[FE_HELP_STR], sub[40], id[12], step[24];
    int p, i, n = Ui_CssGet(FA_CG_N_VIS, 0), lead = 0, cur, slot, ck, tex, tab = Ui_CssGet(FA_CG_TAB, 0);
    int sig = tab * 100000 + n * 10 + Ui_CssGet(FA_CG_N_SLOTS, 0);
    bool online = Ui_CssGet(FA_CG_ONLINE, 0) != 0;
    Ui_CssCols(Ui_SelCols(fas.h));
    if (sig != fas.sig) {
        fas.sig = sig;
        fas_submit_tabs(false, tab);
        Ui_SelBeginCells(fas.h, n);
        for (i = 0; i < n; i++) {
            slot = Ui_CssGet(FA_CG_VIS_SLOT, i);
            ck = Ui_CssGet(FA_CG_SLOT_CK, slot);
            fas_fighter_name(ck, name, 40);
            fas_abbr(ck == FS_CK_RANDOM ? "?" : name, abbr);
            sprintf(id, "f%d", ck);
            if (ck == FS_CK_RANDOM) {
                abbr[0] = '?';
                abbr[1] = '\0';
            }
            tex = -1;
            if (ck >= FA_RETAIL_FIGHTERS) {
                tex = Ui_ArtGeno(ck, 0, 0); /* the package's own icon, when it declares one */
                if (tex >= 0) {
                    fas_geno_art_said(ck, 0, tex);
                }
            }
            if (tex < 0 && ck >= 0 && slot < fcs.n && !fas_ck_is_none(ck)) {
                tex = fas_art(id, fcs.icon[slot]);
            }
            Ui_SelCell(fas.h, i, id, name, abbr, tex,
                       Ui_CssGet(FA_CG_SLOT_OK, slot) ? 0u : (unsigned) FA_CELL_DISABLED, Ui_CssGet(FA_CG_SLOT_TAB, slot) == 2 ? '+' : 0);
        }
    }
    for (p = 0; p < 4; p++) {
        int kind = Ui_CssGet(FA_CG_KIND, p), pck = Ui_CssGet(FA_CG_CK, p), cos = Ui_CssGet(FA_CG_COSTUME, p);
        unsigned flags = 0;
        bool closed = (online && p != 0) || (Ui_CssGet(FA_CG_DUMMY, 0) && p != Ui_CssGet(FA_CG_TRAIN_H, 0) && p != Ui_CssGet(FA_CG_TRAIN_C, 0)) ||
                      (fas.enter >= 0 && p != fas.enter);
        name[0] = sub[0] = abbr[0] = '\0';
        if (kind != FS_OFF) {
            if (pck != FS_CK_NONE) {
                fas_fighter_name(pck, name, 40);
                fas_abbr(pck == FS_CK_RANDOM ? "?" : name, abbr);
                if (pck >= 0) {
                    if (!Geno_DefineCostumeName(pck, cos, sub, 24)) {
                        sprintf(sub, "Costume %d", cos + 1);
                    }
                }
            } else {
                fp_copy(name, kind == FS_CPU ? "CPU" : "Choosing...", 40);
            }
        }
        if (closed) {
            flags |= FA_CARD_CLOSED;
        } else if (kind == FS_OFF) {
            flags |= FA_CARD_OPEN;
        }
        if (kind == FS_HMN && pck != FS_CK_NONE) {
            flags |= FA_CARD_READY;
        }
        Ui_SelCard(fas.h, p, kind, name, sub, abbr, -1, Ui_CssGet(FA_CG_CPU_LV, p), Ui_CssGet(FA_CG_TEAMS, 0) ? Ui_CssGet(FA_CG_TEAM, p) + 1 : 0, flags);
        Ui_SelCursor(fas.h, p, kind == FS_HMN && !(online && p != 0) ? 1 : 0, Ui_CssGet(FA_CG_CUR, p), Ui_CssGet(FA_CG_CARD, p));
    }
    for (p = 0; p < 4; p++) {
        if (Ui_CssGet(FA_CG_KIND, p) == FS_HMN) {
            lead = p;
            break;
        }
    }
    cur = Ui_CssGet(FA_CG_CUR, lead);
    slot = Ui_CssGet(FA_CG_VIS_SLOT, cur);
    ck = Ui_CssGet(FA_CG_SLOT_CK, slot);
    fas_fighter_name(ck, name, 40);
    fas_abbr(ck == FS_CK_RANDOM ? "?" : name, abbr);
    {
        int target = Ui_CssGet(FA_CG_TARGET, lead), tck = Ui_CssGet(FA_CG_CK, target);
        tex = -1;
        step[0] = '\0';
        if (ck >= 0) {
            int cos = tck == ck ? Ui_CssGet(FA_CG_COSTUME, target) : 0;
            if (ck != fas.art_ck || cos != fas.art_cos) { /* the portrait decodes once per (fighter, costume) */
                FsArt a;
                a.img = NULL;
                a.tl = NULL;
                sprintf(id, "p%d:%d", ck, cos);
                fas.art_tex = ck >= FA_RETAIL_FIGHTERS ? Ui_ArtGeno(ck, 1, cos) : -1; /* the package's own portrait first */
                if (fas.art_tex >= 0) {
                    fas_geno_art_said(ck, 1, fas.art_tex);
                }
                if (fas.art_tex < 0 && !fas_ck_is_none(ck)) {
                    mnCharSel_PcArtPortrait(ck, cos, &a.img, &a.tl);
                    fas.art_tex = fas_art(id, a);
                }
                fas.art_ck = ck;
                fas.art_cos = cos;
            }
            tex = fas.art_tex;
        }
        if (tck >= 0) {
            char cname[24];
            if (Geno_DefineCostumeName(tck, Ui_CssGet(FA_CG_COSTUME, target), cname, 24)) { /* step[24]: "9 / 16 " and a 16-character name fit */
                cname[16] = '\0';
                sprintf(step, "%d / %d %s", Ui_CssGet(FA_CG_COSTUME, target) + 1, fs_costumes(tck), cname);
            } else {
                sprintf(step, "%d / %d", Ui_CssGet(FA_CG_COSTUME, target) + 1, fs_costumes(tck));
            }
        }
        Ui_SelExplain(fas.h, ck == FS_CK_RANDOM ? "ANY FIGHTER" : "FIGHTER", ck == FS_CK_NONE ? "" : name, fas_css_hint(buf), abbr, tex,
                      step[0] != '\0' ? "COSTUME" : "", step, ck >= 0 && Ui_CssGet(FA_CG_SLOT_TAB, slot) == 2 ? "Added fighter" : "");
    }
    sprintf(buf, "%d / %d", cur + 1, n);
    Ui_SelCounter(fas.h, buf);
    Ui_SelKeys(fas.h, online ? "A:Pick,X:Costume,S:Lock in,B:Back"
                             : fas.ntab > 1 ? "A:Pick,X:Costume,S:Start,B:Back,L:Tabs" : "A:Pick,X:Costume,S:Start,B:Back");
}

static void fas_finish_css(int go)
{
    int p;
    for (p = 0; p < 4; p++) { /* the model's ports into the record the unchanged legacy writer reads */
        fcs.p[p].kind = Ui_CssGet(FA_CG_KIND, p);
        fcs.p[p].ck = Ui_CssGet(FA_CG_CK, p);
        fcs.p[p].costume = Ui_CssGet(FA_CG_COSTUME, p);
        fcs.p[p].cpu_lv = Ui_CssGet(FA_CG_CPU_LV, p);
        fcs.p[p].team = Ui_CssGet(FA_CG_TEAM, p);
    }
    fas.finished = 1;
    fs_css_finish(go); /* writes CSSData exactly as before, preloads, ends the state: the mode's exit handler takes it from there */
}

static void fas_css_frame(void)
{
    unsigned ev = 0, trig[4], rep[4], mouse = 0;
    int p, k, a, b, mp, before_cur, before_card;
    char buf[80];
    bool online = Ui_CssGet(FA_CG_ONLINE, 0) != 0;
    if (fe.frames >= 4) {
        for (p = 0; p < 4; p++) {
            fas_pad(p, &trig[p], &rep[p]);
            if (p != 0 && Ui_CssGet(FA_CG_KIND, p) == FS_HMN && HSD_PadCopyStatus[p].err != 0 && !online) {
                Ui_CssPort(p, FS_OFF, FS_CK_NONE, 0, 9, p & 1); /* a controller left mid-screen */
            }
        }
        mp = Ui_CssGet(FA_CG_MOUSE_PORT, 0);
        before_cur = Ui_CssGet(FA_CG_CUR, mp);
        before_card = Ui_CssGet(FA_CG_CARD, mp);
        while (Ui_SelPoll(fas.h, &k, &a, &b)) { /* the host's mouse: what it hit and with which button */
            if (k == FA_SELEV_CELL) {
                mouse |= Ui_CssMouse(mp, a, -1, b == 0, b == 1, b == 2, 0);
            } else if (k == FA_SELEV_CARD) {
                mouse |= Ui_CssMouse(mp, -1, a, b == 0, b == 1, b == 2, 0);
            } else if (k == FA_SELEV_TAB) {
                if (a >= 0 && a < fas.ntab) {
                    Ui_CssSetTab(fas.tabid[a]);
                    sfxMove();
                }
            } else if (k == FA_SELEV_KEY) {
                mouse |= fas_key_bits(a);
            } else if (k == FA_SELEV_WHEEL) {
                mouse |= Ui_CssMouse(mp, -1, b, 0, 0, 0, a);
            }
        }
        if (Ui_CssGet(FA_CG_CUR, mp) != before_cur || Ui_CssGet(FA_CG_CARD, mp) != before_card) {
            sfxMove();
        }
        trig[mp] |= mouse;
        if (online) { /* the lobby's pick: any controller drives the one card */
            for (p = 1; p < 4; p++) {
                trig[0] |= trig[p];
                rep[0] |= rep[p];
            }
        }
        Ui_CssCols(Ui_SelCols(fas.h));
        for (p = 0; p < 4 && !(ev & (FA_CE_FINISH_GO | FA_CE_FINISH_BACK)); p++) {
            ev |= Ui_CssStep(p, trig[p], rep[p], fe.frames);
        }
        fas_sfx(ev);
        if (ev & FA_CE_TOAST) {
            Ui_CssToast(buf, 80);
            Ui_SelNote(fas.h, buf, FA_NOTE_INFO);
        }
        if (ev & FA_CE_FINISH_GO) {
            if (online && Ui_CssGet(FA_CG_CK, 0) == CKind_Zelda && Ui_CssGet(FA_CG_VIS_OF_CK, CKind_Seak) == Ui_CssGet(FA_CG_VIS_OF_CK, CKind_Zelda) &&
                fs_sheik_ok() && (fs_held(0) & PAD_BUTTON_A))
            {
                int cos = Ui_CssGet(FA_CG_COSTUME, 0);
                Ui_CssSetPortCk(0, CKind_Seak, cos < fs_costumes(CKind_Seak) ? cos : 0); /* A held while locking in: Sheik, as when a match loads */
            }
            fas_finish_css(1);
            return;
        }
        if (ev & FA_CE_FINISH_BACK) {
            fas_finish_css(0);
            return;
        }
    }
    fas_submit_css();
}

/* ---- the stage select ----------------------------------------------------------------------------- */

static int fas_open_sss(void)
{
    const char *t0, *t1;
    int i, mt = fe_sel_training ? 0x17 : 0;
    {
        int sig = fcs.ns * 7919, k;
        for (k = 0; k < fcs.ns; k++) {
            sig = sig * 31 + fcs.st_ext[k] + 2;
        }
        fas_art_identity(1, sig);
    }
    Ui_SssOpen(0);
    Ui_SssStagesBegin();
    for (i = 0; i < fcs.ns; i++) {
        Ui_SssStageAdd(fcs.st_ext[i], fcs.st_type[i], fcs.st_row[i], fcs.st_ok[i] ? 1 : 0,
                       fcs.st_type[i] == 3 ? 0 : Ui_SssGet(FA_SG_TAB_FOR_EXT, fcs.st_ext[i]));
    }
    Ui_SssStagesEnd();
    if (fcs.st_cur >= 0 && fcs.st_cur < fcs.ns && fcs.st_type[fcs.st_cur] != 3) {
        Ui_SssCursorExt(fcs.st_ext[fcs.st_cur]);
    }
    fas_crumbs(mt, &t0, &t1);
    fas.h = Ui_SelOpen(FA_SEL_SSS, mt, t0, t1, "STAGES");
    if (fas.h < 0) {
        return 0;
    }
    OSReport("frontend: stage select - Atlas, %d stages\n", fcs.ns);
    return 1;
}

static void fas_submit_sss(void)
{
    char name[64], abbr[4], id[12];
    int i, n = Ui_SssGet(FA_SG_N_VIS, 0), tab = Ui_SssGet(FA_SG_TAB, 0), sig = tab * 100000 + n * 10 + Ui_SssGet(FA_SG_N, 0);
    Ui_SssCols(Ui_SelCols(fas.h));
    if (sig != fas.sig) {
        fas.sig = sig;
        fas_submit_tabs(true, tab);
        Ui_SelBeginCells(fas.h, n);
        for (i = 0; i < n; i++) {
            int slot = Ui_SssGet(FA_SG_VIS_SLOT, i), ext = Ui_SssGet(FA_SG_EXT, slot), type = Ui_SssGet(FA_SG_TYPE, slot);
            bool ok = Ui_SssGet(FA_SG_OK, slot) != 0;
            if (type == 3) {
                fp_copy(name, "Random", 64);
                abbr[0] = '?';
                abbr[1] = '\0';
                sprintf(id, "random");
            } else {
                fs_stage_name(ext, name, 64);
                fas_abbr(name, abbr);
                sprintf(id, "s%d", ext);
            }
            Ui_SelCell(fas.h, i, id, name, abbr, type != 3 && slot < fcs.ns ? fas_art(id, fcs.st_icon[slot]) : -1, ok ? 0u : (unsigned) FA_CELL_LOCKED,
                       type != 3 && Ui_SssGet(FA_SG_TAB_OF, slot) == 2 ? '+' : 0);
        }
    }
    Ui_SelCursor(fas.h, 0, 1, Ui_SssGet(FA_SG_CUR, 0), -1);
    {
        char buf[24];
        sprintf(buf, "%d / %d", Ui_SssGet(FA_SG_CUR, 0) + 1, n);
        Ui_SelCounter(fas.h, buf);
    }
    Ui_SelKeys(fas.h, fas.ntab > 1 ? "A:Choose,B:Back,L:Tabs" : "A:Choose,B:Back");
    Ui_SelExplain(fas.h, "", "", "", "", -1, "", "", "");
}

static void fas_sss_frame(void)
{
    unsigned ev, trig = 0, rep = 0, mouse = 0;
    int p, k, a, b, rp = 0;
    char buf[48];
    if (fe.frames >= 4) {
        if (fcs.sss != NULL && fcs.sss->force_stage_id >= 0) {
            fas.finished = 1;
            fs_sss_finish(1, fcs.sss->force_stage_id); /* the rules chose the stage */
            return;
        }
        for (p = 0; p < 4; p++) {
            unsigned t, r;
            fas_pad(p, &t, &r);
            trig |= t;
            rep |= r;
        }
        while (Ui_SelPoll(fas.h, &k, &a, &b)) {
            if (k == FA_SELEV_CELL) {
                mouse |= Ui_SssMouse(a, b == 0, b == 1, 0);
                if (b == 2) {
                    mouse |= FA_CI_B;
                }
            } else if (k == FA_SELEV_TAB) {
                if (a >= 0 && a < fas.ntab) {
                    Ui_SssSetTab(fas.tabid[a]);
                    sfxMove();
                }
            } else if (k == FA_SELEV_KEY) {
                mouse |= fas_key_bits(a);
            } else if (k == FA_SELEV_WHEEL) {
                mouse |= Ui_SssMouse(-1, 0, 0, a);
            }
        }
        trig |= mouse;
        if ((trig & (FA_CI_A | FA_CI_START)) && Ui_SssGet(FA_SG_CUR_TYPE, 0) == 3) {
            int pool = Ui_SssRandomPool();
            rp = Ui_SssPickRandom(HSD_Randi(pool > 0 ? pool : 1)); /* rolled only on a pick, as the legacy did */
        }
        ev = Ui_SssStep(trig, rep, rp);
        fas_sfx(ev);
        if (ev & FA_CE_TOAST) {
            Ui_SssToast(buf, 48);
            Ui_SelNote(fas.h, buf, FA_NOTE_INFO);
        }
        if (ev & FA_CE_FINISH_GO) {
            fas.finished = 1;
            fs_sss_finish(1, Ui_SssGet(FA_SG_PICK_EXT, 0));
            return;
        }
        if (ev & FA_CE_FINISH_BACK) {
            fas.finished = 1;
            fs_sss_finish(0, 0);
            return;
        }
    }
    fas_submit_sss();
}

/* ---- the loading screen ("GET READY") -------------------------------------------------------------- */

/* The warm-up and the hold are the game's (fe_load_step, fl_frame_load: untouched); this only draws who fights where, the stage and the bar. fl_ld is the legacy
 * screen's own list of fighters, read the same way. The online loading screen (the lobby's) follows the lobby's gate. */
static int fas_open_loading(void)
{
    if (fl_ld.online && Settings_Int("atlas_lobby", 0) == 0) {
        return 0;
    }
    fas.h = Ui_SelOpen(FA_SEL_LOAD, 0, "VERSUS", fl_ld.online ? "ONLINE PLAY" : "MELEE", "GET READY");
    if (fas.h < 0) {
        return 0;
    }
    OSReport("frontend: loading screen - Atlas, %d fighter(s)\n", fl_ld.n);
    return 1;
}

static void fas_loading_frame(void)
{
    char name[40], sub[24], abbr[4], stage[64];
    int i, p;
    for (p = 0; p < 4; p++) {
        Ui_SelCard(fas.h, p, 0, "", "", "", -1, 0, 0, 0u);
    }
    for (i = 0; i < fl_ld.n && i < 4; i++) {
        int ck = fl_ld.ck[i], cpu = fl_ld.cpu[i], port = fl_ld.port[i] & 3;
        name[0] = '\0';
        if (ck >= 0) {
            fas_fighter_name(ck, name, 40);
        }
        fas_abbr(name, abbr);
        sub[0] = '\0';
        if (fl_ld.online) {
            fp_copy(sub, i == fl_ld.me ? "YOU" : "OPPONENT", 24);
        }
        Ui_SelCard(fas.h, port, cpu ? FS_CPU : FS_HMN, name, sub, abbr, -1, cpu, 0, (fl_ld.online && i == fl_ld.me) ? (unsigned) FA_CARD_FOCUS : 0u);
    }
    Netplay_StageNameExt(fl_ld.ext, stage, 64);
    Ui_SelExplain(fas.h, "STAGE", stage, "", "", -1, "", "", "");
    Ui_SelProgress(fas.h, (int) (fe.progress * 1000.0F));
    Ui_SelCounter(fas.h, fl_ld.ready ? "READY" : "WARMING UP");
    Ui_SelKeys(fas.h, "");
}

/* ---- the hooks gmfrontend_online.inc calls (fl_open, fl_frame, fl_exit) ------------------------------- */

/* Called from fl_open for the CSS and the SSS, after the legacy open has filled fcs. 1: an Atlas screen is up and the legacy drawing stays off. */
static int fas_open(int art)
{
    if (fas.h >= 0) {
        Ui_SelClose(fas.h); /* a screen of this scene being opened again: the host allows one select at a time */
    }
    memset(&fas, 0, sizeof fas);
    fas.h = -1;
    fas.sig = -1;
    fas.enter = -1;
    fas.art_ck = -9;
    if (Ui_Ready() == 0) {
        return 0; /* MELEE_ATLAS=0 or no Atlas fonts: the legacy screen */
    }
    if (art == FL_CSS) {
        fas.active = fas_open_css();
    } else if (art == FL_SSS) {
        fas.active = fas_open_sss();
    } else if (art == FL_LOAD) {
        fas.active = fas_open_loading();
    }
    fas.art = art;
    return fas.active;
}

static void fas_frame(void)
{
    if (!fas.active || fas.finished) {
        return; /* ending: the last frame stays up until the scene's exit closes it */
    }
    if (fas.art == FL_CSS) {
        fas_css_frame();
    } else if (fas.art == FL_SSS) {
        fas_sss_frame();
    } else {
        fas_loading_frame();
    }
}

/* the scene ends: the host closes every screen of it too (Ui_SceneExit), this only makes sure the handle is gone */
static void fas_exit(void)
{
    fs_preload_only = -1;
    if (fas.h >= 0) {
        Ui_SelClose(fas.h);
    }
    memset(&fas, 0, sizeof fas);
    fas.h = -1;
}
