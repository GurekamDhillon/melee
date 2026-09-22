/* gmfrontend_online.inc - included by gmfrontend.c (TARGET_PC only), after the kit list.
 *
 * ---- THE ONLINE ROOM SCREENS: code entry, waiting room, lobby -----------------------------
 *
 * The flow (one state machine; every screen names its buttons in the footer):
 *
 *   VERSUS > ONLINE (toolkit rows: Host a Room / Join a Room / Random Opponent (soon) / rules)
 *     Host a Room ......... WAITING ROOM (host): the code big, copied; X copies again; B leaves
 *     Join a Room ......... CODE ENTRY: type / paste / d-pad; A joins; B back to ONLINE
 *                           -> WAITING ROOM (guest, "joining"); a bad code comes back here
 *   opponent in ........... LOBBY (both): game 1 blind characters on the real CSS, coin flip,
 *                           strikes 1-2-2; game 2+ winner bans 2, loser picks, then characters;
 *                           START readies (again un-readies); 3-2-1; the match
 *   match -> results ...... LOBBY again (the room persists: it reconnects on its own and the set
 *                           carries on)
 *   leaving ............... B, then B again in the lobby (B once in the other screens): the room
 *                           closes for the host, a guest just goes. An opponent leaving sends the
 *                           host back to its waiting room (same code) and the guest to ONLINE.
 *
 * DRAWING. The art pipeline's layouts (menu/out_lobby: waiting_room_layout.json,
 * code_entry_layout.json, lobby_layout.json; the status strip from out_online's
 * online_layout.json), on the kit's chrome. Their element format is the kit's plus state tables:
 * a colour/texture/uv/opacity written "<axis.key>" is states[axis][current state][key] (null =
 * not drawn), "when": {axis: [states]} draws a quad only in those states, and a state with
 * `offset` lifts the element's lift joints (lobby.py's lookup/visible/draw_el are the reference).
 * The screen is rebuilt into the player (fp) whenever what it shows changes - a signature string
 * of every state and text - so the drawing is always the current state; short motion events from
 * the layouts' *_motion.json (strike stamp, ban lock, pick, ready, the countdown numerals, badge
 * pops, the coin) play on the fresh model.
 *
 * Portraits are the fighters' CSS icons captured on every CSS visit (Frontend_CaptureCharIcon),
 * lent to the player as textures. Stage art is optional in the layout and not captured yet (the
 * tile draws its flat stand-in). */

enum { FL_NONE, FL_CODE, FL_WAIT, FL_LOBBY, FL_LOAD, FL_CSS, FL_SSS };

/* the character and stage select (gmfrontend_select.inc) */
static void fs_css_open(CSSData* css);
static void fs_sss_open(SSSData* sss);
static void fs_css_frame(void);
static void fs_sss_frame(void);
static void fs_build_css(void);
static void fs_build_sss(void);
static void fs_css_sig(char* sig);
static void fs_sss_sig(char* sig);
static void fa_close(void);
static int fs_stage_art_tex(int ext);
static int fs_tex_fit(const char* id, int tex, float x0, float y0, float x1, float y1, const float* rgb);
static void fs_drop_borrowed(void);
static void fl_dense_grid(void);
#define FL_DENSE_COLS 6
#define FL_DENSE_ROWS 3

/* THE LOADING SCREEN (FL_LOAD, loading_layout.json): who fights where, and the warm-up's
 * progress. VS mode's loading state (after the SSS) and the online lobby (after its countdown)
 * both show it; fe_load_step (gmfrontend.c) measures the warm-up. */
static struct {
    int n;                   ///< fighters shown (2 = big cards and the VS mark; 3-4 compact)
    int ck[4], color[4], cpu[4], port[4];
    int online, me;          ///< online: this machine's card
    int ext;                 ///< the stage (external id)
    int game, s1, s2;        ///< online: the set
    char room[8];
    int ready, built;
    int q_fill, q_pct;       ///< the bar's fill and readout quads, updated every frame
    float fill_w;            ///< the fill's full width
    float pct_x;             ///< the readout's right edge
} fl_ld;

typedef struct FlEl {
    const FjNode* el;      ///< the element: quads, slots, states, lift_joint(s)
    const FjNode* tstates; ///< stage tiles: the grid's tile_states (axis "tile")
    float ox, oy;          ///< origin (quad and slot coordinates are relative to it)
    char idx[4];           ///< "<i>" in joint names
    char port[4];          ///< "<p>": "p1" / "p2"
    char pre[8];           ///< quad id prefix, unique per element instance
    int who;               ///< cards: 0 P1, 1 P2; tiles: the index
} FlEl;

static struct {
    bool on;
    int art;        ///< FL_* this model shows
    int frames;     ///< frames on this screen
    char sig[512];  ///< what the model was built from
    int toast_until;
    char toast[FE_STR];
    bool toast_bad;
    /* waiting room */
    int copied_until, found_at;
    /* code entry */
    bool invalid;
    char code_msg[FE_STR];
    bool code_msg_bad;
    int bump_until;
    /* lobby */
    int cur;          ///< the grid cursor (a stage index)
    int leave_until;  ///< B once arms leaving until this frame
    int coin_until;   ///< the coin badge shows until this frame
    int ready_on_until;
    int last_phase, last_seq, last_count, last_game;
    int stage_prev[LB_MAX_UI_STAGES];
    int locked_prev[2], ready_prev[2];
    int rejoin_at;    ///< frame to reconnect after a match (-1 none)
    int pend_event;   ///< an event to play on the next build (FLE_*)
    int pend_arg;
} fl;

static bool fl_coin_done; ///< this game's coin flip has been shown (it survives a CSS visit)
static char fl_pending_note[FE_STR]; ///< for the lobby's status line when it next opens

enum { FLE_NONE, FLE_STRIKE, FLE_BAN, FLE_PICK, FLE_READY, FLE_TICK, FLE_COIN, FLE_BADGE, FLE_SCORE };


/* ---- small helpers ------------------------------------------------------------------------ */

static const char* fl_strchr(const char* s, char c)
{
    for (; s != NULL && *s != 0; s++) {
        if (*s == c) {
            return s;
        }
    }
    return NULL;
}

static void fl_toast(const char* s, bool bad, int frames)
{
    fp_copy(fl.toast, s, FE_STR);
    fl.toast_bad = bad;
    fl.toast_until = fl.frames + frames;
}

static const char* fl_toast_now(void)
{
    return fl.toast[0] != '\0' && fl.frames < fl.toast_until ? fl.toast : NULL;
}

static void fl_sig_add(char* sig, const char* s)
{
    int n = (int) strlen(sig), i;
    for (i = 0; s[i] != '\0' && n < (int) sizeof fl.sig - 2; i++) {
        sig[n++] = s[i];
    }
    sig[n++] = '|';
    sig[n] = '\0';
}

static void fl_sig_int(char* sig, int v)
{
    char b[16];
    sprintf(b, "%d", v);
    fl_sig_add(sig, b);
}

static int fl_joint(const char* name)
{
    int j;
    if (name == NULL || name[0] == '\0') {
        return -1;
    }
    j = fp_find_joint(name);
    return j >= 0 ? j : fp_add_joint(name, -1, NULL);
}

/* "<i>" / "<p>" in a name */
static void fl_subst(char* out, const char* pat, const FlEl* e)
{
    int o = 0;
    while (pat != NULL && *pat != '\0' && o < FP_ID - 1) {
        const char* s = NULL;
        if (strncmp(pat, "<i>", 3) == 0) {
            s = e->idx;
        } else if (strncmp(pat, "<p>", 3) == 0) {
            s = e->port;
        }
        if (s != NULL) {
            while (*s != '\0' && o < FP_ID - 1) {
                out[o++] = *s++;
            }
            pat += 3;
        } else {
            out[o++] = *pat++;
        }
    }
    out[o] = '\0';
}

/* ---- the element format ------------------------------------------------------------------- */

static const char* fl_axis_state(const FlEl* e, const char* axis);
static const char* fl_slot_text(const FlEl* e, const char* id);

static const FjNode* fl_lookup(const FlEl* e, const char* ref)
{
    char axis[24], key[24];
    const char* dot;
    const char* end;
    const char* st;
    int n;
    if (ref == NULL || ref[0] != '<') {
        return NULL;
    }
    dot = fl_strchr(ref, '.');
    end = fl_strchr(ref, '>');
    if (dot == NULL || end == NULL || end < dot) {
        return NULL;
    }
    n = (int) (dot - ref - 1);
    if (n <= 0 || n >= (int) sizeof axis) return NULL;
    memcpy(axis, ref + 1, (size_t) n);
    axis[n] = '\0';
    n = (int) (end - dot - 1);
    if (n <= 0 || n >= (int) sizeof key) return NULL;
    memcpy(key, dot + 1, (size_t) n);
    key[n] = '\0';
    st = fl_axis_state(e, axis);
    if (st == NULL) {
        return NULL;
    }
    {
        const FjNode* S = fj_get(e->el, "states");
        const FjNode* ax = fj_get(S, axis);
        if (ax != NULL) {
            return fj_get(fj_get(ax, st), key);
        }
        if (fj_get(S, st) != NULL) {
            return fj_get(fj_get(S, st), key); /* one axis, its states straight under "states" */
        }
    }
    if (strcmp(axis, "tile") == 0 && e->tstates != NULL) {
        return fj_get(fj_get(e->tstates, st), key);
    }
    return NULL;
}

/* A value that may be "<axis.key>": the node it stands for (NULL = not drawn). */
static const FjNode* fl_val(const FlEl* e, const FjNode* n)
{
    const char* s = fj_str(n);
    if (s != NULL && s[0] == '<' && fl_strchr(s, '.') != NULL) {
        n = fl_lookup(e, s);
        if (n == NULL || n->type == FJ_NULL) {
            return NULL;
        }
    }
    return n;
}

static bool fl_colour(const FlEl* e, const FjNode* n, float* rgb)
{
    const char* s;
    n = fl_val(e, n);
    s = fj_str(n);
    if (s == NULL) {
        return false;
    }
    if (strcmp(s, "port:<p>") == 0) {
        char t[12];
        sprintf(t, "port:%s", e->port);
        return kit_colour(t, KS_VERSUS, rgb);
    }
    return kit_colour(s, KS_VERSUS, rgb);
}

static bool fl_when(const FlEl* e, const FjNode* q)
{
    const FjNode* w = fj_get(q, "when");
    const FjNode* ax;
    for (ax = fj_first(w); ax != NULL; ax = fj_next(ax)) {
        const char* st = fl_axis_state(e, ax->key);
        const FjNode* s;
        bool ok = false;
        for (s = fj_first(ax); s != NULL && st != NULL; s = fj_next(s)) {
            if (fj_str(s) != NULL && strcmp(fj_str(s), st) == 0) {
                ok = true;
            }
        }
        if (!ok) {
            return false;
        }
    }
    return true;
}

/* The element's lift for its current states: the first state (of any axis) with an offset. */
static void fl_offset(const FlEl* e, float* d)
{
    const FjNode* S = fj_get(e->el, "states");
    const FjNode* ax;
    d[0] = d[1] = 0.0F;
    for (ax = fj_first(S); ax != NULL; ax = fj_next(ax)) {
        const char* st = fl_axis_state(e, ax->key);
        const FjNode* off = st != NULL ? fj_get(fj_get(ax, st), "offset") : NULL;
        if (off != NULL) {
            d[0] = fj_num(fj_idx(off, 0), 0);
            d[1] = fj_num(fj_idx(off, 1), 0);
            return;
        }
    }
    if (e->tstates != NULL) {
        const char* st = fl_axis_state(e, "tile");
        const FjNode* off = st != NULL ? fj_get(fj_get(e->tstates, st), "offset") : NULL;
        if (off != NULL) {
            d[0] = fj_num(fj_idx(off, 0), 0);
            d[1] = fj_num(fj_idx(off, 1), 0);
        }
    }
}

