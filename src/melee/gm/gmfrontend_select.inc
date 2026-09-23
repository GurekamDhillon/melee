/* gmfrontend_select.inc - included by gmfrontend.c (TARGET_PC only), after the room screens.
 *
 * ---- THE CHARACTER SELECT AND THE STAGE SELECT, on the kit ---------------------------------
 *
 * VS mode's CSS and SSS states run this frontend scene instead of the native screens
 * (gmFrontend_SelectScene, called from their on_enter; MELEE_NATIVE_CSS=1 keeps the native ones).
 * The scene receives the state's own enter data (gmVsMelee_CssData / gmVsMelee_SssData), fills it
 * exactly as the native screen would - players' kinds, fighters, costumes, CPU levels, teams;
 * the stage and start_game - and ends the scene, so the states' exit handlers run unchanged.
 *
 * WHAT IS SHOWN comes from the disc, like the native screens: the roster is MnSlChr's icon table
 * (retail's 25, or m-ex's mexData table - whatever fighters the disc and mod set offer), the
 * stages MnSlMap's (m-ex: every added stage); the icons and portraits are the disc's own art,
 * borrowed from models mncharsel.c / mnstagesel.c animate for us (PcArt). Nothing is exported.
 *
 * CSS - four ports. Each connected controller moves its own cursor (its port colour) over the
 * grid; A picks for its port, X/Y change the costume, L/R turn pages, START starts once at least
 * two fighters are in and every human has picked. Below the grid, a card per port: move the
 * cursor down onto a card to add a CPU (A on an empty card), pick its fighter (A on a CPU card,
 * then A on a fighter), change its level (X/Y) or remove it (Z). A player leaves with Z on their
 * own card; B undoes, and B twice with nothing picked goes back. Online (the lobby's pick): one
 * card, and a fighter the opponent does not have is greyed out.
 *
 * SSS - one grid, pages, a plate naming the hovered stage. A picks, START picks the hovered one,
 * B goes back to the characters. A stage the disc ships is unlocked (m-ex stages have no unlock
 * bits on a vanilla save); retail stages follow the save, or MELEE_UNLOCK_ALL. */

extern int mnCharSel_PcArtOpen(HSD_Archive* archive);
extern int mnCharSel_PcArtIcon(int i, int* ck, int* ext, HSD_ImageDesc** img, HSD_Tlut** tlut);
extern int mnCharSel_PcArtPortrait(int ck, int costume, HSD_ImageDesc** img, HSD_Tlut** tlut);
extern int mnStageSel_PcArtOpen(HSD_Archive* archive);
extern int mnStageSel_PcArtIcon(int i, int* ext, int* type, HSD_ImageDesc** img, HSD_Tlut** tlut);
extern int mnStageSel_PcArtName(int i, HSD_ImageDesc** img, HSD_Tlut** tlut);

/* One image off the disc: an image and its palette, both living in an archive. */
typedef struct FsArt {
    HSD_ImageDesc* img;
    HSD_Tlut* tl;
} FsArt;
extern int Mex_GrKindForExt(int ext);
extern int Settings_Str(const char* key, char* out, int cap, const char* dflt);
extern const char* Mex_GrFile(int grkind);

#define FS_MAX_FIGHTERS 128 /* CSS_ICON_MAX */
#define FS_MAX_STAGES 256
#define FS_CK_NONE (-1)
#define FS_CK_RANDOM (-2)
enum { FS_OFF, FS_HMN, FS_CPU };

/* ---- the disc's art, borrowed ----------------------------------------------------------------- */

static struct {
    HSD_Archive* chr;
    HSD_Archive* map;
    int nchr, nmap;
} fa;

/* the stage select's icons by external id (fs_stage_art_tex) */
static struct {
    int n;
    int ext[256];
    FsArt art[256];
    int built;
} fa_st;

static void fa_close(void)
{
    if (fa.chr != NULL) {
        lbArchive_80016EFC(fa.chr);
    }
    if (fa.map != NULL) {
        lbArchive_80016EFC(fa.map);
    }
    fa.chr = fa.map = NULL;
    fa.nchr = fa.nmap = 0;
    fa_st.built = 0;
}

static int fa_chr_open(void)
{
    if (fa.chr == NULL) {
        fa.chr = lbArchive_LoadArchive(lbLang_IsSavedLanguageUS() ? "MnSlChr.usd" : "MnSlChr.dat");
        fa.nchr = mnCharSel_PcArtOpen(fa.chr);
        OSReport("frontend: select - MnSlChr %s, %d icons\n", fa.chr != NULL ? "loaded" : "MISSING",
                 fa.nchr);
    }
    return fa.nchr;
}

static int fa_map_open(void)
{
    if (fa.map == NULL) {
        fa.map = lbArchive_LoadArchive(lbLang_IsSavedLanguageUS() ? "MnSlMap.usd" : "MnSlMap.dat");
        fa.nmap = mnStageSel_PcArtOpen(fa.map);
        OSReport("frontend: select - MnSlMap %s, %d icons\n", fa.map != NULL ? "loaded" : "MISSING",
                 fa.nmap);
    }
    return fa.nmap;
}

/* An image off the disc, lent to the player (the pixels stay in the archive). */
static int fs_art_tex(FsArt a)
{
    FpTex* tx;
    HSD_ImageDesc* im = a.img;
    HSD_Tlut* tl = a.tl;
    if (im == NULL || im->image_ptr == NULL || fp.ntex >= FP_MAX_TEX) {
        return -1;
    }
    tx = &fp.tex[fp.ntex];
    memset(tx, 0, sizeof *tx);
    sprintf(tx->name, "@t%d", fp.ntex);
    tx->borrowed = true;
    tx->t.data = im->image_ptr;
    tx->t.w = im->width;
    tx->t.h = im->height;
    if ((im->format == GX_TF_C4 || im->format == GX_TF_C8 || im->format == GX_TF_C14X2) &&
        tl != NULL && tl->lut != NULL)
    {
        tx->t.lut = tl->lut;
        GXInitTlutObj(&tx->t.tlut, tl->lut, (GXTlutFmt) tl->fmt, tl->n_entries);
        GXInitTexObjCI(&tx->t.obj, im->image_ptr, im->width, im->height, (GXTexFmt) im->format,
                       GX_CLAMP, GX_CLAMP, GX_FALSE, GX_TLUT0);
    } else {
        GXInitTexObj(&tx->t.obj, im->image_ptr, im->width, im->height, (GXTexFmt) im->format,
                     GX_CLAMP, GX_CLAMP, GX_FALSE);
    }
    GXInitTexObjLOD(&tx->t.obj, GX_LINEAR, GX_LINEAR, 0.0F, 0.0F, 0.0F, GX_FALSE, GX_FALSE,
                    GX_ANISO_1);
    tx->t.ok = true;
    tx->mask = im->format == GX_TF_I4 || im->format == GX_TF_IA4 || im->format == GX_TF_I8 ||
               im->format == GX_TF_IA8;
    return fp.ntex++;
}

/* Rebuilds lend new textures: drop the previous build's borrowed ones, keep the loaded ones. */
static void fs_drop_borrowed(void)
{
    int i, k = 0;
    for (i = 0; i < fp.ntex; i++) {
        if (!fp.tex[i].borrowed) {
            if (k != i) {
                fp.tex[k] = fp.tex[i];
            }
            k++;
        }
    }
    fp.ntex = k;
}

