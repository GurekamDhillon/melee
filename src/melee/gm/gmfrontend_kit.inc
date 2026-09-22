/* gmfrontend_kit.inc - included by gmfrontend.c (TARGET_PC only), after the player.
 *
 * ---- THE KIT (art brief section 1: C:/Users/Gurek/Desktop/menu/out_kit) --------------------
 *
 *   kit.json            palette, the five sections' colours and band rhythms, port colours
 *   font_manifest.json  the atlas: per role (caption 12 .. display 56) metrics, glyphs (advance,
 *                       page, uv, size, offset from the pen at the baseline) and kerning pairs
 *   chrome_layout.json  header slab + title, breadcrumb, description strip, footer hints, bands
 *   list_layout.json    the list panel and its row template (states ng / sel / disabled)
 *   kit_motion.json     row_select / row_deselect (and more) in hub_motion.json's format, with
 *                       colour TOKENS (@face, gold, ...) where the hub has numbers
 *
 * Kit templates are UNSHEARED: every vertex, glyph quads included, goes through
 * x' = x + (240 - y) * 0.25 last. The hub's files are still in screen space (they move to the
 * kit's convention in section 2), and its motion is applied in screen space; so here layout is
 * built unsheared, sheared into the player's screen-space quads, and the (screen-space) hub
 * motion is applied to those. For translation-only motion the difference is the lift's
 * direction, which both conventions author as (-k, -k) on screen.
 *
 * TEXT is set from the font manifest alone, per its own `setting` rules: kerning[prev+ch], quad
 * at pen + offset with the glyph's size and uv, pen += advance; a missing character falls back
 * to '?'; hero/display are caps-only (uppercased here); a string too wide for its slot steps
 * down to the next smaller role of the same face and is then truncated with an ellipsis - never
 * squashed. The glyphs are upright; the screen shear italicises them.
 *
 * All three parse once per process into static tables (the font manifest is 350 KB of JSON; its
 * DOM lives only for the parse) and the atlas pages load by name like any other texture. With
 * no manifest the menus fall back to HSD_SisLib text. */

/* ---- font ------------------------------------------------------------------------------- */

#define FF_ROLES 10
#define FF_NCH 100 /* 0x20..0x7E, then the ellipsis and the four arrows */
#define FF_ELLIPSIS 95
#define FF_ARROW 96 /* U+2190..2193 (left, up, right, down); in strings bytes 0x02..0x05 */
#define FF_MAX_KERN 12000

typedef struct FfGlyph {
    float adv, off[2], size[2], uv[4];
    s8 page; ///< -1: no quad (space)
    u8 ok;
} FfGlyph;

typedef struct FfRole {
    char name[12];
    char face[8];
    float size, ascent, descent, cap, line;
    bool caps;
    int npages;
    char page[2][FP_ID];
    FfGlyph g[FF_NCH];
    int kern_first[FF_NCH]; ///< index into ff.kern of the first pair starting with this char
    int kern_count[FF_NCH];
} FfRole;

typedef struct FfKern {
    u8 a, b;
    s16 k; ///< 1/100 px
} FfKern;

static struct {
    int state; ///< 0 not tried, 1 loaded, -1 unavailable
    FfRole r[FF_ROLES];
    int nr;
    FfKern kern[FF_MAX_KERN];
    int nk;
} ff;

static int ff_char(const char* key)
{
    const u8* k = (const u8*) key;
    if (k == NULL || k[0] == 0) {
        return -1;
    }
    if (k[1] == 0 && k[0] >= 0x20 && k[0] < 0x7F) {
        return k[0] - 0x20;
    }
    if (k[0] == 0xE2 && k[1] == 0x80 && k[2] == 0xA6 && k[3] == 0) { /* U+2026 */
        return FF_ELLIPSIS;
    }
    if (k[0] == 0xE2 && k[1] == 0x86 && k[2] >= 0x90 && k[2] <= 0x93 && k[3] == 0) {
        return FF_ARROW + (k[2] - 0x90); /* U+2190..2193 */
    }
    return -1;
}