static bool fl_lifts(const FlEl* e, const char* joint)
{
    const FjNode* lj = fj_get(e->el, "lift_joints");
    const FjNode* n;
    char nm[FP_ID];
    if (joint == NULL) {
        return false;
    }
    if (lj != NULL) {
        for (n = fj_first(lj); n != NULL; n = fj_next(n)) {
            fl_subst(nm, fj_str(n), e);
            if (strcmp(nm, joint) == 0) {
                return true;
            }
        }
        return false;
    }
    return fj_str(fj_get(e->el, "lift_joint")) != NULL && fp_strstr(joint, "plate") == NULL;
}

/* A text slot: role, anchor, alignment, max width, colour; the string from the screen. */
static int fl_text_at(const char* id, const char* role, float ax, float ay, const char* align,
                      float max_w, const char* s, const float* rgb, int joint)
{
    int q = fm_text(id, FPR_OTHER, -1, role, ax, ay, s, rgb, max_w);
    if (q >= 0) {
        float w = fm_last_w, sh = 0.0F;
        int k;
        if (align != NULL && (align[0] == 'c')) sh = w * 0.5F;
        if (align != NULL && (align[0] == 'r')) sh = w;
        fp.q[q].ax -= sh;
        for (k = 0; k < 4; k++) {
            fp.q[q].v[k][0] -= sh;
        }
        fp.q[q].joint = (s8) joint;
    }
    return q;
}

static bool fl_special_quad(FlEl* e, const FjNode* q, const char* id, float dx, float dy, int joint);

/* Build one element: its quads (file order), then its text slots. */
static void fl_el(FlEl* e)
{
    const FjNode* q;
    float off[2];
    fl_offset(e, off);
    for (q = fj_first(fj_get(e->el, "quads")); q != NULL; q = fj_next(q)) {
        const char* id = fj_str(fj_get(q, "id"));
        const char* kind = fj_str(fj_get(q, "kind"));
        char jn[FP_ID], qid[FP_ID];
        float dx = e->ox, dy = e->oy, rgb[3];
        int j, qi = -1, k;
        const FjNode* v;
        if (id == NULL || !fl_when(e, q)) {
            continue;
        }
        fl_subst(jn, fj_str(fj_get(q, "joint")), e);
        j = fl_joint(jn);
        if (fl_lifts(e, jn)) {
            dx += off[0];
            dy += off[1];
        }
        if (fl_special_quad(e, q, id, dx, dy, j)) {
            continue;
        }
        if (!fl_colour(e, fj_get(q, "colour"), rgb)) {
            continue;
        }
        sprintf(qid, "%s%.19s", e->pre, id);
        if (kind != NULL && strcmp(kind, "textured") == 0) {
            const FjNode* tn = fl_val(e, fj_get(q, "texture"));
            int t = fp_tex(fj_str(tn));
            if (t < 0) {
                continue;
            }
            qi = fp_add_quad(qid, FPK_TEX, FPR_OTHER, -1);
            if (qi < 0) continue;
            fp.q[qi].tex = (s8) t;
            {
                const FjNode* uv = fl_val(e, fj_get(q, "uv"));
                for (k = 0; k < 4 && uv != NULL; k++) {
                    fp.q[qi].uv[k] = fj_num(fj_idx(uv, k), k < 2 ? 0.0F : 1.0F);
                }
            }
        } else {
            qi = fp_add_quad(qid, FPK_FLAT, FPR_OTHER, -1);
            if (qi < 0) continue;
        }
        fp.q[qi].has_colour = true;
        fm_rgb(fp.q[qi].rgb, rgb);
        for (k = 0, v = fj_first(fj_get(q, "verts")); v != NULL && k < 4; v = fj_next(v), k++) {
            float x = fj_num(fj_idx(v, 0), 0) + dx, y = fj_num(fj_idx(v, 1), 0) + dy;
            fp.q[qi].v[k][0] = x + (fp.shear_y0 - y) * fp.shear_s;
            fp.q[qi].v[k][1] = y;
        }
        fp.q[qi].joint = (s8) j;
        {
            const FjNode* op = fl_val(e, fj_get(q, "opacity"));
            if (op != NULL && op->type == FJ_NUM) {
                fp.q[qi].wa = -op->n; /* negative: an opacity to apply after the channel reset */
            }
        }
    }
    for (q = fj_first(fj_get(e->el, "slots")); q != NULL; q = fj_next(q)) {
        const char* id = fj_str(fj_get(q, "id"));
        const char* s;
        char jn[FP_ID], qid[FP_ID];
        float dx = e->ox, dy = e->oy, rgb[3];
        int j;
        if (id == NULL || !fl_when(e, q)) {
            continue;
        }
        s = fl_slot_text(e, id);
        if (s == NULL || s[0] == '\0' || !fl_colour(e, fj_get(q, "colour"), rgb)) {
            continue;
        }
        fl_subst(jn, fj_str(fj_get(q, "joint")), e);
        j = fl_joint(jn);
        if (fl_lifts(e, jn)) {
            dx += off[0];
            dy += off[1];
        }
        sprintf(qid, "%s%.19s", e->pre, id);
        {
            float ax = fj_num(fj_idx(fj_get(q, "anchor"), 0), 0) + dx;
            const char* align = fj_str(fj_get(q, "align"));
            if (strcmp(id, "action_text") == 0 && fj_str(fl_lookup(e, "<action.icon>")) != NULL &&
                strcmp(fj_str(fl_lookup(e, "<action.icon>")), "pips") != 0)
            {
                /* icon + 6 + text, centred as one group (fl_special_quad places the icon) */
                float iw = fj_num(fl_lookup(e, "<action.icon_w>"), 16);
                float tw = ff_width(ff_role("row"), s);
                ax = 478 - (iw + 6 + tw) * 0.5F + iw + 6 + dx;
                align = "left";
            }
            fl_text_at(qid, fj_str(fj_get(q, "role")), ax,
                       fj_num(fj_idx(fj_get(q, "anchor"), 1), 0) + dy, align,
                       fj_num(fj_get(q, "max_width_1x"), 300) + 6.0F, s, rgb, j); /* +6: our kerning runs a little wider than the pipeline's */
        }
    }
}

/* After a build: joint pivots at the centre of what they draw, rest values, opacities. */
static void fl_finish(void)
{
    int i, k;
    for (i = 0; i < fp.nj; i++) {
        float x0 = 1e9F, y0 = 1e9F, x1 = -1e9F, y1 = -1e9F;
        for (k = 0; k < fp.nq; k++) {
            int c;
            if (fp.q[k].joint != i) continue;
            for (c = 0; c < 4; c++) {
                if (fp.q[k].v[c][0] < x0) x0 = fp.q[k].v[c][0];
                if (fp.q[k].v[c][0] > x1) x1 = fp.q[k].v[c][0];
                if (fp.q[k].v[c][1] < y0) y0 = fp.q[k].v[c][1];
                if (fp.q[k].v[c][1] > y1) y1 = fp.q[k].v[c][1];
            }
        }
        if (x1 >= x0) {
            fp.j[i].has_pivot = true;
            fp.j[i].pivot[0] = (x0 + x1) * 0.5F;
            fp.j[i].pivot[1] = (y0 + y1) * 0.5F;
        }
    }
    fp_reset_channels();
    for (i = 0; i < fp.nq; i++) {
        if (fp.q[i].wa < 0.0F) {
            fp_mch(i, FMC_ALPHA)->rest = -fp.q[i].wa;
            fp.q[i].wa = 0.0F;
        }
    }
    fp.ok = true;
    fp.frame = 0;
}

/* ---- motion: a few of the layouts' events, played on the fresh model --------------------- */

static void fl_play_at(const char* file, const char* event, const FlEl* e, int delay);
static void fl_play(const char* file, const char* event, const FlEl* e)
{
    fl_play_at(file, event, e, 0);
}

static void fl_play_at(const char* file, const char* event, const FlEl* e, int delay)
{
    void *t, *p;
    const FjNode* M = fj_load(file, &t, &p);
    const FjNode* ev = fj_get(fj_get(M, "events"), event);
    const FjNode* tr;
    for (tr = fj_first(fj_get(ev, "tracks")); tr != NULL; tr = fj_next(tr)) {
        char target[FP_ID];
        const char* kind = fj_str(fj_get(tr, "kind"));
        const char* ch = fj_str(fj_get(tr, "channel"));
        FpKey k[FP_MAX_KEYS];
        const FjNode* key;
        int n = 0, c = -1, i;
        bool mat = kind != NULL && strcmp(kind, "material") == 0;
        FpChan* chan = NULL;
        u8 rec = 0;
        if (ch == NULL || strcmp(ch, "COLOUR") == 0) {
            continue; /* colours are the state tables' job here */
        }
        fl_subst(target, fj_str(fj_get(tr, "target")), e);
        if (mat) {
            if (strcmp(ch, "ALPHA") != 0) continue;
            /* a material names a quad; ours carry the element prefix - match by suffix */
            for (i = 0; i < fp.nq; i++) {
                int ln = (int) strlen(fp.q[i].id), lt = (int) strlen(target);
                if (fp.q[i].joint >= 0 && strcmp(fp.j[fp.q[i].joint].name, target) == 0) {
                    chan = fp_mch(i, FMC_ALPHA);
                    break;
                }
                if (ln >= lt && strcmp(fp.q[i].id + ln - lt, target) == 0) {
                    chan = fp_mch(i, FMC_ALPHA);
                    break;
                }
            }
        } else {
            int j = fp_find_joint(target);
            c = fp_channel(ch, &rec);
            if (j < 0 || c >= FJC_N || rec) continue;
            chan = fp_jch(j, c);
        }
        if (chan == NULL) {
            continue;
        }
        {
            /* the player holds FP_MAX_KEYS keys a play: a longer track (the coin's 14) keeps its
               last ones, so it still lands where it should */
            int total = 0, skip;
            for (key = fj_first(fj_get(tr, "keys")); key != NULL; key = fj_next(key)) total++;
            skip = total > FP_MAX_KEYS ? total - FP_MAX_KEYS : 0;
            for (key = fj_first(fj_get(tr, "keys")); key != NULL && skip > 0; key = fj_next(key)) skip--;
        }
        for (; key != NULL && n < FP_MAX_KEYS; key = fj_next(key)) {
            const FjNode* v = fj_get(key, "value");
            const FjNode* fc = fj_get(key, "from_current");
            k[n].frame = (s16) fj_num(fj_get(key, "frame"), 0);
            k[n].interp = fp_interp(fj_str(fj_get(key, "interp")));
            k[n].slope = fj_num(fj_get(key, "slope"), 0);
            k[n].from_current = fc != NULL && fc->type == FJ_TRUE;
            k[n].v = v != NULL && v->type == FJ_NUM ? v->n : (c == FJC_SCA_X || c == FJC_SCA_Y ? 1.0F : 0.0F);
            if (v != NULL && v->type == FJ_STR) {
                k[n].from_current = 1; /* an expression we do not evaluate: hold where it is */
            }
            n++;
        }
        if (n > 0 && !mat && (c == FJC_TRA_X || c == FJC_TRA_Y) && k[n - 1].v != 0.0F) {
            continue; /* a move that ends lifted: the state's offset is built into the model */
        }
        if (n > 0) {
            fp_start(chan, fp.frame + delay, k, n);
        }
    }
    if (t != NULL) HSD_Free(t);
    if (p != NULL) HSD_Free(p);
}

/* ---- portraits: the captured CSS icons, lent to the player --------------------------------- */

static int fl_char_tex(int ck)
{
    char nm[FP_ID];
    FeNpCharIcon* ic;
    FpTex* tx;
    int i;
    if (ck < 0 || ck >= FE_NP_MAX_CK || !fe_np_char_icon[ck].ok) {
        return -1;
    }
    ic = &fe_np_char_icon[ck];
    sprintf(nm, "@ck%d", ck);
    for (i = 0; i < fp.ntex; i++) {
        if (strcmp(fp.tex[i].name, nm) == 0) {
            return i;
        }
    }
    if (fp.ntex >= FP_MAX_TEX) {
        return -1;
    }
    tx = &fp.tex[fp.ntex];
    memset(tx, 0, sizeof *tx);
    fp_copy(tx->name, nm, FP_ID);
    tx->borrowed = true;
    tx->t.data = ic->data;
    tx->t.w = ic->w;
    tx->t.h = ic->h;
    if (ic->tlut_n != 0) {
        tx->t.lut = ic->lut;
        GXInitTlutObj(&tx->t.tlut, ic->lut, (GXTlutFmt) ic->tlut_fmt, (u16) ic->tlut_n);
        GXInitTexObjCI(&tx->t.obj, ic->data, ic->w, ic->h, (GXTexFmt) ic->fmt, GX_CLAMP, GX_CLAMP,
                       GX_FALSE, GX_TLUT0);
    } else {
        GXInitTexObj(&tx->t.obj, ic->data, ic->w, ic->h, (GXTexFmt) ic->fmt, GX_CLAMP, GX_CLAMP,
                     GX_FALSE);
    }
    GXInitTexObjLOD(&tx->t.obj, GX_LINEAR, GX_LINEAR, 0.0F, 0.0F, 0.0F, GX_FALSE, GX_FALSE,
                    GX_ANISO_1);
    tx->t.ok = true;
    tx->mask = false;
    return fp.ntex++;
}

