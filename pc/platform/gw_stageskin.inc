/* gw_stageskin.inc - stage skins (gw_stageskin.h has the design). Included at the end of gw_mods.c, which
 * keeps the mods registry's statics (set_load, the test helpers) the conflict test and the model use.
 *
 * Three layers, bottom up:
 *   1. pure model: the retail table, which file belongs to which stage, how a stored setting becomes a
 *      choice, which provider serves a path for a choice. No game state; the native tests cover it.
 *   2. the skin list of a stage (mounted stage-kind mods that provide its files, each with a content
 *      digest) and the player's choice (settings.cfg skin_<ext>, MELEE_SKIN_<ext>).
 *   3. applying a choice at a stage load by telling the overlay (shim_dvd.c) which provider serves the
 *      stage's files, and the one-shot match override (scene key skin=) the online host and replays use.
 *
 * NATIVE platform code (i686 clang), not gwtool. */
#include "gw_stageskin.h"

extern int gw_Settings_Str(const char *key, char *out, int cap, const char *dflt);
extern void gw_Settings_SetStr(const char *key, const char *value);
/* shim_dvd.c: the overlay keeps every provider of a path */
extern int gw_DVDStageProviders(int r, int *mods, int cap);
extern int gw_DVDStageFile(int r, int mod_index, int n, char *disc, int dcap, char *host, int hcap);
extern int gw_DVDSelectStage(int r, int want);

/* ---- 1. the pure model ------------------------------------------------------------------------- */

/* The retail VS stages by external stage id and file (a stem when the game adds a suffix). gw_mexid.c lists
 * their identities from this table; a skin is a mod that provides one of these stages' files. */
static const struct {
    int ext;
    const char *file, *name;
} sk_retail[] = {
    { 2, "GrIz.dat", "Fountain of Dreams" }, { 3, "GrPs", "Pokemon Stadium" },
    { 4, "GrCs.dat", "Princess Peach's Castle" }, { 5, "GrKg.dat", "Kongo Jungle" },
    { 6, "GrZe.dat", "Brinstar" }, { 7, "GrCn", "Corneria" }, { 8, "GrSt.dat", "Yoshi's Story" },
    { 9, "GrOt", "Onett" }, { 10, "GrMc.dat", "Mute City" }, { 11, "GrRc.dat", "Rainbow Cruise" },
    { 12, "GrGd.dat", "Jungle Japes" }, { 13, "GrGb.dat", "Great Bay" }, { 14, "GrSh.dat", "Temple" },
    { 15, "GrKr.dat", "Brinstar Depths" }, { 16, "GrYt.dat", "Yoshi's Island" },
    { 17, "GrGr.dat", "Green Greens" }, { 18, "GrFs.dat", "Fourside" },
    { 19, "GrI1.dat", "Mushroom Kingdom" }, { 20, "GrI2.dat", "Mushroom Kingdom II" },
    { 22, "GrVe", "Venom" }, { 23, "GrPu.dat", "Poke Floats" }, { 24, "GrBb.dat", "Big Blue" },
    { 25, "GrIm.dat", "Icicle Mountain" }, { 27, "GrFz.dat", "Flat Zone" },
    { 28, "GrOp.dat", "Dream Land" }, { 29, "GrOy.dat", "Yoshi's Island (64)" },
    { 30, "GrOk.dat", "Kongo Jungle (64)" }, { 31, "GrNBa.dat", "Battlefield" },
    { 32, "GrNLa.dat", "Final Destination" },
};
#define SK_NRETAIL ((int) (sizeof sk_retail / sizeof sk_retail[0]))

int gw_Skin_RetailCount(void) { return SK_NRETAIL; }
int gw_Skin_RetailExt(int r) { return r >= 0 && r < SK_NRETAIL ? sk_retail[r].ext : -1; }
const char *gw_Skin_RetailFile(int r) { return r >= 0 && r < SK_NRETAIL ? sk_retail[r].file : ""; }
const char *gw_Skin_RetailName(int r) { return r >= 0 && r < SK_NRETAIL ? sk_retail[r].name : ""; }
int gw_Skin_RetailForExt(int ext) {
    int r;
    for (r = 0; r < SK_NRETAIL; ++r) {
        if (sk_retail[r].ext == ext) return r;
    }
    return -1;
}

