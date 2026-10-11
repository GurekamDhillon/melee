/* gw_mex_modfighter.inc - m-ex fighters that arrive as MODS on a disc with NO MxDt.dat (a vanilla disc).
 * Included near the end of gw_mex_ftfunction_runtime.c (it uses that file's statics). Native platform code.
 *
 * WHY. m-ex adds a fighter by rows in ~33 parallel arrays of MxDt.dat. A vanilla disc has none. A Nucleus fighter post
 * (a MexManager zip) carries the rows for ONE fighter (name, Pl/AJ files, costumes, callbacks as clone-base addresses), so
 * the runtime BUILDS the fighter half of mexData from the enabled fighter mods at boot:
 *
 *   mods/<id>/mexfighter.txt   key=value lines (written by the Nucleus installer, tools/nucleus/install_fighter.py shape)
 *   mods/<id>/files/PlXx.dat   the fighter's files (overlaid on the disc like any mod file)
 *
 * WHAT IT BUILDS. A guest-memory image shaped like the fighter-related parts of mexData: root +0x00 metadata (internal /
 * external fighter counts), +0x08 fighter (the parallel arrays), +0x0C fighter_function (46 slot tables), +0x1C item and
 * +0x20/+0x24 kirby (empty but valid). Retail internal ids 0..26 and the six bosses keep the port's compiled-in data (their
 * rows are empty); mod fighters take internal ids 27.. and external ids 26.., then the six boss ids close the table, the way
 * m-ex lays it out. Everything outside the fighter readers (sound banks, effect banks, music, menu, CSS icons, stages, items)
 * sees "no mexData" exactly as on a vanilla disc: gw_Mex_MexData() returns 0 for this image and gw_Mex_CssIconCount() is 0.
 * A disc that has a real MxDt.dat never uses it (merging into ACE's tables is not supported).
 *
 * CALLBACKS. A callback is a guest address from the zip. The consumer (gw_Mex_FtFunc) resolves it through the bridge when
 * it is retail code; one that does not resolve is logged and left empty (the clone base's callback is used), never jumped
 * to. Addresses inside the fighter's own code blob are installed from the Pl file by the existing ftFunction path.
 */

#include "gw_mods.h"

#define MF_MAX GW_MEX_SLOTS
#define MF_COST_MAX 32
#define MF_NCB 46 /* Arch_FighterFunc slots (gw_mex_ftfunction.c slot_names) */

typedef struct mf_fighter {
    char mod[64], name[48], pl[40], plsym[64], anim[40], post[96];
    char cfile[MF_COST_MAX][40], cjoint[MF_COST_MAX][64], cmat[MF_COST_MAX][64];
    int cvis[MF_COST_MAX];
    char demo[4][64];
    int ncost, animcount, effect, announcer, victory, bgm[2], base, ssm;
    uint32_t cb[MF_NCB];
    int ext, internal;
} mf_fighter;

static mf_fighter mf_list[MF_MAX];
static int mf_n, mf_scanned;

static uint32_t mf_hex(const char *s) { return (uint32_t) strtoul(s, NULL, 0); }

/* Split "a|b|c" into at most n fields (empty ones stay empty). */
static void mf_fields(char *s, char **f, int n) {
    int i = 0;
    while (i < n) {
        f[i++] = s;
        s = strchr(s, '|');
        if (s == NULL) break;
        *s++ = '\0';
    }
    while (i < n) f[i++] = (char *) "";
}

static void mf_copy(char *dst, size_t cap, const char *src) { snprintf(dst, cap, "%s", src); }