/* The icon the SSS captured when the stage was picked (fe_np_icon[1]), lent to the player. */
static int fl_icon_tex(int which)
{
    FeNpIcon* ic = &fe_np_icon[which & 1];
    FpTex* tx;
    int i;
    if (!ic->ok) {
        return -1;
    }
    for (i = 0; i < fp.ntex; i++) {
        if (strcmp(fp.tex[i].name, which ? "@stage" : "@char") == 0) {
            return i;
        }
    }
    if (fp.ntex >= FP_MAX_TEX) {
        return -1;
    }
    tx = &fp.tex[fp.ntex];
    memset(tx, 0, sizeof *tx);
    fp_copy(tx->name, which ? "@stage" : "@char", FP_ID);
    tx->borrowed = true;
    tx->t.data = ic->data;
    tx->t.w = ic->w;
    tx->t.h = ic->h;
    if (ic->tlut_n != 0) {
        tx->t.lut = ic->lut;
        GXInitTlutObj(&tx->t.tlut, ic->lut, (GXTlutFmt) ic->tlut_fmt, (u16) ic->tlut_n);
        GXInitTexObjCI(&tx->t.obj, ic->data, ic->w, ic->h, (GXTexFmt) ic->fmt, GX_CLAMP, GX_CLAMP,
                       GX_FALSE, GX_TLUT0);
    } else {
        GXInitTexObj(&tx->t.obj, ic->data, ic->w, ic->h, (GXTexFmt) ic->fmt, GX_CLAMP, GX_CLAMP,
                     GX_FALSE);
    }
    GXInitTexObjLOD(&tx->t.obj, GX_LINEAR, GX_LINEAR, 0.0F, 0.0F, 0.0F, GX_FALSE, GX_FALSE,
                    GX_ANISO_1);
    tx->t.ok = true;
    tx->mask = false;
    return fp.ntex++;
}

/* A quad at an unsheared rect, sheared like the rest. */
static int fl_rect_quad(const char* id, int tex, float x0, float y0, float x1, float y1,
                        const float* rgb, int joint)
{
    int q = fp_add_quad(id, tex >= 0 ? FPK_TEX : FPK_FLAT, FPR_OTHER, -1);
    if (q < 0) {
        return -1;
    }
    fm_shear_rect(x0, y0, x1, y1, fp.q[q].v);
    fp.q[q].tex = (s8) tex;
    fp.q[q].has_colour = true;
    fm_rgb(fp.q[q].rgb, rgb);
    fp.q[q].joint = (s8) joint;
    return q;
}

/* ---- the screens' state ------------------------------------------------------------------- */

static int fl_ck(int who)
{
    if (fl.art == FL_LOAD) {
        return who >= 0 && who < 4 ? fl_ld.ck[who] : -1;
    }
    return Netplay_LobbyPlayer(who, 0);
}

static bool fl_reconnecting(void)
{
    return fe_np_phase != FE_NP_LOBBY && fe_np_phase != FE_NP_CONNECTED;
}

static int fl_phase(void) { return fl_reconnecting() ? LBP_OFF : Netplay_LobbyPhase(); }

static bool fl_turn_phase(int p)
{
    return p == LBP_STRIKE || p == LBP_BAN || p == LBP_PICK || p == LBP_CHAR_WINNER ||
           p == LBP_CHAR_LOSER;
}

static bool fl_my_stage_turn(void)
{
    int p = fl_phase();
    return (p == LBP_STRIKE || p == LBP_BAN || p == LBP_PICK) &&
           Netplay_LobbyInfo(4) == Netplay_LobbyMe() && fl.frames >= fl.coin_until &&
           Netplay_LobbyInfo(12);
}

static bool fl_can_pick_char(void)
{
    int p = fl_phase(), me = Netplay_LobbyMe();
    if (p == LBP_CHAR_BLIND) {
        return !Netplay_LobbyPlayer(me, 2);
    }
    return (p == LBP_CHAR_WINNER || p == LBP_CHAR_LOSER) && Netplay_LobbyInfo(4) == me;
}

static const char* fl_card_portrait(int w)
{
    int p = fl_phase(), me = Netplay_LobbyMe();
    bool locked = Netplay_LobbyPlayer(w, 2) != 0;
    if (p == LBP_CHAR_BLIND || p == LBP_OFF) {
        if (p == LBP_OFF && Netplay_LobbyInfo(0) > 1) {
            return "shown"; /* between games: last game's fighters */
        }
        if (!locked) return "empty";
        return w == me ? "shown" : "hidden";
    }
    return "shown";
}

static const char* fl_card_badge(int w)
{
    int p = fl_phase();
    bool locked = Netplay_LobbyPlayer(w, 2) != 0, ready = Netplay_LobbyPlayer(w, 3) != 0;
    switch (p) {
    case LBP_CHAR_BLIND: return locked ? "locked" : "picking";
    case LBP_CHAR_WINNER:
    case LBP_CHAR_LOSER:
        if (Netplay_LobbyInfo(4) == w) return "picking";
        return locked ? "locked" : "waiting";
    case LBP_READY:
    case LBP_GO: return ready ? "ready" : "waiting";
    default: return "none";
    }
}

static const char* fl_tile_state(int i)
{
    int s = Netplay_LobbyStage(i), p = fl_phase();
    switch (s) {
    case LBS_P1: return "struck_p1";
    case LBS_P2: return "struck_p2";
    case LBS_BANNED: return "banned";
    case LBS_PICKED: return "picked";
    default: break;
    }
    if (fl_my_stage_turn() && fl.cur == i) return "hovered";
    if (p == LBP_READY || p == LBP_GO || p == LBP_CHAR_WINNER || p == LBP_CHAR_LOSER ||
        p == LBP_OFF)
    {
        return "unavailable";
    }
    return "available";
}

static const char* fl_action_state(void)
{
    int p = fl_phase(), me = Netplay_LobbyMe(), opp = 1 - me;
    if (p == LBP_CHAR_BLIND) {
        return Netplay_LobbyPlayer(me, 2) ? "their_turn" : "your_turn";
    }
    if (fl_turn_phase(p)) {
        if (fl.frames < fl.coin_until) return NULL;
        return Netplay_LobbyInfo(4) == me ? "your_turn" : "their_turn";
    }
    if (p == LBP_READY || p == LBP_GO) {
        if (!Netplay_LobbyPlayer(me, 3)) return "ready_off";
        if (Netplay_LobbyPlayer(opp, 3)) return "ready_both";
        return fl.frames < fl.ready_on_until ? "ready_on" : "ready_wait";
    }
    return NULL;
}

static const char* fl_axis_state(const FlEl* e, const char* a)
{
    const char* id = fj_str(fj_get(e->el, "id"));
    if (fl.art == FL_LOAD) {
        if (strcmp(a, "you") == 0) return fl_ld.online && e->who == fl_ld.me ? "yes" : "no";
        if (strcmp(a, "kind") == 0) return fl_ld.cpu[e->who & 3] ? "cpu" : "human";
        if (strcmp(a, "view") == 0) return fl_ld.online ? "online" : "local";
        if (strcmp(a, "bar") == 0) return fl_ld.ready ? "ready" : "working";
        return NULL;
    }
    if (fl.art == FL_LOBBY) {
        if (strcmp(a, "phase") == 0) {
            int p = fl_phase();
            return p == LBP_STRIKE || p == LBP_BAN || p == LBP_PICK ? "stage"
                   : p == LBP_READY || p == LBP_GO                  ? "ready"
                                                                    : "character";
        }
        if (strcmp(a, "action") == 0) return fl_action_state();
        if (strcmp(a, "turn") == 0) {
            int p = fl_phase(), t = Netplay_LobbyInfo(4);
            bool on = fl_turn_phase(p) && fl.frames >= fl.coin_until;
            if (id != NULL && strncmp(id, "card", 4) == 0) {
                return on && t == e->who ? "active" : "idle";
            }
            return on ? (t == 0 ? "p1" : "p2") : "none";
        }
        if (strcmp(a, "portrait") == 0) return fl_card_portrait(e->who);
        if (strcmp(a, "badge") == 0) return fl_card_badge(e->who);
        if (strcmp(a, "crown") == 0) {
            return Netplay_LobbyInfo(0) > 1 && Netplay_LobbyInfo(1) == e->who ? "yes" : "no";
        }
        if (strcmp(a, "you") == 0) return e->who == Netplay_LobbyMe() ? "yes" : "no";
        if (strcmp(a, "tile") == 0) return fl_tile_state(e->who);
        if (strcmp(a, "coin") == 0) return Netplay_LobbyInfo(6) == 0 ? "p1" : "p2";
        return NULL;
    }
    if (fl.art == FL_WAIT) {
        bool host = Netplay_IsHost() != 0;
        if (strcmp(a, "view") == 0) return host && Netplay_RandomStatus() != 1 ? "host" : "guest";
        if (strcmp(a, "plate") == 0) {
            if (fe_np_phase == FE_NP_FAILED) return "failed";
            if (!host) return "joining";
            return fl.frames < fl.copied_until ? "copied" : "ng";
        }
        if (strcmp(a, "copy") == 0) return fl.frames < fl.copied_until ? "copied" : "idle";
    }
    if (fl.art == FL_CODE) {
        if (strcmp(a, "field") == 0) return fl.invalid ? "invalid" : "editing";
        if (strcmp(a, "slot") == 0) {
            int i = e->who, c = Netplay_CodeChar(i);
            bool act = i == Netplay_CodeSlot();
            if (fl.invalid && c != 0) return "invalid";
            if (act) return c != 0 ? "active_filled" : "active_empty";
            return c != 0 ? "filled" : "empty";
        }
        if (strcmp(a, "arrows") == 0) return "ng";
    }
    if (strcmp(a, "state") == 0) { /* the status strip */
        if (fl.art == FL_CODE) return fl.code_msg_bad ? "failed" : "idle";
        if (fe_np_phase == FE_NP_FAILED) return "failed";
        if (fe_np_phase == FE_NP_LOBBY || fe_np_phase == FE_NP_CONNECTED) return "connected";
        return "working";
    }
    return NULL;
}

/* ---- strings ------------------------------------------------------------------------------ */

static char fl_buf[8][FE_STR];

static const char* fl_stage_name(int i)
{
    static char n[40];
    Netplay_LobbyStageName(i, n, sizeof n);
    return n;
}