/* The stage select's icon for a stage (its EXTERNAL id), lent to the player; -1 if none. The
   icon table is read once per scene (the lobby opens MnSlMap for its stage tiles). */
static int fs_stage_art_tex(int ext)
{
    int i;
    if (fa.map == NULL || !fa_st.built) {
        int n = fa_map_open();
        fa_st.n = 0;
        for (i = 0; i < n && fa_st.n < FS_MAX_STAGES; i++) {
            int e = 0, type = 0;
            FsArt a;
            if (mnStageSel_PcArtIcon(i, &e, &type, &a.img, &a.tl) && type == 2 && a.img != NULL) {
                fa_st.ext[fa_st.n] = e;
                fa_st.art[fa_st.n] = a;
                fa_st.n++;
            }
        }
        fa_st.built = 1;
    }
    for (i = 0; i < fa_st.n; i++) {
        if (fa_st.ext[i] == ext) {
            return fs_art_tex(fa_st.art[i]);
        }
    }
    return -1;
}

/* A quad showing a texture inside a rect, keeping the texture's aspect (centred). */
static int fs_tex_fit(const char* id, int tex, float x0, float y0, float x1, float y1,
                      const float* rgb)
{
    float w, h, tw, th, s;
    if (tex < 0) {
        return -1;
    }
    tw = (float) fp.tex[tex].t.w;
    th = (float) fp.tex[tex].t.h;
    w = x1 - x0;
    h = y1 - y0;
    if (tw <= 0.0F || th <= 0.0F) {
        return -1;
    }
    s = w / tw < h / th ? w / tw : h / th;
    tw *= s;
    th *= s;
    x0 += (w - tw) * 0.5F;
    y0 += (h - th) * 0.5F;
    return fl_rect_quad(id, tex, x0, y0, x0 + tw, y0 + th, rgb, -1);
}

static void fs_tok(const char* tok, float* rgb) { fm_tok(tok, KS_VERSUS, rgb); }

static void fs_port_rgb(int p, float* rgb)
{
    char t[8];
    sprintf(t, "port:p%d", (p & 3) + 1);
    if (!kit_colour(t, KS_VERSUS, rgb)) {
        rgb[0] = rgb[1] = rgb[2] = 1.0F;
    }
}

/* A rectangle's outline, `w` thick (4 flats). */
static void fs_outline(const char* id, float x0, float y0, float x1, float y1, float w,
                       const float* rgb)
{
    char q[FP_ID];
    sprintf(q, "%.20s_t", id);
    fm_flat(q, FPR_OTHER, -1, x0, y0, x1, y0 + w, rgb);
    sprintf(q, "%.20s_b", id);
    fm_flat(q, FPR_OTHER, -1, x0, y1 - w, x1, y1, rgb);
    sprintf(q, "%.20s_l", id);
    fm_flat(q, FPR_OTHER, -1, x0, y0, x0 + w, y1, rgb);
    sprintf(q, "%.20s_r", id);
    fm_flat(q, FPR_OTHER, -1, x1 - w, y0, x1, y1, rgb);
}

/* Per-port input: edges (A, B, X, Y, Z, L, R, START) and held-repeat directions. */
typedef struct FsIn {
    u64 trig, rep;
} FsIn;

static FsIn fs_in(int port)
{
    FsIn r;
    r.trig = gm_GetButtonsTriggered((u8) port);
    r.rep = gm_801A36C0((u8) port);
    return r;
}

/* ---- CSS -------------------------------------------------------------------------------------- */

typedef struct FsPort {
    int kind;    ///< FS_OFF / FS_HMN / FS_CPU
    int ck;      ///< FS_CK_NONE, FS_CK_RANDOM, or a CharacterKind
    int costume, cpu_lv, team;
    int cur;     ///< the cursor: a roster slot (random = n)
    int card;    ///< -1 on the grid, else the card it is on
    int target;  ///< the port its picks go to (itself, or a CPU it took)
} FsPort;

static struct {
    CSSData* css;
    SSSData* sss;
    int online;
    int n;                        ///< fighters on offer (the Random tile is slot n)
    int ck[FS_MAX_FIGHTERS];
    FsArt icon[FS_MAX_FIGHTERS];
    bool ok[FS_MAX_FIGHTERS];     ///< online: the opponent has it too
    int page, pages, cols, rows, per;
    FsPort p[4];
    int back_until;
    int done;                     ///< the scene is ending
    int teams;
    char toast[FE_STR];
    int toast_until;
    /* SSS */
    int ns;
    int st_ext[FS_MAX_STAGES], st_type[FS_MAX_STAGES];
    bool st_ok[FS_MAX_STAGES];
    FsArt st_icon[FS_MAX_STAGES];
    int st_row[FS_MAX_STAGES];       ///< its row in the disc's icon table (for its name art)
    int st_cur;
} fcs;

/* Buttons held now on this port (online: on any port, like fs_css_port's input). */
static u32 fs_held(int port)
{
    u32 b = HSD_PadCopyStatus[port].button;
    if (fcs.online) {
        int q;
        for (q = 0; q < 4; q++) {
            b |= HSD_PadCopyStatus[q].button;
        }
    }
    return b;
}

static void fs_toast(const char* s)
{
    fp_copy(fcs.toast, s, FE_STR);
    fcs.toast_until = fl.frames + 150;
}

static int fs_slots(void) { return fcs.n + 1; } /* the fighters and the Random tile */

/* ZELDA / SHEIK. Retail's roster has no Sheik tile: she shares Zelda's, and the game swaps them
 * when A is held as the match loads (gmvs.c). Here A on the Zelda tile once more swaps the pick
 * between Zelda and Sheik (the card names which), and online - where the lobby agrees the fighter
 * before the match, so the load-time swap is off - A held while START locks the pick in gives
 * Sheik too, the game's own gesture. A disc whose roster has a Sheik tile of its own (an m-ex
 * layout can) needs none of this. */
static bool fs_sheik_shares_zelda(void)
{
    int i;
    bool zelda = false;
    for (i = 0; i < fcs.n; i++) {
        if (fcs.ck[i] == CKind_Seak) {
            return false;
        }
        zelda |= fcs.ck[i] == CKind_Zelda;
    }
    return zelda;
}

static bool fs_sheik_ok(void)
{
    return !fcs.online || Netplay_FighterAvailable(CKind_Seak);
}

static int fs_slot_ck(int slot)
{
    return slot >= 0 && slot < fcs.n ? fcs.ck[slot] : slot == fcs.n ? FS_CK_RANDOM : FS_CK_NONE;
}

static bool fs_slot_ok(int slot)
{
    return slot == fcs.n || (slot >= 0 && slot < fcs.n && fcs.ok[slot]);
}

static int fs_costumes(int ck)
{
    int n = ck >= 0 ? gm_GetNumCostumesForCKind((u8) ck) : 1;
    return n > 0 ? n : 1;
}

/* The first costume of `ck` nobody else in the match wears (from `from`, stepping by `dir`). */
static int fs_free_costume(int port, int ck, int from, int dir)
{
    int n = fs_costumes(ck), k, c, q;
    for (k = 0; k < n; k++) {
        bool taken = false;
        c = ((from + dir * k) % n + n) % n;
        for (q = 0; q < 4; q++) {
            if (q != port && fcs.p[q].kind != FS_OFF && fcs.p[q].ck == ck &&
                fcs.p[q].costume == c)
            {
                taken = true;
            }
        }
        if (!taken) {
            return c;
        }
    }
    return from;
}