/* <stem>[digits].<letters>, at the payload root: GrPs.dat, GrPs.usd, GrPs1.dat. GrI1.dat is Mushroom Kingdom's own stem. */
int gw_Skin_FileRetail(const char *path) {
    int r;
    if (path == NULL) return -1;
    while (*path == '/' || *path == '\\') ++path;
    if (strchr(path, '/') != NULL || strchr(path, '\\') != NULL) return -1;
    for (r = 0; r < SK_NRETAIL; ++r) {
        size_t n = strlen(sk_retail[r].file);
        const char *p;
        if (n > 4 && _stricmp(sk_retail[r].file + n - 4, ".dat") == 0) n -= 4;
        if (_strnicmp(path, sk_retail[r].file, n) != 0) continue;
        p = path + n;
        while (*p >= '0' && *p <= '9') ++p;
        if (*p != '.' || p[1] == '\0') continue;
        for (++p; (*p >= 'a' && *p <= 'z') || (*p >= 'A' && *p <= 'Z'); ++p) {
        }
        if (*p == '\0') return r;
    }
    return -1;
}

/* A stage's archive name as the StageData row spells it ("/GrPs", the game adds .dat / .usd; or "GrSt.dat"): its retail row, or -1. */
int gw_Skin_NameRetail(const char *name) {
    int r;
    if (name == NULL) return -1;
    while (*name == '/' || *name == '\\') ++name;
    for (r = 0; r < SK_NRETAIL; ++r) {
        size_t n = strlen(sk_retail[r].file);
        if (n > 4 && _stricmp(sk_retail[r].file + n - 4, ".dat") == 0) n -= 4;
        if (_strnicmp(name, sk_retail[r].file, n) == 0 && (name[n] == '\0' || _stricmp(name + n, ".dat") == 0)) return r;
    }
    return -1;
}

uint32_t gw_Skin_PayloadMask(const char *dir) {
    char pattern[MAX_PATH];
    WIN32_FIND_DATAA fd;
    HANDLE h;
    uint32_t mask = 0;
    if (dir == NULL || dir[0] == '\0') return 0;
    snprintf(pattern, sizeof pattern, "%s\\*", dir);
    h = FindFirstFileA(pattern, &fd);
    if (h == INVALID_HANDLE_VALUE) return 0;
    do {
        int r;
        if (fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) continue;
        r = gw_Skin_FileRetail(fd.cFileName);
        if (r >= 0) mask |= 1u << r;
    } while (FindNextFileA(h, &fd));
    FindClose(h);
    return mask;
}

int gw_Skin_Resolve(const char *setting, const char *const *ids, int n) {
    int k;
    if (setting != NULL && _stricmp(setting, "orig") == 0) return 0;
    if (setting != NULL && setting[0] != '\0' && _stricmp(setting, "auto") != 0) {
        for (k = 0; k < n; ++k) {
            if (ids[k] != NULL && _stricmp(ids[k], setting) == 0) return k + 1;
        }
    }
    return n; /* auto: the last mounted skin (0 = Original when there is none) */
}

int gw_Skin_Step(int cur, int n, int dir) {
    int c = n + 1;
    if (c <= 1) return 0;
    cur = ((cur % c) + c) % c;
    return ((cur + (dir < 0 ? -1 : 1)) % c + c) % c;
}

int gw_skin_select(const gw_skin_prov *prov, int nprov, int on_disc, int want) {
    int i, orig = -1;
    if (nprov <= 0) return -1;
    if (want == GW_SKIN_DEFAULT) return nprov - 1;
    for (i = 0; i < nprov; ++i) {
        if (!prov[i].is_skin) orig = i;
    }
    if (want >= 0) {
        for (i = 0; i < nprov; ++i) {
            if (prov[i].is_skin && prov[i].mod_index == want) return i;
        }
    }
    if (orig >= 0) return orig;
    return on_disc ? -1 : nprov - 1;
}