static void fl_instruction(char* out)
{
    int p = fl_phase(), me = Netplay_LobbyMe(), turn = Netplay_LobbyInfo(4),
        left = Netplay_LobbyInfo(5);
    bool mine = turn == me;
    const char* pl = left == 1 ? "" : "s";
    switch (p) {
    case LBP_CHAR_BLIND:
        sprintf(out, "%s", Netplay_LobbyPlayer(me, 2) ? "Locked in. Waiting for your opponent."
                                                      : "Press A to pick. It stays hidden till both lock.");
        break;
    case LBP_STRIKE:
        if (fl.frames < fl.coin_until) sprintf(out, "Coin flip: P%d strikes first.", Netplay_LobbyInfo(6) + 1);
        else if (mine) sprintf(out, "Strike %d stage%s: move, then press A.", left, pl);
        else sprintf(out, "P%d strikes %d stage%s.", turn + 1, left, pl);
        break;
    case LBP_BAN:
        if (!Netplay_LobbyInfo(12)) sprintf(out, "%s", "Getting the stage list...");
        else if (Netplay_LobbyInfo(0) <= 1 && mine) sprintf(out, "You won the coin flip: ban %d stage%s.", left, pl);
        else if (Netplay_LobbyInfo(0) <= 1) sprintf(out, "P%d won the coin flip: bans %d stage%s.", turn + 1, left, pl);
        else if (mine) sprintf(out, "You won: ban %d stage%s, then P%d picks.", left, pl, 2 - me);
        else sprintf(out, "P%d (last game's winner) bans %d stage%s.", turn + 1, left, pl);
        break;
    case LBP_PICK:
        if (mine) sprintf(out, "%s", "Pick the stage for this game.");
        else sprintf(out, "P%d picks the stage.", turn + 1);
        break;
    case LBP_CHAR_WINNER:
    case LBP_CHAR_LOSER:
        if (mine) sprintf(out, "%s", "Your pick: press A for the character screen.");
        else sprintf(out, "P%d is picking a character.", turn + 1);
        break;
    case LBP_READY:
        if (Netplay_LobbyInfo(8) > 0) sprintf(out, "%s", "Starting... B to cancel.");
        else if (Netplay_LobbyPlayer(me, 3)) sprintf(out, "%s", "Waiting for your opponent to ready up.");
        else sprintf(out, "%s", "Press START (or A) when you're ready.");
        break;
    case LBP_GO:
        sprintf(out, "%s", "Loading the match...");
        break;
    default:
        if (fe_np_phase == FE_NP_FAILED) Netplay_MenuStatus(out, FE_STR);
        else if (Netplay_IsHost()) sprintf(out, "%s", "Waiting for your opponent to come back.");
        else sprintf(out, "%s", "Getting back into the room...");
        break;
    }
}

static const char* fl_phase_title(void)
{
    switch (fl_phase()) {
    case LBP_CHAR_BLIND:
    case LBP_CHAR_WINNER:
    case LBP_CHAR_LOSER: return "CHARACTER PICK";
    case LBP_STRIKE: return "STAGE STRIKING";
    case LBP_BAN: return Netplay_LobbyInfo(0) <= 1 ? "STAGE BAN" : "COUNTERPICK";
    case LBP_PICK: return Netplay_LobbyInfo(0) <= 1 ? "STAGE PICK" : "COUNTERPICK";
    case LBP_READY: return "READY CHECK";
    case LBP_GO: return "GET READY";
    default: return "RECONNECTING";
    }
}

static void fl_status_line(char* out)
{
    const char* t = fl_toast_now();
    char code[8];
    if (t != NULL) {
        sprintf(out, "%s", t);
        return;
    }
    Netplay_RoomCode(code, sizeof code);
    if (fl.art == FL_LOBBY && !fl_reconnecting() && Netplay_LobbyInfo(9) > 6 && Netplay_LobbyInfo(12)) {
        int per = FL_DENSE_COLS * FL_DENSE_ROWS, n = Netplay_LobbyInfo(9);
        sprintf(out, "Room %s. Stages page %d of %d (L/R). B twice leaves.", code, fl.cur / per + 1,
                (n + per - 1) / per);
    } else if (fl.art == FL_LOBBY && !fl_reconnecting()) {
        sprintf(out, "Room %s. You are P%d%s. B twice leaves.", code, Netplay_LobbyMe() + 1,
                Netplay_IsHost() ? " (host)" : "");
    } else {
        Netplay_MenuStatus(out, FE_STR);
    }
}

static const char* fl_slot_text(const FlEl* e, const char* id)
{
    const char* el = fj_str(fj_get(e->el, "id"));
    char* b = fl_buf[e->who & 7];
    if (el == NULL) el = "";
    /* the status strip (waiting room, code entry) and the lobby's */
    if (strcmp(id, "status") == 0 || strcmp(id, "status_text") == 0) {
        if (fl.art == FL_CODE) {
            const char* t = fl_toast_now();
            return t != NULL ? t : fl.code_msg[0] != '\0' ? fl.code_msg
                                                          : "Enter the code, then press A to join.";
        }
        fl_status_line(fl_buf[6]);
        return fl_buf[6];
    }
    if (strcmp(id, "ping_ms") == 0) {
        int ms = Netplay_Ping();
        if (ms < 0 || fl_reconnecting()) return NULL;
        sprintf(fl_buf[7], "%d ms", ms);
        return fl_buf[7];
    }
    if (fl.art == FL_LOAD) {
        int w = e->who & 3;
        if (strcmp(id, "port_label") == 0) {
            sprintf(b, "P%d", fl_ld.port[w] + 1);
            return b;
        }
        if (strcmp(id, "you_tag") == 0) return "YOU";
        if (strcmp(id, "tag") == 0) {
            if (fl_ld.online) return w == 0 ? "HOST" : "GUEST";
            return fl_ld.cpu[w] ? "CPU" : NULL;
        }
        if (strcmp(id, "cpu_text") == 0) {
            sprintf(b, "CPU LV %d", fl_ld.cpu[w]);
            return b;
        }
        if (strcmp(id, "name") == 0) {
            Netplay_FighterName(fl_ld.ck[w], b, 24);
            return b;
        }
        if (strcmp(id, "vs_text") == 0) return "VS";
        if (strcmp(id, "stage_label") == 0) return "STAGE";
        if (strcmp(id, "stage_name") == 0) {
            Netplay_StageNameExt(fl_ld.ext, fl_buf[5], FE_STR);
            return fl_buf[5];
        }
        if (strcmp(id, "set_line") == 0) {
            sprintf(fl_buf[6], "GAME %d - SET %d - %d", fl_ld.game, fl_ld.s1, fl_ld.s2);
            return fl_buf[6];
        }
        if (strcmp(id, "room_line") == 0) {
            sprintf(fl_buf[7], "ROOM %s", fl_ld.room);
            return fl_buf[7];
        }
        if (strcmp(id, "bar_label") == 0) return fl_ld.ready ? "READY" : "WARMING UP";
        if (strcmp(id, "bar_pct") == 0) {
            sprintf(b, "%d%%", fe.percent < 0 ? 0 : fe.percent);
            return b;
        }
        return NULL;
    }
    if (fl.art == FL_WAIT) {
        char code[8];
        Netplay_RoomCode(code, sizeof code);
        if (strcmp(id, "code_label") == 0) return "ROOM CODE";
        if (strncmp(id, "char_", 5) == 0) {
            int i = id[5] - '0';
            static char c[2][4][2];
            int h = Netplay_IsHost() != 0;
            c[h][i][0] = i >= 0 && i < 4 && (int) strlen(code) > i ? code[i] : '\0';
            c[h][i][1] = '\0';
            return c[h][i];
        }
        if (strcmp(id, "copy_text") == 0) {
            const FjNode* n = fl_lookup(e, "<copy.text_string>");
            return fj_str(n);
        }
        if (Netplay_RandomStatus() == 1) {
            if (strcmp(id, "code_label") == 0) return "RANDOM OPPONENT";
            if (strncmp(id, "char_", 5) == 0) return NULL;
            if (strcmp(id, "share_line") == 0) {
                sprintf(b, "Looking for an opponent... %ds", Netplay_RandomSeconds());
                return b;
            }
            if (strcmp(id, "sub_line") == 0) return "B stops looking.";
        }
        if (strcmp(id, "share_line") == 0) {
            if (Netplay_IsHost()) return code[0] != '\0' ? "Send it to your opponent. They pick Join and type it in."
                                                         : "Asking the server for a room...";
            sprintf(b, "Joining room %s...", code);
            return b;
        }
        if (strcmp(id, "sub_line") == 0) {
            return Netplay_IsHost() ? "Codes use A-Z and 2-9, never I, O, 0 or 1." : NULL;
        }
        return NULL;
    }
    if (fl.art == FL_CODE) {
        if (strcmp(id, "instruction") == 0) return "Type the 4-character room code from your host.";
        if (strcmp(id, "char") == 0) {
            static char c[4][2];
            int i = e->who;
            c[i][0] = (char) Netplay_CodeChar(i);
            c[i][1] = '\0';
            return c[i];
        }
        if (strcmp(id, "arrow_up") == 0) return "\x03";
        if (strcmp(id, "arrow_down") == 0) return "\x05";
        if (strcmp(id, "keyboard_text") == 0) return "Or type it on a keyboard";
        if (strcmp(id, "paste_text") == 0) return "Paste";
        if (strcmp(id, "alphabet_note") == 0) return "Codes use A-Z and 2-9, never I, O, 0 or 1.";
        return NULL;
    }
    /* the lobby */
    if (strcmp(id, "set_label") == 0) return "SET";
    if (strcmp(id, "set_score") == 0) {
        sprintf(b, "%d - %d", Netplay_LobbyInfo(2), Netplay_LobbyInfo(3));
        return b;
    }
    if (strcmp(id, "phase_title") == 0) return fl_phase_title();
    if (strcmp(id, "instruction") == 0) {
        fl_instruction(fl_buf[5]);
        return fl_buf[5];
    }
    if (strcmp(id, "action_text") == 0) {
        const char* st = fl_action_state();
        if (st != NULL && fl_phase() == LBP_CHAR_BLIND) {
            return strcmp(st, "your_turn") == 0 ? "PICK" : "WAITING";
        }
        return fj_str(fl_lookup(e, "<action.string>"));
    }
    if (strncmp(el, "card", 4) == 0) {
        int w = e->who;
        if (strcmp(id, "port_label") == 0) return w == 0 ? "P1" : "P2";
        if (strcmp(id, "you_tag") == 0) return "YOU";
        if (strcmp(id, "hidden_mark") == 0) return "?";
        if (strcmp(id, "name") == 0) {
            const char* pt = fl_card_portrait(w);
            if (strcmp(pt, "shown") != 0) return strcmp(pt, "hidden") == 0 ? "Locked in" : "Choosing...";
            Netplay_FighterName(fl_ck(w), b, 24);
            return b;
        }
        if (strcmp(id, "badge_text") == 0) {
            const char* bd = fl_card_badge(w);
            return strcmp(bd, "locked") == 0    ? "LOCKED IN"
                   : strcmp(bd, "picking") == 0 ? "PICKING"
                   : strcmp(bd, "ready") == 0   ? "READY"
                                                : "WAITING";
        }
        return NULL;
    }
    if (strncmp(el, "tile", 4) == 0) {
        int s = Netplay_LobbyStage(e->who);
        if (strcmp(id, "name") == 0) return fl_stage_name(e->who);
        if (strcmp(id, "tab_text") == 0) {
            if (s == LBS_P1) return "P1";
            if (s == LBS_P2) return "P2";
            if (s == LBS_BANNED) return Netplay_LobbyInfo(1) == 1 ? "P2" : "P1";
            return NULL;
        }
        return NULL;
    }
    if (strcmp(id, "coin_label") == 0) return Netplay_LobbyInfo(6) == 0 ? "P1" : "P2";
    if (strcmp(id, "coin_caption") == 0) return "COIN FLIP";
    if (strcmp(id, "coin_text") == 0) {
        sprintf(b, "P%d STRIKES FIRST", Netplay_LobbyInfo(6) + 1);
        return b;
    }
    if (strcmp(id, "numeral") == 0) {
        sprintf(b, "%d", Netplay_LobbyInfo(8));
        return b;
    }
    if (strcmp(id, "stage_line") == 0) {
        int c = Netplay_LobbyInfo(7);
        return c >= 0 ? fl_stage_name(c) : NULL;
    }
    return NULL;
}