static void fs_css_roster(void)
{
    int i, n = fa_chr_open();
    fcs.n = 0;
    for (i = 0; i < n && fcs.n < FS_MAX_FIGHTERS; i++) {
        int ck = -1, ext = -1, k;
        FsArt t;
        bool dup = false;
        if (!mnCharSel_PcArtIcon(i, &ck, &ext, &t.img, &t.tl)) {
            continue;
        }
        /* a fighter the port lacks, or locked on this save (MELEE_UNLOCK_ALL opens all) */
        if (ck < 0 || ck >= FE_NP_MAX_CK || !gm_IsCKindUnlocked((u8) ck)) {
            continue;
        }
        for (k = 0; k < fcs.n; k++) {
            dup |= fcs.ck[k] == ck;
        }
        if (dup) {
            continue;
        }
        fcs.ck[fcs.n] = ck;
        fcs.icon[fcs.n] = t;
        fcs.ok[fcs.n] = !fcs.online || Netplay_FighterAvailable(ck);
        fcs.n++;
    }
    fcs.cols = 11;
    fcs.rows = 4;
    fcs.per = fcs.cols * fcs.rows;
    fcs.pages = (fs_slots() + fcs.per - 1) / fcs.per;
    OSReport("frontend: character select - %d fighters on offer, %d page(s)%s\n", fcs.n, fcs.pages,
             fcs.online ? " (online)" : "");
    {
        /* the roster's order, for tests that walk the grid (ck per slot) */
        char line[FS_MAX_FIGHTERS * 4 + 8];
        int k, o = 0;
        for (k = 0; k < fcs.n && o < (int) sizeof line - 5; k++) {
            o += sprintf(line + o, "%s%d", k ? "," : "", fcs.ck[k]);
        }
        line[o] = '\0';
        OSReport("frontend: character select - roster %s\n", line);
    }
}

static int fs_slot_of(int ck)
{
    int i;
    if (ck == CKind_Seak && fs_sheik_shares_zelda()) {
        ck = CKind_Zelda; /* Sheik lives on Zelda's tile */
    }
    for (i = 0; i < fcs.n; i++) {
        if (fcs.ck[i] == ck) {
            return i;
        }
    }
    return 0;
}

static void fs_css_open(CSSData* css)
{
    int p;
    memset(&fcs, 0, sizeof fcs);
    fcs.css = css;
    fcs.online = Frontend_OnlinePick() == 1;
    fcs.teams = css != NULL && css->vs.start.rules.is_teams && !fcs.online;
    fs_css_roster();
    for (p = 0; p < 4; p++) {
        FsPort* s = &fcs.p[p];
        PlayerInitData* pl = css != NULL ? &css->vs.start.players[p] : NULL;
        s->card = -1;
        s->target = p;
        s->ck = FS_CK_NONE;
        s->cpu_lv = 9;
        s->team = p & 1;
        if (pl != NULL) {
            if (pl->slot_type == Gm_PKind_Human) {
                s->kind = FS_HMN;
            } else if (pl->slot_type == Gm_PKind_Cpu) {
                s->kind = FS_CPU;
                s->cpu_lv = pl->cpu_level >= 1 && pl->cpu_level <= 9 ? pl->cpu_level : 9;
            }
            if (s->kind != FS_OFF && pl->ckind >= 0 && pl->ckind < FE_NP_MAX_CK &&
                pl->ckind != ChKind_None)
            {
                s->ck = pl->ckind;
                s->costume = pl->color;
            }
            if (pl->team <= 2) {
                s->team = pl->team;
            }
        }
        if (fcs.online) {
            /* the lobby's pick: one fighter, this player's; nothing chosen until A */
            s->kind = p == 0 ? FS_HMN : FS_OFF;
            s->ck = FS_CK_NONE;
        }
        if (s->kind == FS_HMN && p != 0 && HSD_PadCopyStatus[p].err != 0) {
            s->kind = FS_OFF; /* no controller in that port any more */
        }
        s->cur = s->ck >= 0 ? fs_slot_of(s->ck) : 0;
    }
    if (fcs.p[0].kind == FS_OFF) {
        fcs.p[0].kind = FS_HMN; /* whoever came here is P1 */
    }
    fcs.page = fcs.p[0].cur / fcs.per;
}

static int fs_count_in(void)
{
    int p, k = 0;
    for (p = 0; p < 4; p++) {
        k += fcs.p[p].kind != FS_OFF && fcs.p[p].ck != FS_CK_NONE;
    }
    return k;
}

/* Who is still missing, or NULL when the match can start. */
static const char* fs_css_blocker(char* buf)
{
    int p;
    for (p = 0; p < 4; p++) {
        if (fcs.p[p].kind == FS_HMN && fcs.p[p].ck == FS_CK_NONE) {
            sprintf(buf, "P%d: pick a fighter.", p + 1);
            return buf;
        }
    }
    if (fs_count_in() < (fcs.online ? 1 : 2)) {
        sprintf(buf, "%s", "Two fighters needed: pick one, or add a CPU below.");
        return buf;
    }
    return NULL;
}

static int fs_random_ck(void)
{
    int k = 0, i, pick;
    for (i = 0; i < fcs.n; i++) {
        k += fcs.ok[i];
    }
    if (k == 0) {
        return fcs.n > 0 ? fcs.ck[0] : 0;
    }
    pick = HSD_Randi(k);
    for (i = 0; i < fcs.n; i++) {
        if (fcs.ok[i] && pick-- == 0) {
            return fcs.ck[i];
        }
    }
    return fcs.ck[0];
}

/* The preload cache has to hear who is playing and where, as the native screens tell it
 * (mncharsel.c every frame, mnstagesel.c on start). Without it nothing is preloaded, and the
 * match loads each fighter's animation file on demand into the ARAM heap - which a VS state
 * leaves at ~160 KB, so any fighter ended in lbMemory's "memp_kouho" assert. */
static void fs_preload_players(const CSSData* css)
{
    struct GameCache* cache = &lbDvd_GetPreloadCacheScene()->game_cache;
    int p;
    for (p = 0; p < 4; p++) {
        const PlayerInitData* pl = &css->vs.start.players[p];
        if (pl->slot_type == Gm_PKind_Human || pl->slot_type == Gm_PKind_Cpu) {
            cache->entries[p].char_id = pl->ckind;
            cache->entries[p].color = pl->color;
        } else {
            cache->entries[p].char_id = ChKind_None;
        }
    }
    lbDvd_80018254();
}

static void fs_preload_stage(int ext)
{
    lbDvd_GetPreloadCacheScene()->game_cache.stkind = ext;
    lbDvd_80018254();
}