/* ---- 2. skins and the choice -------------------------------------------------------------------- */

typedef struct sk_skin {
    int mod_index;
    char id[GW_MODS_ID_MAX];
    char name[GW_MODS_TEXT_MAX];
    uint64_t digest;
} sk_skin;

static struct {
    int built;
    int n;
    sk_skin s[GW_SKIN_MAX];
} sk_by_r[GW_SKIN_RETAIL_MAX];

#define SK_MASK48 0xFFFFFFFFFFFFull

/* FNV-1a over the bytes, a 64-bit digest of the skin's stage files (names, sizes, content). */
static uint64_t sk_fnv(uint64_t h, const unsigned char *p, size_t n) {
    size_t i;
    for (i = 0; i < n; ++i) {
        h ^= p[i];
        h *= 0x100000001B3ull;
    }
    return h;
}

static uint64_t sk_digest(int r, int mod_index) {
    uint64_t h = 0xCBF29CE484222325ull;
    char disc[96], host[MAX_PATH];
    int n;
    for (n = 0; gw_DVDStageFile(r, mod_index, n, disc, sizeof disc, host, sizeof host); ++n) {
        FILE *f = fopen(host, "rb");
        unsigned char buf[16384];
        size_t got, total = 0;
        char *c;
        for (c = disc; *c; ++c) {
            if (*c >= 'A' && *c <= 'Z') *c = (char) (*c - 'A' + 'a');
        }
        h = sk_fnv(h, (const unsigned char *) disc, strlen(disc) + 1);
        if (f == NULL) continue;
        while ((got = fread(buf, 1, sizeof buf, f)) > 0) {
            h = sk_fnv(h, buf, got);
            total += got;
        }
        fclose(f);
        h = sk_fnv(h, (const unsigned char *) &total, sizeof total);
    }
    return h != 0 ? h : 1;
}

static void sk_build(int r) {
    int mods[GW_SKIN_MAX], n, i;
    if (r < 0 || r >= SK_NRETAIL || sk_by_r[r].built) return;
    sk_by_r[r].built = 1;
    sk_by_r[r].n = 0;
    n = gw_DVDStageProviders(r, mods, GW_SKIN_MAX);
    for (i = 0; i < n; ++i) {
        sk_skin *s = &sk_by_r[r].s[sk_by_r[r].n++];
        s->mod_index = mods[i];
        copy_str(s->id, sizeof s->id, gw_Mods_Id(mods[i]));
        copy_str(s->name, sizeof s->name, gw_Mods_Name(mods[i]));
        s->digest = sk_digest(r, mods[i]);
        gw_log("stageskin: %s skin %d: %s (%s) digest %012llx", sk_retail[r].file, i + 1, s->name, s->id,
               (unsigned long long) (s->digest & SK_MASK48));
    }
}

int gw_StageSkin_Count(int ext) {
    int r = gw_Skin_RetailForExt(ext);
    if (r < 0) return 0;
    sk_build(r);
    return sk_by_r[r].n;
}
const char *gw_StageSkin_Id(int ext, int k) {
    int r = gw_Skin_RetailForExt(ext);
    if (r < 0 || k < 0 || k >= (sk_build(r), sk_by_r[r].n)) return "";
    return sk_by_r[r].s[k].id;
}
const char *gw_StageSkin_Name(int ext, int k) {
    int r = gw_Skin_RetailForExt(ext);
    if (r < 0 || k < 0 || k >= (sk_build(r), sk_by_r[r].n)) return "";
    return sk_by_r[r].s[k].name;
}
uint64_t gw_StageSkin_Digest(int ext, int k) {
    int r = gw_Skin_RetailForExt(ext);
    if (r < 0 || k < 0 || k >= (sk_build(r), sk_by_r[r].n)) return 0;
    return sk_by_r[r].s[k].digest;
}

static int sk_choice_r(int r) {
    const char *ids[GW_SKIN_MAX];
    char key[16], val[GW_MODS_ID_MAX];
    int k;
    sk_build(r);
    if (sk_by_r[r].n == 0) return 0;
    for (k = 0; k < sk_by_r[r].n; ++k) ids[k] = sk_by_r[r].s[k].id;
    snprintf(key, sizeof key, "skin_%d", sk_retail[r].ext);
    gw_Settings_Str(key, val, sizeof val, "");
    return gw_Skin_Resolve(val, ids, sk_by_r[r].n);
}