/* Quads positioned by a rule rather than numbers, and the disc-art quads. */
static bool fl_special_quad(FlEl* e, const FjNode* q, const char* id, float dx, float dy, int joint)
{
    const char* kind = fj_str(fj_get(q, "kind"));
    char qid[FP_ID];
    float rgb[3];
    sprintf(qid, "%s%.19s", e->pre, id);
    if (kind != NULL && strcmp(kind, "disc_art") == 0) {
        const FjNode* v0 = fj_idx(fj_get(q, "verts"), 0);
        const FjNode* v2 = fj_idx(fj_get(q, "verts"), 2);
        float x0 = fj_num(fj_idx(v0, 0), 0) + dx, y0 = fj_num(fj_idx(v0, 1), 0) + dy;
        float x1 = fj_num(fj_idx(v2, 0), 0) + dx, y1 = fj_num(fj_idx(v2, 1), 0) + dy;
        if (strcmp(id, "portrait_art") == 0) {
            int t = fl_char_tex(fl_ck(e->who));
            float white[3] = { 1, 1, 1 };
            if (t >= 0) {
                fl_rect_quad(qid, t, x0, y0, x1, y1, white, joint);
            } else {
                fm_tok("@face_hi", KS_VERSUS, rgb);
                fl_rect_quad(qid, -1, x0, y0, x1, y1, rgb, joint);
            }
        } else if (fl.art == FL_LOAD && !fl_ld.online && fl_icon_tex(1) >= 0) {
            float white[3] = { 1, 1, 1 };
            fl_rect_quad(qid, fl_icon_tex(1), x0, y0, x1, y1, white, joint); /* the SSS's icon */
        } else if (fl.art == FL_LOBBY && fs_stage_art_tex(Netplay_LobbyStageExt(e->who)) >= 0) {
            /* the stage's select icon, off the disc's MnSlMap */
            const FjNode* op = fl_val(e, fj_get(q, "opacity"));
            float white[3] = { 1, 1, 1 };
            int qi = fs_tex_fit(qid, fs_stage_art_tex(Netplay_LobbyStageExt(e->who)), x0, y0, x1, y1, white);
            if (qi >= 0) {
                fp.q[qi].joint = (s8) joint;
                fp.q[qi].wa = -fj_num(op, 1.0F);
            }
        } else {
            /* stage art: not captured (online: no SSS) - the layout's flat stand-in */
            const FjNode* op = fl_val(e, fj_get(q, "opacity"));
            int qi;
            fm_tok("@face_hi", KS_VERSUS, rgb);
            qi = fl_rect_quad(qid, -1, x0, y0, x1, y1, rgb, joint);
            if (qi >= 0) fp.q[qi].wa = -(0.5F * fj_num(op, 1.0F));
        }
        return true;
    }
    if (strcmp(id, "badge") == 0) {
        /* from the text column edge (P1 left = 90, P2 right = 130): 5 + 12 + 4 + text + 6 */
        const char* s = fl_slot_text(e, "badge_text");
        float w = 5 + 12 + 4 + (s != NULL ? ff_width(ff_role("caption"), s) : 40) + 6;
        float x0 = e->who == 0 ? 90 : 130 - w;
        if (!fl_colour(e, fj_get(q, "colour"), rgb)) return true;
        fl_rect_quad(qid, -1, x0 + dx, 47 + dy, x0 + w + dx, 65 + dy, rgb, joint);
        return true;
    }
    if (strcmp(id, "action_icon") == 0) {
        /* icon + 6 + text, centred on the plate as one group; the text slot follows */
        const char* s = fl_slot_text(e, "action_text");
        const FjNode* tn = fl_val(e, fj_get(q, "texture"));
        float iw = fj_num(fl_lookup(e, "<action.icon_w>"), 16);
        float tw = s != NULL ? ff_width(ff_role("row"), s) : 0;
        float x0 = 478 - (iw + 6 + tw) * 0.5F;
        int t = fp_tex(fj_str(tn));
        if (t < 0 || !fl_colour(e, fj_get(q, "colour"), rgb)) return true;
        fl_rect_quad(qid, t, x0 + dx, 95 + dy, x0 + iw + dx, 95 + (iw > 16 ? 16 : iw) + dy, rgb,
                     joint);
        return true;
    }
    if (strcmp(id, "paste_glyph") == 0) {
        float adv = ff_width(ff_role("body"), "Paste");
        int t = fp_tex("glyph_y");
        if (t < 0 || !fl_colour(e, fj_get(q, "colour"), rgb)) return true;
        fl_rect_quad(qid, t, 491 - 4 - adv - 16, 284, 491 - 4 - adv, 300, rgb, joint);
        return true;
    }
    if (strncmp(id, "bar_", 4) == 0 && id[4] >= '1' && id[4] <= '4' && id[5] == 0) {
        /* ping bars: the first `bars` lit in the level's colour, the rest unlit */
        int ms = Netplay_Ping(), bars, n = id[4] - '0', t;
        const char* col;
        const FjNode* v0 = fj_idx(fj_get(q, "verts"), 0);
        const FjNode* v2 = fj_idx(fj_get(q, "verts"), 2);
        char tex[16];
        if (ms < 0 || fl_reconnecting() || fe_np_phase == FE_NP_WORKING) return true;
        bars = ms <= 60 ? 4 : ms <= 100 ? 3 : ms <= 150 ? 2 : 1;
        col = n <= bars ? (bars >= 3 ? "ok" : bars == 2 ? "gold" : "danger") : "@face";
        fm_tok(col, KS_VERSUS, rgb);
        sprintf(tex, "sig_bar_%d", n);
        t = fp_tex(tex);
        if (t < 0) return true;
        fl_rect_quad(qid, t, fj_num(fj_idx(v0, 0), 0) + dx, fj_num(fj_idx(v0, 1), 0) + dy,
                     fj_num(fj_idx(v2, 0), 0) + dx, fj_num(fj_idx(v2, 1), 0) + dy, rgb, joint);
        return true;
    }
    if (strncmp(id, "pip_", 4) == 0 || strcmp(id, "sweep") == 0) {
        /* the status strip's activity: only while working (the waiting room's sweep: guests) */
        const char* st = fl_axis_state(e, "state");
        const char* el = fj_str(fj_get(e->el, "id"));
        if (el != NULL && strcmp(el, "room_plate") == 0) return false; /* the plate's own sweep */
        return st == NULL || strcmp(st, "working") != 0;
    }
    return false;
}

/* ---- building ----------------------------------------------------------------------------- */

static void fl_desc(const char* s)
{
    float bone[3];
    fm_tok("bone", KS_VERSUS, bone);
    fm_text("fl_desc", FPR_OTHER, -1, kit.desc_role, kit.desc_anchor[0], kit.desc_anchor[1], s,
            bone, kit.desc_max);
}

/* The status strip, from online_layout.json. */
static void fl_status_strip(void)
{
    void *t, *p;
    const FjNode* L = fj_load("online_layout.json", &t, &p);
    FlEl e;
    memset(&e, 0, sizeof e);
    e.el = fj_get(L, "status_strip");
    sprintf(e.pre, "ss_");
    if (e.el != NULL) {
        fl_el(&e);
    }
    if (t != NULL) HSD_Free(t);
    if (p != NULL) HSD_Free(p);
}

static void fl_build_wait(void)
{
    static const char* const hg_h[2] = { "glyph_b", "glyph_x" };
    static const char* const ht_h[2] = { "Leave", "Copy Code" };
    static const char* const hg_g[1] = { "glyph_b" };
    static const char* const ht_g[1] = { "Cancel" };
    const char* crumbs_h[3] = { "VERSUS", "ONLINE PLAY", "HOST" };
    const char* crumbs_g[3] = { "VERSUS", "ONLINE PLAY", "JOIN" };
    const char* crumbs_r[3] = { "VERSUS", "ONLINE PLAY", "RANDOM" };
    bool host = Netplay_IsHost() != 0, rnd = Netplay_RandomStatus() == 1;
    void *t, *p;
    const FjNode* L;
    FlEl e;
    if (rnd) {
        fm_chrome_with("FINDING OPPONENT", crumbs_r, 3, KS_VERSUS, 1, hg_g, ht_g);
        fl_desc("You'll meet in a lobby as soon as someone else is looking.");
    } else {
        fm_chrome_with(host ? "WAITING ROOM" : "JOINING ROOM", host ? crumbs_h : crumbs_g, 3, KS_VERSUS,
                       host ? 2 : 1, host ? hg_h : hg_g, host ? ht_h : ht_g);
        fl_desc(host ? "Your room stays open until you leave." : "The host sees you as soon as you're in.");
    }
    L = fj_load("waiting_room_layout.json", &t, &p);
    memset(&e, 0, sizeof e);
    e.el = fj_get(L, "room_plate");
    sprintf(e.pre, "wr_");
    if (e.el != NULL) {
        fl_el(&e);
    }
    if (t != NULL) HSD_Free(t);
    if (p != NULL) HSD_Free(p);
    fl_status_strip();
}

static void fl_build_code(void)
{
    static const char* const hg[4] = { "glyph_dpad", "glyph_a", "glyph_y", "glyph_b" };
    static const char* const ht[4] = { "Move", "Join", "Paste", "Back" };
    const char* crumbs[3] = { "VERSUS", "ONLINE PLAY", "JOIN" };
    void *t, *p;
    const FjNode* L;
    const FjNode* F;
    FlEl e;
    int i;
    fm_chrome_with("JOIN ROOM", crumbs, 3, KS_VERSUS, 4, hg, ht);
    fl_desc("Up/down change a letter, left/right move between slots.");
    L = fj_load("code_entry_layout.json", &t, &p);
    memset(&e, 0, sizeof e);
    e.el = fj_get(L, "panel");
    sprintf(e.pre, "pn_");
    fl_el(&e);
    F = fj_get(L, "field");
    e.el = F;
    sprintf(e.pre, "fd_");
    fl_el(&e);
    for (i = 0; i < 4; i++) {
        e.el = fj_get(F, "slot_template");
        e.ox = 161.0F + 82.0F * i;
        e.oy = 150.0F;
        e.who = i;
        sprintf(e.idx, "%d", i);
        sprintf(e.pre, "s%d_", i);
        fl_el(&e);
    }
    e.el = fj_get(F, "arrows");
    e.ox = 161.0F + 82.0F * Netplay_CodeSlot() + 36.0F;
    e.oy = 0.0F;
    e.who = 0;
    sprintf(e.pre, "ar_");
    fl_el(&e);
    memset(&e, 0, sizeof e);
    e.el = fj_get(L, "hints_row");
    sprintf(e.pre, "hr_");
    fl_el(&e);
    if (t != NULL) HSD_Free(t);
    if (p != NULL) HSD_Free(p);
    fl_status_strip();
}