static void ff_load(void)
{
    void *mt, *mp;
    const FjNode* M;
    const FjNode* rn;
    if (ff.state != 0) {
        return;
    }
    ff.state = -1;
    M = fj_load("font_manifest.json", &mt, &mp);
    ff.nr = 0;
    ff.nk = 0;
    for (rn = fj_first(fj_get(M, "roles")); rn != NULL && ff.nr < FF_ROLES; rn = fj_next(rn)) {
        FfRole* r = &ff.r[ff.nr];
        const FjNode* m = fj_get(rn, "metrics");
        const FjNode* g;
        const FjNode* p;
        const char* cs = fj_str(fj_get(rn, "charset"));
        int i;
        memset(r, 0, sizeof *r);
        fp_copy(r->name, rn->key, 12);
        fp_copy(r->face, fj_str(fj_get(rn, "face")), 8);
        r->size = fj_num(fj_get(rn, "size"), 16);
        r->ascent = fj_num(fj_get(m, "ascent"), r->size);
        r->descent = fj_num(fj_get(m, "descent"), r->size * 0.25F);
        r->cap = fj_num(fj_get(m, "cap_height"), r->size * 0.66F);
        r->line = fj_num(fj_get(m, "line_height"), r->size * 1.25F);
        r->caps = cs != NULL && strcmp(cs, "caps") == 0;
        for (p = fj_first(fj_get(fj_get(rn, "pages"), "latin")); p != NULL && r->npages < 2;
             p = fj_next(p))
        {
            fp_copy(r->page[r->npages++], fj_str(p), FP_ID);
        }
        for (g = fj_first(fj_get(rn, "glyphs")); g != NULL; g = fj_next(g)) {
            int c = ff_char(g->key);
            FfGlyph* d;
            if (c < 0) {
                continue;
            }
            d = &r->g[c];
            d->ok = 1;
            d->adv = fj_num(fj_get(g, "advance"), 0);
            d->page = fj_get(g, "uv") != NULL ? (s8) fj_num(fj_get(g, "page"), 0) : -1;
            for (i = 0; i < 4; i++) {
                d->uv[i] = fj_num(fj_idx(fj_get(g, "uv"), i), 0);
            }
            for (i = 0; i < 2; i++) {
                d->size[i] = fj_num(fj_idx(fj_get(g, "size"), i), 0);
                d->off[i] = fj_num(fj_idx(fj_get(g, "offset"), i), 0);
            }
        }
        /* kerning pairs, bucketed by first character (the manifest lists them sorted, but
           nothing relies on that) */
        {
            const FjNode* k;
            int start = ff.nk, c;
            for (c = 0; c < FF_NCH; c++) {
                int first = ff.nk;
                for (k = fj_first(fj_get(rn, "kerning")); k != NULL && ff.nk < FF_MAX_KERN;
                     k = fj_next(k))
                {
                    const u8* key = (const u8*) k->key;
                    if (key == NULL || key[0] != c + 0x20 || key[1] < 0x20 || key[1] >= 0x7F ||
                        key[2] != 0)
                    {
                        continue;
                    }
                    ff.kern[ff.nk].a = key[0];
                    ff.kern[ff.nk].b = key[1];
                    ff.kern[ff.nk].k = (s16) (fj_num(k, 0) * 100.0F);
                    ff.nk++;
                }
                r->kern_first[c] = first;
                r->kern_count[c] = ff.nk - first;
            }
            (void) start;
        }
        ff.nr++;
    }
    if (mt != NULL) HSD_Free(mt);
    if (mp != NULL) HSD_Free(mp);
    if (ff.nr > 0) {
        ff.state = 1;
        OSReport("frontend: font atlas - %d roles, %d kerning pairs\n", ff.nr, ff.nk);
    }
}

static int ff_role(const char* name)
{
    int i;
    for (i = 0; ff.state == 1 && i < ff.nr; i++) {
        if (strcmp(ff.r[i].name, name) == 0) {
            return i;
        }
    }
    return -1;
}

static float ff_kern(const FfRole* r, int prev, int ch)
{
    int i;
    if (prev < 0 || prev >= 95 || ch >= 95 || prev >= FF_NCH) {
        return 0.0F;
    }
    for (i = r->kern_first[prev]; i < r->kern_first[prev] + r->kern_count[prev]; i++) {
        if (ff.kern[i].b == ch + 0x20) {
            return (float) ff.kern[i].k / 100.0F;
        }
    }
    return 0.0F;
}

