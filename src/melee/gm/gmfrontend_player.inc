/* gmfrontend_player.inc - included by gmfrontend.c (TARGET_PC only). Not a TU of its own: build
 * with `tools/port/build.sh --tu src/melee/gm/gmfrontend.c`, which build.sh also does by itself
 * when this file is newer than gmfrontend.c's object.
 *
 * ---- THE LAYOUT + MOTION PLAYER ------------------------------------------------------------
 *
 * Plays the art pipeline's screen format (C:/Users/Gurek/Desktop/menu, pipeline/hub.py and
 * hub_motion.py; the contract is out_hub/hub_layout.json + hub_motion.json):
 *
 *   LAYOUT  quads with explicit vertices in 640x480 (y down): flat (colour) or textured (a white
 *           mask + normalised UV rect, tinted by its material colour). Top-level "quads" are the
 *           chrome; "tiles" carry plate/face/icon/label quads plus a footer "desc" quad; "states"
 *           give the ng/sel colours, the selection offset and the plate colour.
 *   MOTION  a joint tree in parent-before-child order (parent, pivot, channels, drawn quad ids),
 *           a material map (target -> quad), events (keyframed tracks on templated targets
 *           "lift_<id>"), and sequences (cursor move, confirm, screen out/in with staggers).
 *
 * WHY RUNTIME JSON, NOT A CONVERTER. The files are small (57 KB for the hub) and the art agent
 * iterates on them; parsing them here means a new delivery is `png2gx.py --layout` and a
 * restart, with no second format to keep in step. The parser is 150 lines, runs once per scene,
 * and its DOM is freed as soon as the model is built. It lives in this game TU rather than in a
 * native shim because everything it produces is floats read by game code: gwtool byte-swaps
 * game memory, so the host only hands over the file's BYTES (gw_UiFile_Read), which read the
 * same on both sides.
 *
 * THE MATHS, exactly as hub_motion.json states it:
 *   - keys (frame, value, interp, slope); interp applies to the segment starting at the key;
 *     before the first key its value, after the last the last value. CON holds, LIN lerps, SPL
 *     is cubic Hermite with u = (t - f0)/(f1 - f0), d = f1 - f0,
 *     v = h00 v0 + h10 d s0 + h01 v1 + h11 d s1 (slopes in value/frame).
 *   - from_current: at the event's start, the key's value is replaced by the channel's current
 *     value. A channel plays the most recently STARTED event covering it (insertion order among
 *     those whose start frame has been reached, as the reference sequencer does - that is what
 *     makes a scheduled "hold off-screen, then slide in" work).
 *   - a joint maps p -> pivot + S (p - pivot) + T; world = parent after child. RECOIL is a
 *     scalar r adding r * px * unit(pivot(tile) - pivot(selected)) to a tile's T.
 *   - draw order: top-level quads in file order; tiles by rank, then the previous selection,
 *     then the selection; each tile plate (only while visible), face, icon, label; then every
 *     tile's description. Opacity = material ALPHA x joint ALPHA down the chain.
 *
 * Textured quads are drawn with one TEV stage taking RGB from the vertex (material) colour and
 * only ALPHA from the texture, which is the setup the README asks for I4 masks (an RGBA texture
 * is modulated normally). Quads are parallelograms, so two triangles map the texture exactly.
 *
 * TEXT QUADS are this engine's addition for screens it generates (gmfrontend_menus.inc): a quad
 * whose content is a string instead of a texture. The player moves, scales and recolours it like
 * any quad and hands the result to a HSD_SisLib text each frame. */

/* ---- a small JSON DOM -------------------------------------------------------------------- */

enum { FJ_NULL, FJ_FALSE, FJ_TRUE, FJ_NUM, FJ_STR, FJ_ARR, FJ_OBJ };

typedef struct FjNode {
    u8 type;
    const char* key; ///< member name when the parent is an object
    const char* s;   ///< FJ_STR
    float n;         ///< FJ_NUM
    int child;       ///< first element/member, -1 none
    int next;        ///< next sibling, -1 none
} FjNode;

static struct {
    char* p;
    char* end;
    FjNode* pool;
    int n, cap;
    bool bad;
} fj;

static void fj_ws(void)
{
    while (fj.p < fj.end && (*fj.p == ' ' || *fj.p == '\t' || *fj.p == '\n' || *fj.p == '\r')) {
        fj.p++;
    }
}

static int fj_new(u8 type)
{
    FjNode* nd;
    if (fj.n >= fj.cap) {
        fj.bad = true;
        return -1;
    }
    nd = &fj.pool[fj.n];
    nd->type = type;
    nd->key = NULL;
    nd->s = NULL;
    nd->n = 0.0F;
    nd->child = -1;
    nd->next = -1;
    return fj.n++;
}

/* A string, unescaped in place and NUL-terminated there (the closing quote is overwritten). */
static const char* fj_string(void)
{
    char* out;
    char* start;
    if (*fj.p != '"') {
        fj.bad = true;
        return "";
    }
    start = out = ++fj.p;
    while (fj.p < fj.end && *fj.p != '"') {
        char c = *fj.p++;
        if (c == '\\' && fj.p < fj.end) {
            c = *fj.p++;
            switch (c) {
            case 'n': c = '\n'; break;
            case 't': c = '\t'; break;
            case 'r': c = '\r'; break;
            case 'b': c = '\b'; break;
            case 'f': c = '\f'; break;
            case 'u': {
                int v = 0, i;
                for (i = 0; i < 4 && fj.p < fj.end; i++, fj.p++) {
                    char h = *fj.p;
                    v = v * 16 + (h >= 'a' ? h - 'a' + 10 : h >= 'A' ? h - 'A' + 10 : h - '0');
                }
                c = v < 128 ? (char) v : '?'; /* the SIS font is ASCII */
                break;
            }
            default: break; /* \" \\ \/ */
            }
        }
        *out++ = c;
    }
    if (fj.p < fj.end) {
        fj.p++;
    }
    *out = '\0';
    return start;
}

static float fj_number(void)
{
    double v = 0.0, frac = 0.1;
    int neg = 0, eneg = 0, e = 0;
    if (*fj.p == '-') {
        neg = 1;
        fj.p++;
    }
    while (fj.p < fj.end && *fj.p >= '0' && *fj.p <= '9') {
        v = v * 10.0 + (*fj.p++ - '0');
    }
    if (fj.p < fj.end && *fj.p == '.') {
        fj.p++;
        while (fj.p < fj.end && *fj.p >= '0' && *fj.p <= '9') {
            v += (*fj.p++ - '0') * frac;
            frac *= 0.1;
        }
    }
    if (fj.p < fj.end && (*fj.p == 'e' || *fj.p == 'E')) {
        fj.p++;
        if (*fj.p == '-' || *fj.p == '+') {
            eneg = *fj.p++ == '-';
        }
        while (fj.p < fj.end && *fj.p >= '0' && *fj.p <= '9') {
            e = e * 10 + (*fj.p++ - '0');
        }
        while (e-- > 0) {
            v = eneg ? v * 0.1 : v * 10.0;
        }
    }
    return (float) (neg ? -v : v);
}