/* Parse one mexfighter.txt body. Returns 1 when it names a fighter (a name and a Pl file). */
static int mf_parse(const char *text, mf_fighter *o) {
    const char *p = text;
    memset(o, 0, sizeof *o);
    o->effect = 255;
    o->announcer = o->victory = -1;
    o->bgm[0] = o->bgm[1] = 0xFFFF;
    o->base = -1;
    o->ssm = 255;
    while (*p) {
        char line[512], *eq;
        size_t n = 0;
        while (p[n] && p[n] != '\n' && p[n] != '\r') ++n;
        if (n >= sizeof line) n = sizeof line - 1;
        memcpy(line, p, n);
        line[n] = '\0';
        p += n;
        while (*p == '\n' || *p == '\r') ++p;
        eq = strchr(line, '=');
        if (eq == NULL || line[0] == '#') continue;
        *eq++ = '\0';
        if (!strcmp(line, "name")) mf_copy(o->name, sizeof o->name, eq);
        else if (!strcmp(line, "pl")) mf_copy(o->pl, sizeof o->pl, eq);
        else if (!strcmp(line, "plsym")) mf_copy(o->plsym, sizeof o->plsym, eq);
        else if (!strcmp(line, "anim")) mf_copy(o->anim, sizeof o->anim, eq);
        else if (!strcmp(line, "post")) mf_copy(o->post, sizeof o->post, eq);
        else if (!strcmp(line, "animcount")) o->animcount = atoi(eq);
        else if (!strcmp(line, "effect")) o->effect = atoi(eq);
        else if (!strcmp(line, "announcer")) o->announcer = atoi(eq);
        else if (!strcmp(line, "victory")) o->victory = atoi(eq);
        else if (!strcmp(line, "base")) o->base = atoi(eq);
        else if (!strcmp(line, "bgm")) {
            char *q = strchr(eq, ',');
            o->bgm[0] = atoi(eq);
            o->bgm[1] = q ? atoi(q + 1) : 0xFFFF;
        } else if (!strcmp(line, "demo")) {
            char *f[4], buf[300];
            int i;
            mf_copy(buf, sizeof buf, eq);
            mf_fields(buf, f, 4);
            for (i = 0; i < 4; ++i) mf_copy(o->demo[i], sizeof o->demo[i], f[i]);
        } else if (line[0] == 'c' && line[1] >= '0' && line[1] <= '9') { /* c<i>=file|joint|matanim|vis */
            int i = atoi(line + 1);
            if (i >= 0 && i < MF_COST_MAX) {
                char *f[4], buf[300];
                mf_copy(buf, sizeof buf, eq);
                mf_fields(buf, f, 4);
                mf_copy(o->cfile[i], sizeof o->cfile[i], f[0]);
                mf_copy(o->cjoint[i], sizeof o->cjoint[i], f[1]);
                mf_copy(o->cmat[i], sizeof o->cmat[i], f[2]);
                o->cvis[i] = f[3][0] ? atoi(f[3]) : i;
                if (i + 1 > o->ncost) o->ncost = i + 1;
            }
        } else if (line[0] == 'c' && line[1] == 'b' && line[2] >= '0' && line[2] <= '9') { /* cb<slot>=0xADDR */
            int s = atoi(line + 2);
            if (s >= 0 && s < MF_NCB) o->cb[s] = mf_hex(eq);
        }
    }
    if (o->name[0] == '\0' || o->pl[0] == '\0' || strncmp(o->pl, "Pl", 2) != 0 || strchr(o->pl, '/') != NULL) return 0;
    if (o->ncost == 0) { /* a fighter needs one costume: the Pl file itself, symbols as the clone base's naming */
        mf_copy(o->cfile[0], sizeof o->cfile[0], o->pl);
        o->ncost = 1;
    }
    return 1;
}