/* The glyph slot a byte sets in role r: hero/display uppercase, missing -> '?'. */
static int ff_slot(const FfRole* r, u8 c)
{
    int s;
    if (r->caps && c >= 'a' && c <= 'z') {
        c = (u8) (c - 'a' + 'A');
    }
    if (c == 0x01) {
        s = FF_ELLIPSIS; /* our internal marker for the ellipsis */
    } else if (c >= 0x02 && c <= 0x05) {
        s = FF_ARROW + (c - 0x02);
    } else {
        s = (c >= 0x20 && c < 0x7F) ? c - 0x20 : -1;
    }
    if (s < 0 || !r->g[s].ok) {
        s = '?' - 0x20;
    }
    return s;
}

static float ff_width(int role, const char* s)
{
    const FfRole* r = &ff.r[role];
    float x = 0.0F;
    int prev = -1;
    for (; *s != '\0'; s++) {
        int c = ff_slot(r, (u8) *s);
        x += ff_kern(r, prev, c) + r->g[c].adv;
        prev = c;
    }
    return x;
}

/* The fit rule: the next smaller role of the same face while too wide, then truncate with an
 * ellipsis (written as byte 0x01 into `out`). */
static int ff_fit(int role, const char* s, float max_w, char* out, int cap)
{
    int i, n;
    fp_copy(out, s, cap);
    if (max_w <= 0.0F) {
        return role;
    }
    while (ff_width(role, out) > max_w) {
        int best = -1;
        for (i = 0; i < ff.nr; i++) {
            if (strcmp(ff.r[i].face, ff.r[role].face) == 0 && ff.r[i].size < ff.r[role].size &&
                (best < 0 || ff.r[i].size > ff.r[best].size))
            {
                best = i;
            }
        }
        if (best < 0) {
            break;
        }
        role = best;
    }
    n = (int) strlen(out);
    while (n > 1 && ff_width(role, out) > max_w) {
        out[n - 2] = 0x01;
        out[n - 1] = '\0';
        n--;
    }
    return role;
}

/* ---- kit palette and sections --------------------------------------------------------- */

#define KIT_SECTIONS 5
enum { KS_VERSUS, KS_SOLO, KS_COLLECTION, KS_OPTIONS, KS_DATA };
static const char* const kit_section_names[KIT_SECTIONS] = { "versus", "solo", "collection",
                                                              "options", "data" };

typedef struct KitSection {
    float face[3], bg[3], band[3], face_hi[3];
    float band_x[6], band_w[6];
    int nbands;
} KitSection;

static struct {
    int state;
    char pal_name[20][12];
    float pal[20][3];
    int npal;
    KitSection sec[KIT_SECTIONS];
    float port[5][3];
    /* chrome_layout.json */
    float slab[4], rule[4], crumb_icon[4], desc_strip[4], footer[4], footer_mark[4];
    float title_anchor[2], crumb_anchor[2], desc_anchor[2], hint_anchor_y, hint_glyph_y0,
        hint_glyph_h;
    float title_max, crumb_max, desc_max, hint_max;
    char title_role[12], crumb_role[12], desc_role[12], hint_role[12];
    /* list_layout.json */
    float rows_top, row_h, pitch, row_x0, row_x1, row_label_x, row_label_base, row_label_max;
    int visible;
    char row_role[12];
    float row_sel_offset[2];
    /* kit_motion.json hold-repeat */
    int hold_delay, hold_interval;
} kit;

static void kit_rect(const FjNode* verts, float* r)
{
    r[0] = fj_num(fj_idx(fj_idx(verts, 0), 0), 0);
    r[1] = fj_num(fj_idx(fj_idx(verts, 0), 1), 0);
    r[2] = fj_num(fj_idx(fj_idx(verts, 2), 0), 0);
    r[3] = fj_num(fj_idx(fj_idx(verts, 2), 1), 0);
}

static bool kit_palette(const char* name, float* rgb)
{
    int i;
    for (i = 0; i < kit.npal; i++) {
        if (strcmp(kit.pal_name[i], name) == 0) {
            memcpy(rgb, kit.pal[i], 3 * sizeof(float));
            return true;
        }
    }
    return false;
}

/* A kit colour token: @face/@bg/@band/@face_hi from the section, port:pN, a palette name, or
 * a #rrggbb literal. */