int gw_StageSkin_Choice(int ext) {
    int r = gw_Skin_RetailForExt(ext);
    return r < 0 ? 0 : sk_choice_r(r);
}

int gw_StageSkin_Cycle(int ext, int dir) {
    int r = gw_Skin_RetailForExt(ext), cur, nxt;
    char key[16];
    if (r < 0) return 0;
    cur = sk_choice_r(r);
    if (sk_by_r[r].n == 0) return 0;
    nxt = gw_Skin_Step(cur, sk_by_r[r].n, dir);
    snprintf(key, sizeof key, "skin_%d", sk_retail[r].ext);
    gw_Settings_SetStr(key, nxt == 0 ? "orig" : sk_by_r[r].s[nxt - 1].id);
    gw_log("stageskin: %s choice %d -> %d (%s)", sk_retail[r].file, cur, nxt, nxt == 0 ? "Original" : sk_by_r[r].s[nxt - 1].name);
    return nxt;
}

int gw_StageSkin_Label(int ext, char *out, int cap) {
    int r = gw_Skin_RetailForExt(ext), c;
    if (out != NULL && cap > 0) out[0] = '\0';
    if (r < 0) return 0;
    sk_build(r);
    if (sk_by_r[r].n == 0) return 0;
    c = sk_choice_r(r);
    if (out != NULL && cap > 0) snprintf(out, (size_t) cap, "%s", c == 0 ? "Original" : sk_by_r[r].s[c - 1].name);
    return sk_by_r[r].n;
}

/* ---- 3. applying -------------------------------------------------------------------------------- */

static struct {
    int on, orig;
    uint64_t dig48;
    char id[GW_MODS_ID_MAX];
    DWORD tick;
} sk_ov;                /* the match's override (scene key skin=), one load */
static DWORD sk_net_tick; /* an online match is launching: no override means Original */
static struct {
    int ext;
    char name[GW_MODS_TEXT_MAX];
    uint64_t digest;
} sk_last = { -1, "", 0 };

#define SK_STALE_MS 300000u

int gw_StageSkin_SceneSet(const char *v) {
    size_t n;
    char *end;
    unsigned long long d;
    memset(&sk_ov, 0, sizeof sk_ov);
    if (v == NULL || v[0] == '\0') return -1;
    sk_ov.on = 1;
    sk_ov.tick = GetTickCount();
    if (_stricmp(v, "orig") == 0 || _stricmp(v, "original") == 0) {
        sk_ov.orig = 1;
        return 0;
    }
    n = strlen(v);
    d = _strtoui64(v, &end, 16);
    if (n >= 12 && n <= 16 && *end == '\0') {
        sk_ov.dig48 = d & SK_MASK48;
        return 0;
    }
    copy_str(sk_ov.id, sizeof sk_ov.id, v);
    return 0;
}

void gw_StageSkin_NetBegin(void) {
    sk_net_tick = GetTickCount();
}

int gw_StageSkin_Last(int *ext, char *name, int cap, uint64_t *digest) {
    if (ext != NULL) *ext = sk_last.ext;
    if (name != NULL && cap > 0) snprintf(name, (size_t) cap, "%s", sk_last.name);
    if (digest != NULL) *digest = sk_last.digest;
    return sk_last.ext >= 0 && sk_last.digest != 0;
}

static struct {
    int armed, grkind, choice;
} sk_decided; /* the preload's decision, reused by the same stage's init */