static void mf_scan(void) {
    int n, k;
    if (mf_scanned) return;
    mf_scanned = 1;
    mf_n = 0;
    for (n = 0; n < gw_Mods_ActiveCount() && mf_n < MF_MAX; ++n) {
        int i = gw_Mods_ActiveAt(n);
        char dir[260], path[300];
        FILE *f;
        char text[8192];
        size_t got, len;
        if (strcmp(gw_Mods_Kind(i), "fighter") != 0) continue;
        snprintf(dir, sizeof dir, "%s", gw_Mods_PayloadDir(i));
        if (!gw_Mods_PayloadIsModDir(i)) {
            len = strlen(dir);
            while (len > 0 && (dir[len - 1] == '/' || dir[len - 1] == '\\')) dir[--len] = '\0';
            while (len > 0 && dir[len - 1] != '/' && dir[len - 1] != '\\') dir[--len] = '\0';
        }
        snprintf(path, sizeof path, "%smexfighter.txt", dir);
        f = fopen(path, "rb");
        if (f == NULL) continue;
        got = fread(text, 1, sizeof text - 1, f);
        fclose(f);
        text[got] = '\0';
        if (!mf_parse(text, &mf_list[mf_n])) {
            gw_log("modfighter: %s: mexfighter.txt does not name a fighter (needs name= and pl=PlXx.dat) - skipped", gw_Mods_Id(i));
            continue;
        }
        snprintf(mf_list[mf_n].mod, sizeof mf_list[mf_n].mod, "%s", gw_Mods_Id(i));
        ++mf_n;
    }
    if (n < gw_Mods_ActiveCount() && mf_n == MF_MAX) {
        gw_log("modfighter: ERROR no m-ex fighter slot left (the port holds %d): further fighter mods are left out", MF_MAX);
    }
    for (k = 0; k < mf_n; ++k) {
        mf_list[k].internal = GW_MEX_FIRST_NEW + k;
        mf_list[k].ext = 26 + k;
        gw_log("modfighter: fighter %d (ext %d) \"%s\" %s from %s", mf_list[k].internal, mf_list[k].ext, mf_list[k].name, mf_list[k].pl, mf_list[k].mod);
    }
}

/* ---- the public view (host data only) ---- */
int gw_ModFighter_Count(void) { uint32_t b, s; if (gw_Mex_MexData(&b, &s) != 0u) return 0; mf_scan(); return mf_n; }
const char *gw_ModFighter_Name(int i) { return i >= 0 && i < gw_ModFighter_Count() ? mf_list[i].name : ""; }
const char *gw_ModFighter_File(int i) { return i >= 0 && i < gw_ModFighter_Count() ? mf_list[i].pl : ""; }
const char *gw_ModFighter_Mod(int i) { return i >= 0 && i < gw_ModFighter_Count() ? mf_list[i].mod : ""; }
const char *gw_ModFighter_Post(int i) { return i >= 0 && i < gw_ModFighter_Count() ? mf_list[i].post : ""; }
int gw_ModFighter_Internal(int i) { return i >= 0 && i < gw_ModFighter_Count() ? mf_list[i].internal : -1; }
/* The port CharacterKind of mod fighter i (its slot is allocated when its Pl file is on the disc or in a mod), or -1. */
int gw_ModFighter_CKAt(int i) {
    extern int gw_Mex_ExtToPortCKind(int ext);
    int n = gw_ModFighter_Count();
    if (i < 0 || i >= n) return -1;
    (void) gw_Mex_SlotInternal(0); /* builds the slot table */
    return gw_Mex_ExtToPortCKind(mf_list[i].ext);
}
/* 1 when the fighter-table image is the synthetic one (no MxDt.dat on the disc) */
int gw_Mex_FtSynth(void) { return gw_mexdt_synth && gw_mexdt != 0u; }

/* ---- the image ---- */
static uint32_t mf_at, mf_end;
static void mf_w32(uint32_t at, uint32_t v) { gw_w32((void *) (uintptr_t) at, v); }
static uint32_t mf_alloc(uint32_t n, uint32_t align) {
    uint32_t a = (mf_at + align - 1u) & ~(align - 1u);
    if (a + n > mf_end) return 0u;
    mf_at = a + n;
    return a;
}
static uint32_t mf_str(const char *s) {
    size_t l = strlen(s) + 1;
    uint32_t a;
    if (s[0] == '\0') return 0u;
    a = mf_alloc((uint32_t) l, 1u);
    if (a != 0u) memcpy((void *) (uintptr_t) a, s, l);
    return a;
}