static bool kit_colour(const char* tok, int section, float* rgb)
{
    const KitSection* s = &kit.sec[section < 0 ? KS_VERSUS : section];
    if (tok == NULL) {
        return false;
    }
    if (tok[0] == '#') {
        return fp_hex(tok, rgb);
    }
    if (tok[0] == '@') {
        const float* src = strcmp(tok, "@face") == 0      ? s->face
                           : strcmp(tok, "@bg") == 0      ? s->bg
                           : strcmp(tok, "@band") == 0    ? s->band
                           : strcmp(tok, "@face_hi") == 0 ? s->face_hi
                                                          : NULL;
        if (src != NULL) {
            memcpy(rgb, src, 3 * sizeof(float));
            return true;
        }
        return false;
    }
    if (strncmp(tok, "port:p", 6) == 0 && tok[6] >= '1' && tok[6] <= '4') {
        memcpy(rgb, kit.port[tok[6] - '1'], 3 * sizeof(float));
        return true;
    }
    return kit_palette(tok, rgb);
}

static void kit_slot(const FjNode* slots, const char* id, float* anchor, float* max_w, char* role)
{
    const FjNode* s;
    for (s = fj_first(slots); s != NULL; s = fj_next(s)) {
        const char* sid = fj_str(fj_get(s, "id"));
        const FjNode* t = s;
        if (sid == NULL) {
            continue;
        }
        if (strcmp(sid, id) != 0) {
            continue;
        }
        if (fj_get(s, "text") != NULL) {
            t = fj_get(s, "text");
        }
        anchor[0] = fj_num(fj_idx(fj_get(t, "anchor"), 0), anchor[0]);
        anchor[1] = fj_num(fj_idx(fj_get(t, "anchor"), 1), anchor[1]);
        *max_w = fj_num(fj_get(t, "max_width_1x"), *max_w);
        fp_copy(role, fj_str(fj_get(t, "role")), 12);
        return;
    }
}