static void fs_css_finish(int go)
{
    int p;
    CSSData* css = fcs.css;
    fcs.done = 1;
    for (p = 0; p < fcs.n; p++) {
        fe_np_store(fcs.ck[p], fcs.icon[p].img, fcs.icon[p].tl); /* the lobby's and loading screen's portraits */
    }
    if (css != NULL) {
        if (go) {
            for (p = 0; p < 4; p++) {
                FsPort* s = &fcs.p[p];
                PlayerInitData* pl = &css->vs.start.players[p];
                int ck = s->ck == FS_CK_RANDOM ? fs_random_ck() : s->ck;
                if (s->kind == FS_OFF || ck < 0) {
                    pl->slot_type = Gm_PKind_NA;
                    continue;
                }
                pl->slot_type = s->kind == FS_HMN ? Gm_PKind_Human : Gm_PKind_Cpu;
                pl->ckind = (s8) ck;
                pl->color = (u8) (s->costume < fs_costumes(ck) ? s->costume : 0);
                if (fcs.teams) {
                    pl->team = (u8) s->team;
                    pl->color = s->team == 1   ? gm_801692BC((u8) ck)
                                : s->team == 2 ? gm_80169290((u8) ck)
                                               : gm_80169264((u8) ck);
                }
                if (s->kind == FS_CPU) {
                    pl->cpu_level = (u8) s->cpu_lv;
                }
                OSReport("frontend: character select - P%d %s ck %d costume %d%s\n", p + 1,
                         s->kind == FS_HMN ? "human" : "CPU", ck, pl->color,
                         s->ck == FS_CK_RANDOM ? " (random)" : "");
            }
            fs_preload_players(css);
            css->pending_scene_change = 1;
        } else {
            css->pending_scene_change = CSSPendingSceneChange_2;
        }
    }
    gm_801A4B60(); /* the state's exit handler (gmVsMelee_ExitCss) takes it from here */
}

/* Pick the slot under the cursor for the port this cursor picks for. */
static void fs_pick(int port, int slot)
{
    FsPort* me = &fcs.p[port];
    FsPort* t = &fcs.p[me->target];
    int ck = fs_slot_ck(slot);
    if (ck == FS_CK_NONE) {
        return;
    }
    if (!fs_slot_ok(slot)) {
        sfxBack();
        fs_toast("Your opponent doesn't have this fighter - pick another.");
        return;
    }
    if (ck == CKind_Zelda && fs_sheik_shares_zelda() && (t->ck == CKind_Zelda || t->ck == CKind_Seak)) {
        /* A on the Zelda tile again: swap Zelda and Sheik */
        if (t->ck == CKind_Zelda && !fs_sheik_ok()) {
            sfxBack();
            fs_toast("Your opponent doesn't have Sheik.");
            return;
        }
        sfxForward();
        t->ck = t->ck == CKind_Zelda ? CKind_Seak : CKind_Zelda;
        t->costume = fs_free_costume(me->target, t->ck, t->costume < fs_costumes(t->ck) ? t->costume : 0, 1);
        if (me->target != port) {
            me->target = port;
        }
        return;
    }
    sfxForward();
    t->ck = ck;
    t->costume = ck >= 0 ? fs_free_costume(me->target, ck, 0, 1) : 0;
    if (me->target != port) {
        me->target = port; /* the CPU has its fighter: back to picking for yourself */
    }
}

static void fs_css_port(int p)
{
    FsPort* s = &fcs.p[p];
    FsIn in = fs_in(p);
    int cols = fcs.cols, slot, n = fs_slots();
    if (fcs.online && p != 0) {
        return;
    }
    if (fcs.online) {
        /* the lobby's pick: any controller drives the one card */
        int q;
        for (q = 1; q < 4; q++) {
            FsIn o = fs_in(q);
            in.trig |= o.trig;
            in.rep |= o.rep;
        }
    }
    if (s->kind != FS_HMN) {
        if (in.trig & (PAD_BUTTON_A | PAD_BUTTON_START)) {
            s->kind = FS_HMN; /* a controller joins (a CPU in its port becomes this player) */
            s->target = p;
            s->card = -1;
            sfxForward();
        }
        return;
    }
    if (in.trig & PAD_TRIGGER_L) {
        fcs.page = (fcs.page + fcs.pages - 1) % fcs.pages;
        sfxMove();
    } else if (in.trig & PAD_TRIGGER_R) {
        fcs.page = (fcs.page + 1) % fcs.pages;
        sfxMove();
    }
    if (s->cur / fcs.per != fcs.page) {
        s->cur = fcs.page * fcs.per; /* the page turned: the cursor follows */
    }
    slot = s->cur - fcs.page * fcs.per;
    if (s->card < 0) {
        int page_n = n - fcs.page * fcs.per < fcs.per ? n - fcs.page * fcs.per : fcs.per;
        int row = slot / cols, col = slot % cols;
        if (in.rep & PAD_ANY_LEFT) {
            col = col == 0 ? cols - 1 : col - 1;
        } else if (in.rep & PAD_ANY_RIGHT) {
            col = col == cols - 1 ? 0 : col + 1;
        } else if (in.rep & PAD_ANY_UP) {
            row = row == 0 ? (page_n - 1) / cols : row - 1;
        } else if (in.rep & PAD_ANY_DOWN) {
            if (row + 1 > (page_n - 1) / cols || (row + 1) * cols + col >= page_n) {
                if (!fcs.online) {
                    s->card = col * 4 / cols; /* down off the grid: onto the cards */
                    sfxMove();
                    return;
                }
                row = 0;
            } else {
                row++;
            }
        }
        if (row * cols + col >= page_n) {
            col = page_n - 1 - row * cols;
            if (col < 0) {
                row = 0;
                col = 0;
            }
        }
        if (row * cols + col != slot) {
            sfxMove();
        }
        s->cur = fcs.page * fcs.per + row * cols + col;
        if (in.trig & PAD_BUTTON_A) {
            fs_pick(p, s->cur);
        } else if (in.trig & (PAD_BUTTON_X | PAD_BUTTON_Y)) {
            FsPort* t = &fcs.p[s->target];
            if (t->ck >= 0) {
                int dir = (in.trig & PAD_BUTTON_X) ? 1 : -1;
                t->costume = fs_free_costume(s->target, t->ck, t->costume + dir, dir);
                sfxMove();
            }
        } else if (in.trig & PAD_BUTTON_B) {
            sfxBack();
            if (s->target != p) {
                s->target = p; /* stop picking for that CPU */
            } else if (s->ck != FS_CK_NONE) {
                s->ck = FS_CK_NONE; /* undo */
            } else if (fl.frames < fcs.back_until) {
                fs_css_finish(0);
            } else {
                fcs.back_until = fl.frames + 150;
                fs_toast("Press B again to go back.");
            }
        }
    } else {
        /* on the cards */
        FsPort* c = &fcs.p[s->card];
        if (in.rep & PAD_ANY_LEFT) {
            s->card = (s->card + 3) % 4;
            sfxMove();
        } else if (in.rep & PAD_ANY_RIGHT) {
            s->card = (s->card + 1) % 4;
            sfxMove();
        } else if (in.rep & PAD_ANY_UP) {
            s->cur = fcs.page * fcs.per + (fcs.rows - 1) * cols + s->card * cols / 4;
            if (s->cur >= n) {
                s->cur = n - 1;
            }
            s->card = -1;
            sfxMove();
        } else if (in.trig & PAD_BUTTON_A) {
            if (s->card == p) {
                s->card = -1; /* your own card: back to the grid */
            } else if (c->kind == FS_OFF) {
                c->kind = FS_CPU; /* add a CPU, and pick its fighter now */
                c->ck = FS_CK_RANDOM;
                c->cpu_lv = 9;
                s->target = s->card;
                s->card = -1;
                sfxForward();
            } else if (c->kind == FS_CPU) {
                s->target = s->card; /* pick this CPU's fighter */
                s->card = -1;
                sfxForward();
            }
        } else if (in.trig & (PAD_BUTTON_X | PAD_BUTTON_Y)) {
            int dir = (in.trig & PAD_BUTTON_X) ? 1 : -1;
            if (c->kind == FS_CPU) {
                c->cpu_lv = c->cpu_lv + dir < 1 ? 9 : c->cpu_lv + dir > 9 ? 1 : c->cpu_lv + dir;
                sfxMove();
            } else if (s->card == p && s->ck >= 0) {
                s->costume = fs_free_costume(p, s->ck, s->costume + dir, dir);
                sfxMove();
            }
        } else if (in.trig & PAD_TRIGGER_Z) {
            if (c->kind == FS_CPU) {
                c->kind = FS_OFF;
                c->ck = FS_CK_NONE;
                sfxBack();
            } else if (s->card == p && p != 0) {
                s->kind = FS_OFF; /* leave */
                s->ck = FS_CK_NONE;
                s->card = -1;
                sfxBack();
            }
        } else if (in.trig & PAD_BUTTON_B) {
            s->card = -1;
            sfxBack();
        }
        if (fcs.teams && (in.trig & PAD_BUTTON_Y) && s->card == p && s->ck < 0) {
            s->team = (s->team + 1) % 3;
        }
    }
    if ((in.trig & PAD_BUTTON_START) && !fcs.done) {
        char buf[FE_STR];
        const char* why = fs_css_blocker(buf);
        if (why == NULL) {
            if (fcs.online && s->ck == CKind_Zelda && fs_sheik_shares_zelda() && fs_sheik_ok() &&
                (fs_held(p) & PAD_BUTTON_A))
            {
                s->ck = CKind_Seak; /* A held while locking in: Sheik, as when a match loads */
                s->costume = s->costume < fs_costumes(s->ck) ? s->costume : 0;
            }
            sfxForward();
            fs_css_finish(1);
        } else {
            sfxBack();
            fs_toast(why);
        }
    }
}