/* Build the image into the persistent region and make it the current mexData (fighter readers only). Returns 1 when built. */
static int gw_mf_build(void) {
    uint32_t total, base, root, meta, fighter, ff, menu, item, kirby, kfun, ncost_tbl;
    uint32_t names, pl, insig, desc, cinfo, demo, animf, animn, effi, resf, ress, vict, ann, ssm, itl, bgm;
    int I, k, j, c;
    extern void *gw_Mex_PersistAlloc(uint32_t size);
    if (gw_mexdt != 0u) return 0;
    mf_scan();
    if (mf_n <= 0) return 0;
    I = GW_MEX_FIRST_NEW + mf_n + 6;
    total = 0x2000u + (uint32_t) I * 640u + (uint32_t) mf_n * 12288u;
    base = (uint32_t) (uintptr_t) gw_Mex_PersistAlloc(total);
    mf_at = base;
    mf_end = base + total;
    root = mf_alloc(0x40u, 4u);
    meta = mf_alloc(0x40u, 4u);
    menu = mf_alloc(0x40u, 4u);
    item = mf_alloc(0x40u, 4u);
    kirby = mf_alloc(0x40u, 4u);
    kfun = mf_alloc(0x40u, 4u);
    fighter = mf_alloc(0x80u, 4u);
    ff = mf_alloc(MF_NCB * 4u, 4u);
    names = mf_alloc((uint32_t) I * 4u, 4u);
    pl = mf_alloc((uint32_t) I * 8u, 4u);
    insig = mf_alloc((uint32_t) I, 4u);
    desc = mf_alloc((uint32_t) I * 3u, 4u);
    cinfo = mf_alloc((uint32_t) I * 4u, 4u);
    ncost_tbl = mf_alloc((uint32_t) I * 4u, 4u);
    demo = mf_alloc((uint32_t) I * 4u, 4u);
    animf = mf_alloc((uint32_t) I * 4u, 4u);
    animn = mf_alloc((uint32_t) I * 8u, 4u);
    effi = mf_alloc((uint32_t) I, 4u);
    resf = mf_alloc((uint32_t) I * 4u, 4u);
    ress = mf_alloc((uint32_t) I * 4u, 4u);
    vict = mf_alloc((uint32_t) I * 4u, 4u);
    ann = mf_alloc((uint32_t) I * 4u, 4u);
    ssm = mf_alloc((uint32_t) I * 0x10u, 4u);
    itl = mf_alloc((uint32_t) I * 8u, 4u);
    bgm = mf_alloc((uint32_t) I * 4u, 4u);
    if (root == 0u || bgm == 0u) { gw_log("modfighter: image allocation failed"); return 0; }
    mf_w32(root + 0x00u, meta);
    mf_w32(root + 0x04u, menu);
    mf_w32(root + 0x08u, fighter);
    mf_w32(root + 0x0Cu, ff);
    mf_w32(root + 0x1Cu, item);
    mf_w32(root + 0x20u, kirby);
    mf_w32(root + 0x24u, kfun);
    mf_w32(meta + 0x04u, (uint32_t) I); /* internal ids */
    mf_w32(meta + 0x08u, (uint32_t) I); /* external ids */
    mf_w32(fighter + 0x00u, names);
    mf_w32(fighter + 0x04u, pl);
    mf_w32(fighter + 0x08u, insig);
    mf_w32(fighter + 0x0Cu, desc);
    mf_w32(fighter + 0x10u, cinfo);
    mf_w32(fighter + 0x14u, ncost_tbl);
    mf_w32(fighter + 0x18u, demo);
    mf_w32(fighter + 0x1Cu, animf);
    mf_w32(fighter + 0x20u, animn);
    mf_w32(fighter + 0x24u, effi);
    mf_w32(fighter + 0x28u, resf);
    mf_w32(fighter + 0x2Cu, ress);
    mf_w32(fighter + 0x30u, vict);
    mf_w32(fighter + 0x34u, ann);
    mf_w32(fighter + 0x38u, ssm);
    mf_w32(fighter + 0x4Cu, itl);
    mf_w32(fighter + 0x54u, bgm);
    /* unused external rows resolve to nothing (255), not to Mario; retail ext 0..25 map to themselves */
    for (j = 0; j < I; ++j) {
        ((uint8_t *) (uintptr_t) desc)[j * 3] = (uint8_t) (j <= 25 ? j : 0xFF);
        ((uint8_t *) (uintptr_t) effi)[j] = 255;
        ((uint8_t *) (uintptr_t) ssm)[j * 0x10] = 255;
        mf_w32(vict + (uint32_t) j * 4u, 0xFFFFFFFFu);
        mf_w32(ann + (uint32_t) j * 4u, 0xFFFFFFFFu);
        mf_w32(ress + (uint32_t) j * 4u, 0x3F800000u);
        mf_w32(bgm + (uint32_t) j * 4u, 0xFFFFFFFFu);
    }
    for (j = 0; j < MF_NCB; ++j) {
        uint32_t tbl = mf_alloc((uint32_t) I * 4u, 4u);
        if (tbl == 0u) { gw_log("modfighter: image allocation failed (callbacks)"); return 0; }
        mf_w32(ff + (uint32_t) j * 4u, tbl);
        for (k = 0; k < mf_n; ++k) mf_w32(tbl + (uint32_t) mf_list[k].internal * 4u, mf_list[k].cb[j]);
    }
    for (k = 0; k < mf_n; ++k) {
        mf_fighter *m = &mf_list[k];
        uint32_t cf, dm, row;
        int e = m->ext, in = m->internal;
        mf_w32(names + (uint32_t) e * 4u, mf_str(m->name));
        mf_w32(pl + (uint32_t) in * 8u, mf_str(m->pl));
        mf_w32(pl + (uint32_t) in * 8u + 4u, mf_str(m->plsym));
        ((uint8_t *) (uintptr_t) desc)[e * 3] = (uint8_t) in;
        ((uint8_t *) (uintptr_t) cinfo)[e * 4] = (uint8_t) m->ncost;
        ((uint8_t *) (uintptr_t) cinfo)[e * 4 + 1] = (uint8_t) (m->ncost > 1 ? 1 : 0);
        ((uint8_t *) (uintptr_t) cinfo)[e * 4 + 2] = (uint8_t) (m->ncost > 2 ? 2 : 0);
        ((uint8_t *) (uintptr_t) cinfo)[e * 4 + 3] = (uint8_t) (m->ncost > 3 ? 3 : 0);
        cf = mf_alloc((uint32_t) m->ncost * 16u, 4u);
        dm = mf_alloc(16u, 4u);
        if (cf == 0u || dm == 0u) { gw_log("modfighter: image allocation failed (%s)", m->mod); return 0; }
        for (c = 0; c < m->ncost; ++c) {
            row = cf + (uint32_t) c * 16u;
            mf_w32(row + 0u, mf_str(m->cfile[c]));
            mf_w32(row + 4u, mf_str(m->cjoint[c]));
            mf_w32(row + 8u, mf_str(m->cmat[c]));
            mf_w32(row + 12u, (uint32_t) m->cvis[c]);
        }
        mf_w32(ncost_tbl + (uint32_t) in * 4u, cf);
        for (c = 0; c < 4; ++c) mf_w32(dm + (uint32_t) c * 4u, mf_str(m->demo[c]));
        mf_w32(demo + (uint32_t) in * 4u, dm);
        mf_w32(animf + (uint32_t) in * 4u, mf_str(m->anim));
        mf_w32(animn + (uint32_t) in * 8u + 4u, (uint32_t) m->animcount);
        ((uint8_t *) (uintptr_t) effi)[in] = (uint8_t) m->effect;
        mf_w32(vict + (uint32_t) e * 4u, (uint32_t) m->victory);
        mf_w32(ann + (uint32_t) e * 4u, (uint32_t) m->announcer);
        mf_w32(bgm + (uint32_t) e * 4u, m->bgm[0] == 0xFFFF && m->bgm[1] == 0xFFFF ? 0xFFFFFFFFu
                                                                                 : ((uint32_t) (m->bgm[0] & 0xFFFF) << 16) | (uint32_t) (m->bgm[1] & 0xFFFF));
    }
    gw_mexdt = root;
    gw_mexdt_base = base;
    gw_mexdt_size = total;
    gw_mexdt_synth = 1;
    gw_log("modfighter: %d fighter mod(s) -> internal %d..%d, image @ 0x%08X (%u bytes of %u)", mf_n, GW_MEX_FIRST_NEW,
           GW_MEX_FIRST_NEW + mf_n - 1, root, mf_at - base, total);
    return 1;
}