static void kit_load(void)
{
    void *t, *p;
    const FjNode* K;
    const FjNode* n;
    int i;
    if (kit.state != 0) {
        return;
    }
    kit.state = -1;
    /* defaults: the kit's own values at delivery, so a missing file still draws the system */
    kit.rows_top = 84;
    kit.row_h = 30;
    kit.pitch = 34;
    kit.visible = 9;
    kit.row_x0 = 96;
    kit.row_x1 = 540;
    kit.row_label_x = 16;
    kit.row_label_base = 20.28F;
    kit.row_label_max = 211.2F;
    kit.row_sel_offset[0] = kit.row_sel_offset[1] = -3.0F;
    fp_copy(kit.row_role, "row", 12);
    kit.slab[0] = 84; kit.slab[1] = 24; kit.slab[2] = 408; kit.slab[3] = 52;
    kit.rule[0] = 404; kit.rule[1] = 46; kit.rule[2] = 550; kit.rule[3] = 52;
    kit.crumb_icon[0] = 96; kit.crumb_icon[1] = 56; kit.crumb_icon[2] = 112; kit.crumb_icon[3] = 72;
    kit.desc_strip[0] = 90; kit.desc_strip[1] = 398; kit.desc_strip[2] = 596; kit.desc_strip[3] = 422;
    kit.footer[0] = 90; kit.footer[1] = 430; kit.footer[2] = 596; kit.footer[3] = 456;
    kit.footer_mark[0] = 100; kit.footer_mark[1] = 437; kit.footer_mark[2] = 106; kit.footer_mark[3] = 449;
    kit.title_anchor[0] = 100; kit.title_anchor[1] = 45.92F; kit.title_max = 288;
    kit.crumb_anchor[0] = 118; kit.crumb_anchor[1] = 68.62F; kit.crumb_max = 432;
    kit.desc_anchor[0] = 106; kit.desc_anchor[1] = 414.62F; kit.desc_max = 474;
    kit.hint_anchor_y = 447.62F; kit.hint_glyph_y0 = 435; kit.hint_glyph_h = 16; kit.hint_max = 67.2F;
    fp_copy(kit.title_role, "title", 12);
    fp_copy(kit.crumb_role, "body", 12);
    fp_copy(kit.desc_role, "body", 12);
    fp_copy(kit.hint_role, "body", 12);
    kit.hold_delay = 24;
    kit.hold_interval = 4;

    K = fj_load("kit.json", &t, &p);
    if (K != NULL) {
        kit.npal = 0;
        for (n = fj_first(fj_get(K, "palette")); n != NULL && kit.npal < 20; n = fj_next(n)) {
            fp_copy(kit.pal_name[kit.npal], n->key, 12);
            fp_hex(fj_str(n), kit.pal[kit.npal]);
            kit.npal++;
        }
        for (i = 0; i < KIT_SECTIONS; i++) {
            const FjNode* s = fj_get(fj_get(K, "sections"), kit_section_names[i]);
            const FjNode* b;
            KitSection* d = &kit.sec[i];
            fp_hex(fj_str(fj_get(s, "face")), d->face);
            fp_hex(fj_str(fj_get(s, "bg")), d->bg);
            fp_hex(fj_str(fj_get(s, "band")), d->band);
            fp_hex(fj_str(fj_get(s, "face_hi")), d->face_hi);
            d->nbands = 0;
            for (b = fj_first(fj_get(s, "bands")); b != NULL && d->nbands < 6; b = fj_next(b)) {
                d->band_x[d->nbands] = fj_num(fj_idx(b, 0), 0);
                d->band_w[d->nbands] = fj_num(fj_idx(b, 1), 0);
                d->nbands++;
            }
        }
        for (i = 0; i < 4; i++) {
            char k[4] = { 'p', (char) ('1' + i), 0, 0 };
            fp_hex(fj_str(fj_get(fj_get(fj_get(K, "ports"), "colours"), k)), kit.port[i]);
        }
        kit.state = 1;
    }
    if (t != NULL) HSD_Free(t);
    if (p != NULL) HSD_Free(p);

    K = fj_load("chrome_layout.json", &t, &p);
    if (K != NULL) {
        for (n = fj_first(fj_get(K, "quads")); n != NULL; n = fj_next(n)) {
            const char* id = fj_str(fj_get(n, "id"));
            float* r = id == NULL                         ? NULL
                       : strcmp(id, "header_slab") == 0  ? kit.slab
                       : strcmp(id, "header_rule") == 0  ? kit.rule
                       : strcmp(id, "crumb_icon") == 0   ? kit.crumb_icon
                       : strcmp(id, "desc_strip") == 0   ? kit.desc_strip
                       : strcmp(id, "footer_strip") == 0 ? kit.footer
                       : strcmp(id, "footer_mark") == 0  ? kit.footer_mark
                                                         : NULL;
            if (r != NULL) {
                kit_rect(fj_get(n, "verts"), r);
            }
        }
        n = fj_get(K, "slots");
        kit_slot(n, "title", kit.title_anchor, &kit.title_max, kit.title_role);
        kit_slot(n, "breadcrumb", kit.crumb_anchor, &kit.crumb_max, kit.crumb_role);
        kit_slot(n, "description", kit.desc_anchor, &kit.desc_max, kit.desc_role);
        {
            float a[2] = { 0, kit.hint_anchor_y };
            const FjNode* s;
            kit_slot(n, "hint_0", a, &kit.hint_max, kit.hint_role);
            kit.hint_anchor_y = a[1];
            for (s = fj_first(n); s != NULL; s = fj_next(s)) {
                const char* sid = fj_str(fj_get(s, "id"));
                if (sid != NULL && strcmp(sid, "hint_0") == 0) {
                    kit.hint_glyph_y0 = fj_num(fj_get(s, "glyph_y0"), kit.hint_glyph_y0);
                    kit.hint_glyph_h = fj_num(fj_get(s, "glyph_h_1x"), kit.hint_glyph_h);
                }
            }
        }
    }
    if (t != NULL) HSD_Free(t);
    if (p != NULL) HSD_Free(p);

    K = fj_load("list_layout.json", &t, &p);
    if (K != NULL) {
        const FjNode* pn = fj_get(K, "panel");
        const FjNode* row = fj_idx(fj_get(K, "templates"), 0);
        const FjNode* s;
        kit.rows_top = fj_num(fj_get(pn, "rows_top"), kit.rows_top);
        kit.row_h = fj_num(fj_get(pn, "row_h"), kit.row_h);
        kit.pitch = fj_num(fj_get(pn, "pitch"), kit.pitch);
        kit.visible = (int) fj_num(fj_get(pn, "visible"), (float) kit.visible);
        kit.row_x0 = fj_num(fj_get(pn, "x0"), kit.row_x0);
        kit.row_x1 = fj_num(fj_get(pn, "x1"), kit.row_x1);
        for (s = fj_first(fj_get(row, "slots")); s != NULL; s = fj_next(s)) {
            const char* sid = fj_str(fj_get(s, "id"));
            if (sid != NULL && strcmp(sid, "label") == 0) {
                kit.row_label_x = fj_num(fj_idx(fj_get(s, "anchor"), 0), kit.row_label_x);
                kit.row_label_base = fj_num(fj_idx(fj_get(s, "anchor"), 1), kit.row_label_base);
                kit.row_label_max = fj_num(fj_get(s, "max_width_1x"), kit.row_label_max);
                fp_copy(kit.row_role, fj_str(fj_get(s, "role")), 12);
            }
        }
        s = fj_get(fj_get(fj_get(row, "states"), "sel"), "offset");
        kit.row_sel_offset[0] = fj_num(fj_idx(s, 0), kit.row_sel_offset[0]);
        kit.row_sel_offset[1] = fj_num(fj_idx(s, 1), kit.row_sel_offset[1]);
    }
    if (t != NULL) HSD_Free(t);
    if (p != NULL) HSD_Free(p);
    OSReport("frontend: kit %s (%d palette colours)\n", kit.state == 1 ? "loaded" : "MISSING - "
             "using the delivery's values", kit.npal);
}