static void fl_build_lobby(void)
{
    const char* hg[4];
    const char* ht[4];
    const char* crumbs[3] = { "VERSUS", "ONLINE PLAY", NULL };
    char room[24], code[8], title[16];
    int nh = 0, i, p = fl_phase(), n, cols;
    void *t, *pp;
    const FjNode* L;
    const FjNode* G;
    FlEl e;
    Netplay_RoomCode(code, sizeof code);
    sprintf(room, "ROOM %s", code);
    crumbs[2] = room;
    if (fl_can_pick_char()) {
        hg[nh] = "glyph_a"; ht[nh++] = "Pick";
    } else if (fl_my_stage_turn()) {
        hg[nh] = "glyph_a";
        ht[nh++] = p == LBP_STRIKE ? "Strike" : p == LBP_BAN ? "Ban" : "Pick";
        hg[nh] = "glyph_dpad"; ht[nh++] = "Move";
    } else if (p == LBP_READY) {
        bool ready = Netplay_LobbyPlayer(Netplay_LobbyMe(), 3) != 0;
        hg[nh] = "glyph_start"; ht[nh++] = ready ? "Not Ready" : "Ready";
    }
    hg[nh] = "glyph_b";
    ht[nh++] = p == LBP_READY && Netplay_LobbyInfo(8) > 0 &&
                       Netplay_LobbyPlayer(Netplay_LobbyMe(), 3)
                   ? "Cancel"
               : fl.frames < fl.leave_until ? "Leave?"
                                            : "Leave";
    sprintf(title, "GAME %d", Netplay_LobbyInfo(0) > 0 ? Netplay_LobbyInfo(0) : 1);
    fm_chrome_with(title, crumbs, 3, KS_VERSUS, nh, hg, ht);
    L = fj_load("lobby_layout.json", &t, &pp);
    memset(&e, 0, sizeof e);
#define FL_EL(name, prefix)                                                                       \
    e.el = fj_get(L, name);                                                                       \
    e.ox = e.oy = 0.0F;                                                                           \
    sprintf(e.pre, prefix);                                                                       \
    if (e.el != NULL) fl_el(&e);
    FL_EL("score_plaque", "sp_");
    FL_EL("phase_banner", "pb_");
    if (fl_action_state() != NULL) {
        FL_EL("action_plate", "ap_");
    }
    for (i = 0; i < 2; i++) {
        e.el = fj_idx(fj_get(L, "cards"), i);
        e.ox = fj_num(fj_idx(fj_get(e.el, "origin"), 0), 88);
        e.oy = fj_num(fj_idx(fj_get(e.el, "origin"), 1), 134);
        e.who = i;
        sprintf(e.port, "p%d", i + 1);
        sprintf(e.pre, "c%d_", i);
        fl_el(&e);
    }
    e.port[0] = '\0';
    FL_EL("turn_arrow", "ta_");
    /* the stage grid: tile_0 is the template, placed per index on the grid's columns, so the list
       can be as long as the rules make it (rows beyond the second belong to a scrolling list) */
    G = fj_get(L, "stage_grid");
    n = Netplay_LobbyInfo(9);
    cols = (int) fj_num(fj_get(G, "columns"), 3);
    if (!Netplay_LobbyInfo(12)) {
        n = 0; /* the host's stage list is still on its way */
    }
    if (n > 6) {
        fl_dense_grid(); /* a long list (all stages): smaller tiles, pages */
        n = 0;
    }
    {
        float tw = fj_num(fj_idx(fj_get(G, "tile"), 0), 148), th = fj_num(fj_idx(fj_get(G, "tile"), 1), 86);
        float gx = fj_num(fj_idx(fj_get(G, "gutter"), 0), 12), gy = fj_num(fj_idx(fj_get(G, "gutter"), 1), 8);
        float x0 = fj_num(fj_idx(fj_get(G, "rect"), 0), 88), y0 = fj_num(fj_idx(fj_get(G, "rect"), 1), 214);
        int k, last = -1;
        for (k = 0; k <= n; k++) {
            /* the hovered or picked tile last, so its lift draws over its neighbours */
            const char* st;
            i = k < n ? k : last;
            if (i < 0 || i >= LB_MAX_UI_STAGES) continue;
            st = fl_tile_state(i);
            if (k < n && (strcmp(st, "hovered") == 0 || strcmp(st, "picked") == 0)) {
                last = i;
                continue;
            }
            if (i / cols >= 2) continue; /* two rows on screen for now */
            e.el = fj_idx(fj_get(G, "tiles"), 0);
            e.tstates = fj_get(G, "tile_states");
            e.ox = x0 + (tw + gx) * (float) (i % cols);
            e.oy = y0 + (th + gy) * (float) (i / cols);
            e.who = i;
            sprintf(e.idx, "%d", i);
            sprintf(e.port, "p%d", Netplay_LobbyInfo(1) == 1 ? 2 : 1); /* a ban's colour: the banner */
            sprintf(e.pre, "t%d_", i);
            fl_el(&e);
            if (k == n) break;
        }
        e.tstates = NULL;
        e.port[0] = '\0';
        e.idx[0] = '\0';
    }
    if ((p == LBP_STRIKE || p == LBP_BAN) && fl.frames < fl.coin_until) {
        FL_EL("coin_badge", "cb_");
    }
    if (p == LBP_READY && Netplay_LobbyInfo(8) > 0) {
        FL_EL("countdown", "cd_");
    }
    FL_EL("lobby_status", "ls_");
#undef FL_EL
    if (t != NULL) HSD_Free(t);
    if (pp != NULL) HSD_Free(pp);
}

/* Who fights where: the online lobby's two players, or VS mode's players and stage. */
static void fl_load_init(void)
{
    int i;
    memset(&fl_ld, 0, sizeof fl_ld);
    fl_ld.q_fill = fl_ld.q_pct = -1;
    if (fe.screen == &fe_screen_online_load) {
        fl_ld.online = 1;
        fl_ld.n = 2;
        fl_ld.me = Netplay_LobbyMe();
        for (i = 0; i < 2; i++) {
            fl_ld.ck[i] = Netplay_LobbyPlayer(i, 0);
            fl_ld.color[i] = Netplay_LobbyPlayer(i, 1);
            fl_ld.port[i] = i;
        }
        fl_ld.ext = Netplay_LobbyStageExt(Netplay_LobbyInfo(7) >= 0 ? Netplay_LobbyInfo(7) : 0);
        fl_ld.game = Netplay_LobbyInfo(0);
        fl_ld.s1 = Netplay_LobbyInfo(2);
        fl_ld.s2 = Netplay_LobbyInfo(3);
        Netplay_RoomCode(fl_ld.room, sizeof fl_ld.room);
        return;
    }
    {
        struct VsModeData* vs = gmVsMelee_GetVsData();
        for (i = 0; i < 4 && vs != NULL; i++) {
            PlayerInitData* pl = &vs->start.players[i];
            if ((pl->slot_type != Gm_PKind_Human && pl->slot_type != Gm_PKind_Cpu) ||
                pl->ckind < 0 || pl->ckind == ChKind_None)
            {
                continue;
            }
            fl_ld.ck[fl_ld.n] = pl->ckind;
            fl_ld.color[fl_ld.n] = pl->color;
            fl_ld.cpu[fl_ld.n] =
                pl->slot_type == Gm_PKind_Cpu ? (pl->cpu_level > 0 ? pl->cpu_level : 1) : 0;
            fl_ld.port[fl_ld.n] = i;
            fl_ld.n++;
        }
        fl_ld.ext = vs != NULL ? vs->start.rules.stkind : 0;
    }
    if (fl_ld.n < 1) {
        fl_ld.n = 1;
    }
}

static void fl_build_load(void)
{
    const char* crumbs[3];
    char room[16];
    void *t, *p;
    const FjNode* L;
    const FjNode* C;
    const FjNode* O;
    FlEl e;
    int i, n = fl_ld.n;
    if (fl_ld.online) {
        sprintf(room, "ROOM %s", fl_ld.room);
        crumbs[0] = "VERSUS";
        crumbs[1] = "ONLINE PLAY";
        crumbs[2] = room;
    } else {
        crumbs[0] = "VERSUS";
        crumbs[1] = "MELEE";
        crumbs[2] = "LOADING";
    }
    fm_chrome_with("GET READY", crumbs, 3, KS_VERSUS, 0, NULL, NULL);
    fl_desc(fl_ld.ready ? "Here we go!" : "Loading the match...");
    L = fj_load("loading_layout.json", &t, &p);
    C = fj_get(fj_get(L, "cards"), n == 2 ? "big" : "small");
    O = fj_get(fj_get(fj_get(L, "card_places"), n == 2 ? "two" : "up_to_four"), "origins");
    memset(&e, 0, sizeof e);
    for (i = 0; i < n && i < 4; i++) {
        const FjNode* o = fj_idx(O, i);
        e.el = C;
        /* three compact cards centre as a group; one sits in the middle */
        e.ox = fj_num(fj_idx(o, 0), 92) + (n == 3 ? 60.0F : n == 1 ? 180.0F : 0.0F);
        e.oy = fj_num(fj_idx(o, 1), 84);
        e.who = i;
        sprintf(e.port, "p%d", fl_ld.port[i] + 1);
        sprintf(e.pre, "c%d_", i);
        fl_el(&e);
    }
    memset(&e, 0, sizeof e);
    if (n == 2) {
        e.el = fj_get(L, "vs_mark");
        sprintf(e.pre, "vs_");
        fl_el(&e);
    }
    e.el = fj_get(L, "stage_plate");
    sprintf(e.pre, "st_");
    fl_el(&e);
    e.el = fj_get(L, "progress");
    sprintf(e.pre, "pg_");
    fl_el(&e);
    if (t != NULL) HSD_Free(t);
    if (p != NULL) HSD_Free(p);
    fl_ld.q_fill = fp_find_quad("pg_bar_fill");
    fl_ld.q_pct = fp_find_quad("pg_bar_pct");
    if (fl_ld.q_fill >= 0) {
        FpQuad* q = &fp.q[fl_ld.q_fill];
        fl_ld.fill_w = q->v[1][0] - q->v[0][0];
    }
    if (fl_ld.q_pct >= 0) {
        FpQuad* d = &fp.q[fl_ld.q_pct];
        fl_ld.pct_x = d->ax + (d->ff_role >= 0 ? ff_width(d->ff_role, d->text) : 0.0F);
    }
}

/* Every frame: the bar's fill follows the warm-up, and so does its readout. */
static void fl_load_update(void)
{
    if (fl_ld.q_fill >= 0) {
        FpQuad* q = &fp.q[fl_ld.q_fill];
        q->v[1][0] = q->v[0][0] + fl_ld.fill_w * fe.progress;
        q->v[2][0] = q->v[3][0] + fl_ld.fill_w * fe.progress;
    }
    if (fl_ld.q_pct >= 0) {
        char b[8];
        sprintf(b, "%d%%", fe.percent < 0 ? 0 : fe.percent);
        fk_retext(fl_ld.q_pct, b, fl_ld.pct_x, 2, 60.0F);
    }
}

/* A long stage list (all stages): 6 x 3 small tiles a page, the stage art, the name, the same
   marks as the big tiles (P1 cross, P2 ring, padlock for a ban, gold for the pick). */
static void fl_dense_grid(void)
{
    int n = Netplay_LobbyInfo(9), per = FL_DENSE_COLS * FL_DENSE_ROWS, page, i, first, last;
    float x0 = 88, y0 = 214, gx = 6, gy = 6;
    float w = (468 - (FL_DENSE_COLS - 1) * gx) / FL_DENSE_COLS, h = (180 - (FL_DENSE_ROWS - 1) * gy) / FL_DENSE_ROWS;
    float ink[3], face[3], gold[3], bone[3], muted[3], white[3] = { 1, 1, 1 }, p1[3], p2[3];
    char id[FP_ID], name[40], buf[24];
    if (fl.cur >= n) fl.cur = n - 1;
    if (fl.cur < 0) fl.cur = 0;
    page = fl.cur / per;
    first = page * per;
    last = first + per < n ? first + per : n;
    fm_tok("ink", KS_VERSUS, ink);
    fm_tok("@face", KS_VERSUS, face);
    fm_tok("gold", KS_VERSUS, gold);
    fm_tok("bone", KS_VERSUS, bone);
    fm_tok("muted", KS_VERSUS, muted);
    kit_colour("port:p1", KS_VERSUS, p1);
    kit_colour("port:p2", KS_VERSUS, p2);
    for (i = first; i < last; i++) {
        int s_ = Netplay_LobbyStage(i), k = i - first, tex;
        float rx = x0 + (k % FL_DENSE_COLS) * (w + gx), ry = y0 + (k / FL_DENSE_COLS) * (h + gy);
        bool hov = fl_my_stage_turn() && fl.cur == i;
        bool out = s_ == LBS_P1 || s_ == LBS_P2 || s_ == LBS_BANNED;
        sprintf(id, "dg%d", i);
        fm_flat(id, FPR_OTHER, -1, rx - 3, ry - 3, rx + w + 3, ry + h + 3,
                hov || s_ == LBS_PICKED ? gold : ink);
        sprintf(id, "df%d", i);
        fm_flat(id, FPR_OTHER, -1, rx, ry, rx + w, ry + h, out ? ink : face);
        tex = fs_stage_art_tex(Netplay_LobbyStageExt(i));
        if (tex >= 0) {
            int q;
            sprintf(id, "da%d", i);
            q = fs_tex_fit(id, tex, rx + 2, ry + 2, rx + w - 2, ry + h - 13, white);
            if (q >= 0 && out) fp.q[q].wa = -0.3F;
        }
        sprintf(id, "dn%d", i);
        Netplay_LobbyStageName(i, name, sizeof name);
        fl_text_at(id, "caption", rx + w * 0.5F, ry + h - 3, "centre", w - 4, name, out ? muted : bone, -1);
        if (out) {
            int t = fp_tex(s_ == LBS_P1 ? "ico_strike_x" : s_ == LBS_P2 ? "ico_strike_o" : "ico_lock");
            const float* c = s_ == LBS_P1 ? p1 : s_ == LBS_P2 ? p2 : Netplay_LobbyInfo(1) == 1 ? p2 : p1;
            if (t >= 0) {
                sprintf(id, "dm%d", i);
                fl_rect_quad(id, t, rx + w * 0.5F - 13, ry + 6, rx + w * 0.5F + 13, ry + 32, c, -1);
            }
        }
        if (s_ == LBS_PICKED) {
            int t = fp_tex("ico_pick");
            sprintf(id, "dp%d", i);
            fm_flat(id, FPR_OTHER, -1, rx + w - 18, ry, rx + w, ry + 14, ink);
            if (t >= 0) {
                sprintf(id, "dq%d", i);
                fl_rect_quad(id, t, rx + w - 16, ry + 1, rx + w - 4, ry + 13, gold, -1);
            }
        }
    }
    (void) buf;
}