static void fs_css_frame(void)
{
    int p;
    if (fcs.done || fe.frames < 4) {
        return;
    }
    for (p = 0; p < 4 && !fcs.done; p++) {
        fs_css_port(p);
    }
    /* a CPU someone was picking for went away */
    for (p = 0; p < 4; p++) {
        if (fcs.p[fcs.p[p].target].kind != FS_CPU) {
            fcs.p[p].target = p;
        }
    }
}

/* Geometry: the grid over the top, the cards along the bottom. */
#define FS_GX0 92.0F
#define FS_GX1 548.0F
#define FS_GY0 84.0F
#define FS_GAP 4.0F
#define FS_CY0 236.0F
#define FS_CY1 392.0F

static void fs_tile_rect(int slot, float* r)
{
    float w = (FS_GX1 - FS_GX0 - (fcs.cols - 1) * FS_GAP) / (float) fcs.cols;
    float h = w * 7.0F / 8.0F;
    int s = slot - fcs.page * fcs.per;
    r[0] = FS_GX0 + (s % fcs.cols) * (w + FS_GAP);
    r[1] = FS_GY0 + (s / fcs.cols) * (h + FS_GAP);
    r[2] = r[0] + w;
    r[3] = r[1] + h;
}

static void fs_card_rect(int c, float* r)
{
    float w = (FS_GX1 - FS_GX0 - 3 * 8.0F) / 4.0F;
    if (fcs.online) {
        r[0] = 250.0F;
        r[2] = 390.0F;
    } else {
        r[0] = FS_GX0 + c * (w + 8.0F);
        r[2] = r[0] + w;
    }
    r[1] = FS_CY0;
    r[3] = FS_CY1;
}

static const char* fs_css_hint(char* buf)
{
    FsPort* s = &fcs.p[0];
    char why[FE_STR];
    if (fcs.toast[0] != 0 && fl.frames < fcs.toast_until) {
        return fcs.toast;
    }
    if (s->target != 0) {
        sprintf(buf, "P1 is picking for the P%d CPU: A on a fighter, B to stop.", s->target + 1);
        return buf;
    }
    if (fs_css_blocker(why) == NULL) {
        if ((s->ck == CKind_Zelda || s->ck == CKind_Seak) && fs_sheik_shares_zelda() && fs_sheik_ok()) {
            sprintf(buf, "A on Zelda again for %s. START %s.", s->ck == CKind_Zelda ? "Sheik" : "Zelda",
                    fcs.online ? "locks it in" : "chooses the stage");
            return buf;
        }
        return fcs.online ? "Press START to lock in this fighter."
                          : "Ready! Press START to choose the stage.";
    }
    if (s->card >= 0) {
        FsPort* c = &fcs.p[s->card];
        if (s->card == 0) return "Your card: X/Y costume, A back to the fighters.";
        if (c->kind == FS_OFF) return "A: add a CPU here.";
        if (c->kind == FS_CPU) return "CPU: A pick its fighter, X/Y level, Z remove.";
        return "Another player's card.";
    }
    return fs_css_blocker(why) != NULL ? fcs.online ? "Pick your fighter: A. START locks it in."
                                                    : "A: pick. X/Y: costume. L/R: pages. Down: CPUs."
                                       : "";
}