/* widgets_layout.json's colour names per row state (ng / sel), as kit tokens. */
#define KW_NAMES 11
static const char* const kw_names[KW_NAMES] = { "row",  "text",  "aux",         "track",
                                                "track_text", "fill", "knob", "toggle_knob",
                                                "knob_text",  "plate", "plate_glyph" };
static char kw_tok[3][KW_NAMES][12]; /* ng, sel, disabled */
static int kw_state = 0;

static void kit_widgets_load(void)
{
    void *t, *p;
    const FjNode* W;
    static const char* const st[3] = { "ng", "sel", "disabled" };
    static const char* const def[3][KW_NAMES] = {
        { "@face", "bone", "muted", "ink", "muted", "@face_hi", "bone", "bone", "ink", "ink",
          "bone" },
        { "gold", "ink", "ink", "ink", "gold_dk", "gold_dk", "ink", "gold_lt", "ink", "ink",
          "gold_lt" },
        { "@face", "disabled", "disabled", "ink", "disabled", "disabled", "disabled", "disabled",
          "ink", "ink", "disabled" },
    };
    int i, k;
    if (kw_state != 0) {
        return;
    }
    kw_state = 1;
    for (i = 0; i < 3; i++) {
        for (k = 0; k < KW_NAMES; k++) {
            fp_copy(kw_tok[i][k], def[i][k], 12);
        }
    }
    W = fj_load("widgets_layout.json", &t, &p);
    for (i = 0; W != NULL && i < 3; i++) {
        const FjNode* c = fj_get(fj_get(W, "colours"), st[i]);
        for (k = 0; k < KW_NAMES; k++) {
            const char* v = fj_str(fj_get(c, kw_names[k]));
            if (v != NULL) {
                fp_copy(kw_tok[i][k], v, 12);
            }
        }
    }
    if (t != NULL) HSD_Free(t);
    if (p != NULL) HSD_Free(p);
}

static int kw_section = KS_VERSUS;

/* A widget colour name in a row state (0 ng, 1 sel, 2 disabled) -> rgb. */
static void kit_widget_state_colour(int state, const char* name, float* rgb)
{
    int k;
    for (k = 0; k < KW_NAMES; k++) {
        if (strcmp(kw_names[k], name) == 0) {
            if (!kit_colour(kw_tok[state][k], kw_section, rgb)) {
                rgb[0] = rgb[1] = rgb[2] = 1.0F;
            }
            return;
        }
    }
    rgb[0] = rgb[1] = rgb[2] = 1.0F;
}

/* In a motion key, a widget name ("aux") means the selected row's colour: value changes happen
 * on the row the cursor is on. */
static void kit_widget_colour(const char* name, float* rgb)
{
    kit_widget_state_colour(1, name, rgb);
}

/* kit_motion.json's events into fp's event table, for a screen of `section`. Targets are the
 * list row's ("row", "row_plate", "row_face", "row_label"), rewritten to the player's per-tile
 * names; COLOUR tracks become DIFFUSE_R/G/B with the tokens resolved for the section. Events
 * whose values are expressions the screens here do not use (scroll's "-34*delta") are read
 * with those keys left numeric-zero and are played by code instead. */