/* Which callback slots a fighter mod gave an address that does NOT resolve through the bridge (and lies outside the code blob it ships
 * is not known here): the count, for the log and the tests. */
int gw_ModFighter_UnresolvedCallbacks(int i, char *out, int cap) {
    int s, n = 0;
    if (out != NULL && cap > 0) out[0] = '\0';
    if (i < 0 || i >= gw_ModFighter_Count()) return 0;
    for (s = 0; s < MF_NCB; ++s) {
        uint32_t g = mf_list[i].cb[s];
        int kind = 0;
        if (g == 0u) continue;
        if (gw_mex_bridge_lookup(g, &kind) == 0u) {
            ++n;
            if (out != NULL && (int) strlen(out) + 12 < cap) {
                size_t l = strlen(out);
                snprintf(out + l, (size_t) cap - l, "%s%d", l ? "," : "", s);
            }
        }
    }
    return n;
}

/* The retail fighter this mod fighter was cloned from: the descriptor's base=, else the retail kind whose onLoad its onLoad callback is
 * (the port's ftData_OnLoad[r] == the bridge's native for the zip's address), else Mario. Retail ids are their own base. */
static int gw_mf_base_kind(int k) {
    extern int gw_ftData_PcRetailKindOfOnLoad(uint32_t native);
    int i;
    if (k >= 0 && k <= 26) return k;
    for (i = 0; i < mf_n; ++i) {
        if (mf_list[i].internal == k) {
            uint32_t nat;
            int r;
            if (mf_list[i].base >= 0 && mf_list[i].base <= 26) return mf_list[i].base;
            nat = mf_list[i].cb[0] != 0u ? gw_mex_bridge_lookup(mf_list[i].cb[0], NULL) : 0u;
            r = nat != 0u ? gw_ftData_PcRetailKindOfOnLoad(nat) : -1;
            return r >= 0 ? r : 0;
        }
    }
    return 0;
}