static void fs_build_css(void)
{
    static const char* const hg[5] = { "glyph_a", "glyph_x", "glyph_start", "glyph_b", "glyph_dpad" };
    static const char* const ht[5] = { "Pick", "Costume", "Start", "Back", "CPUs" };
    const char* crumbs[3] = { "VERSUS", "MELEE", "CHARACTERS" };
    const char* crumbs_on[3] = { "VERSUS", "ONLINE PLAY", "PICK A FIGHTER" };
    float ink[3], face[3], bone[3], muted[3], gold[3], white[3] = { 1, 1, 1 }, dim[3], disabled[3];
    float r[4];
    char id[FP_ID], buf[FE_STR];
    int slot, p, first = fcs.page * fcs.per, last = first + fcs.per, q;
    fs_drop_borrowed();
    fs_tok("ink", ink);
    fs_tok("@face", face);
    fs_tok("bone", bone);
    fs_tok("muted", muted);
    fs_tok("gold", gold);
    fs_tok("@bg", dim);
    fs_tok("disabled", disabled);
    fm_chrome_with(fcs.online ? "PICK YOUR FIGHTER" : "CHARACTER SELECT", fcs.online ? crumbs_on : crumbs,
                   3, KS_VERSUS, fcs.online ? 3 : 5, hg, ht);
    fl_desc(fs_css_hint(buf));
    if (last > fs_slots()) {
        last = fs_slots();
    }
    /* the port cursors, under the tiles: a port-coloured plate peeking out round the tile */
    for (p = 3; p >= 0; p--) {
        FsPort* s = &fcs.p[p];
        float c[3], k;
        if (s->kind != FS_HMN || s->card >= 0 || s->cur < first || s->cur >= last) {
            continue;
        }
        fs_tile_rect(s->cur, r);
        fs_port_rgb(p, c);
        k = 3.0F;
        for (q = 0; q < p; q++) {
            k += fcs.p[q].kind == FS_HMN && fcs.p[q].card < 0 && fcs.p[q].cur == s->cur ? 3.0F : 0.0F;
        }
        sprintf(id, "cur%d", p);
        fm_flat(id, FPR_OTHER, -1, r[0] - k, r[1] - k, r[2] + k, r[3] + k, c);
    }
    for (slot = first; slot < last; slot++) {
        int ck = fs_slot_ck(slot);
        fs_tile_rect(slot, r);
        sprintf(id, "tf%d", slot);
        fm_flat(id, FPR_OTHER, -1, r[0], r[1], r[2], r[3], slot == fcs.n ? ink : face);
        if (slot == fcs.n) {
            sprintf(id, "tq%d", slot);
            fl_text_at(id, "heading", (r[0] + r[2]) * 0.5F, r[3] - 8.0F, "centre", 40, "?", gold, -1);
            continue;
        }
        sprintf(id, "ti%d", slot);
        fs_tex_fit(id, fs_art_tex(fcs.icon[slot]), r[0], r[1], r[2], r[3], white);
        if (!fcs.ok[slot]) {
            int qd;
            sprintf(id, "tx%d", slot);
            qd = fm_flat(id, FPR_OTHER, -1, r[0], r[1], r[2], r[3], ink);
            if (qd >= 0) fp.q[qd].wa = -0.7F; /* dimmed: the opponent lacks it */
        }
        (void) ck;
    }
    /* the port tabs over the hovered tiles */
    for (p = 0; p < 4; p++) {
        FsPort* s = &fcs.p[p];
        float c[3];
        char t[4];
        if (s->kind != FS_HMN || s->card >= 0 || s->cur < first || s->cur >= last) {
            continue;
        }
        fs_tile_rect(s->cur, r);
        fs_port_rgb(p, c);
        sprintf(id, "ct%d", p);
        fm_flat(id, FPR_OTHER, -1, r[0] - 3 + p * 0.0F, r[1] - 15, r[0] + 21, r[1] - 2, c);
        sprintf(id, "cl%d", p);
        sprintf(t, "P%d", p + 1);
        fl_text_at(id, "caption", r[0] + 9, r[1] - 4.5F, "centre", 22, t, ink, -1);
    }
    if (fcs.pages > 1) {
        sprintf(buf, "PAGE %d/%d  L/R", fcs.page + 1, fcs.pages);
        fl_text_at("pg", "caption", FS_GX1 + 6, FS_GY0 - 6, "right", 120, buf, muted, -1);
    }
    /* the cards */
    for (p = 0; p < 4; p++) {
        FsPort* s = &fcs.p[p];
        float c[3], pr[4];
        int hov = -1;
        if (fcs.online && p != 0) {
            continue;
        }
        fs_card_rect(p, r);
        fs_port_rgb(fcs.teams ? s->team : p, c);
        for (q = 0; q < 4; q++) {
            if (fcs.p[q].kind == FS_HMN && fcs.p[q].card == p) hov = q;
        }
        sprintf(id, "co%d", p);
        if (hov >= 0) {
            float hc[3];
            fs_port_rgb(hov, hc);
            fm_flat(id, FPR_OTHER, -1, r[0] - 5, r[1] - 5, r[2] + 5, r[3] + 5, hc);
        } else {
            fm_flat(id, FPR_OTHER, -1, r[0] - 3, r[1] - 3, r[2] + 3, r[3] + 3, ink);
        }
        sprintf(id, "cf%d", p);
        fm_flat(id, FPR_OTHER, -1, r[0], r[1], r[2], r[3], s->kind == FS_OFF ? dim : face);
        sprintf(id, "cb%d", p);
        fm_flat(id, FPR_OTHER, -1, r[0], r[1], r[2], r[1] + 6, s->kind == FS_OFF ? muted : c);
        sprintf(id, "cp%d", p);
        fm_flat(id, FPR_OTHER, -1, r[0] + 8, r[1] + 11, r[0] + 32, r[1] + 25, s->kind == FS_OFF ? muted : c);
        sprintf(id, "cn%d", p);
        sprintf(buf, "P%d", p + 1);
        fl_text_at(id, "caption", r[0] + 20, r[1] + 22.5F, "centre", 22, buf, ink, -1);
        sprintf(id, "ck%d", p);
        if (s->kind == FS_CPU) {
            sprintf(buf, "CPU LV %d", s->cpu_lv);
        } else {
            sprintf(buf, "%s", s->kind == FS_HMN ? (fcs.online ? "YOU" : "HMN") : "");
            if (s->kind == FS_HMN && p == 0) {
                /* P1's name tag: SETTINGS > Online > Player Name */
                char nm[24];
                Settings_Str("name", nm, sizeof nm, "");
                if (nm[0] != 0) sprintf(buf, "%s", nm);
            }
        }
        fl_text_at(id, "caption", r[2] - 8, r[1] + 22.5F, "right", 70, buf, s->kind == FS_CPU ? gold : muted, -1);
        pr[0] = r[0] + 8;
        pr[1] = r[1] + 30;
        pr[2] = r[2] - 8;
        pr[3] = r[3] - 36;
        sprintf(id, "cr%d", p);
        fm_flat(id, FPR_OTHER, -1, pr[0], pr[1], pr[2], pr[3], ink);
        if (s->kind == FS_OFF) {
            sprintf(id, "cj%d", p);
            fl_text_at(id, "caption", (r[0] + r[2]) * 0.5F, (pr[1] + pr[3]) * 0.5F + 4, "centre",
                       r[2] - r[0] - 12, p == 0 || HSD_PadCopyStatus[p].err == 0 ? "PRESS A TO JOIN" : "A: ADD A CPU",
                       muted, -1);
            continue;
        }
        if (s->ck >= 0) {
            FsArt a;
            sprintf(id, "cs%d", p);
            mnCharSel_PcArtPortrait(s->ck, s->costume, &a.img, &a.tl);
            fs_tex_fit(id, fs_art_tex(a), pr[0] + 2, pr[1] + 2, pr[2] - 2, pr[3] - 2, white);
            Netplay_FighterName(s->ck, buf, 32);
        } else if (s->ck == FS_CK_RANDOM) {
            sprintf(id, "cq%d", p);
            fl_text_at(id, "heading", (pr[0] + pr[2]) * 0.5F, (pr[1] + pr[3]) * 0.5F + 10, "centre", 40, "?",
                       gold, -1);
            sprintf(buf, "%s", "Random");
        } else {
            sprintf(buf, "%s", s->kind == FS_CPU ? "CPU" : "Choosing...");
        }
        sprintf(id, "cm%d", p);
        fl_text_at(id, "body", (r[0] + r[2]) * 0.5F, r[3] - 17, "centre", r[2] - r[0] - 10, buf, bone, -1);
        if (s->ck >= 0) {
            sprintf(id, "cc%d", p);
            sprintf(buf, "COSTUME %d/%d", s->costume + 1, fs_costumes(s->ck));
            fl_text_at(id, "caption", (r[0] + r[2]) * 0.5F, r[3] - 5, "centre", r[2] - r[0] - 10, buf, muted, -1);
        }
        for (q = 0; q < 4; q++) {
            if (q != p && fcs.p[q].target == p && fcs.p[q].kind == FS_HMN) {
                float g[3];
                fs_port_rgb(q, g);
                sprintf(id, "cg%d", p);
                fm_flat(id, FPR_OTHER, -1, pr[0], pr[3] - 14, pr[2], pr[3], gold);
                sprintf(id, "ch%d", p);
                sprintf(buf, "P%d PICKS", q + 1);
                fl_text_at(id, "caption", (pr[0] + pr[2]) * 0.5F, pr[3] - 3, "centre", pr[2] - pr[0], buf, ink, -1);
                break;
            }
        }
    }
    (void) disabled;
}