static void kit_read_motion(int section)
{
    void *t, *p;
    const FjNode* M = fj_load("kit_motion.json", &t, &p);
    const FjNode* e;
    if (M == NULL) {
        return;
    }
    kit.hold_delay =
        (int) fj_num(fj_get(fj_get(fj_get(M, "sequences"), "hold_repeat"), "delay_frames"), 24);
    kit.hold_interval = (int) fj_num(
        fj_get(fj_get(fj_get(M, "sequences"), "hold_repeat"), "interval_frames"), 4);
    for (e = fj_first(fj_get(M, "events")); e != NULL && fp.nev < FP_MAX_EVENTS; e = fj_next(e)) {
        FpEvent* ev;
        const FjNode* tr;
        if (strcmp(e->key, "row_select") != 0 && strcmp(e->key, "row_deselect") != 0 &&
            strcmp(e->key, "choice_step") != 0 && strcmp(e->key, "slider_step") != 0 &&
            strcmp(e->key, "toggle_set") != 0 && strcmp(e->key, "bump") != 0 &&
            strcmp(e->key, "toast_in") != 0 && strcmp(e->key, "toast_out") != 0)
        {
            continue; /* dialogs, cells, cursors and the stepper: not on a screen yet */
        }
        ev = &fp.ev[fp.nev++];
        fp_copy(ev->name, e->key, 16);
        ev->length = (int) fj_num(fj_get(e, "length_frames"), 1);
        ev->n = 0;
        for (tr = fj_first(fj_get(e, "tracks")); tr != NULL; tr = fj_next(tr)) {
            const char* target = fj_str(fj_get(tr, "target"));
            const char* kind = fj_str(fj_get(tr, "kind"));
            const char* ch = fj_str(fj_get(tr, "channel"));
            /* the kit's widget joints and materials, per row: see gmfrontend_kitlist.inc */
            const char* mapped = target == NULL                    ? NULL
                                 : strcmp(target, "row") == 0       ? "lift_<id>"
                                 : strcmp(target, "row_plate") == 0 ? "plate_<id>"
                                 : strcmp(target, "row_face") == 0  ? "face_<id>"
                                 : strcmp(target, "row_label") == 0 ? "label_<id>"
                                 : strcmp(target, "choice_value_out") == 0 ? "cvo_<id>"
                                 : strcmp(target, "choice_value") == 0     ? "cv_<id>"
                                 : strcmp(target, "choice_arrow_<dir>") == 0 ? "ca<dir>_<id>"
                                 : strcmp(target, "<blocked_arrow>") == 0  ? "ca<dir>_<id>"
                                 : strcmp(target, "<value>") == 0          ? "<value>_<id>"
                                 : strcmp(target, "toggle_knob") == 0      ? "tk_<id>"
                                 : strcmp(target, "slider_fill") == 0      ? "sf_<id>"
                                 : strcmp(target, "slider_knob") == 0      ? "sk_<id>"
                                 : strcmp(target, "toast") == 0            ? "toast"
                                                                           : NULL;
            bool colour = ch != NULL && strcmp(ch, "COLOUR") == 0;
            int c, nc = colour ? 3 : 1;
            if (mapped == NULL) {
                continue;
            }
            for (c = 0; c < nc && ev->n < FP_MAX_TRACKS; c++) {
                FpTrack* k = &ev->t[ev->n++];
                const FjNode* key;
                k->expr_mask = 0;
                fp_copy(k->target, mapped, FP_ID);
                k->is_material = kind != NULL && strcmp(kind, "material") == 0;
                k->channel = colour ? (u8) c : fp_channel(ch, &k->recoil);
                k->recoil = 0;
                k->n = 0;
                for (key = fj_first(fj_get(tr, "keys")); key != NULL && k->n < FP_MAX_KEYS;
                     key = fj_next(key))
                {
                    FpKey* d = &k->k[k->n++];
                    const FjNode* v = fj_get(key, "value");
                    const FjNode* fc = fj_get(key, "from_current");
                    d->frame = (s16) fj_num(fj_get(key, "frame"), 0);
                    d->interp = fp_interp(fj_str(fj_get(key, "interp")));
                    d->slope = fj_num(fj_get(key, "slope"), 0);
                    d->from_current = fc != NULL && fc->type == FJ_TRUE;
                    if (v != NULL && v->type == FJ_STR && !colour) {
                        /* an expression ("60*dir", "value/max"), evaluated when played */
                        k->expr_mask |= (u8) (1 << (k->n - 1));
                        fp_copy(k->expr[k->n - 1], v->s, 24);
                        d->v = 0.0F;
                    } else if (v != NULL && v->type == FJ_STR) {
                        float rgb[3] = { 0, 0, 0 };
                        if (!kit_colour(v->s, section, rgb)) {
                            kit_widget_colour(v->s, rgb); /* "aux" and the other widget names */
                        }
                        d->v = rgb[c];
                    } else {
                        d->v = fj_num(v, 0);
                    }
                }
            }
        }
    }
    if (t != NULL) HSD_Free(t);
    if (p != NULL) HSD_Free(p);
}