/* The persistent region was rewound: the image is gone, the descriptors stay. */
static void gw_mf_invalidate(void) { gw_mexdt_synth = 0; }

/* ---- tests ---- */
static int test_modfighter_parse(void) {
    mf_fighter f;
    const char *ok = "name=Test Guy\npl=PlZz.dat\nplsym=ftDataZz\nanim=PlZzAJ.dat\nanimcount=316\nc0=PlZzNr.dat|J0|M0|0\nc1=PlZzRe.dat|J0|M0|0\n"
                     "cb0=0x8014f6b8\ncb3=0x803d35e8\ncb45=1\ncb99=7\ndemo=a|b|c|d\nannouncer=510008\nvictory=22\nbgm=98,97\nbase=11\n";
    if (!mf_parse(ok, &f)) { gw_test_fail("a complete mexfighter.txt must parse"); return 1; }
    if (strcmp(f.name, "Test Guy") || strcmp(f.pl, "PlZz.dat") || f.animcount != 316 || f.ncost != 2) { gw_test_fail("fields wrong: %s %s %d %d", f.name, f.pl, f.animcount, f.ncost); return 1; }
    if (f.cb[0] != 0x8014F6B8u || f.cb[3] != 0x803D35E8u || f.cb[45] != 1u) { gw_test_fail("callbacks wrong: %08X %08X %08X", f.cb[0], f.cb[3], f.cb[45]); return 1; }
    if (f.victory != 22 || f.bgm[0] != 98 || f.bgm[1] != 97 || f.base != 11 || strcmp(f.demo[3], "d")) { gw_test_fail("misc fields wrong"); return 1; }
    if (mf_parse("name=No file\n", &f) || mf_parse("pl=PlZz.dat\n", &f) || mf_parse("name=x\npl=../PlZz.dat\n", &f) || mf_parse("name=x\npl=GrZz.dat\n", &f) ||
        mf_parse("", &f) || mf_parse("garbage\n=\n\xff\xfe", &f)) {
        gw_test_fail("a descriptor without a name, a Pl file, with a path or garbage must be refused");
        return 1;
    }
    return 0;
}