void gw_StageSkin_ApplyForLoad(int grkind, const char *file, int phase) {
    int r = gw_Skin_NameRetail(file), choice = 0, k, ov = sk_ov.on, net = sk_net_tick != 0;
    const char *why = "choice";
    DWORD now = GetTickCount();
    if (file == NULL || file[0] == 0) return; /* a row with no archive (the menus' dummy stage): not a match's stage, leave what the last match loaded */
    if (phase == 2 && sk_decided.armed && sk_decided.grkind == grkind) { /* the stage's init, after its preload decided: the same file */
        sk_decided.armed = 0;
        r = gw_Skin_NameRetail(file);
        if (r >= 0 && sk_by_r[r].n > 0 && sk_decided.choice <= sk_by_r[r].n) {
            gw_DVDSelectStage(r, sk_decided.choice == 0 ? GW_SKIN_ORIGINAL : sk_by_r[r].s[sk_decided.choice - 1].mod_index);
        }
        return;
    }
    sk_decided.armed = 0;
    if (ov && now - sk_ov.tick > SK_STALE_MS) ov = 0;
    if (net && now - sk_net_tick > SK_STALE_MS) net = 0;
    sk_last.ext = -1;
    sk_last.name[0] = '\0';
    sk_last.digest = 0;
    if (r < 0) {
        memset(&sk_ov, 0, sizeof sk_ov); /* the override was for this load, whatever stage it was */
        sk_net_tick = 0;
        return;
    }
    sk_build(r);
    if (sk_by_r[r].n == 0) {
        memset(&sk_ov, 0, sizeof sk_ov);
        sk_net_tick = 0;
        return;
    }
    if (ov) {
        why = "match";
        if (!sk_ov.orig) {
            for (k = 0; k < sk_by_r[r].n; ++k) {
                if ((sk_ov.dig48 != 0 && (sk_by_r[r].s[k].digest & SK_MASK48) == sk_ov.dig48) ||
                    (sk_ov.id[0] != '\0' && _stricmp(sk_by_r[r].s[k].id, sk_ov.id) == 0)) {
                    choice = k + 1;
                }
            }
            if (choice == 0) {
                gw_log("stageskin: %s: the match names a skin that is not installed here (%012llx %s) - Original", sk_retail[r].file,
                       (unsigned long long) sk_ov.dig48, sk_ov.id);
            }
        }
    } else if (net) {
        why = "online match without a skin";
    } else {
        choice = sk_choice_r(r);
    }
    gw_DVDSelectStage(r, choice == 0 ? GW_SKIN_ORIGINAL : sk_by_r[r].s[choice - 1].mod_index);
    sk_decided.armed = phase == 1;
    sk_decided.grkind = grkind;
    sk_decided.choice = choice;
    sk_last.ext = sk_retail[r].ext;
    if (choice > 0) {
        copy_str(sk_last.name, sizeof sk_last.name, sk_by_r[r].s[choice - 1].name);
        sk_last.digest = sk_by_r[r].s[choice - 1].digest;
    }
    gw_log("stageskin: %s loads %s (%s, %d skin(s) installed, stage %d)", sk_retail[r].file, choice == 0 ? "Original" : sk_by_r[r].s[choice - 1].name, why,
           sk_by_r[r].n, grkind);
    memset(&sk_ov, 0, sizeof sk_ov);
    sk_net_tick = 0;
}

int gw_StageSkin_HostToken(int ext, int (*peer_has)(int ext, uint64_t digest), char *out, int cap) {
    int r = gw_Skin_RetailForExt(ext), c;
    uint64_t d;
    if (out != NULL && cap > 0) out[0] = '\0';
    if (r < 0 || out == NULL || cap < 24) return 0;
    c = sk_choice_r(r);
    if (c == 0) {
        gw_log("stageskin: %s: the host plays Original", sk_retail[r].file);
        return 0; /* no token = Original online */
    }
    d = sk_by_r[r].s[c - 1].digest;
    if (peer_has != NULL && peer_has(ext, d)) {
        snprintf(out, (size_t) cap, ";skin=%012llx", (unsigned long long) (d & SK_MASK48));
        gw_log("stageskin: %s: the host sends \"%s\" (the opponent has it too)", sk_retail[r].file, sk_by_r[r].s[c - 1].name);
        return 0;
    }
    gw_log("stageskin: %s: the opponent does not have \"%s\" - Original for both", sk_retail[r].file, sk_by_r[r].s[c - 1].name);
    return 1;
}

/* ---- tests ----------------------------------------------------------------------------------- */