static int fj_value(int depth)
{
    int id = -1;
    fj_ws();
    if (fj.p >= fj.end || depth > 32) {
        fj.bad = true;
        return -1;
    }
    switch (*fj.p) {
    case '{':
    case '[': {
        char close = *fj.p == '{' ? '}' : ']';
        int last = -1;
        id = fj_new(close == '}' ? FJ_OBJ : FJ_ARR);
        if (id < 0) {
            return -1;
        }
        fj.p++;
        fj_ws();
        while (fj.p < fj.end && *fj.p != close && !fj.bad) {
            const char* key = NULL;
            int c;
            if (close == '}') {
                key = fj_string();
                fj_ws();
                if (*fj.p == ':') {
                    fj.p++;
                }
            }
            c = fj_value(depth + 1);
            if (c < 0) {
                return -1;
            }
            fj.pool[c].key = key;
            if (last < 0) {
                fj.pool[id].child = c;
            } else {
                fj.pool[last].next = c;
            }
            last = c;
            fj_ws();
            if (*fj.p == ',') {
                fj.p++;
                fj_ws();
            }
        }
        if (fj.p < fj.end) {
            fj.p++;
        }
        return id;
    }
    case '"':
        id = fj_new(FJ_STR);
        if (id >= 0) {
            fj.pool[id].s = fj_string();
        }
        return id;
    case 't':
        fj.p += 4;
        return fj_new(FJ_TRUE);
    case 'f':
        fj.p += 5;
        return fj_new(FJ_FALSE);
    case 'n':
        fj.p += 4;
        return fj_new(FJ_NULL);
    default:
        id = fj_new(FJ_NUM);
        if (id >= 0) {
            fj.pool[id].n = fj_number();
        }
        return id;
    }
}

static const FjNode* fj_at(int id)
{
    return id >= 0 ? &fj.pool[id] : NULL;
}

static const FjNode* fj_get(const FjNode* o, const char* key)
{
    int c;
    if (o == NULL || o->type != FJ_OBJ) {
        return NULL;
    }
    for (c = o->child; c >= 0; c = fj.pool[c].next) {
        if (strcmp(fj.pool[c].key, key) == 0) {
            return &fj.pool[c];
        }
    }
    return NULL;
}

static const FjNode* fj_first(const FjNode* a)
{
    return (a != NULL && (a->type == FJ_ARR || a->type == FJ_OBJ)) ? fj_at(a->child) : NULL;
}

static const FjNode* fj_next(const FjNode* n)
{
    return n != NULL ? fj_at(n->next) : NULL;
}

static const FjNode* fj_idx(const FjNode* a, int i)
{
    const FjNode* n = fj_first(a);
    while (n != NULL && i-- > 0) {
        n = fj_next(n);
    }
    return n;
}

static float fj_num(const FjNode* n, float def)
{
    return (n != NULL && n->type == FJ_NUM) ? n->n : def;
}

static const char* fj_str(const FjNode* n)
{
    return (n != NULL && n->type == FJ_STR) ? n->s : NULL;
}

/* Reads a UI file (gw_UiFile_Read) into the scene heap and parses it. The text buffer and the
 * node pool are returned through `mem` so the caller frees them once the model is built. */
static const FjNode* fj_load(const char* name, void** mem_text, void** mem_pool)
{
    extern int UiFile_Read(const char* name, void* dst, int cap);
    int size = UiFile_Read(name, NULL, 0);
    char* text;
    int root;
    *mem_text = *mem_pool = NULL;
    if (size <= 0) {
        OSReport("frontend: %s not found beside the exe (ui/) - run pc/tools/png2gx.py "
                 "--layout\n",
                 name);
        return NULL;
    }
    text = HSD_MemAlloc(size + 1);
    fj.cap = size / 5 + 64; /* every value costs at least ~5 bytes of JSON */
    fj.pool = HSD_MemAlloc(fj.cap * (int) sizeof(FjNode));
    if (text == NULL || fj.pool == NULL) {
        OSReport("frontend: no heap for %s (%d bytes)\n", name, size);
        return NULL;
    }
    UiFile_Read(name, text, size);
    text[size] = '\0';
    *mem_text = text;
    *mem_pool = fj.pool;
    fj.p = text;
    fj.end = text + size;
    fj.n = 0;
    fj.bad = false;
    root = fj_value(0);
    if (fj.bad || root < 0) {
        OSReport("frontend: %s: JSON parse failed near byte %d\n", name, (int) (fj.p - text));
        return NULL;
    }
    OSReport("frontend: %s: %d bytes, %d nodes\n", name, size, fj.n);
    return fj_at(root);
}

/* ---- the model --------------------------------------------------------------------------- */

#define FP_MAX_QUADS 112
#define FP_MAX_JOINTS 64
#define FP_MAX_TILES 12
#define FP_MAX_TEX 24
#define FP_MAX_EVENTS 12
#define FP_MAX_TRACKS 16
#define FP_MAX_KEYS 6
#define FP_PLAYS 4
#define FP_ID 28

enum { FPK_FLAT, FPK_TEX, FPK_TEXT };
enum {
    FPR_DECOR, FPR_CHROME, FPR_PLATE, FPR_FACE, FPR_ICON, FPR_LABEL, FPR_DESC, FPR_TITLE,
    FPR_OTHER
};
enum { FPI_CON, FPI_LIN, FPI_SPL };
/* joint channels, then material channels */
enum { FJC_TRA_X, FJC_TRA_Y, FJC_SCA_X, FJC_SCA_Y, FJC_ALPHA, FJC_RECOIL, FJC_N };
enum { FMC_R, FMC_G, FMC_B, FMC_ALPHA, FMC_N };

typedef struct FpKey {
    s16 frame;
    u8 interp;
    u8 from_current;
    float v, slope;
} FpKey;

typedef struct FpPlay {
    int start;
    u8 n;
    FpKey k[FP_MAX_KEYS];
} FpPlay;

typedef struct FpChan {
    float rest;
    u8 n; ///< plays, in insertion order
    FpPlay play[FP_PLAYS];
} FpChan;

typedef struct FpQuad {
    char id[FP_ID];
    u8 kind, role;
    s8 tile;  ///< owning tile, -1 for chrome
    s8 tex;   ///< FpTex index, -1 none/missing
    s8 joint; ///< the joint that draws it, -1 none
    bool has_colour;
    float v[4][2]; ///< TL TR BR BL at rest
    float uv[4];   ///< u0 v0 u1 v1
    float rgb[3];
    /* FPK_TEXT */
    const char* text;
    float text_h;  ///< cap height in 640x480 units
    u8 align;      ///< 0 left, 1 centre
    HSD_Text* sis;
    int sis_entry;
    GXColor sis_col;
    /* evaluated each frame */
    float w[4][2];
    float wrgb[3];
    float wa;
    float wscale;
} FpQuad;

typedef struct FpTile {
    char id[FP_ID];
    int rank;
    int q_plate, q_face, q_icon, q_label, q_desc; ///< quad indices, -1 none
    int j_tile, j_lift, j_plate, j_desc;          ///< joint indices, -1 none
} FpTile;

typedef struct FpJoint {
    char name[FP_ID];
    s8 parent;
    bool has_pivot;
    float pivot[2];
    float recoil_dir[2];
    /* world transform, p -> s * p + o (per axis) */
    float ws[2], wo[2], wa;
} FpJoint;

typedef struct FpTrack {
    char target[FP_ID];
    u8 is_material;
    u8 channel;
    u8 recoil;
    u8 n;
    FpKey k[FP_MAX_KEYS];
} FpTrack;

typedef struct FpEvent {
    char name[16];
    int length;
    int n;
    FpTrack t[FP_MAX_TRACKS];
} FpEvent;

typedef struct FpTex {
    char name[FP_ID];
    FeTex t;
    bool mask; ///< I4/IA4: RGB from the material, alpha from the texture
} FpTex;

typedef struct FpSeqBar {
    bool ok;
    u8 n;
    FpKey k[FP_MAX_KEYS];
} FpSeqBar;

typedef struct FpSeqScreen {
    char cluster_event[16];
    int cluster_at;
    FpSeqBar header, footer;
    char tile_event[16];
    bool selected_last; ///< "unselected by rank, then selected"
    int stagger;
    int offsets[8];
    int n_offsets;
    bool hold;
    float hold_value;
} FpSeqScreen;