/* ---- drawing text and glyph masks --------------------------------------------------------- */

static void fp_mask_state(FpTex* tx)
{
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
    GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_ZERO, GX_CC_ZERO, GX_CC_ZERO, GX_CC_RASC);
    GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_ZERO, GX_CA_TEXA, GX_CA_RASA, GX_CA_ZERO);
    GXSetTevColorOp(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, GX_TRUE, GX_TEVPREV);
    GXSetTevAlphaOp(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, GX_TRUE, GX_TEVPREV);
    GXSetNumChans(1);
    GXSetChanCtrl(GX_COLOR0A0, GX_FALSE, GX_SRC_REG, GX_SRC_VTX, GX_LIGHT_NULL, GX_DF_NONE,
                  GX_AF_NONE);
    GXSetBlendMode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_NOOP);
    GXSetAlphaCompare(GX_ALWAYS, 0, GX_AOP_AND, GX_ALWAYS, 0);
    GXSetZMode(GX_FALSE, GX_ALWAYS, GX_FALSE);
    GXSetCullMode(GX_CULL_NONE);
    GXLoadTexObj(&tx->t.obj, GX_TEXMAP0);
}

/* A text quad set in the atlas: laid out unsheared from its baseline anchor, sheared, then
 * moved by its joint like every other quad. */
static void fp_draw_text(FpQuad* q)
{
    const FfRole* r = &ff.r[q->ff_role];
    int page, n, prev;
    const char* s;
    GXColor c;
    float S = fp.shear_s, Y0 = fp.shear_y0;
    FpJoint* j = q->joint >= 0 ? &fp.j[q->joint] : NULL;
    c.r = fp_u8(q->wrgb[0]);
    c.g = fp_u8(q->wrgb[1]);
    c.b = fp_u8(q->wrgb[2]);
    c.a = fp_u8(q->wa);
    for (page = 0; page < r->npages; page++) {
        int ti = fp_tex(r->page[page]);
        float pen;
        if (ti < 0) {
            continue;
        }
        n = 0;
        prev = -1;
        for (s = q->text; *s != '\0'; s++) {
            int ch = ff_slot(r, (u8) *s);
            if (r->g[ch].page == page && r->g[ch].size[0] > 0.0F) {
                n++;
            }
        }
        if (n == 0) {
            continue;
        }
        fp_mask_state(&fp.tex[ti]);
        GXBegin(GX_QUADS, GX_VTXFMT0, (u16) (4 * n));
        pen = q->ax;
        for (s = q->text; *s != '\0'; s++) {
            int ch = ff_slot(r, (u8) *s);
            const FfGlyph* g = &r->g[ch];
            pen += ff_kern(r, prev, ch);
            prev = ch;
            if (g->page == page && g->size[0] > 0.0F) {
                float x0 = pen + g->off[0], y0 = q->ay + g->off[1];
                float x1 = x0 + g->size[0], y1 = y0 + g->size[1];
                float vx[4], vy[4], u[4], v[4];
                int k;
                vx[0] = x0; vy[0] = y0; u[0] = g->uv[0]; v[0] = g->uv[1];
                vx[1] = x1; vy[1] = y0; u[1] = g->uv[2]; v[1] = g->uv[1];
                vx[2] = x1; vy[2] = y1; u[2] = g->uv[2]; v[2] = g->uv[3];
                vx[3] = x0; vy[3] = y1; u[3] = g->uv[0]; v[3] = g->uv[3];
                for (k = 0; k < 4; k++) {
                    float X = vx[k] + (Y0 - vy[k]) * S, Y = vy[k];
                    if (j != NULL) {
                        X = j->ws[0] * X + j->wo[0];
                        Y = j->ws[1] * Y + j->wo[1];
                    }
                    GXPosition2f32(X, -Y);
                    GXColor4u8(c.r, c.g, c.b, c.a);
                    GXTexCoord2f32(u[k], v[k]);
                }
            }
            pen += g->adv;
        }
    }
    HSD_StateInvalidate(-1);
    hsd_80391A04(1.0F, 1.0F, 1);
}