static int test_skin_model(void) {
    const char *ids[3] = { "skin-a", "Skin-B", "skin-c" };
    gw_skin_prov p[4] = { { 0, 0 }, { 5, 1 }, { 7, 1 }, { 9, 1 } };
    int r3 = gw_Skin_RetailForExt(3), r32 = gw_Skin_RetailForExt(32), r8 = gw_Skin_RetailForExt(8);
    if (r3 < 0 || r32 < 0 || r8 < 0 || gw_Skin_RetailForExt(99) != -1 || gw_Skin_RetailExt(r3) != 3) {
        gw_test_fail("retail table lookup");
        return 1;
    }
    if (gw_Skin_FileRetail("GrPs.dat") != r3 || gw_Skin_FileRetail("/GrPs.usd") != r3 || gw_Skin_FileRetail("grps1.DAT") != r3 ||
        gw_Skin_FileRetail("GrNLa.dat") != r32 || gw_Skin_FileRetail("GrSt.dat") != r8 ||
        gw_Skin_FileRetail("GrI1.dat") != gw_Skin_RetailForExt(19)) {
        gw_test_fail("file -> stage: a stage's file did not map to it");
        return 1;
    }
    if (gw_Skin_FileRetail("GrXx.dat") != -1 || gw_Skin_FileRetail("audio/GrPs.dat") != -1 || gw_Skin_FileRetail("PlFx.dat") != -1 ||
        gw_Skin_FileRetail("GrPsExtra.dat") != -1 || gw_Skin_FileRetail("GrPs") != -1 || gw_Skin_FileRetail("GrPs.") != -1 ||
        gw_Skin_FileRetail(NULL) != -1 || gw_Skin_FileRetail("GrNBa.dat") == r32) {
        gw_test_fail("file -> stage: a file that is not a stage's mapped to one");
        return 1;
    }
    /* settings -> choice: auto = last mounted skin, orig = 0, an id = its position, an unknown id = auto */
    if (gw_Skin_Resolve(NULL, ids, 3) != 3 || gw_Skin_Resolve("", ids, 3) != 3 || gw_Skin_Resolve("auto", ids, 3) != 3 ||
        gw_Skin_Resolve("orig", ids, 3) != 0 || gw_Skin_Resolve("skin-b", ids, 3) != 2 || gw_Skin_Resolve("gone", ids, 3) != 3 ||
        gw_Skin_Resolve(NULL, ids, 0) != 0 || gw_Skin_Resolve("skin-a", ids, 0) != 0) {
        gw_test_fail("setting -> choice");
        return 1;
    }
    /* cycling: Original, 1, 2, 3, wrap; backwards from Original reaches the last */
    if (gw_Skin_Step(0, 3, 1) != 1 || gw_Skin_Step(3, 3, 1) != 0 || gw_Skin_Step(0, 3, -1) != 3 || gw_Skin_Step(2, 3, -1) != 1 ||
        gw_Skin_Step(0, 0, 1) != 0 || gw_Skin_Step(7, 3, 1) != 0) {
        gw_test_fail("choice stepping");
        return 1;
    }
    /* providers: [0] a base mod (not a skin), then skins from mods 5, 7, 9 */
    if (gw_skin_select(p, 4, 1, GW_SKIN_DEFAULT) != 3 || gw_skin_select(p, 4, 1, 7) != 2 || gw_skin_select(p, 4, 1, GW_SKIN_ORIGINAL) != 0 ||
        gw_skin_select(p, 4, 1, 42) != 0) {
        gw_test_fail("provider selection with a non-skin provider");
        return 1;
    }
    /* only skins: Original is the disc; a path the disc lacks keeps the default provider whatever is asked */
    if (gw_skin_select(p + 1, 3, 1, GW_SKIN_ORIGINAL) != -1 || gw_skin_select(p + 1, 3, 1, 5) != 0 || gw_skin_select(p + 1, 3, 1, GW_SKIN_DEFAULT) != 2 ||
        gw_skin_select(p + 1, 3, 0, GW_SKIN_ORIGINAL) != 2 || gw_skin_select(p, 0, 1, 5) != -1) {
        gw_test_fail("provider selection among skins only");
        return 1;
    }
    /* a skin that lacks this path (its .usd) falls back to Original for that path */
    if (gw_skin_select(p + 1, 2, 1, 9) != -1) {
        gw_test_fail("a skin without the path must give the original");
        return 1;
    }
    return 0;
}