static void fs_css_sig(char* sig)
{
    int p;
    fl_sig_int(sig, fcs.page);
    for (p = 0; p < 4; p++) {
        FsPort* s = &fcs.p[p];
        fl_sig_int(sig, s->kind * 1000000 + (s->ck + 2) * 1000 + s->costume * 10 + s->cpu_lv);
        fl_sig_int(sig, s->cur * 100 + (s->card + 1) * 10 + s->target);
        fl_sig_int(sig, s->team);
        fl_sig_int(sig, HSD_PadCopyStatus[p].err == 0);
    }
    fl_sig_int(sig, fcs.toast[0] != 0 && fl.frames < fcs.toast_until);
}

/* ---- SSS -------------------------------------------------------------------------------------- */

/* A stage's display name: the disc's own for an added stage, else the retail name. */
static void fs_stage_name(int ext, char* out, int cap) { Netplay_StageNameExt(ext, out, cap); }

static bool fs_stage_unlocked(int ext)
{
    int gk = ext > 0 ? Mex_GrKindForExt(ext) : -1;
    if (ext >= 288) {
        /* an added stage: no unlock bits on any save - playable when the disc ships its file */
        return gk >= 0 && Mex_GrFile(gk) != NULL;
    }
    return gm_80164430(ext) != 0;
}

static void fs_sss_open(SSSData* sss)
{
    int i, n;
    memset(&fcs, 0, sizeof fcs);
    fcs.sss = sss;
    n = fa_map_open();
    for (i = 0; i < n && fcs.ns < FS_MAX_STAGES; i++) {
        int ext = 0, type = 0;
        FsArt t;
        if (!mnStageSel_PcArtIcon(i, &ext, &type, &t.img, &t.tl) || type == 0) {
            continue;
        }
        if (type == 1 && ext <= 0) {
            continue; /* an empty slot of the layout */
        }
        fcs.st_ext[fcs.ns] = type == 3 ? -1 : ext;
        fcs.st_type[fcs.ns] = type;
        fcs.st_icon[fcs.ns] = t;
        fcs.st_row[fcs.ns] = i;
        fcs.st_ok[fcs.ns] = type == 3 || fs_stage_unlocked(ext);
        fcs.ns++;
    }
    /* one Random tile, at the end */
    {
        int k, r = -1;
        for (k = 0; k < fcs.ns; k++) {
            if (fcs.st_type[k] == 3) {
                if (r < 0) {
                    r = k;
                } else {
                    fcs.st_type[k] = 0; /* a second Random icon (Akaneia has two) */
                }
            }
        }
        for (k = 0, i = 0; k < fcs.ns; k++) {
            if (fcs.st_type[k] != 0 && fcs.st_type[k] != 3) {
                fcs.st_ext[i] = fcs.st_ext[k];
                fcs.st_type[i] = fcs.st_type[k];
                fcs.st_icon[i] = fcs.st_icon[k];
                fcs.st_row[i] = fcs.st_row[k];
                fcs.st_ok[i] = fcs.st_ok[k];
                i++;
            }
        }
        if (r >= 0 || i < FS_MAX_STAGES) {
            FsArt rt;
            rt.img = NULL;
            rt.tl = NULL;
            if (r >= 0) rt = fcs.st_icon[r];
            fcs.st_ext[i] = -1;
            fcs.st_type[i] = 3;
            fcs.st_icon[i] = rt;
            fcs.st_ok[i] = true;
            i++;
        }
        fcs.ns = i;
    }
    fcs.cols = 8;
    fcs.rows = 4;
    fcs.per = fcs.cols * fcs.rows;
    fcs.pages = (fcs.ns + fcs.per - 1) / fcs.per;
    if (fcs.pages < 1) fcs.pages = 1;
    for (i = 0; i < fcs.ns; i++) {
        if (sss != NULL && fcs.st_ext[i] == sss->vs.start.rules.stkind) {
            fcs.st_cur = i;
        }
    }
    fcs.page = fcs.st_cur / fcs.per;
    OSReport("frontend: stage select - %d stages on offer, %d page(s)\n", fcs.ns, fcs.pages);
}

static void fs_sss_finish(int go, int ext)
{
    SSSData* sss = fcs.sss;
    int i;
    fcs.done = 1;
    if (sss != NULL) {
        sss->start_game = go != 0;
        if (go) {
            for (i = 0; i < fcs.ns; i++) {
                if (fcs.st_ext[i] == ext) {
                    fe_np_store(-1, fcs.st_icon[i].img, fcs.st_icon[i].tl); /* the loading screen's */
                }
            }
            sss->vs.start.rules.stkind = (u16) ext;
            fs_preload_stage(ext);
            OSReport("frontend: stage select - stage %d\n", ext);
        }
    }
    gm_801A4B60(); /* gmVsMelee_ExitSss: on to the loading screen, or back to the characters */
}

static int fs_random_stage(void)
{
    int k = 0, i, pick;
    for (i = 0; i < fcs.ns; i++) {
        k += fcs.st_ok[i] && fcs.st_type[i] != 3;
    }
    if (k == 0) {
        return 31; /* Battlefield */
    }
    pick = HSD_Randi(k);
    for (i = 0; i < fcs.ns; i++) {
        if (fcs.st_ok[i] && fcs.st_type[i] != 3 && pick-- == 0) {
            return fcs.st_ext[i];
        }
    }
    return 31;
}

static void fs_sss_frame(void)
{
    FsIn in;
    int p, cols = fcs.cols, slot, page_n, row, col;
    if (fcs.done || fe.frames < 4) {
        return;
    }
    if (fcs.sss != NULL && fcs.sss->force_stage_id >= 0) {
        fs_sss_finish(1, fcs.sss->force_stage_id); /* the rules chose the stage */
        return;
    }
    in.trig = in.rep = 0;
    for (p = 0; p < 4; p++) {
        FsIn o = fs_in(p);
        in.trig |= o.trig;
        in.rep |= o.rep;
    }
    if (in.trig & PAD_TRIGGER_L) {
        fcs.page = (fcs.page + fcs.pages - 1) % fcs.pages;
        fcs.st_cur = fcs.page * fcs.per;
        sfxMove();
    } else if (in.trig & PAD_TRIGGER_R) {
        fcs.page = (fcs.page + 1) % fcs.pages;
        fcs.st_cur = fcs.page * fcs.per;
        sfxMove();
    }
    page_n = fcs.ns - fcs.page * fcs.per < fcs.per ? fcs.ns - fcs.page * fcs.per : fcs.per;
    slot = fcs.st_cur - fcs.page * fcs.per;
    row = slot / cols;
    col = slot % cols;
    if (in.rep & PAD_ANY_LEFT) {
        if (col == 0 && fcs.pages > 1) {
            fcs.page = (fcs.page + fcs.pages - 1) % fcs.pages; /* off the edge: the page before */
            fcs.st_cur = fcs.page * fcs.per + row * cols + cols - 1;
            if (fcs.st_cur >= fcs.ns) fcs.st_cur = fcs.ns - 1;
            sfxMove();
            return;
        }
        col = col == 0 ? cols - 1 : col - 1;
    } else if (in.rep & PAD_ANY_RIGHT) {
        if ((col == cols - 1 || row * cols + col + 1 >= page_n) && fcs.pages > 1) {
            fcs.page = (fcs.page + 1) % fcs.pages;
            fcs.st_cur = fcs.page * fcs.per + row * cols;
            if (fcs.st_cur >= fcs.ns) fcs.st_cur = fcs.page * fcs.per;
            sfxMove();
            return;
        }
        col = col == cols - 1 ? 0 : col + 1;
    } else if (in.rep & PAD_ANY_UP) {
        row = row == 0 ? (page_n - 1) / cols : row - 1;
    } else if (in.rep & PAD_ANY_DOWN) {
        row = row >= (page_n - 1) / cols ? 0 : row + 1;
    }
    if (row * cols + col >= page_n) {
        col = page_n - 1 - row * cols;
        if (col < 0) {
            row = 0;
            col = 0;
        }
    }
    if (fcs.page * fcs.per + row * cols + col != fcs.st_cur) {
        fcs.st_cur = fcs.page * fcs.per + row * cols + col;
        sfxMove();
    }
    if (in.trig & (PAD_BUTTON_A | PAD_BUTTON_START)) {
        int i = fcs.st_cur;
        if (!fcs.st_ok[i]) {
            sfxBack();
            fs_toast("That stage is locked.");
        } else {
            sfxForward();
            fs_sss_finish(1, fcs.st_type[i] == 3 ? fs_random_stage() : fcs.st_ext[i]);
        }
    } else if (in.trig & PAD_BUTTON_B) {
        sfxBack();
        fs_sss_finish(0, 0);
    }
}