typedef struct FpSeqStep {
    char event[16];
    u8 on; ///< 0 previous, 1 new/selected, 2 others
    int at;
} FpSeqStep;

static struct {
    bool ok;        ///< a layout is built
    bool motion_ok; ///< and the motion file was read
    float shear_s, shear_y0;
    float clear[3];
    float ng_face[3], ng_icon[3], ng_label[3];
    float sel_face[3], sel_icon[3], sel_label[3], sel_plate[3];
    float sel_offset[2];
    FpQuad q[FP_MAX_QUADS];
    int nq;
    FpTile tile[FP_MAX_TILES];
    int nt;
    FpJoint j[FP_MAX_JOINTS];
    int nj;
    FpTex tex[FP_MAX_TEX];
    int ntex;
    /* materials: target name pattern -> quad id pattern, with "<id>" where the tile id goes */
    char mat_key[8][FP_ID];
    char mat_val[8][FP_ID];
    int nmat;
    FpEvent ev[FP_MAX_EVENTS];
    int nev;
    float recoil_px;
    FpSeqStep cursor_move[8];
    int n_cursor_move;
    char confirm_event[16];
    int screen_out_at; ///< confirm -> screen_out delay
    FpSeqScreen s_out, s_in;
    FpChan ch[FP_MAX_JOINTS * FJC_N + FP_MAX_QUADS * FMC_N];
    int frame;
    int sel, prev; ///< tile indices, -1 none
} fp;

/* -- helpers -- */

static void fp_cat(char* dst, const char* src)
{
    int n = (int) strlen(dst);
    while (*src != ' ' && n < FP_ID - 1) {
        dst[n++] = *src++;
    }
    dst[n] = ' ';
}

static const char* fp_strstr(const char* s, const char* sub)
{
    int n = (int) strlen(sub);
    for (; s != NULL && *s != '\0'; s++) {
        if (strncmp(s, sub, n) == 0) {
            return s;
        }
    }
    return NULL;
}

static void fp_copy(char* dst, const char* src, int cap)
{
    int i = 0;
    if (src != NULL) {
        for (; src[i] != '\0' && i < cap - 1; i++) {
            dst[i] = src[i];
        }
    }
    dst[i] = '\0';
}

static int fp_hexv(char c)
{
    return c >= 'a' ? c - 'a' + 10 : c >= 'A' ? c - 'A' + 10 : c - '0';
}

static bool fp_hex(const char* s, float* rgb)
{
    int i;
    if (s == NULL || s[0] != '#') {
        return false;
    }
    for (i = 0; i < 3; i++) {
        rgb[i] = (float) (fp_hexv(s[1 + 2 * i]) * 16 + fp_hexv(s[2 + 2 * i])) / 255.0F;
    }
    return true;
}

static int fp_find_quad(const char* id)
{
    int i;
    for (i = 0; i < fp.nq; i++) {
        if (strcmp(fp.q[i].id, id) == 0) {
            return i;
        }
    }
    return -1;
}

static int fp_find_joint(const char* name)
{
    int i;
    for (i = 0; i < fp.nj; i++) {
        if (strcmp(fp.j[i].name, name) == 0) {
            return i;
        }
    }
    return -1;
}

static int fp_find_event(const char* name)
{
    int i;
    for (i = 0; i < fp.nev; i++) {
        if (strcmp(fp.ev[i].name, name) == 0) {
            return i;
        }
    }
    return -1;
}

/* "lift_<id>" with id "solo" -> "lift_solo" */
static void fp_subst(char* out, const char* pat, const char* id)
{
    int o = 0;
    while (*pat != '\0' && o < FP_ID - 1) {
        if (pat[0] == '<' && strncmp(pat, "<id>", 4) == 0) {
            const char* s = id;
            while (*s != '\0' && o < FP_ID - 1) {
                out[o++] = *s++;
            }
            pat += 4;
        } else {
            out[o++] = *pat++;
        }
    }
    out[o] = '\0';
}

static u8 fp_role(const char* r)
{
    static const char* const names[] = { "decor", "chrome", "plate", "face",
                                         "icon",  "label",  "desc",  "title" };
    int i;
    for (i = 0; r != NULL && i < (int) (sizeof names / sizeof names[0]); i++) {
        if (strcmp(r, names[i]) == 0) {
            return (u8) i;
        }
    }
    return FPR_OTHER;
}

static u8 fp_interp(const char* s)
{
    if (s != NULL && s[0] == 'C') {
        return FPI_CON;
    }
    if (s != NULL && s[0] == 'L') {
        return FPI_LIN;
    }
    return FPI_SPL;
}

static float fp_sqrt(float x)
{
    float r = x > 1.0F ? x : 1.0F;
    int i;
    if (x <= 0.0F) {
        return 0.0F;
    }
    for (i = 0; i < 24; i++) {
        r = 0.5F * (r + x / r);
    }
    return r;
}

/* -- textures (loaded by name through GxTex_OpenUI, into the scene heap) -- */

/* What each texture name is waiting on, so a missing one says where it should come from. */
static const char* fp_expected_from(const char* name)
{
    if (strncmp(name, "ico_", 4) == 0) {
        return "brief section 2 (hub item icon masks)";
    }
    if (strncmp(name, "lbl_", 4) == 0 || strncmp(name, "desc_", 5) == 0) {
        return "out_hub (preview-only baked text; menus draw strings)";
    }
    if (strncmp(name, "glyph_", 6) == 0) {
        return "brief section 1 (button glyphs)";
    }
    return "brief section 1/2";
}

static int fp_tex(const char* name)
{
    int i;
    extern int GxTex_OpenUI(const char* name);
    extern int GxTex_Format(int h);
    extern void GxTex_Close(int h);
    if (name == NULL || name[0] == '\0') {
        return -1;
    }
    for (i = 0; i < fp.ntex; i++) {
        if (strcmp(fp.tex[i].name, name) == 0) {
            return fp.tex[i].t.ok ? i : -1;
        }
    }
    if (fp.ntex >= FP_MAX_TEX) {
        return -1;
    }
    i = fp.ntex++;
    fp_copy(fp.tex[i].name, name, FP_ID);
    fe_tex_load(&fp.tex[i].t, name);
    fp.tex[i].mask = false;
    if (fp.tex[i].t.ok) {
        int h = GxTex_OpenUI(name);
        int f = h >= 0 ? GxTex_Format(h) : -1;
        if (h >= 0) {
            GxTex_Close(h);
        }
        fp.tex[i].mask = f == GX_TF_I4 || f == GX_TF_IA4 || f == GX_TF_I8 || f == GX_TF_IA8;
        return i;
    }
    OSReport("frontend: art missing: %s.gxtex - expected from %s; drawing without it\n", name,
             fp_expected_from(name));
    return -1;
}

static void fp_free_textures(void)
{
    int i;
    for (i = 0; i < fp.ntex; i++) {
        if (fp.tex[i].t.ok) {
            HSD_Free(fp.tex[i].t.data);
            if (fp.tex[i].t.lut != NULL) {
                HSD_Free(fp.tex[i].t.lut);
            }
        }
        fp.tex[i].t.ok = false;
    }
    fp.ntex = 0;
}

/* -- building the model -- */

static void fp_reset(void)
{
    fp.ok = fp.motion_ok = false;
    fp.nq = fp.nt = fp.nj = fp.nev = fp.nmat = 0;
    fp.n_cursor_move = 0;
    fp.frame = 0;
    fp.sel = fp.prev = -1;
    fp.recoil_px = 2.0F;
    fp.shear_s = 0.25F;
    fp.shear_y0 = 240.0F;
    fp.screen_out_at = 14;
    fp.confirm_event[0] = '\0';
    memset(&fp.s_out, 0, sizeof fp.s_out);
    memset(&fp.s_in, 0, sizeof fp.s_in);
}