/* skins of one stage no longer conflict (Nucleus lists them as mutual conflicts); other conflicts stand */
static int test_skin_mods_coexist(void) {
    char saved[MAX_PATH];
    gw_mods_set *s = t_set();
    int bad = 0;
    char tmp[MAX_PATH];
    DWORD n = GetTempPathA(sizeof tmp, tmp);
    copy_str(saved, sizeof saved, gw_mods_test_root);
    if (n == 0 || n >= sizeof tmp) return 1;
    snprintf(gw_mods_test_root, sizeof gw_mods_test_root, "%sgw_skin_test_%lu", tmp, (unsigned long) GetCurrentProcessId());
    t_rmtree(gw_mods_test_root);
    CreateDirectoryA(gw_mods_test_root, NULL);
    t_write("ps-a\\mod.json", "{\"kind\":\"stage\",\"conflicts\":[\"ps-b\",\"ps-c\"]}");
    t_write("ps-a\\files\\GrPs.dat", "aaaa");
    t_write("ps-b\\mod.json", "{\"kind\":\"stage\",\"conflicts\":[\"ps-a\"]}");
    t_write("ps-b\\files\\GrPs.dat", "bbbb");
    t_write("ps-b\\files\\GrPs.usd", "usd");
    t_write("ps-c\\mod.json", "{\"kind\":\"stage\",\"conflicts\":[\"ps-a\"]}");
    t_write("ps-c\\files\\GrPs.dat", "cccc");
    t_write("st-a\\mod.json", "{\"kind\":\"stage\",\"conflicts\":[\"ps-c\"]}");
    t_write("st-a\\files\\GrSt.dat", "dddd");
    t_write("zz-fx\\mod.json", "{\"kind\":\"fighter\",\"conflicts\":[\"ps-b\"]}");
    t_write("zz-fx\\files\\PlFx.dat", "eeee");
    set_load(s, gw_mods_test_root, 0);
    if (t_status(s, "ps-a") != GW_MOD_ACTIVE || t_status(s, "ps-b") != GW_MOD_ACTIVE || t_status(s, "ps-c") != GW_MOD_ACTIVE) {
        gw_test_fail("three Pokemon Stadium skins must all mount (%d %d %d: %s | %s)", t_status(s, "ps-a"), t_status(s, "ps-b"), t_status(s, "ps-c"),
                     s->mod[set_find(s, "ps-b")].status_text, s->mod[set_find(s, "ps-b")].payload);
        bad = 1;
    } else if (t_status(s, "st-a") != GW_MOD_CONFLICT) {
        gw_test_fail("a Yoshi's Story mod that conflicts with a Stadium skin must still conflict (%d)", t_status(s, "st-a"));
        bad = 1;
    } else if (t_status(s, "zz-fx") != GW_MOD_CONFLICT) {
        gw_test_fail("a fighter mod that conflicts with a skin must still conflict (%d)", t_status(s, "zz-fx"));
        bad = 1;
    } else if (gw_Skin_PayloadMask("") != 0) {
        gw_test_fail("payload mask of no folder");
        bad = 1;
    } else {
        char pay[MAX_PATH];
        snprintf(pay, sizeof pay, "%s\\ps-b\\files", gw_mods_test_root);
        if (gw_Skin_PayloadMask(pay) != (1u << gw_Skin_RetailForExt(3))) {
            gw_test_fail("payload mask of the Stadium skin");
            bad = 1;
        }
    }
    t_rmtree(gw_mods_test_root);
    copy_str(gw_mods_test_root, sizeof gw_mods_test_root, saved);
    return bad;
}

void gw_stageskin_tests_register(void) {
    gw_test_register("stageskin_model", test_skin_model);
    gw_test_register("stageskin_mods_coexist", test_skin_mods_coexist);
}