static void fs_build_sss(void)
{
    static const char* const hg[4] = { "glyph_a", "glyph_dpad", "glyph_b", "glyph_l" };
    static const char* const ht[4] = { "Choose", "Move", "Back", "Pages" };
    const char* crumbs[3] = { "VERSUS", "MELEE", "STAGES" };
    float ink[3], face[3], bone[3], muted[3], gold[3], white[3] = { 1, 1, 1 }, r[4], c[3];
    char id[FP_ID], buf[FE_STR];
    int i, first = fcs.page * fcs.per, last = first + fcs.per;
    float w = (FS_GX1 - FS_GX0 - (fcs.cols - 1) * 6.0F) / (float) fcs.cols, h = w * 7.0F / 8.0F;
    fs_drop_borrowed();
    fs_tok("ink", ink);
    fs_tok("@face", face);
    fs_tok("bone", bone);
    fs_tok("muted", muted);
    fs_tok("gold", gold);
    fs_port_rgb(0, c);
    fm_chrome_with("STAGE SELECT", crumbs, 3, KS_VERSUS, fcs.pages > 1 ? 4 : 3, hg, ht);
    fl_desc(fcs.toast[0] != 0 && fl.frames < fcs.toast_until ? fcs.toast
                                                              : "Choose the stage: A. B goes back to the characters.");
    if (last > fcs.ns) {
        last = fcs.ns;
    }
    for (i = first; i < last; i++) {
        int s = i - first;
        r[0] = FS_GX0 + (s % fcs.cols) * (w + 6.0F);
        r[1] = FS_GY0 + (s / fcs.cols) * (h + 6.0F);
        r[2] = r[0] + w;
        r[3] = r[1] + h;
        if (i == fcs.st_cur) {
            sprintf(id, "sc%d", i);
            fm_flat(id, FPR_OTHER, -1, r[0] - 4, r[1] - 4, r[2] + 4, r[3] + 4, gold);
        }
        sprintf(id, "sf%d", i);
        fm_flat(id, FPR_OTHER, -1, r[0], r[1], r[2], r[3], fcs.st_type[i] == 3 ? ink : face);
        sprintf(id, "si%d", i);
        if (fcs.st_type[i] == 3 && fcs.st_icon[i].img == NULL) {
            fl_text_at(id, "heading", (r[0] + r[2]) * 0.5F, r[3] - 10, "centre", 40, "?", gold, -1);
        } else {
            fs_tex_fit(id, fs_art_tex(fcs.st_icon[i]), r[0] + 1, r[1] + 1, r[2] - 1, r[3] - 1, white);
        }
        if (!fcs.st_ok[i]) {
            int q;
            sprintf(id, "sx%d", i);
            q = fm_flat(id, FPR_OTHER, -1, r[0], r[1], r[2], r[3], ink);
            if (q >= 0) fp.q[q].wa = -0.75F;
            sprintf(id, "sl%d", i);
            {
                int t = fp_tex("ico_lock");
                if (t >= 0) {
                    fl_rect_quad(id, t, (r[0] + r[2]) * 0.5F - 9, (r[1] + r[3]) * 0.5F - 9,
                                 (r[0] + r[2]) * 0.5F + 9, (r[1] + r[3]) * 0.5F + 9, muted, -1);
                }
            }
        }
    }
    /* the hovered stage's plate */
    {
        float y0 = 296.0F, y1 = 352.0F;
        int cur = fcs.st_cur;
        fm_flat("sp_ink", FPR_OTHER, -1, FS_GX0 - 3, y0 - 3, FS_GX1 + 3, y1 + 3, ink);
        fm_flat("sp_face", FPR_OTHER, -1, FS_GX0, y0, FS_GX1, y1, face);
        fm_flat("sp_acc", FPR_OTHER, -1, FS_GX0, y0, FS_GX0 + 4, y1, gold);
        fm_flat("sp_fr", FPR_OTHER, -1, FS_GX0 + 12, y0 + 4, FS_GX0 + 70, y1 - 4, ink);
        if (cur < fcs.ns && fcs.st_icon[cur].img != NULL) {
            fs_tex_fit("sp_art", fs_art_tex(fcs.st_icon[cur]), FS_GX0 + 13, y0 + 5, FS_GX0 + 69, y1 - 5, white);
        }
        fl_text_at("sp_lbl", "caption", FS_GX0 + 80, y0 + 18, "left", 100,
                   cur < fcs.ns && fcs.st_type[cur] == 3 ? "ANY STAGE" : "STAGE", muted, -1);
        if (cur < fcs.ns && fcs.st_type[cur] == 3) {
            sprintf(buf, "%s", "Random");
        } else if (cur < fcs.ns) {
            fs_stage_name(fcs.st_ext[cur], buf, sizeof buf);
        } else {
            buf[0] = 0;
        }
        {
            /* the SSS's own name art for the stage; our text when there is none */
            int nt = -1;
            FsArt na;
            if (cur < fcs.ns && fcs.st_type[cur] != 3 &&
                mnStageSel_PcArtName(fcs.st_row[cur], &na.img, &na.tl))
            {
                nt = fs_art_tex(na);
            }
            if (nt >= 0 && fp.tex[nt].t.w >= 64) {
                fs_tex_fit("sp_nimg", nt, FS_GX0 + 78, y0 + 22, FS_GX0 + 380, y1 - 3,
                           cur < fcs.ns && !fcs.st_ok[cur] ? muted : white);
            } else {
                fl_text_at("sp_name", "title", FS_GX0 + 80, y1 - 9, "left", 300, buf,
                           cur < fcs.ns && !fcs.st_ok[cur] ? muted : bone, -1);
            }
        }
        if (cur < fcs.ns && !fcs.st_ok[cur]) {
            fl_text_at("sp_lock", "caption", FS_GX1 - 12, y0 + 18, "right", 120, "LOCKED", muted, -1);
        }
        if (fcs.pages > 1) {
            sprintf(buf, "PAGE %d/%d", fcs.page + 1, fcs.pages);
            fl_text_at("sp_pg", "caption", FS_GX1 - 12, y1 - 10, "right", 120, buf, gold, -1);
        }
    }
}

static void fs_sss_sig(char* sig)
{
    fl_sig_int(sig, fcs.page);
    fl_sig_int(sig, fcs.st_cur);
    fl_sig_int(sig, fcs.toast[0] != 0 && fl.frames < fcs.toast_until);
}