static int fp_add_quad(const char* id, u8 kind, u8 role, int tile)
{
    FpQuad* q;
    if (fp.nq >= FP_MAX_QUADS) {
        return -1;
    }
    q = &fp.q[fp.nq];
    memset(q, 0, sizeof *q);
    fp_copy(q->id, id, FP_ID);
    q->kind = kind;
    q->role = role;
    q->tile = (s8) tile;
    q->tex = -1;
    q->joint = -1;
    q->uv[2] = q->uv[3] = 1.0F;
    return fp.nq++;
}

static int fp_quad_from_json(const FjNode* n, int tile)
{
    const char* kind = fj_str(fj_get(n, "kind"));
    int qi = fp_add_quad(fj_str(fj_get(n, "id")),
                         (kind != NULL && strcmp(kind, "textured") == 0) ? FPK_TEX : FPK_FLAT,
                         fp_role(fj_str(fj_get(n, "role"))), tile);
    const FjNode* v;
    int i;
    FpQuad* q;
    if (qi < 0) {
        return -1;
    }
    q = &fp.q[qi];
    for (i = 0, v = fj_first(fj_get(n, "verts")); v != NULL && i < 4; v = fj_next(v), i++) {
        q->v[i][0] = fj_num(fj_idx(v, 0), 0);
        q->v[i][1] = fj_num(fj_idx(v, 1), 0);
    }
    for (i = 0, v = fj_first(fj_get(n, "uv")); v != NULL && i < 4; v = fj_next(v), i++) {
        q->uv[i] = fj_num(v, 0);
    }
    q->has_colour = fp_hex(fj_str(fj_get(n, "colour")), q->rgb);
    if (q->kind == FPK_TEX) {
        q->tex = (s8) fp_tex(fj_str(fj_get(n, "texture")));
    }
    return qi;
}

static void fp_state_colours(const FjNode* st, float* face, float* icon, float* label)
{
    fp_hex(fj_str(fj_get(st, "face")), face);
    fp_hex(fj_str(fj_get(st, "icon")), icon);
    fp_hex(fj_str(fj_get(st, "label")), label);
}

/* The palette and state rules every screen shares, from a *_layout.json. */
static void fp_read_states(const FjNode* L)
{
    const FjNode* st = fj_get(L, "states");
    const FjNode* sel = fj_get(st, "sel");
    const FjNode* sh = fj_get(L, "shear");
    fp_hex(fj_str(fj_get(L, "clear_colour")), fp.clear);
    fp_state_colours(fj_get(st, "ng"), fp.ng_face, fp.ng_icon, fp.ng_label);
    fp_state_colours(sel, fp.sel_face, fp.sel_icon, fp.sel_label);
    fp_hex(fj_str(fj_get(sel, "plate")), fp.sel_plate);
    fp.sel_offset[0] = fj_num(fj_idx(fj_get(sel, "offset"), 0), -5.0F);
    fp.sel_offset[1] = fj_num(fj_idx(fj_get(sel, "offset"), 1), -5.0F);
    fp.shear_s = fj_num(fj_get(sh, "S"), 0.25F);
    fp.shear_y0 = fj_num(fj_get(sh, "Y0"), 240.0F);
}

/* The whole screen from a *_layout.json (the hub demo): chrome quads, then the tiles. */
static void fp_build_from_layout(const FjNode* L)
{
    const FjNode* n;
    fp_read_states(L);
    for (n = fj_first(fj_get(L, "quads")); n != NULL; n = fj_next(n)) {
        fp_quad_from_json(n, -1);
    }
    for (n = fj_first(fj_get(L, "tiles")); n != NULL && fp.nt < FP_MAX_TILES; n = fj_next(n)) {
        FpTile* t = &fp.tile[fp.nt];
        const FjNode* qn;
        fp_copy(t->id, fj_str(fj_get(n, "id")), FP_ID);
        t->rank = (int) fj_num(fj_get(n, "rank"), (float) fp.nt);
        t->q_plate = t->q_face = t->q_icon = t->q_label = t->q_desc = -1;
        for (qn = fj_first(fj_get(n, "quads")); qn != NULL; qn = fj_next(qn)) {
            int qi = fp_quad_from_json(qn, fp.nt);
            if (qi < 0) {
                continue;
            }
            switch (fp.q[qi].role) {
            case FPR_PLATE: t->q_plate = qi; break;
            case FPR_FACE: t->q_face = qi; break;
            case FPR_ICON: t->q_icon = qi; break;
            case FPR_LABEL: t->q_label = qi; break;
            default: break;
            }
        }
        if (fj_get(n, "desc") != NULL) {
            t->q_desc = fp_quad_from_json(fj_get(n, "desc"), fp.nt);
        }
        fp.nt++;
    }
}

static int fp_add_joint(const char* name, int parent, const float* pivot)
{
    FpJoint* j;
    if (fp.nj >= FP_MAX_JOINTS) {
        return -1;
    }
    j = &fp.j[fp.nj];
    memset(j, 0, sizeof *j);
    fp_copy(j->name, name, FP_ID);
    j->parent = (s8) parent;
    j->has_pivot = pivot != NULL;
    if (pivot != NULL) {
        j->pivot[0] = pivot[0];
        j->pivot[1] = pivot[1];
    }
    return fp.nj++;
}

static void fp_joint_draws(int j, const char* quad_id)
{
    int qi = fp_find_quad(quad_id);
    if (j >= 0 && qi >= 0) {
        fp.q[qi].joint = (s8) j;
    }
}

static void fp_keys_from_json(const FjNode* keys, FpKey* out, u8* n)
{
    const FjNode* k;
    *n = 0;
    for (k = fj_first(keys); k != NULL && *n < FP_MAX_KEYS; k = fj_next(k)) {
        FpKey* d = &out[(*n)++];
        if (k->type == FJ_ARR) { /* sequences write keys as [frame, value, interp, slope] */
            d->frame = (s16) fj_num(fj_idx(k, 0), 0);
            d->v = fj_num(fj_idx(k, 1), 0);
            d->interp = fp_interp(fj_str(fj_idx(k, 2)));
            d->slope = fj_num(fj_idx(k, 3), 0);
            d->from_current = 0;
        } else {
            const FjNode* fc = fj_get(k, "from_current");
            d->frame = (s16) fj_num(fj_get(k, "frame"), 0);
            d->v = fj_num(fj_get(k, "value"), 0);
            d->interp = fp_interp(fj_str(fj_get(k, "interp")));
            d->slope = fj_num(fj_get(k, "slope"), 0);
            d->from_current = fc != NULL && fc->type == FJ_TRUE;
        }
    }
}

static u8 fp_channel(const char* c, u8* is_recoil)
{
    *is_recoil = 0;
    if (c == NULL) {
        return 0;
    }
    if (strcmp(c, "TRA_X/TRA_Y") == 0 || strcmp(c, "RECOIL") == 0) {
        *is_recoil = 1;
        return FJC_RECOIL;
    }
    if (strcmp(c, "TRA_X") == 0) return FJC_TRA_X;
    if (strcmp(c, "TRA_Y") == 0) return FJC_TRA_Y;
    if (strcmp(c, "SCA_X") == 0) return FJC_SCA_X;
    if (strcmp(c, "SCA_Y") == 0) return FJC_SCA_Y;
    if (strcmp(c, "DIFFUSE_R") == 0) return FMC_R;
    if (strcmp(c, "DIFFUSE_G") == 0) return FMC_G;
    if (strcmp(c, "DIFFUSE_B") == 0) return FMC_B;
    return FJC_ALPHA; /* "ALPHA": FJC_ALPHA on a joint, FMC_ALPHA (3) is remapped below */
}