/* Build an image from two synthetic descriptors and read it back through the REAL accessors. */
static int test_modfighter_image(void) {
    uint32_t sr = gw_mexdt, sb = gw_mexdt_base, ss = gw_mexdt_size;
    int sy = gw_mexdt_synth, sn = mf_n, sc = mf_scanned, rc = 0;
    mf_fighter save[2];
    memcpy(save, mf_list, sizeof save);
    gw_mexdt = 0u;
    gw_mexdt_synth = 0;
    mf_scanned = 1;
    mf_n = 2;
    mf_parse("name=Alpha\npl=PlAa.dat\nplsym=ftDataAa\nanim=PlAaAJ.dat\nanimcount=300\nc0=PlAaNr.dat|JA|MA|0\nc1=PlAaRe.dat|JA|MA|0\ncb0=0x8014f6b8\nannouncer=7\n", &mf_list[0]);
    mf_parse("name=Beta\npl=PlBb.dat\nplsym=ftDataBb\nanim=PlBbAJ.dat\nanimcount=310\nbase=2\n", &mf_list[1]);
    mf_list[0].internal = 27; mf_list[0].ext = 26;
    mf_list[1].internal = 28; mf_list[1].ext = 27;
    if (!gw_mf_build()) { gw_test_fail("the image did not build"); rc = 1; goto out; }
    if (gw_Mex_InternalCount() != 27 + 2 + 6) { gw_test_fail("internal count %d, not 35", gw_Mex_InternalCount()); rc = 1; }
    if (strcmp(gw_Mex_FtPlFile(27), "PlAa.dat") || strcmp(gw_Mex_FtPlFile(28), "PlBb.dat") || gw_Mex_FtPlFile(0) != NULL) { gw_test_fail("Pl rows wrong"); rc = 1; }
    if (strcmp(gw_Mex_FtPlSymbol(27), "ftDataAa") || strcmp(gw_Mex_FtAnimFile(28), "PlBbAJ.dat") || gw_Mex_FtAnimCount(27) != 300 || gw_Mex_FtAnimCount(28) != 310) { gw_test_fail("symbol/anim rows wrong"); rc = 1; }
    if (gw_Mex_FtCostumeCount(27) != 2 || gw_Mex_FtCostumeCount(28) != 1 || gw_Mex_FtCostumeCount(3) != 0) { gw_test_fail("costume counts wrong: %d %d", gw_Mex_FtCostumeCount(27), gw_Mex_FtCostumeCount(28)); rc = 1; }
    if (strcmp(gw_Mex_FtCostumeString(27, 1, 0), "PlAaRe.dat") || strcmp(gw_Mex_FtCostumeString(27, 0, 1), "JA")) { gw_test_fail("costume strings wrong"); rc = 1; }
    if (gw_Mex_ExtForInternal(27) != 26 || gw_Mex_InternalForExt(27) != 28 || gw_Mex_InternalForExt(5) != 5 || strcmp(gw_Mex_FighterName(26), "Alpha")) { gw_test_fail("ext/internal maps wrong"); rc = 1; }
    if (gw_Mex_FtBaseKind(28) != 2 || gw_Mex_FtBaseKind(4) != 4) { gw_test_fail("base kinds wrong: %d", gw_Mex_FtBaseKind(28)); rc = 1; }
    if (gw_Mex_MexData(NULL, NULL) != 0u || gw_Mex_CssIconCount() != 0) { gw_test_fail("a synthetic image must not look like a disc's MxDt.dat"); rc = 1; }
out:
    memcpy(mf_list, save, sizeof save);
    gw_mexdt = sr; gw_mexdt_base = sb; gw_mexdt_size = ss; gw_mexdt_synth = sy; mf_n = sn; mf_scanned = sc;
    return rc;
}

void gw_modfighter_tests_register(void) {
    gw_test_register("modfighter_parse", test_modfighter_parse);
    gw_test_register("modfighter_image", test_modfighter_image);
}