/* What the current screen shows, as a string: a rebuild whenever it changes. */
static void fl_signature(char* sig)
{
    char b[FE_STR];
    int i;
    sig[0] = '\0';
    fl_sig_int(sig, fl.art);
    fl_sig_int(sig, fe_np_phase);
    fl_sig_add(sig, fl_toast_now() != NULL ? fl_toast_now() : "");
    if (fl.art == FL_LOBBY) {
        fl_sig_int(sig, Netplay_LobbySeq());
        fl_sig_int(sig, fl.cur);
        fl_sig_int(sig, Netplay_LobbyInfo(8));
        fl_sig_int(sig, fl.frames < fl.coin_until);
        fl_sig_int(sig, fl.frames < fl.leave_until);
        fl_sig_int(sig, fl.frames < fl.ready_on_until);
        fl_sig_int(sig, Netplay_Ping() / 25);
        fl_sig_int(sig, fe_np_char_icon[fl_ck(0) & (FE_NP_MAX_CK - 1)].ok);
        fl_sig_int(sig, fe_np_char_icon[fl_ck(1) & (FE_NP_MAX_CK - 1)].ok);
        fl_instruction(b);
        fl_sig_add(sig, b);
        fl_status_line(b);
        fl_sig_add(sig, b);
    } else if (fl.art == FL_WAIT) {
        Netplay_RoomCode(b, 8);
        fl_sig_add(sig, b);
        Netplay_MenuStatus(b, FE_STR);
        fl_sig_add(sig, b);
        fl_sig_int(sig, fl.frames < fl.copied_until);
        fl_sig_int(sig, Netplay_RandomStatus() * 1000 + Netplay_RandomSeconds());
        fl_sig_int(sig, Netplay_Ping() / 25);
    } else if (fl.art == FL_CSS) {
        fs_css_sig(sig);
    } else if (fl.art == FL_SSS) {
        fs_sss_sig(sig);
    } else if (fl.art == FL_LOAD) {
        fl_sig_int(sig, fl_ld.ready);
        for (i = 0; i < fl_ld.n; i++) {
            fl_sig_int(sig, fe_np_char_icon[fl_ld.ck[i] & (FE_NP_MAX_CK - 1)].ok);
        }
    } else if (fl.art == FL_CODE) {
        for (i = 0; i < 4; i++) {
            fl_sig_int(sig, Netplay_CodeChar(i));
        }
        fl_sig_int(sig, Netplay_CodeSlot());
        fl_sig_int(sig, fl.invalid);
        fl_sig_add(sig, fl.code_msg);
    }
}

static void fl_build(void)
{
    kit_load();
    ff_load();
    kit_widgets_load();
    kw_section = KS_VERSUS;
    fp_reset(); /* the textures stay loaded: rebuilds come often */
    fs_drop_borrowed(); /* but the lent ones (disc art) are lent again by this build */
    fm_default_states(KS_VERSUS);
    fp.recolour = false;
    switch (fl.art) {
    case FL_WAIT: fl_build_wait(); break;
    case FL_CODE: fl_build_code(); break;
    case FL_LOBBY: fl_build_lobby(); break;
    case FL_LOAD: fl_build_load(); break;
    case FL_CSS: fs_build_css(); break;
    case FL_SSS: fs_build_sss(); break;
    default: break;
    }
    fl_finish();
    fl.on = true;
    if (fl.art == FL_LOAD) {
        FlEl e;
        memset(&e, 0, sizeof e);
        if (!fl_ld.built) {
            /* arriving: the cards slide in, the VS mark pops, the stage plate rises */
            fl_play("loading_motion.json", "cards_in", &e);
            fl_play("loading_motion.json", "vs_pop", &e);
            fl_play("loading_motion.json", "stage_in", &e);
            fl_ld.built = 1;
        } else if (fl_ld.ready) {
            fl_play("loading_motion.json", "ready", &e);
        }
        fl_load_update();
    }
    /* the event this change deserves */
    switch (fl.pend_event) {
    case FLE_STRIKE:
    case FLE_BAN:
    case FLE_PICK: {
        FlEl e;
        memset(&e, 0, sizeof e);
        sprintf(e.idx, "%d", fl.pend_arg);
        fl_play("lobby_motion.json", fl.pend_event == FLE_STRIKE ? "strike_stamp"
                                     : fl.pend_event == FLE_BAN  ? "ban_lock"
                                                                 : "pick_confirm", &e);
        break;
    }
    case FLE_READY: {
        FlEl e;
        memset(&e, 0, sizeof e);
        fl_play("lobby_motion.json", "ready_toggle", &e);
        break;
    }
    case FLE_TICK: {
        FlEl e;
        memset(&e, 0, sizeof e);
        fl_play("lobby_motion.json", "countdown_tick", &e);
        break;
    }
    case FLE_COIN: {
        FlEl e;
        memset(&e, 0, sizeof e);
        fl_play("lobby_motion.json", "coin_flip", &e);
        break;
    }
    case FLE_BADGE: {
        FlEl e;
        memset(&e, 0, sizeof e);
        sprintf(e.port, "p%d", fl.pend_arg + 1);
        fl_play("lobby_motion.json", "badge_set", &e);
        break;
    }
    case FLE_SCORE: {
        FlEl e;
        memset(&e, 0, sizeof e);
        fl_play("lobby_motion.json", "score_update", &e);
        break;
    }
    default: break;
    }
    fl.pend_event = FLE_NONE;
}

/* ---- per-frame logic ---------------------------------------------------------------------- */

static void fl_open(int art)
{
    if (art == FL_CSS) {
        fs_css_open(&gmVsMelee_CssData);
    } else if (art == FL_SSS) {
        fs_sss_open(&gmVsMelee_SssData);
    }
    if (art == FL_LOAD) {
        fe_load_begin();
        fl_load_init();
    }
    fl.art = art;
    fl.frames = 0;
    fl.sig[0] = '\0';
    fl.toast[0] = '\0';
    fl.toast_until = 0;
    fl.leave_until = 0;
    fl.pend_event = FLE_NONE;
    if (art == FL_WAIT) {
        fl.copied_until = Netplay_IsHost() ? 40 : 0;
        fl.found_at = -1;
    }
    if (art == FL_LOBBY) {
        int i;
        fl.last_phase = -1;
        fl.last_seq = -1;
        fl.last_count = 0;
        fl.coin_until = 0;
        fl.ready_on_until = 0;
        fl.last_game = Netplay_LobbyInfo(0);
        for (i = 0; i < LB_MAX_UI_STAGES; i++) fl.stage_prev[i] = Netplay_LobbyStage(i);
        fl.locked_prev[0] = fl.locked_prev[1] = 0;
        fl.ready_prev[0] = fl.ready_prev[1] = 0;
        fl.rejoin_at = -1;
        if (fl_pending_note[0] != 0) {
            fl_toast(fl_pending_note, true, 240);
            fl_pending_note[0] = 0;
        }
    }
}

/* Join a Room: a fresh code entry (a code typed before stays, a failed join's marks do not). */
static void fl_code_reset(void)
{
    fl.invalid = false;
    fl.code_msg[0] = '\0';
    fl.code_msg_bad = false;
}

/* A message for the lobby's status line the next time it opens (back from the CSS). */
static void fl_lobby_notice(const char* s)
{
    fp_copy(fl_pending_note, s, FE_STR);
}

/* Back from a match: the lobby reconnects to the room. */
static void fl_rejoin_reset(void)
{
    fl.rejoin_at = -1;
}

static void fl_leave_room(const char* why, bool bad)
{
    Netplay_Leave();
    fe_np_phase = FE_NP_IDLE;
    fe_ol_notice(why, bad);
    fe_switch_screen(&fe_screen_online);
}

/* The first free stage from `from` in direction `dir` (1/-1), wrapping; `from` itself if free. */
static int fl_free_stage(int from, int dir)
{
    int n = Netplay_LobbyInfo(9), k;
    for (k = 0; k < n; k++) {
        int i = ((from + dir * k) % n + n) % n;
        if (Netplay_LobbyStage(i) == LBS_FREE) return i;
    }
    return from;
}

static void fl_frame_code(u32 in)
{
    int k = Netplay_CodeKeys(), before[5], i;
    bool edited = false;
    for (i = 0; i < 4; i++) before[i] = Netplay_CodeChar(i);
    before[4] = Netplay_CodeSlot();
    if (in & MenuInput_Left) {
        if (Netplay_CodeMove(-1)) sfxMove(); else fl.bump_until = fl.frames + 8;
    } else if (in & MenuInput_Right) {
        if (Netplay_CodeMove(+1)) sfxMove(); else fl.bump_until = fl.frames + 8;
    } else if (in & MenuInput_Up) {
        Netplay_CodeStep(+1);
        sfxMove();
    } else if (in & MenuInput_Down) {
        Netplay_CodeStep(-1);
        sfxMove();
    } else if (in & MenuInput_YButton) {
        if (Netplay_CodePaste()) {
            sfxForward();
            fl_toast("Room code pasted - press A to join.", false, 120);
        } else {
            fl_toast("The clipboard doesn't hold a room code.", true, 150);
        }
    }
    if (k == 4) {
        fl.bump_until = fl.frames + 8;
    }
    for (i = 0; i < 4; i++) edited |= before[i] != Netplay_CodeChar(i);
    if (edited && fl.invalid) {
        fl.invalid = false; /* the first edit clears a failed join */
        fl.code_msg[0] = '\0';
        fl.code_msg_bad = false;
    }
    if ((in & MenuInput_Confirm) || k == 2) {
        if (!Netplay_CodeComplete()) {
            Netplay_CodeFirstEmpty();
            fl.bump_until = fl.frames + 8;
            fl_toast("Fill in all 4 characters first.", true, 120);
        } else {
            sfxForward();
            if (Netplay_MenuBegin(0, fe_np_ck, fe_np_color, fe_np_stocks, fe_np_minutes,
                                  fe_np_delay) == 0)
            {
                fe_np_phase = FE_NP_WORKING;
                fe_switch_screen(&fe_screen_wait);
            } else {
                Netplay_MenuStatus(fl.code_msg, FE_STR);
                fl.code_msg_bad = true;
            }
        }
    } else if ((in & MenuInput_Back) || k == 3) {
        sfxBack();
        fe_switch_screen(&fe_screen_online);
    }
}