static void fp_seq_screen(const FjNode* s, FpSeqScreen* out)
{
    const FjNode* c = fj_get(s, "cluster");
    const FjNode* t = fj_get(s, "tiles");
    const FjNode* o;
    const char* order;
    const char* hold;
    FpSeqBar* bars[2];
    const char* names[2] = { "header", "footer" };
    int i;
    bars[0] = &out->header;
    bars[1] = &out->footer;
    fp_copy(out->cluster_event, fj_str(fj_get(c, "event")), 16);
    out->cluster_at = (int) fj_num(fj_get(c, "at"), 0);
    for (i = 0; i < 2; i++) {
        const FjNode* b = fj_get(s, names[i]);
        bars[i]->ok = b != NULL;
        fp_keys_from_json(fj_get(b, "keys"), bars[i]->k, &bars[i]->n);
    }
    fp_copy(out->tile_event, fj_str(fj_get(t, "event")), 16);
    order = fj_str(fj_get(t, "order"));
    out->selected_last = order != NULL && fp_strstr(order, "then selected") != NULL;
    out->stagger = (int) fj_num(fj_get(t, "stagger_frames"), 0);
    out->n_offsets = 0;
    for (o = fj_first(fj_get(t, "offsets")); o != NULL && out->n_offsets < 8; o = fj_next(o)) {
        out->offsets[out->n_offsets++] = (int) fj_num(o, 0);
    }
    /* "hold TRA_X at 720 (off-screen)": the number is the hold value */
    hold = fj_str(fj_get(t, "before_start"));
    out->hold = hold != NULL;
    out->hold_value = 720.0F;
    if (hold != NULL) {
        const char* p = fp_strstr(hold, " at ");
        if (p != NULL) {
            float v = 0.0F;
            int neg = 0;
            p += 4;
            if (*p == '-') {
                neg = 1;
                p++;
            }
            while (*p >= '0' && *p <= '9') {
                v = v * 10.0F + (float) (*p++ - '0');
            }
            out->hold_value = neg ? -v : v;
        }
    }
}

/* True when `nm` is a per-tile joint of the file: it ends in "_<id>" for an id some "tile_<id>"
 * joint names. */
static bool fp_authored_tile_joint(const FjNode* M, const char* nm)
{
    const FjNode* n;
    int ln = (int) strlen(nm);
    for (n = fj_first(fj_get(M, "joints")); n != NULL; n = fj_next(n)) {
        const char* t = fj_str(fj_get(n, "name"));
        int tl;
        if (t == NULL || strncmp(t, "tile_", 5) != 0) {
            continue;
        }
        t += 5;
        tl = (int) strlen(t);
        if (ln > tl + 1 && nm[ln - tl - 1] == '_' && strcmp(nm + ln - tl, t) == 0) {
            return true;
        }
    }
    return false;
}

/* The motion file. `instantiate` builds the joint tree for the tiles fp already has, from the
 * per-tile pattern in the file (the joints whose names end in the first tile's id), rather than
 * taking the file's joints literally: a generated screen has other tiles than the hub it was
 * authored for. The hub demo, whose tiles ARE the file's, takes it literally. */
static void fp_read_motion(const FjNode* M, bool instantiate)
{
    const FjNode* n;
    const FjNode* tr = fj_get(M, "transform");
    const char* tile0 = fp.nt > 0 ? fp.tile[0].id : "";
    const char* first_tile = NULL;
    int i;

    /* the first tile named in the file's joints is the template for per-tile joints */
    for (n = fj_first(fj_get(M, "joints")); n != NULL; n = fj_next(n)) {
        const char* nm = fj_str(fj_get(n, "name"));
        if (nm != NULL && strncmp(nm, "tile_", 5) == 0) {
            first_tile = nm + 5;
            break;
        }
    }
    for (n = fj_first(fj_get(M, "joints")); n != NULL; n = fj_next(n)) {
        const char* nm = fj_str(fj_get(n, "name"));
        const char* parent = fj_str(fj_get(n, "parent"));
        const FjNode* pv = fj_get(n, "pivot");
        const FjNode* d;
        int ln, fl, t;
        if (nm == NULL) {
            continue;
        }
        ln = (int) strlen(nm);
        fl = first_tile != NULL ? (int) strlen(first_tile) : 0;
        if (instantiate && fp_authored_tile_joint(M, nm) &&
            !(ln > fl + 1 && nm[ln - fl - 1] == '_' && strcmp(nm + ln - fl, first_tile) == 0))
        {
            continue; /* another authored tile's joint: the template makes ours */
        }
        if (!instantiate || !fp_authored_tile_joint(M, nm)) {
            /* not per-tile (header, footer, cluster), or the demo: as written */
            float p[2];
            int j;
            p[0] = fj_num(fj_idx(pv, 0), 0);
            p[1] = fj_num(fj_idx(pv, 1), 0);
            j = fp_add_joint(nm, parent != NULL ? fp_find_joint(parent) : -1,
                             (pv != NULL && pv->type == FJ_ARR) ? p : NULL);
            for (d = fj_first(fj_get(n, "draws")); d != NULL; d = fj_next(d)) {
                fp_joint_draws(j, fj_str(d));
            }
            continue;
        }
        /* a per-tile template joint: one per tile of ours */
        for (t = 0; t < fp.nt; t++) {
            char name[FP_ID], pname[FP_ID], pat[FP_ID];
            int j, k;
            float p[2];
            fp_copy(pat, nm, FP_ID);
            pat[ln - fl] = '\0';
            fp_cat(pat, "<id>");
            fp_subst(name, pat, fp.tile[t].id);
            pname[0] = '\0';
            if (parent != NULL) {
                int pl = (int) strlen(parent);
                if (pl > fl && strcmp(parent + pl - fl, first_tile) == 0) {
                    fp_copy(pat, parent, FP_ID);
                    pat[pl - fl] = '\0';
                    fp_cat(pat, "<id>");
                    fp_subst(pname, pat, fp.tile[t].id);
                } else {
                    fp_copy(pname, parent, FP_ID);
                }
            }
            /* pivot: the face's centre, which is how the file's pivots are made */
            {
                FpTile* tt = &fp.tile[t];
                p[0] = p[1] = 0.0F;
                if (tt->q_face >= 0) {
                    for (k = 0; k < 4; k++) {
                        p[0] += fp.q[tt->q_face].v[k][0] * 0.25F;
                        p[1] += fp.q[tt->q_face].v[k][1] * 0.25F;
                    }
                }
            }
            j = fp_add_joint(name, pname[0] ? fp_find_joint(pname) : -1,
                             (pv != NULL && pv->type == FJ_ARR) ? p : NULL);
            for (d = fj_first(fj_get(n, "draws")); d != NULL; d = fj_next(d)) {
                const char* qid = fj_str(d);
                int ql = qid != NULL ? (int) strlen(qid) : 0;
                char qname[FP_ID];
                /* "versus_face" -> "<id>_face" */
                if (qid != NULL && ql > fl && strncmp(qid, first_tile, fl) == 0) {
                    fp_copy(pat, "<id>", FP_ID);
                    fp_cat(pat, qid + fl);
                    fp_subst(qname, pat, fp.tile[t].id);
                    fp_joint_draws(j, qname);
                }
            }
        }
    }
    (void) tile0;

    /* materials: "face_versus" -> "versus_face", kept as patterns */
    fp.nmat = 0;
    for (n = fj_first(fj_get(fj_get(M, "materials"), "map")); n != NULL && fp.nmat < 8;
         n = fj_next(n))
    {
        const char* k = n->key;
        const char* v = fj_str(n);
        int kl, fl, m;
        if (k == NULL || v == NULL || first_tile == NULL) {
            continue;
        }
        kl = (int) strlen(k);
        fl = (int) strlen(first_tile);
        if (kl <= fl || strcmp(k + kl - fl, first_tile) != 0 || strncmp(v, first_tile, fl) != 0) {
            continue;
        }
        m = fp.nmat++;
        fp_copy(fp.mat_key[m], k, FP_ID);
        fp.mat_key[m][kl - fl] = '\0';
        fp_cat(fp.mat_key[m], "<id>");
        fp_copy(fp.mat_val[m], "<id>", FP_ID);
        fp_cat(fp.mat_val[m], v + fl);
    }

    /* "RECOIL is a scalar r; tile_<id> adds r * 2 * unit(...)" */
    for (n = fj_first(tr); n != NULL; n = fj_next(n)) {
        const char* s = fj_str(n);
        const char* p = s != NULL ? fp_strstr(s, "r * ") : NULL;
        if (p != NULL) {
            float v = 0.0F;
            p += 4;
            while (*p >= '0' && *p <= '9') {
                v = v * 10.0F + (float) (*p++ - '0');
            }
            if (v > 0.0F) {
                fp.recoil_px = v;
            }
        }
    }

    for (n = fj_first(fj_get(M, "events")); n != NULL && fp.nev < FP_MAX_EVENTS; n = fj_next(n))
    {
        FpEvent* e = &fp.ev[fp.nev++];
        const FjNode* t;
        fp_copy(e->name, n->key, 16);
        e->length = (int) fj_num(fj_get(n, "length_frames"), 1);
        e->n = 0;
        for (t = fj_first(fj_get(n, "tracks")); t != NULL && e->n < FP_MAX_TRACKS;
             t = fj_next(t))
        {
            FpTrack* k = &e->t[e->n++];
            const char* kind = fj_str(fj_get(t, "kind"));
            fp_copy(k->target, fj_str(fj_get(t, "target")), FP_ID);
            k->is_material = kind != NULL && strcmp(kind, "material") == 0;
            k->channel = fp_channel(fj_str(fj_get(t, "channel")), &k->recoil);
            fp_keys_from_json(fj_get(t, "keys"), k->k, &k->n);
        }
    }

    n = fj_get(M, "sequences");
    {
        const FjNode* s;
        fp.n_cursor_move = 0;
        for (s = fj_first(fj_get(n, "cursor_move")); s != NULL && fp.n_cursor_move < 8;
             s = fj_next(s))
        {
            FpSeqStep* st = &fp.cursor_move[fp.n_cursor_move++];
            const char* on = fj_str(fj_get(s, "on"));
            fp_copy(st->event, fj_str(fj_get(s, "event")), 16);
            st->at = (int) fj_num(fj_get(s, "at"), 0);
            st->on = on == NULL ? 1 : strcmp(on, "previous") == 0 ? 0 : strcmp(on, "others") == 0 ? 2 : 1;
        }
        for (s = fj_first(fj_get(n, "confirm")); s != NULL; s = fj_next(s)) {
            if (fj_get(s, "event") != NULL) {
                fp_copy(fp.confirm_event, fj_str(fj_get(s, "event")), 16);
            } else if (fj_get(s, "sequence") != NULL) {
                fp.screen_out_at = (int) fj_num(fj_get(s, "at"), 14);
            }
        }
        fp_seq_screen(fj_get(n, "screen_out"), &fp.s_out);
        fp_seq_screen(fj_get(n, "screen_in"), &fp.s_in);
    }
    for (i = 0; i < fp.nt; i++) {
        char nm[FP_ID];
        FpTile* t = &fp.tile[i];
        fp_subst(nm, "tile_<id>", t->id);
        t->j_tile = fp_find_joint(nm);
        fp_subst(nm, "lift_<id>", t->id);
        t->j_lift = fp_find_joint(nm);
        fp_subst(nm, "plate_<id>", t->id);
        t->j_plate = fp_find_joint(nm);
        fp_subst(nm, "desc_<id>", t->id);
        t->j_desc = fp_find_joint(nm);
    }
    fp.motion_ok = fp.nev > 0;
}

/* ---- channels ---------------------------------------------------------------------------- */

static FpChan* fp_jch(int j, int c)
{
    return &fp.ch[j * FJC_N + c];
}

static FpChan* fp_mch(int q, int c)
{
    return &fp.ch[FP_MAX_JOINTS * FJC_N + q * FMC_N + c];
}

static float fp_eval(const FpKey* k, int n, float t)
{
    int i;
    if (n <= 0) {
        return 0.0F;
    }
    if (t <= (float) k[0].frame) {
        return k[0].v;
    }
    for (i = 0; i + 1 < n; i++) {
        float f0 = (float) k[i].frame, f1 = (float) k[i + 1].frame;
        if (t < f1) {
            float u, d, u2, u3;
            if (k[i].interp == FPI_CON) {
                return k[i].v;
            }
            u = (t - f0) / (f1 - f0);
            if (k[i].interp == FPI_LIN) {
                return k[i].v + (k[i + 1].v - k[i].v) * u;
            }
            d = f1 - f0;
            u2 = u * u;
            u3 = u2 * u;
            return (2 * u3 - 3 * u2 + 1) * k[i].v + (u3 - 2 * u2 + u) * d * k[i].slope +
                   (-2 * u3 + 3 * u2) * k[i + 1].v + (u3 - u2) * d * k[i + 1].slope;
        }
    }
    return k[n - 1].v;
}

static float fp_value(FpChan* c, int f)
{
    int i;
    for (i = c->n - 1; i >= 0; i--) {
        if (c->play[i].start <= f) {
            return fp_eval(c->play[i].k, c->play[i].n, (float) (f - c->play[i].start));
        }
    }
    return c->rest;
}

static void fp_start(FpChan* c, int f, const FpKey* keys, int n)
{
    FpPlay* p;
    int i;
    float cur = fp_value(c, f);
    if (c->n == FP_PLAYS) {
        for (i = 1; i < FP_PLAYS; i++) {
            c->play[i - 1] = c->play[i];
        }
        c->n--;
    }
    p = &c->play[c->n++];
    p->start = f;
    p->n = (u8) (n < FP_MAX_KEYS ? n : FP_MAX_KEYS);
    for (i = 0; i < p->n; i++) {
        p->k[i] = keys[i];
        if (keys[i].from_current) {
            p->k[i].v = cur;
        }
    }
}

/* Hold a channel at a value from frame f on. */
static void fp_set(FpChan* c, int f, float v)
{
    FpKey k;
    k.frame = 0;
    k.interp = FPI_CON;
    k.from_current = 0;
    k.v = v;
    k.slope = 0.0F;
    fp_start(c, f, &k, 1);
}

static void fp_reset_channels(void)
{
    int i, c;
    for (i = 0; i < FP_MAX_JOINTS * FJC_N + FP_MAX_QUADS * FMC_N; i++) {
        fp.ch[i].n = 0;
        fp.ch[i].rest = 0.0F;
    }
    for (i = 0; i < fp.nj; i++) {
        fp_jch(i, FJC_SCA_X)->rest = 1.0F;
        fp_jch(i, FJC_SCA_Y)->rest = 1.0F;
        fp_jch(i, FJC_ALPHA)->rest = 1.0F;
    }
    /* State rules: the plate shows only on the selection and only the selection's description
       shows, so their joints rest invisible. */
    for (i = 0; i < fp.nt; i++) {
        if (fp.tile[i].j_plate >= 0) {
            fp_jch(fp.tile[i].j_plate, FJC_ALPHA)->rest = 0.0F;
        }
        if (fp.tile[i].j_desc >= 0) {
            fp_jch(fp.tile[i].j_desc, FJC_ALPHA)->rest = 0.0F;
        }
    }
    for (i = 0; i < fp.nq; i++) {
        for (c = 0; c < 3; c++) {
            fp_mch(i, c)->rest = fp.q[i].rgb[c];
        }
        fp_mch(i, FMC_ALPHA)->rest = 1.0F;
    }
}