static void fl_frame_wait(u32 in)
{
    bool host = Netplay_IsHost() != 0;
    if (fe_np_phase == FE_NP_WORKING || fe_np_phase == FE_NP_LOBBY) {
        fe_np_phase = Netplay_MenuPoll();
    }
    if (fe_np_phase == FE_NP_LOBBY) {
        if (fl.found_at < 0) {
            fl.found_at = fl.frames;
            sfxForward();
        } else if (fl.frames - fl.found_at >= 45) {
            fe_switch_screen(&fe_screen_lobby);
            return;
        }
    }
    if (Netplay_RandomStatus() >= 3 || (fe_np_phase == FE_NP_FAILED && Netplay_RandomStatus() != 0 &&
                                          Netplay_RandomStatus() != 2)) {
        /* nobody was looking (3), or the search failed (4) */
        char why[FE_STR];
        if (Netplay_RandomStatus() == 3) sprintf(why, "%s", "Nobody else is looking right now - try again soon.");
        else Netplay_MenuStatus(why, FE_STR);
        Netplay_RandomCancel();
        Netplay_Leave();
        fe_np_phase = FE_NP_IDLE;
        fe_ol_notice(why, true);
        fe_switch_screen(&fe_screen_online);
        return;
    }
    if (fe_np_phase == FE_NP_FAILED && !host) {
        /* a bad code or a dead room: back to the code, marked */
        Netplay_MenuStatus(fl.code_msg, FE_STR);
        fl.code_msg_bad = true;
        fl.invalid = true;
        fe_np_phase = FE_NP_IDLE;
        fe_switch_screen(&fe_screen_code);
        return;
    }
    if ((in & MenuInput_XButton) && host) {
        Netplay_CopyCode();
        fl.copied_until = fl.frames + 40;
        sfxForward();
    }
    if ((in & MenuInput_Back) && Netplay_RandomStatus() == 1) {
        sfxBack();
        Netplay_RandomCancel();
        fe_np_phase = FE_NP_IDLE;
        fe_ol_notice("You stopped looking for an opponent.", false);
        fe_switch_screen(&fe_screen_online);
        return;
    }
    if (in & MenuInput_Back) {
        sfxBack();
        Netplay_Leave();
        fe_np_phase = FE_NP_IDLE;
        if (host) {
            fe_ol_notice("You closed the room.", false);
            fe_switch_screen(&fe_screen_online);
        } else {
            fe_switch_screen(&fe_screen_code);
        }
    }
}

static void fl_lobby_events(void)
{
    int p = Netplay_LobbyPhase(), i, n = Netplay_LobbyInfo(9), cd = Netplay_LobbyInfo(8);
    if (fl_reconnecting()) {
        return;
    }
    if (p == LBP_CHAR_BLIND) {
        fl_coin_done = false;
    }
    if (p != fl.last_phase) {
        bool none_struck = true;
        for (i = 0; i < n; i++) {
            none_struck = none_struck && Netplay_LobbyStage(i) == LBS_FREE;
        }
        if ((p == LBP_STRIKE || (p == LBP_BAN && Netplay_LobbyInfo(11) == 1)) && Netplay_LobbyInfo(0) <= 1 &&
            none_struck && !fl_coin_done) {
            /* the coin flip, then the striking - also for the player who locked last and only
               sees the lobby again once the striking has begun */
            fl.coin_until = fl.frames + 150;
            fl.pend_event = FLE_COIN;
            fl_coin_done = true;
            OSReport("frontend: lobby - the coin flip (P%d strikes first)%c", Netplay_LobbyInfo(6) + 1, 10);
        }
        if (p == LBP_STRIKE || p == LBP_BAN || p == LBP_PICK) {
            fl.cur = fl_free_stage(fl.cur, 1);
        }
        fl.last_phase = p;
    }
    for (i = 0; i < n && i < LB_MAX_UI_STAGES; i++) {
        int s = Netplay_LobbyStage(i);
        if (s != fl.stage_prev[i]) {
            if (s == LBS_P1 || s == LBS_P2) { fl.pend_event = FLE_STRIKE; fl.pend_arg = i; }
            if (s == LBS_BANNED) { fl.pend_event = FLE_BAN; fl.pend_arg = i; }
            if (s == LBS_PICKED) { fl.pend_event = FLE_PICK; fl.pend_arg = i; }
            fl.stage_prev[i] = s;
            if (Netplay_LobbyStage(fl.cur) != LBS_FREE) fl.cur = fl_free_stage(fl.cur, 1);
        }
    }
    for (i = 0; i < 2; i++) {
        int lk = Netplay_LobbyPlayer(i, 2), rd = Netplay_LobbyPlayer(i, 3);
        if ((lk && !fl.locked_prev[i]) || (rd && !fl.ready_prev[i])) {
            if (fl.pend_event == FLE_NONE) { fl.pend_event = FLE_BADGE; fl.pend_arg = i; }
        }
        fl.locked_prev[i] = lk;
        fl.ready_prev[i] = rd;
    }
    if (cd != fl.last_count) {
        if (cd > 0) fl.pend_event = FLE_TICK;
        fl.last_count = cd;
    }
    if (Netplay_LobbyInfo(0) != fl.last_game) {
        fl.pend_event = FLE_SCORE;
        fl.last_game = Netplay_LobbyInfo(0);
    }
}

static void fl_frame_lobby(u32 in)
{
    int p, me = Netplay_LobbyMe();
    /* back from a match: reconnect to the room (the host at once, the guest a moment later) */
    if (fe_np_phase == FE_NP_IDLE && Netplay_RematchPending() && fl.rejoin_at < 0) {
        fl.rejoin_at = fl.frames + (Netplay_IsHost() ? 10 : 150);
    }
    if (fl.rejoin_at >= 0 && fl.frames >= fl.rejoin_at) {
        fl.rejoin_at = -1;
        fe_np_phase = Netplay_Rejoin() == 0 ? FE_NP_WORKING : FE_NP_FAILED;
    }
    if (fe_np_phase == FE_NP_WORKING || fe_np_phase == FE_NP_LOBBY) {
        fe_np_phase = Netplay_MenuPoll();
    }
    if (fe_np_phase == FE_NP_FAILED) {
        char why[FE_STR];
        Netplay_MenuStatus(why, FE_STR);
        if (Netplay_PeerLeft() && Netplay_IsHost()) {
            /* the host keeps its room: back to waiting for the next opponent, same code */
            fe_np_phase = Netplay_Rejoin() == 0 ? FE_NP_WORKING : FE_NP_FAILED;
            fe_switch_screen(&fe_screen_wait);
            fl_toast("Your opponent left. Waiting for a new one.", true, 300);
            return;
        }
        fl_leave_room(Netplay_PeerLeft() ? "The host closed the room." : why, true);
        return;
    }
    if (fe_np_phase == FE_NP_CONNECTED) {
        /* the match is agreed: the loading screen warms the renderer, then the match */
        fe_switch_screen(&fe_screen_online_load);
        return;
    }
    fl_lobby_events();
    p = fl_phase();
    if (in & MenuInput_Back) {
        if (p == LBP_READY && Netplay_LobbyInfo(8) > 0 && Netplay_LobbyPlayer(me, 3)) {
            sfxBack();
            Netplay_LobbyReady(0); /* B during the count takes the ready back */
        } else if (fl.frames < fl.leave_until) {
            sfxBack();
            fl_leave_room("You left the room.", false);
            return;
        } else {
            sfxBack();
            fl.leave_until = fl.frames + 150;
            fl_toast("Press B again to leave the room.", true, 150);
        }
        return;
    }
    if (fl_reconnecting()) {
        return;
    }
    if (p == LBP_READY && (in & (MenuInput_StartButton | MenuInput_AButton))) {
        /* START toggles; A only readies (so a hurried A never takes it back) */
        bool on = !Netplay_LobbyPlayer(me, 3);
        if (!on && !(in & MenuInput_StartButton)) {
            return;
        }
        sfxForward();
        Netplay_LobbyReady(on);
        if (on) {
            fl.ready_on_until = fl.frames + 30;
            fl.pend_event = FLE_READY;
        }
        return;
    }
    if (fl_can_pick_char() && (in & MenuInput_Confirm)) {
        sfxForward();
        fe_np_pick_char(); /* the real CSS; the pick comes back through Frontend_OnlinePicked */
        return;
    }
    if (fl_my_stage_turn() && Netplay_LobbyInfo(9) > 6) {
        int n = Netplay_LobbyInfo(9), per = FL_DENSE_COLS * FL_DENSE_ROWS, c = fl.cur;
        if (in & MenuInput_LTrigger) c = c - per < 0 ? c : c - per;
        else if (in & MenuInput_RTrigger) c = c + per >= n ? n - 1 : c + per;
        else if (in & MenuInput_Left) c = c > 0 ? c - 1 : n - 1;
        else if (in & MenuInput_Right) c = c < n - 1 ? c + 1 : 0;
        else if (in & MenuInput_Up) c = c - FL_DENSE_COLS >= 0 ? c - FL_DENSE_COLS : c;
        else if (in & MenuInput_Down) c = c + FL_DENSE_COLS < n ? c + FL_DENSE_COLS : c;
        if (c != fl.cur) {
            fl.cur = c;
            sfxMove();
        }
        if (in & MenuInput_Confirm) {
            if (Netplay_LobbyStage(fl.cur) == LBS_FREE) {
                sfxForward();
                Netplay_LobbyStageAct(fl.cur);
            } else {
                sfxBack();
                fl_toast("That stage is already out - pick another.", true, 90);
            }
        }
    } else if (fl_my_stage_turn()) {
        int n = Netplay_LobbyInfo(9), cols = 3, c = fl.cur;
        if (in & MenuInput_Left) c = c % cols == 0 ? c + cols - 1 : c - 1;
        else if (in & MenuInput_Right) c = c % cols == cols - 1 ? c - cols + 1 : c + 1;
        else if (in & MenuInput_Up) c -= cols;
        else if (in & MenuInput_Down) c += cols;
        if (c != fl.cur) {
            if (c < 0) c += ((n + cols - 1) / cols) * cols;
            if (c >= n) c = c % cols < n ? c % cols : n - 1;
            if (c >= 0 && c < n) {
                fl.cur = c;
                sfxMove();
            }
        }
        if (in & MenuInput_Confirm) {
            if (Netplay_LobbyStage(fl.cur) == LBS_FREE) {
                sfxForward();
                Netplay_LobbyStageAct(fl.cur);
            } else {
                sfxBack();
                fl_toast("That stage is already out - pick another.", true, 90);
            }
        }
    } else if (in & MenuInput_Confirm) {
        int t = Netplay_LobbyInfo(4);
        if (fl_turn_phase(p) && t != me) {
            fl_toast("It's your opponent's turn.", false, 90);
        }
    }
}

/* The online loading screen: the warm-up, then the match (the transport holds its start until
 * both machines have loaded). VS mode's loading state runs fe_loading_frame instead. */
static void fl_frame_load(void)
{
    fl_ld.ready = fe.progress >= 1.0F;
    if (fe.screen != &fe_screen_online_load) {
        return;
    }
    fe_np_phase = Netplay_MenuPoll();
    if (fe_np_phase == FE_NP_FAILED) {
        char why[FE_STR];
        Netplay_MenuStatus(why, FE_STR);
        fl_leave_room(Netplay_PeerLeft() ? "Your opponent left before the match." : why, true);
        return;
    }
    if (fe_load_step()) {
        Netplay_MenuLaunch(); /* the agreed match is the configured scene */
        fe.warmed = true;     /* the match's own hold only covers its first frame */
        fe.continue_to = GM_VS;
        fe.leaving = 1;
        fe_np_phase = FE_NP_IDLE;
    }
}

static void fl_frame(void)
{
    u32 in = 0;
    fl.frames++;
    if (fe.frames >= 4) { /* let the button that brought us here go */
        in = mn_80229624(4);
    }
    switch (fl.art) {
    case FL_CODE: fl_frame_code(in); break;
    case FL_WAIT: fl_frame_wait(in); break;
    case FL_LOBBY: fl_frame_lobby(in); break;
    case FL_LOAD: fl_frame_load(); break;
    case FL_CSS: fs_css_frame(); break;
    case FL_SSS: fs_sss_frame(); break;
    default: break;
    }
    if (fe.screen == NULL || fe.screen->art != fl.art || fe.leaving != 0) {
        return; /* switched away (fe_switch_screen rebuilt) or leaving */
    }
    {
        char sig[sizeof fl.sig];
        fl_signature(sig);
        if (strcmp(sig, fl.sig) != 0) {
            memcpy(fl.sig, sig, sizeof sig);
            fl_build();
        }
    }
    if (fl.art == FL_LOAD) {
        fl_load_update();
    }
    fp.frame++;
    fp_evaluate(fp.frame);
}

/* A screen that is leaving keeps its motion running. */
static void fl_frame_anim(void)
{
    if (!fl.on) {
        return;
    }
    if (fl.art == FL_LOAD) {
        fl_load_update();
    }
    fp.frame++;
    fp_evaluate(fp.frame);
}

static void fl_exit(void)
{
    fa_close(); /* the select screens' archives */
    if (fl.on) {
        fp_free_textures();
        fp.ok = false;
        fl.on = false;
    }
}