/* Play an event on tile `ti` from frame f. `joint_override` sends every joint track to that
 * joint instead (the cluster's backplate rides the tiles' slide). */
static void fp_event(const char* name, int ti, int f, int joint_override)
{
    int e = fp_find_event(name), k;
    const char* id = ti >= 0 ? fp.tile[ti].id : "";
    if (e < 0) {
        return;
    }
    for (k = 0; k < fp.ev[e].n; k++) {
        FpTrack* t = &fp.ev[e].t[k];
        char target[FP_ID];
        int m, q = -1, j = -1;
        fp_subst(target, t->target, id);
        if (t->is_material) {
            for (m = 0; m < fp.nmat; m++) {
                char key[FP_ID], val[FP_ID];
                fp_subst(key, fp.mat_key[m], id);
                if (strcmp(key, target) == 0) {
                    fp_subst(val, fp.mat_val[m], id);
                    q = fp_find_quad(val);
                    break;
                }
            }
        }
        if (q >= 0) {
            fp_start(fp_mch(q, t->channel == FJC_ALPHA ? FMC_ALPHA : t->channel), f, t->k, t->n);
            continue;
        }
        /* a material track on a target that names a joint (plate_<id>, desc_<id>) drives that
           joint's ALPHA, which is how the file's joints list those channels */
        j = joint_override >= 0 ? joint_override : fp_find_joint(target);
        if (j < 0) {
            continue;
        }
        if (t->recoil && ti >= 0 && fp.sel >= 0 && fp.sel != ti) {
            FpTile* a = &fp.tile[ti];
            FpTile* b = &fp.tile[fp.sel];
            if (a->j_lift >= 0 && b->j_lift >= 0) {
                float dx = fp.j[a->j_lift].pivot[0] - fp.j[b->j_lift].pivot[0];
                float dy = fp.j[a->j_lift].pivot[1] - fp.j[b->j_lift].pivot[1];
                float l = fp_sqrt(dx * dx + dy * dy);
                if (l > 0.0F) {
                    fp.j[j].recoil_dir[0] = dx / l;
                    fp.j[j].recoil_dir[1] = dy / l;
                }
            }
        }
        fp_start(fp_jch(j, t->channel), f, t->k, t->n);
    }
}

/* "the selected tile is already at its 'sel' rest values" */
static void fp_select_initial(int ti, int f)
{
    FpTile* t = &fp.tile[ti];
    int c;
    fp.sel = ti;
    fp.prev = -1;
    if (t->j_lift >= 0) {
        fp_set(fp_jch(t->j_lift, FJC_TRA_X), f, fp.sel_offset[0]);
        fp_set(fp_jch(t->j_lift, FJC_TRA_Y), f, fp.sel_offset[1]);
    }
    if (t->j_plate >= 0) {
        fp_set(fp_jch(t->j_plate, FJC_ALPHA), f, 1.0F);
    }
    if (t->j_desc >= 0) {
        fp_set(fp_jch(t->j_desc, FJC_ALPHA), f, 1.0F);
    }
    for (c = 0; c < 3; c++) {
        if (t->q_face >= 0) fp_set(fp_mch(t->q_face, c), f, fp.sel_face[c]);
        if (t->q_icon >= 0) fp_set(fp_mch(t->q_icon, c), f, fp.sel_icon[c]);
        if (t->q_label >= 0) fp_set(fp_mch(t->q_label, c), f, fp.sel_label[c]);
    }
}

/* sequences.cursor_move */
static void fp_cursor_move(int to, int f)
{
    int i, k;
    int old = fp.sel;
    if (to == old || to < 0) {
        return;
    }
    fp.prev = old;
    fp.sel = to;
    for (i = 0; i < fp.n_cursor_move; i++) {
        FpSeqStep* s = &fp.cursor_move[i];
        if (s->on == 0) {
            if (old >= 0) fp_event(s->event, old, f + s->at, -1);
        } else if (s->on == 1) {
            fp_event(s->event, to, f + s->at, -1);
        } else {
            for (k = 0; k < fp.nt; k++) {
                if (k != to && k != old) {
                    fp_event(s->event, k, f + s->at, -1);
                }
            }
        }
    }
}

static int fp_by_rank(int* order)
{
    int i, k, n = 0;
    for (i = 0; i < fp.nt; i++) {
        order[n++] = i;
    }
    for (i = 1; i < n; i++) { /* insertion sort by rank */
        int v = order[i];
        for (k = i; k > 0 && fp.tile[order[k - 1]].rank > fp.tile[v].rank; k--) {
            order[k] = order[k - 1];
        }
        order[k] = v;
    }
    return n;
}

static void fp_bar(FpSeqBar* b, const char* joint, int f)
{
    int j = fp_find_joint(joint);
    if (b->ok && j >= 0) {
        fp_start(fp_jch(j, FJC_TRA_Y), f, b->k, b->n);
    }
}

/* Frames until a screen_out started now has finished. */
static int fp_screen_out_length(void)
{
    int e = fp_find_event(fp.s_out.tile_event);
    int len = e >= 0 ? fp.ev[e].length : 13;
    return fp.s_out.stagger * (fp.nt > 0 ? fp.nt - 1 : 0) + len;
}

static void fp_screen_out(int f)
{
    int order[FP_MAX_TILES], n = fp_by_rank(order), i, s = 0;
    int cl = fp_find_joint("cluster");
    for (i = 0; i < n; i++) {
        if (fp.s_out.selected_last && order[i] == fp.sel) {
            continue;
        }
        fp_event(fp.s_out.tile_event, order[i], f + fp.s_out.stagger * s++, -1);
    }
    if (fp.s_out.selected_last && fp.sel >= 0) {
        fp_event(fp.s_out.tile_event, fp.sel, f + fp.s_out.stagger * s, -1);
    }
    if (cl >= 0) {
        fp_event(fp.s_out.cluster_event, fp.nt > 0 ? 0 : -1, f + fp.s_out.cluster_at, cl);
    }
    fp_bar(&fp.s_out.header, "header", f);
    fp_bar(&fp.s_out.footer, "footer", f);
}

static void fp_screen_in(int f)
{
    int order[FP_MAX_TILES], n = fp_by_rank(order), i;
    int cl = fp_find_joint("cluster");
    for (i = 0; i < n; i++) {
        FpTile* t = &fp.tile[order[i]];
        int off;
        if (i < fp.s_in.n_offsets) {
            off = fp.s_in.offsets[i];
        } else { /* more tiles than the authored offsets: keep the last step */
            int m = fp.s_in.n_offsets;
            int step = m >= 2 ? fp.s_in.offsets[m - 1] - fp.s_in.offsets[m - 2] : 2;
            off = (m > 0 ? fp.s_in.offsets[m - 1] : 0) + step * (i - m + 1);
        }
        if (fp.s_in.hold && t->j_tile >= 0) {
            fp_set(fp_jch(t->j_tile, FJC_TRA_X), f, fp.s_in.hold_value);
        }
        fp_event(fp.s_in.tile_event, order[i], f + off, -1);
    }
    if (cl >= 0) {
        fp_event(fp.s_in.cluster_event, fp.nt > 0 ? 0 : -1, f + fp.s_in.cluster_at, cl);
    }
    fp_bar(&fp.s_in.header, "header", f);
    fp_bar(&fp.s_in.footer, "footer", f);
}

static int fp_screen_in_length(void)
{
    int e = fp_find_event(fp.s_in.tile_event);
    int last = fp.s_in.n_offsets > 0 ? fp.s_in.offsets[fp.s_in.n_offsets - 1] : 0;
    if (fp.nt > fp.s_in.n_offsets) {
        last += 2 * (fp.nt - fp.s_in.n_offsets);
    }
    return last + (e >= 0 ? fp.ev[e].length : 18);
}

/* ---- evaluation and drawing -------------------------------------------------------------- */

static void fp_evaluate(int f)
{
    int i, k, c;
    for (i = 0; i < fp.nj; i++) {
        FpJoint* j = &fp.j[i];
        float sx = fp_value(fp_jch(i, FJC_SCA_X), f);
        float sy = fp_value(fp_jch(i, FJC_SCA_Y), f);
        float r = fp_value(fp_jch(i, FJC_RECOIL), f) * fp.recoil_px;
        float tx = fp_value(fp_jch(i, FJC_TRA_X), f) + r * j->recoil_dir[0];
        float ty = fp_value(fp_jch(i, FJC_TRA_Y), f) + r * j->recoil_dir[1];
        float px = j->has_pivot ? j->pivot[0] : 0.0F;
        float py = j->has_pivot ? j->pivot[1] : 0.0F;
        /* local: s * p + (pivot - s * pivot + T) */
        float ls[2], lo[2];
        ls[0] = sx;
        ls[1] = sy;
        lo[0] = px - sx * px + tx;
        lo[1] = py - sy * py + ty;
        j->wa = fp_value(fp_jch(i, FJC_ALPHA), f);
        if (j->parent >= 0) {
            FpJoint* p = &fp.j[j->parent];
            for (c = 0; c < 2; c++) {
                j->ws[c] = p->ws[c] * ls[c];
                j->wo[c] = p->ws[c] * lo[c] + p->wo[c];
            }
            j->wa *= p->wa;
        } else {
            for (c = 0; c < 2; c++) {
                j->ws[c] = ls[c];
                j->wo[c] = lo[c];
            }
        }
    }
    for (i = 0; i < fp.nq; i++) {
        FpQuad* q = &fp.q[i];
        for (c = 0; c < 3; c++) {
            q->wrgb[c] = fp_value(fp_mch(i, c), f);
        }
        q->wa = fp_value(fp_mch(i, FMC_ALPHA), f);
        if (q->role == FPR_PLATE) {
            for (c = 0; c < 3; c++) {
                q->wrgb[c] = fp.sel_plate[c];
            }
        }
        if (q->joint >= 0) {
            FpJoint* j = &fp.j[q->joint];
            for (k = 0; k < 4; k++) {
                q->w[k][0] = j->ws[0] * q->v[k][0] + j->wo[0];
                q->w[k][1] = j->ws[1] * q->v[k][1] + j->wo[1];
            }
            q->wa *= j->wa;
            q->wscale = j->ws[1];
        } else {
            for (k = 0; k < 4; k++) {
                q->w[k][0] = q->v[k][0];
                q->w[k][1] = q->v[k][1];
            }
            q->wscale = 1.0F;
        }
    }
}

static u8 fp_u8(float v)
{
    v = v * 255.0F + 0.5F;
    return (u8) (v < 0.0F ? 0 : v > 255.0F ? 255 : (int) v);
}

static void fp_flat(const float w[4][2], GXColor c)
{
    int k;
    GXBegin(GX_QUADS, GX_VTXFMT0, 4);
    for (k = 0; k < 4; k++) {
        GXPosition2f32(w[k][0], -w[k][1]);
        GXColor4u8(c.r, c.g, c.b, c.a);
    }
}

static void fp_textured(FpTex* tx, const float w[4][2], const float* uv, GXColor c)
{
    float u[4], v[4];
    int k;
    u[0] = uv[0]; v[0] = uv[1];
    u[1] = uv[2]; v[1] = uv[1];
    u[2] = uv[2]; v[2] = uv[3];
    u[3] = uv[0]; v[3] = uv[3];
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
    if (tx->mask) {
        /* I4: RGB from the material colour only, alpha = texture x material alpha */
        GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_ZERO, GX_CC_ZERO, GX_CC_ZERO, GX_CC_RASC);
        GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_ZERO, GX_CA_TEXA, GX_CA_RASA, GX_CA_ZERO);
        GXSetTevColorOp(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, GX_TRUE,
                        GX_TEVPREV);
        GXSetTevAlphaOp(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, GX_TRUE,
                        GX_TEVPREV);
    } else {
        GXSetTevOp(GX_TEVSTAGE0, GX_MODULATE);
    }
    GXSetNumChans(1);
    GXSetChanCtrl(GX_COLOR0A0, GX_FALSE, GX_SRC_REG, GX_SRC_VTX, GX_LIGHT_NULL, GX_DF_NONE,
                  GX_AF_NONE);
    GXSetBlendMode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_NOOP);
    GXSetAlphaCompare(GX_ALWAYS, 0, GX_AOP_AND, GX_ALWAYS, 0);
    GXSetZMode(GX_FALSE, GX_ALWAYS, GX_FALSE);
    GXSetCullMode(GX_CULL_NONE);
    if (tx->t.lut != NULL) {
        GXLoadTlut(&tx->t.tlut, GX_TLUT0);
    }
    GXLoadTexObj(&tx->t.obj, GX_TEXMAP0);
    GXBegin(GX_QUADS, GX_VTXFMT0, 4);
    for (k = 0; k < 4; k++) {
        GXPosition2f32(w[k][0], -w[k][1]);
        GXColor4u8(c.r, c.g, c.b, c.a);
        GXTexCoord2f32(u[k], v[k]);
    }
    HSD_StateInvalidate(-1);
    hsd_80391A04(1.0F, 1.0F, 1);
}

static void fp_draw_quad(int i)
{
    FpQuad* q = &fp.q[i];
    GXColor c;
    if (i < 0 || q->kind == FPK_TEXT || q->wa <= 0.01F) {
        return;
    }
    if (q->kind == FPK_FLAT && !q->has_colour && q->role != FPR_PLATE) {
        return; /* a flat quad with no colour is a placeholder with nothing to show */
    }
    c.r = fp_u8(q->wrgb[0]);
    c.g = fp_u8(q->wrgb[1]);
    c.b = fp_u8(q->wrgb[2]);
    c.a = fp_u8(q->wa);
    if (q->kind == FPK_TEX) {
        if (q->tex >= 0) {
            fp_textured(&fp.tex[q->tex], q->w, q->uv, c);
        }
        return;
    }
    fp_flat(q->w, c);
}

static void fp_draw(void)
{
    int i, k;
    int order[FP_MAX_TILES], n;
    float bg[4][2];
    GXColor clear;
    if (!fp.ok) {
        return;
    }
    hsd_80391A04(1.0F, 1.0F, 1);
    bg[0][0] = -8; bg[0][1] = -8;
    bg[1][0] = 648; bg[1][1] = -8;
    bg[2][0] = 648; bg[2][1] = 488;
    bg[3][0] = -8; bg[3][1] = 488;
    clear.r = fp_u8(fp.clear[0]);
    clear.g = fp_u8(fp.clear[1]);
    clear.b = fp_u8(fp.clear[2]);
    clear.a = 255;
    fp_flat(bg, clear);
    for (i = 0; i < fp.nq; i++) {
        if (fp.q[i].tile < 0) {
            fp_draw_quad(i);
        }
    }
    /* tiles by rank, then the previous selection, then the selection */
    n = fp_by_rank(order);
    for (k = 0; k < n + 2; k++) {
        int ti = k < n ? order[k] : k == n ? fp.prev : fp.sel;
        FpTile* t;
        if (ti < 0 || (k < n && (ti == fp.sel || ti == fp.prev))) {
            continue;
        }
        t = &fp.tile[ti];
        fp_draw_quad(t->q_plate);
        fp_draw_quad(t->q_face);
        fp_draw_quad(t->q_icon);
        fp_draw_quad(t->q_label);
    }
    for (i = 0; i < fp.nt; i++) {
        fp_draw_quad(fp.tile[i].q_desc);
    }
}
