/* gw_mexid.c - content identities for fighters, stages and global game data, and the netplay
 * intersection built from them. See gw_mexid.h for the rules; docs/mods-packaging.md (root repo)
 * for the design and its limits.
 *
 * Everything here is host data (hashes, copied names, ints). Nothing caches a guest pointer, so
 * the test harness's MEM1 restore cannot leave a stale one behind.
 *
 * NATIVE platform code (i686 clang), not gwtool. */
#define _CRT_SECURE_NO_WARNINGS
#include "gw.h"
#include "gw_mexid.h"
#include "gw_mex_ftfunction.h"
#include "gw_mex_grfunction.h"
#include "gw_test.h"
#include "gw_uigen.h"
#include "geno_define_online.h"

#define WIN32_LEAN_AND_MEAN
#ifdef _WIN32
#include <windows.h>
#else
#include "gw_compat_linux.h"
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern void *gw_DVDReadFileAlloc(const char *path, uint32_t *out_size);
extern int gw_DVDFileExists(const char *path);
extern int gw_Mex_SlotInternal(int slot);
extern const char *gw_Mex_FtPlFile(int k);
extern const char *gw_Mex_FtAnimFile(int k);
extern const char *gw_Mex_KirbyCapFile(int k);
extern int gw_Mex_ExtForInternal(int k);
extern const char *gw_Mex_FighterName(int ext);
extern int gw_Geno_DefineCount(void); /* slice 7: native defines are fighters of the table (gw_Geno_DefineOnlineInfo) */

#define MX_CK_MEX0 0x22 /* ChKind_Mex0: m-ex slot s is CharacterKind 0x22 + s */
#define MX_MEX_SLOTS GW_MEX_SLOT_COUNT
#define MX_MAX 512
#define MX_MAXDEF 128 /* slice 7: GF1 admits at most 94 defines */
#define MX_WIRE_HASH_BYTES 6 /* 48 bits on the wire: accidental collisions are not a concern */
#define MX_WIRE_ENTRY (1 + MX_WIRE_HASH_BYTES + 2)
#define MX_WIRE_HDR 5 /* 'M' 'X' 'E' index total */
#define MX_WIRE_CAP 200 /* GW_NET_LOBBY_MAX */
#define MX_WIRE_PER_CHUNK ((MX_WIRE_CAP - MX_WIRE_HDR) / MX_WIRE_ENTRY)
#define MX_WIRE_MASK ((1ull << (8 * MX_WIRE_HASH_BYTES)) - 1ull)

typedef struct mx_entry {
    int kind;
    int local;
    uint64_t hash;
    char name[40];
    char hex[17];
    int define;       /* slice 7: a native Geno define (its local id is a resident alias) */
    char key[40];     /* a define's key, for the reason text and the MXD message */
} mx_entry;

typedef struct mx_table {
    int built;
    int n;
    mx_entry e[MX_MAX];
} mx_table;

typedef struct mx_peer {
    int total_chunks; /* 0 = nothing received yet */
    int got_chunks;
    unsigned char got[64];
    int n;
    struct {
        int kind;
        int local;
        uint64_t hash48;
    } e[MX_MAX];
    int send_next; /* our next chunk to send (MXE chunks first, then one MXD message per define) */
    int nd;        /* slice 7: the peer's defines (key + short id), from MXD messages */
    int nd_total;  /* how many it says it has (from the messages), 0 = none heard */
    int nd_got;    /* how many distinct ones arrived */
    unsigned char nd_seen[MX_MAXDEF];
    struct {
        uint64_t hash48;
        char key[40];
    } d[MX_MAXDEF];
} mx_peer;

static mx_table mx_local;
static mx_peer mx_remote;

/* ---- hashing ------------------------------------------------------------------------------------ */

static uint64_t mx_mix(uint64_t h, uint64_t v) {
    h ^= v;
    h *= 0x9E3779B97F4A7C15ull;
    h ^= h >> 31;
    h *= 0xBF58476D1CE4E5B9ull;
    h ^= h >> 29;
    return h;
}

static uint64_t mx_hash_bytes(const unsigned char *p, uint32_t n) {
    uint64_t h = mx_mix(0x6A09E667F3BCC909ull, n);
    uint32_t i = 0;
    for (; i + 8u <= n; i += 8u) {
        uint64_t v;
        memcpy(&v, p + i, 8);
        h = mx_mix(h, v);
    }
    {
        uint64_t v = 0;
        memcpy(&v, p + i, n - i);
        h = mx_mix(h, v ^ 0xA5ull);
    }
    return h;
}

static uint64_t mx_hash_str(uint64_t h, const char *s) {
    return mx_mix(h, mx_hash_bytes((const unsigned char *) s, (uint32_t) strlen(s)));
}

/* Content hash of a disc path as the game would read it (disc or mounted mod), cached. 0 when the
 * file is absent. The NAME is only the cache key - it never enters the identity. */
#define MX_FILE_CACHE 1024
static struct {
    char path[64];
    uint64_t hash;
} mx_files[MX_FILE_CACHE];
static int mx_nfiles;
static uint64_t mx_bytes_hashed;

static uint64_t mx_file(const char *path) {
    char key[64];
    size_t i, n;
    uint32_t size = 0;
    unsigned char *buf;
    uint64_t h;
    if (path == NULL) return 0;
    while (*path == '/' || *path == '\\') ++path;
    n = strlen(path);
    if (n == 0 || n >= sizeof key) return 0;
    for (i = 0; i <= n; ++i) {
        char c = path[i];
        key[i] = (c >= 'A' && c <= 'Z') ? (char) (c - 'A' + 'a') : c;
    }
    for (i = 0; i < (size_t) mx_nfiles; ++i) {
        if (strcmp(mx_files[i].path, key) == 0) return mx_files[i].hash;
    }
    h = 0;
    if (gw_DVDFileExists(path)) {
        char slashed[80];
        snprintf(slashed, sizeof slashed, "/%s", path);
        buf = (unsigned char *) gw_DVDReadFileAlloc(slashed, &size);
        if (buf != NULL) {
            h = mx_hash_bytes(buf, size);
            if (h == 0) h = 1;
            mx_bytes_hashed += size;
            free(buf);
        }
    }
    if (mx_nfiles < MX_FILE_CACHE) {
        strcpy(mx_files[mx_nfiles].path, key);
        mx_files[mx_nfiles].hash = h;
        mx_nfiles++;
    }
    return h;
}

/* ---- the local table ------------------------------------------------------------------------------ */

/* CharacterKind order 0..25 (= m-ex external ids 0..25): the Pl code, the partner loaded with it
 * (Sheik and Zelda transform into each other; Popo brings Nana) and a display name. */
static const struct {
    const char *code, *partner, *name;
} mx_vanilla_fighters[26] = {
    { "Ca", NULL, "Captain Falcon" }, { "Dk", NULL, "Donkey Kong" }, { "Fx", NULL, "Fox" },
    { "Gw", NULL, "Mr. Game & Watch" }, { "Kb", NULL, "Kirby" }, { "Kp", NULL, "Bowser" },
    { "Lk", NULL, "Link" }, { "Lg", NULL, "Luigi" }, { "Mr", NULL, "Mario" },
    { "Ms", NULL, "Marth" }, { "Mt", NULL, "Mewtwo" }, { "Ns", NULL, "Ness" },
    { "Pe", NULL, "Peach" }, { "Pk", NULL, "Pikachu" }, { "Pp", "Nn", "Ice Climbers" },
    { "Pr", NULL, "Jigglypuff" }, { "Ss", NULL, "Samus" }, { "Ys", NULL, "Yoshi" },
    { "Zd", "Sk", "Zelda" }, { "Sk", "Zd", "Sheik" }, { "Fc", NULL, "Falco" },
    { "Cl", NULL, "Young Link" }, { "Dr", NULL, "Dr. Mario" }, { "Fe", NULL, "Roy" },
    { "Pc", NULL, "Pichu" }, { "Gn", NULL, "Ganondorf" },
};

/* The retail VS stages by external stage id and file (a prefix when the game adds a suffix). */
static const struct {
    int ext;
    const char *file, *name;
} mx_vanilla_stages[] = {
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

static void mx_hex(uint64_t h, char out[17]) {
    snprintf(out, 17, "%016llx", (unsigned long long) h);
}

static mx_entry *mx_add(mx_table *t, int kind, int local, uint64_t hash, const char *name) {
    mx_entry *e;
    if (t->n >= MX_MAX) return NULL;
    e = &t->e[t->n++];
    e->kind = kind;
    e->local = local;
    e->hash = hash != 0 ? hash : 1;
    snprintf(e->name, sizeof e->name, "%s", name != NULL ? name : "?");
    mx_hex(e->hash, e->hex);
    return e;
}

/* Identity of a fighter from its files, in a fixed order. Returns 0 when its Pl file is absent. */
static uint64_t mx_fighter_identity(const char *pl, const char *aj, const char *pl2, const char *aj2,
                                    const char *cap) {
    uint64_t h = mx_hash_str(0, "fighter");
    uint64_t f = mx_file(pl);
    if (f == 0) return 0;
    h = mx_mix(h, f);
    h = mx_mix(h, mx_file(aj));
    h = mx_mix(h, pl2 != NULL ? mx_file(pl2) : 0);
    h = mx_mix(h, aj2 != NULL ? mx_file(aj2) : 0);
    h = mx_mix(h, cap != NULL ? mx_file(cap) : 0);
    {
        /* A Geno overlay (mods/<id>/geno.json) changes how this fighter plays, so its stable id is
         * part of the identity. 0 - no overlay - leaves the hash exactly as it always was. */
        extern uint64_t gw_Geno_SaltForPlFile(const char *pl);
        uint64_t salt = gw_Geno_SaltForPlFile(pl);
        if (salt != 0) h = mx_mix(h, salt);
    }
    return h;
}

static int mx_ends_dat(const char *s) {
    size_t n = strlen(s);
    return n >= 4 && _stricmp(s + n - 4, ".dat") == 0;
}

/* Identity of a stage from its file(s). 0 when no file is present. */
static uint64_t mx_stage_identity(const char *file) {
    uint64_t h = mx_hash_str(0, "stage"), f;
    int any = 0, i;
    while (*file == '/') ++file;
    if (mx_ends_dat(file)) {
        f = mx_file(file);
        return f != 0 ? mx_mix(h, f) : 0;
    }
    for (i = -1; i <= 9; ++i) { /* <prefix>.dat, <prefix>1.dat .. <prefix>9.dat */
        char p[64];
        if (i < 0) snprintf(p, sizeof p, "%s.dat", file);
        else snprintf(p, sizeof p, "%s%d.dat", file, i);
        f = mx_file(p);
        h = mx_mix(h, f);
        any |= f != 0;
    }
    return any ? h : 0;
}

/* Slice 7: the online identity of a native Geno define. The entry's content id (the whole geno.json entry, its overlay words and
 * its Lua module: geno_registry.c) says what the fighter DOES; the files the registry names say what it is built on (Mario's
 * model and clips for a donor define, the package's own bank, plan and costume models for a "none" define). The resident alias
 * is NOT part of it: it is this install's local id, mapped through the peer's list like an m-ex slot. A define whose resource
 * file is missing has no identity and so cannot be played online (it would not load offline either). */
static void mx_add_defines(mx_table *t) {
    int d, n = gw_Geno_DefineCount();
    for (d = 0; d < n; ++d) {
        GenoDefineOnline o;
        uint64_t h;
        int f, ok = 1;
        mx_entry *e;
        if (!gw_Geno_DefineOnlineInfo(d, &o)) continue;
        h = mx_mix(mx_hash_str(0, "geno-define"), o.id);
        h = mx_mix(h, (uint64_t) o.none);
        for (f = 0; f < o.nfiles && ok; ++f) {
            uint64_t fh = mx_file(o.files[f]);
            if (fh == 0) {
                gw_log("mexid: define %s has no online identity: resource file %s is missing", o.key, o.files[f]);
                ok = 0;
            } else {
                h = mx_mix(h, fh);
            }
        }
        if (!ok) continue;
        e = mx_add(t, GW_MEXID_FIGHTER, o.ck, h, o.name);
        if (e != NULL) {
            e->define = 1;
            snprintf(e->key, sizeof e->key, "%s", o.key);
        }
    }
}

static void mx_build(mx_table *t) {
    DWORD t0 = GetTickCount();
    int i, s, n;
    uint64_t before = mx_bytes_hashed;
    memset(t, 0, sizeof *t);
    t->built = 1;
    for (i = 0; i < 26; ++i) {
        char pl[16], aj[16], pl2[16], aj2[16];
        uint64_t h;
        snprintf(pl, sizeof pl, "Pl%s.dat", mx_vanilla_fighters[i].code);
        snprintf(aj, sizeof aj, "Pl%sAJ.dat", mx_vanilla_fighters[i].code);
        if (mx_vanilla_fighters[i].partner != NULL) {
            snprintf(pl2, sizeof pl2, "Pl%s.dat", mx_vanilla_fighters[i].partner);
            snprintf(aj2, sizeof aj2, "Pl%sAJ.dat", mx_vanilla_fighters[i].partner);
        }
        h = mx_fighter_identity(pl, aj, mx_vanilla_fighters[i].partner ? pl2 : NULL,
                                mx_vanilla_fighters[i].partner ? aj2 : NULL, NULL);
        if (h != 0) mx_add(t, GW_MEXID_FIGHTER, i, h, mx_vanilla_fighters[i].name);
    }
    for (s = 0; s < MX_MEX_SLOTS; ++s) {
        int k = gw_Mex_SlotInternal(s);
        const char *pl, *name;
        uint64_t h;
        if (k < 0) break;
        pl = gw_Mex_FtPlFile(k);
        h = mx_fighter_identity(pl, gw_Mex_FtAnimFile(k), NULL, NULL, gw_Mex_KirbyCapFile(k));
        name = gw_Mex_FighterName(gw_Mex_ExtForInternal(k));
        if (h != 0) mx_add(t, GW_MEXID_FIGHTER, MX_CK_MEX0 + s, h, name != NULL ? name : pl);
    }
    mx_add_defines(t);
    for (i = 0; i < (int) (sizeof mx_vanilla_stages / sizeof mx_vanilla_stages[0]); ++i) {
        uint64_t h = mx_stage_identity(mx_vanilla_stages[i].file);
        if (h != 0) mx_add(t, GW_MEXID_STAGE, mx_vanilla_stages[i].ext, h, mx_vanilla_stages[i].name);
    }
    n = gw_UI_StageCount(); /* the m-ex stage-select icons: added stages whose file is here */
    for (i = 0; i < n; ++i) {
        GwUiStage st;
        const char *file;
        char name[40];
        size_t j;
        uint64_t h;
        if (!gw_UI_StageAt(i, &st) || st.grkind < GW_MEX_GR_FIRST_NEW || !st.has_file) continue;
        if (gw_MexId_FindStage(st.external_id) >= 0) continue; /* two icons, one stage */
        file = gw_Mex_GrFile(st.grkind);
        if (file == NULL) continue;
        while (*file == '/') ++file;
        snprintf(name, sizeof name, "%s", file);
        for (j = 0; name[j]; ++j) {
            if (name[j] == '.') name[j] = '\0';
        }
        h = mx_stage_identity(file);
        if (h != 0) mx_add(t, GW_MEXID_STAGE, st.external_id, h, name);
    }
    gw_log("mexid: %d identities (%u fighter/stage bytes hashed) in %lu ms", t->n,
           (unsigned) (mx_bytes_hashed - before), (unsigned long) (GetTickCount() - t0));
}

static mx_table *mx_tab(void) {
    if (!mx_local.built) mx_build(&mx_local);
    return &mx_local;
}

int gw_MexId_Count(void) { return mx_tab()->n; }
static mx_entry *mx_at(int i) {
    mx_table *t = mx_tab();
    return (i >= 0 && i < t->n) ? &t->e[i] : NULL;
}
int gw_MexId_Kind(int i) { return mx_at(i) ? mx_at(i)->kind : 0; }
int gw_MexId_LocalId(int i) { return mx_at(i) ? mx_at(i)->local : -1; }
const char *gw_MexId_Name(int i) { return mx_at(i) ? mx_at(i)->name : ""; }
const char *gw_MexId_HashHex(int i) { return mx_at(i) ? mx_at(i)->hex : ""; }
uint64_t gw_MexId_Hash(int i) { return mx_at(i) ? mx_at(i)->hash : 0; }

static int mx_find(const mx_table *t, int kind, int local) {
    int i;
    for (i = 0; i < t->n; ++i) {
        if (t->e[i].kind == kind && t->e[i].local == local) return i;
    }
    return -1;
}
int gw_MexId_FindFighter(int ck) { return mx_find(mx_tab(), GW_MEXID_FIGHTER, ck); }
int gw_MexId_FindStage(int ext) {
    /* also used while the table is being built (duplicate SSS icons) */
    return mx_find(&mx_local, GW_MEXID_STAGE, ext);
}

const char *gw_MexId_TokenForCk(int ck) {
    static char tok[24];
    int i = gw_MexId_FindFighter(ck);
    if (i >= 0) snprintf(tok, sizeof tok, "id:%s", mx_local.e[i].hex);
    else snprintf(tok, sizeof tok, "ck:%d", ck);
    return tok;
}
const char *gw_MexId_TokenForExt(int ext) {
    static char tok[24];
    int i = (mx_tab(), gw_MexId_FindStage(ext));
    if (i >= 0) snprintf(tok, sizeof tok, "id:%s", mx_local.e[i].hex);
    else snprintf(tok, sizeof tok, "ext:%d", ext);
    return tok;
}

static int mx_local_for_hex(int kind, const char *hex) {
    mx_table *t = mx_tab();
    uint64_t h;
    char *end;
    int i;
    if (hex == NULL) return -1;
    h = _strtoui64(hex, &end, 16);
    if (end == hex) return -1;
    for (i = 0; i < t->n; ++i) {
        if (t->e[i].kind == kind && t->e[i].hash == h) return t->e[i].local;
    }
    return -1;
}
int gw_MexId_CkForHex(const char *hex16) { return mx_local_for_hex(GW_MEXID_FIGHTER, hex16); }
int gw_MexId_ExtForHex(const char *hex16) { return mx_local_for_hex(GW_MEXID_STAGE, hex16); }

/* Slice 7: the peer announced a define whose short id is the one in `hex16` (the peer's scene names it): say its key, and whether this
 * install lacks it or holds another version of the same key. 0 when the peer did not announce it (an older build, or the list is not here). */
static int mx_peer_define_note(const char *hex16, char *out, int cap) {
    const mx_table *t = &mx_local;
    char *end;
    uint64_t h = _strtoui64(hex16, &end, 16) & MX_WIRE_MASK;
    int j, i;
    if (end == hex16) return 0;
    for (j = 0; j < mx_remote.nd; ++j) {
        if (mx_remote.d[j].hash48 != h) continue;
        for (i = 0; i < t->n; ++i) {
            if (t->e[i].define && strcmp(t->e[i].key, mx_remote.d[j].key) == 0) {
                snprintf(out, (size_t) cap, "Geno fighter %.40s (you have another version of it)", mx_remote.d[j].key);
                return 1;
            }
        }
        snprintf(out, (size_t) cap, "Geno fighter %.40s (not installed here)", mx_remote.d[j].key);
        return 1;
    }
    return 0;
}

int gw_MexId_SceneCheck(const char *scene, char *why, int cap) {
    (void) mx_tab();
    const char *p = scene;
    int missing = 0;
    if (why != NULL && cap > 0) why[0] = '\0';
    while (p != NULL && (p = strstr(p, "id:")) != NULL) {
        /* "p1=id:<hex>/..." is a fighter, "stage=id:<hex>" a stage */
        int is_stage = (p - scene >= 6 && strncmp(p - 6, "stage=", 6) == 0);
        int ok = is_stage ? gw_MexId_ExtForHex(p + 3) >= 0 : gw_MexId_CkForHex(p + 3) >= 0;
        if (!ok) {
            ++missing;
            if (why != NULL && cap > 0) {
                size_t l = strlen(why);
                char named[120];
                if (!is_stage && mx_peer_define_note(p + 3, named, sizeof named)) {
                    snprintf(why + l, (size_t) cap - l, "%s%s", l ? ", " : "", named);
                } else {
                    snprintf(why + l, (size_t) cap - l, "%s%s %.16s", l ? ", " : "",
                             is_stage ? "stage" : "fighter", p + 3);
                }
            }
        }
        p += 3;
    }
    return missing;
}

/* ---- global game data ------------------------------------------------------------------------- *
 * PlCo.dat's ftLoadCommonData is 23 pointers (Fighter_LoadCommonData, ft/fighter.c). [4]
 * (ftPartsTable) and [5] are PER-KIND tables that an m-ex build extends for its roster; they are
 * left out - per-kind data belongs to the fighter, not to the global sim. The other 21 are hashed
 * at their retail sizes, pointer words as a marker (their value moves with the file layout). An
 * offline walk found every retail table and everything under it identical in vanilla, ACE 2.0 and
 * Akaneia 1.0.1 (m-ex only APPENDS, and edits rows >= 27 of the two per-kind tables); this hash
 * does not descend into the nested tables, so a mod that edits those is not caught. THAT WALK WAS NOT
 * ENOUGH: ACE's PlCo.dat still changes vanilla fighters' animation poses (see mx_global_build), so the
 * global component also hashes the whole file. */
static const uint16_t mx_plco_sizes[23] = { 2072, 312, 120, 36, 0, 0, 984, 48, 8, 24, 8, 8,
                                            156, 60, 36, 8, 88, 20, 20, 20, 88, 68, 48 };

static uint64_t mx_plco(void) {
    uint32_t size = 0, data_size, nb_reloc, i, base;
    unsigned char *dat = (unsigned char *) gw_DVDReadFileAlloc("/PlCo.dat", &size);
    uint64_t h = mx_hash_str(0, "plco");
    const unsigned char *data, *rel;
    int32_t sym;
    if (dat == NULL) return mx_mix(h, 0);
    data_size = size >= 0x20 ? gw_r32(dat + 4) : 0;
    nb_reloc = size >= 0x20 ? gw_r32(dat + 8) : 0;
    if (size < 0x20 || 0x20ull + data_size + (uint64_t) nb_reloc * 4u > size) {
        free(dat);
        return mx_mix(h, 0xBADu);
    }
    data = dat + 0x20;
    rel = data + data_size;
    sym = gw_ftfunction_find_public(dat, size, "ftLoadCommonData");
    if (sym < 0 || (uint32_t) sym + 23u * 4u > data_size) {
        free(dat);
        return mx_mix(h, 0xBADu + 1u);
    }
    base = (uint32_t) sym;
    for (i = 0; i < 23; ++i) {
        uint32_t p = gw_r32(data + base + i * 4u), o, r;
        if (mx_plco_sizes[i] == 0) continue;
        h = mx_mix(h, i);
        for (o = 0; o + 4u <= mx_plco_sizes[i]; o += 4u) {
            int is_ptr = 0;
            if ((uint64_t) p + o + 4u > data_size) {
                h = mx_mix(h, 0xEEEEu);
                break;
            }
            for (r = 0; r < nb_reloc; ++r) { /* the table is small; a scan is fine */
                if (gw_r32(rel + r * 4u) == p + o) {
                    is_ptr = 1;
                    break;
                }
            }
            h = mx_mix(h, is_ptr ? 0x50545221u : (uint64_t) gw_r32(data + p + o) << 1);
        }
    }
    free(dat);
    return h;
}

static uint64_t mx_mexflags(void) {
    uint64_t h = mx_hash_str(0, "mexflags");
    const char *env = getenv("MELEE_MEX");
    char path[MAX_PATH];
    DWORD n;
    h = mx_hash_str(h, env != NULL ? env : "");
    n = GetModuleFileNameA(NULL, path, (DWORD) sizeof path);
    if (n > 0 && n < (DWORD) sizeof path) { /* gw_runtime.c's gw_mex_load reads exactly this */
        char *slash = gw_path_separator(path);
        FILE *f;
        if (slash != NULL) {
            slash[1] = '\0';
            strncat(path, "mods\\mex.txt", sizeof path - strlen(path) - 1);
            f = fopen(path, "rb");
            if (f != NULL) {
                unsigned char buf[4096];
                size_t got = fread(buf, 1, sizeof buf, f);
                fclose(f);
                h = mx_mix(h, mx_hash_bytes(buf, (uint32_t) got));
            }
        }
    }
    return h;
}

static const struct {
    const char *key, *what;
} mx_global_names[3] = {
    { "plco", "PlCo.dat (common fighter data: a different disc or mod pack)" },
    { "itco", "ItCo.dat (common items)" },
    { "mexflags", "m-ex feature flags (MELEE_MEX / mods\\mex.txt)" },
};
static uint64_t mx_global[3];
static int mx_global_done;
static char mx_global_desc[96];

static unsigned mx_fold16(uint64_t h) {
    return (unsigned) ((h ^ (h >> 16) ^ (h >> 32) ^ (h >> 48)) & 0xFFFFu);
}

static void mx_global_build(void) {
    if (mx_global_done) return;
    mx_global_done = 1;
    /* The whole PlCo.dat file, not only its 21 global tables (mx_plco): vanilla fighters played with the
     * SSBM ACE Build v2.0.0 PlCo.dat (vanilla disc + that one file as a loose mod) stop matching a
     * vanilla PlCo.dat at an animation's first frame (frames 3560 and 5218 of a scripted Falco v Sheik
     * match; the netplay checksum fails at 5244) although every table the shallow walk reaches is equal.
     * The same PlCo.dat re-padded to another size changes nothing (identical over 6000 frames), so it is
     * the content, not the heap layout. Until the field is named, equal PlCo.dat bytes are required. */
    mx_global[0] = mx_mix(mx_plco(), mx_file("PlCo.dat"));
    mx_global[1] = mx_mix(mx_hash_str(0, "itco"), mx_file("ItCo.dat"));
    mx_global[2] = mx_mexflags();
    snprintf(mx_global_desc, sizeof mx_global_desc, "%s#%04x,%s#%04x,%s#%04x", mx_global_names[0].key,
             mx_fold16(mx_global[0]), mx_global_names[1].key, mx_fold16(mx_global[1]),
             mx_global_names[2].key, mx_fold16(mx_global[2]));
    gw_log("mexid: global game data %s", mx_global_desc);
}

uint64_t gw_MexId_GlobalHash(void) {
    uint64_t h;
    mx_global_build();
    h = mx_mix(mx_mix(mx_mix(0, mx_global[0]), mx_global[1]), mx_global[2]);
    return h != 0 ? h : 1;
}

const char *gw_MexId_GlobalDescribe(void) {
    mx_global_build();
    return mx_global_desc;
}

/* Compare two "key#hhhh,..." strings component by component. */
static int mx_global_diff(const char *mine, const char *peer, char *out, int cap) {
    int i, ndiff = 0;
    if (out != NULL && cap > 0) out[0] = '\0';
    for (i = 0; i < 3; ++i) {
        char pat[16];
        const char *a, *b;
        unsigned va = 0, vb = 0;
        snprintf(pat, sizeof pat, "%s#", mx_global_names[i].key);
        a = strstr(mine, pat);
        b = peer != NULL ? strstr(peer, pat) : NULL;
        if (a != NULL) va = (unsigned) strtoul(a + strlen(pat), NULL, 16);
        if (b != NULL) vb = (unsigned) strtoul(b + strlen(pat), NULL, 16);
        if (a == NULL || b == NULL || va != vb) {
            ++ndiff;
            if (out != NULL && cap > 0) {
                size_t l = strlen(out);
                snprintf(out + l, (size_t) cap - l, "%s%s", l ? ", " : "", mx_global_names[i].what);
            }
        }
    }
    if (ndiff == 0 && out != NULL && cap > 0) snprintf(out, (size_t) cap, "same global data");
    return ndiff;
}

int gw_MexId_GlobalDiff(const char *peer_desc, char *out, int cap) {
    return mx_global_diff(gw_MexId_GlobalDescribe(), peer_desc, out, cap);
}

/* ---- the peer ----------------------------------------------------------------------------------- */

static int mx_chunks_for(int n) {
    int c = (n + MX_WIRE_PER_CHUNK - 1) / MX_WIRE_PER_CHUNK;
    return c > 0 ? c : 1;
}

void gw_MexId_PeerReset(void) {
    memset(&mx_remote, 0, sizeof mx_remote);
}

static int mx_wire_chunk(const mx_table *t, int idx, uint8_t *out, int cap) {
    int total = mx_chunks_for(t->n), first = idx * MX_WIRE_PER_CHUNK, i, len;
    uint8_t *p = out;
    if (idx >= total || cap < MX_WIRE_CAP) return 0;
    *p++ = 'M';
    *p++ = 'X';
    *p++ = 'E';
    *p++ = (uint8_t) idx;
    *p++ = (uint8_t) total;
    for (i = first; i < t->n && i < first + MX_WIRE_PER_CHUNK; ++i) {
        uint64_t h = t->e[i].hash & MX_WIRE_MASK;
        int b;
        *p++ = (uint8_t) t->e[i].kind;
        for (b = MX_WIRE_HASH_BYTES - 1; b >= 0; --b) *p++ = (uint8_t) (h >> (8 * b));
        *p++ = (uint8_t) ((unsigned) t->e[i].local >> 8);
        *p++ = (uint8_t) t->e[i].local;
    }
    len = (int) (p - out);
    return len;
}

/* Slice 7: one message per define, after the identity chunks: 'M' 'X' 'D' index count hash48(6) key. It lets each side say WHICH
 * define the other lacks or holds a different copy of (the reason text); nothing is decided from it. A build that does not know
 * it ignores the message (its WireFeed and the lobby both pass over an unknown first letter). */
#define MX_WIRE_DHDR 5
static int mx_wire_define(const mx_table *t, int idx, uint8_t *out, int cap) {
    int i, seen = 0, count = 0, b;
    uint8_t *p = out;
    size_t kl;
    for (i = 0; i < t->n; ++i) count += t->e[i].define != 0;
    if (idx < 0 || idx >= count || cap < MX_WIRE_CAP) return 0;
    for (i = 0; i < t->n; ++i) {
        uint64_t h;
        if (!t->e[i].define) continue;
        if (seen++ != idx) continue;
        h = t->e[i].hash & MX_WIRE_MASK;
        *p++ = 'M';
        *p++ = 'X';
        *p++ = 'D';
        *p++ = (uint8_t) idx;
        *p++ = (uint8_t) count;
        for (b = MX_WIRE_HASH_BYTES - 1; b >= 0; --b) *p++ = (uint8_t) (h >> (8 * b));
        kl = strlen(t->e[i].key);
        memcpy(p, t->e[i].key, kl);
        p += kl;
        return (int) (p - out);
    }
    return 0;
}

int gw_MexId_WireNext(uint8_t *out, int cap) {
    const mx_table *t = mx_tab();
    int total = mx_chunks_for(t->n);
    if (mx_remote.send_next < total) return mx_wire_chunk(t, mx_remote.send_next, out, cap);
    return mx_wire_define(t, mx_remote.send_next - total, out, cap);
}

void gw_MexId_WireSent(void) { mx_remote.send_next++; }

static int mx_wire_feed_define(mx_peer *r, const uint8_t *msg, int len) {
    int idx = msg[3], b, k, kl;
    uint64_t h = 0;
    if (len < MX_WIRE_DHDR + MX_WIRE_HASH_BYTES || idx >= MX_MAXDEF || idx >= msg[4]) {
        gw_log("mexid: malformed define message - ignored");
        return 1;
    }
    for (b = 0; b < MX_WIRE_HASH_BYTES; ++b) h = (h << 8) | msg[MX_WIRE_DHDR + b];
    kl = len - MX_WIRE_DHDR - MX_WIRE_HASH_BYTES;
    if (kl > (int) sizeof r->d[0].key - 1) kl = (int) sizeof r->d[0].key - 1;
    r->d[idx].hash48 = h;
    r->nd_total = msg[4];
    if (!r->nd_seen[idx]) {
        r->nd_seen[idx] = 1;
        r->nd_got++;
    }
    for (k = 0; k < kl; ++k) {
        uint8_t c = msg[MX_WIRE_DHDR + MX_WIRE_HASH_BYTES + k];
        r->d[idx].key[k] = (c >= 32 && c < 127) ? (char) c : '?';
    }
    r->d[idx].key[kl] = '\0';
    if (idx + 1 > r->nd) r->nd = idx + 1;
    return 1;
}

static int mx_wire_feed(mx_peer *r, const uint8_t *msg, int len) {
    int idx, total, i, count, first;
    if (msg != NULL && len >= MX_WIRE_DHDR && msg[0] == 'M' && msg[1] == 'X' && msg[2] == 'D') return mx_wire_feed_define(r, msg, len);
    if (msg == NULL || len < MX_WIRE_HDR || msg[0] != 'M' || msg[1] != 'X' || msg[2] != 'E') return 0;
    idx = msg[3];
    total = msg[4];
    if (total == 0 || total > 64 || idx >= total || (r->total_chunks != 0 && total != r->total_chunks)) {
        gw_log("mexid: malformed identity chunk %d/%d - ignored", idx, total);
        return 1;
    }
    r->total_chunks = total;
    if (r->got[idx]) return 1; /* the lobby channel is exactly-once; be safe anyway */
    count = (len - MX_WIRE_HDR) / MX_WIRE_ENTRY;
    first = idx * MX_WIRE_PER_CHUNK;
    for (i = 0; i < count && first + i < MX_MAX; ++i) {
        const uint8_t *e = msg + MX_WIRE_HDR + i * MX_WIRE_ENTRY;
        uint64_t h = 0;
        int b;
        for (b = 0; b < MX_WIRE_HASH_BYTES; ++b) h = (h << 8) | e[1 + b];
        r->e[first + i].kind = e[0];
        r->e[first + i].hash48 = h;
        r->e[first + i].local = (e[1 + MX_WIRE_HASH_BYTES] << 8) | e[2 + MX_WIRE_HASH_BYTES];
        if (first + i + 1 > r->n) r->n = first + i + 1;
    }
    r->got[idx] = 1;
    r->got_chunks++;
    if (r->got_chunks == r->total_chunks) {
        gw_log("mexid: the peer's %d identities are here", r->n);
    }
    return 1;
}

int gw_MexId_WireFeed(const uint8_t *msg, int len) {
    int was_ready = gw_MexId_PeerReady(), r = mx_wire_feed(&mx_remote, msg, len);
    if (r && !was_ready && gw_MexId_PeerReady()) {
        gw_log("mexid: online in common: %d fighter(s), %d stage(s)", gw_MexId_CommonFighterCount(),
               gw_MexId_CommonStageCount());
    }
    return r;
}

/* 1 when every define the peer announced has arrived (it announces its count with each one); 0 while some are missing or none came. */
int gw_MexId_PeerDefinesComplete(void) {
    return mx_remote.nd_total != 0 && mx_remote.nd_got >= mx_remote.nd_total;
}

int gw_MexId_PeerReady(void) {
    return mx_remote.total_chunks != 0 && mx_remote.got_chunks == mx_remote.total_chunks;
}
int gw_MexId_PeerCount(void) { return mx_remote.n; }

/* The peer's local id for our entry i, or -1. */
static int mx_peer_local(const mx_table *t, const mx_peer *r, int i) {
    int j;
    if (i < 0) return -1;
    for (j = 0; j < r->n; ++j) {
        if (r->e[j].kind == t->e[i].kind && r->e[j].hash48 == (t->e[i].hash & MX_WIRE_MASK)) {
            return r->e[j].local;
        }
    }
    return -1;
}

/* Our local id for the peer's (kind, local), or -1. */
static int mx_local_for_peer(const mx_table *t, const mx_peer *r, int kind, int peer_local) {
    int i, j;
    for (j = 0; j < r->n; ++j) {
        if (r->e[j].kind != kind || r->e[j].local != peer_local) continue;
        for (i = 0; i < t->n; ++i) {
            if (t->e[i].kind == kind && (t->e[i].hash & MX_WIRE_MASK) == r->e[j].hash48) return t->e[i].local;
        }
        return -1;
    }
    return -1;
}

int gw_MexId_OnlineFighter(int ck) {
    if (!gw_MexId_PeerReady()) return -1;
    return mx_peer_local(mx_tab(), &mx_remote, gw_MexId_FindFighter(ck)) >= 0;
}
int gw_MexId_OnlineStage(int ext) {
    if (!gw_MexId_PeerReady()) return -1;
    return mx_peer_local(mx_tab(), &mx_remote, (mx_tab(), gw_MexId_FindStage(ext))) >= 0;
}
int gw_MexId_PeerCkForLocal(int ck) {
    return gw_MexId_PeerReady() ? mx_peer_local(mx_tab(), &mx_remote, gw_MexId_FindFighter(ck)) : -1;
}
int gw_MexId_LocalCkForPeer(int peer_ck) {
    return gw_MexId_PeerReady() ? mx_local_for_peer(mx_tab(), &mx_remote, GW_MEXID_FIGHTER, peer_ck) : -1;
}
int gw_MexId_PeerExtForLocal(int ext) {
    return gw_MexId_PeerReady() ? mx_peer_local(mx_tab(), &mx_remote, (mx_tab(), gw_MexId_FindStage(ext))) : -1;
}
int gw_MexId_LocalExtForPeer(int peer_ext) {
    return gw_MexId_PeerReady() ? mx_local_for_peer(mx_tab(), &mx_remote, GW_MEXID_STAGE, peer_ext) : -1;
}

/* Slice 7: why a fighter this install has cannot be played against this peer. 1 = playable, or the peer's list is not complete yet
 * (nothing is refused on a guess); 0 = refused, and `why` says in the player's words which fighter and what differs. */
int gw_MexId_FighterWhy(int ck, char *why, int cap) {
    mx_table *t = mx_tab();
    int i = gw_MexId_FindFighter(ck), j;
    char nm[48];
    if (why != NULL && cap > 0) why[0] = '\0';
    if (i < 0) {
        char dn[48];
        extern int gw_Geno_DefineName(int ck, char *out, int cap);
        if (gw_Geno_DefineName(ck, dn, (int) sizeof dn)) { /* a define whose package lacks a resource file: no identity, so no online play */
            if (why != NULL && cap > 0) snprintf(why, (size_t) cap, "%.46s: its package is incomplete (see the log)", dn);
            return 0;
        }
        return 1; /* any other fighter without an identity: not decided here (the lobby has always let those through) */
    }
    if (!gw_MexId_PeerReady() || mx_peer_local(t, &mx_remote, i) >= 0) return 1;
    snprintf(nm, sizeof nm, "%s", t->e[i].name);
    /* the text is short on purpose: the lobby's notice line holds 80 characters. The two ids go to the log, once per fighter. */
    {
        static int said[MX_MAX];
        int differs = -1;
        if (t->e[i].define) {
            for (j = 0; j < mx_remote.nd; ++j) {
                if (strcmp(mx_remote.d[j].key, t->e[i].key) == 0) differs = j;
            }
        }
        if (i < MX_MAX && !said[i]) {
            said[i] = 1;
            if (differs >= 0) {
                gw_log("mexid: define %s (%s) is not in common: the peer's copy is %012llx, ours %012llx", t->e[i].key, nm,
                       (unsigned long long) mx_remote.d[differs].hash48, (unsigned long long) (t->e[i].hash & MX_WIRE_MASK));
            } else {
                gw_log("mexid: %s %s (%012llx) is not in common: the peer does not have it", t->e[i].define ? "define" : "fighter", nm,
                       (unsigned long long) (t->e[i].hash & MX_WIRE_MASK));
            }
        }
        if (why != NULL && cap > 0) {
            if (differs >= 0) snprintf(why, (size_t) cap, "%.46s: your opponent has another version", nm);
            else snprintf(why, (size_t) cap, "%.46s: your opponent doesn't have it", nm);
        }
    }
    return 0;
}

/* A define's local key for a CharacterKind ("" when it is not a define). */
const char *gw_MexId_DefineKey(int ck) {
    int i = gw_MexId_FindFighter(ck);
    return i >= 0 && mx_local.e[i].define ? mx_local.e[i].key : "";
}

static int mx_common(int kind) {
    mx_table *t = mx_tab();
    int i, n = 0;
    if (!gw_MexId_PeerReady()) return -1;
    for (i = 0; i < t->n; ++i) {
        if (t->e[i].kind == kind && mx_peer_local(t, &mx_remote, i) >= 0) ++n;
    }
    return n;
}
int gw_MexId_CommonFighterCount(void) { return mx_common(GW_MEXID_FIGHTER); }
int gw_MexId_CommonStageCount(void) { return mx_common(GW_MEXID_STAGE); }

/* ---- tests -------------------------------------------------------------------------------------- */

/* Two synthetic installs sharing some content under different local ids: the intersection and the
 * id mapping must come out right through the real wire encoding. */
static int test_mexid_wire_intersection(void) {
    static mx_table a, b, saved;
    static mx_peer saved_peer;
    uint8_t buf[MX_WIRE_CAP];
    int i, idx, len, rv = 0;
    memset(&a, 0, sizeof a);
    memset(&b, 0, sizeof b);
    a.built = b.built = 1;
    /* 60 entries each so the list spans several chunks */
    for (i = 0; i < 26; ++i) {
        mx_add(&a, GW_MEXID_FIGHTER, i, 0x1000 + i, "vanilla");
        mx_add(&b, GW_MEXID_FIGHTER, i, 0x1000 + i, "vanilla");
    }
    mx_add(&a, GW_MEXID_FIGHTER, 0x22, 0xAAAA, "Wolf");   /* a: Wolf at ck 34 */
    mx_add(&a, GW_MEXID_FIGHTER, 0x23, 0xBBBB, "Sonic");  /* a: Sonic at ck 35 */
    mx_add(&b, GW_MEXID_FIGHTER, 0x22, 0xBBBB, "Sonic");  /* b: Sonic at ck 34 */
    mx_add(&b, GW_MEXID_FIGHTER, 0x23, 0xCCCC, "Tails");  /* b: Tails, a lacks it */
    for (i = 0; i < 30; ++i) {
        mx_add(&a, GW_MEXID_STAGE, 100 + i, 0x5000 + i, "st");
        mx_add(&b, GW_MEXID_STAGE, 200 + i, 0x5000 + i + (i == 7), "st"); /* stage 7 differs */
    }
    saved = mx_local;
    saved_peer = mx_remote;
    /* b sends to a */
    memset(&mx_remote, 0, sizeof mx_remote);
    for (idx = 0; (len = mx_wire_chunk(&b, idx, buf, sizeof buf)) > 0; ++idx) {
        if (len > MX_WIRE_CAP) {
            gw_test_fail("chunk %d is %d bytes, over the lobby limit", idx, len);
            rv = 1;
        }
        if (!mx_wire_feed(&mx_remote, buf, len)) {
            gw_test_fail("chunk %d not recognised", idx);
            rv = 1;
        }
    }
    mx_local = a;
    if (rv == 0 && !gw_MexId_PeerReady()) {
        gw_test_fail("peer list incomplete after %d chunks", idx);
        rv = 1;
    }
    if (rv == 0 && (gw_MexId_OnlineFighter(0x23) != 1 || gw_MexId_PeerCkForLocal(0x23) != 0x22 ||
                    gw_MexId_LocalCkForPeer(0x22) != 0x23 || gw_MexId_OnlineFighter(0x22) != 0 ||
                    gw_MexId_LocalCkForPeer(0x23) != -1 || gw_MexId_OnlineFighter(2) != 1)) {
        gw_test_fail("fighter mapping wrong: Sonic %d/%d/%d Wolf %d Tails %d Fox %d",
                     gw_MexId_OnlineFighter(0x23), gw_MexId_PeerCkForLocal(0x23),
                     gw_MexId_LocalCkForPeer(0x22), gw_MexId_OnlineFighter(0x22),
                     gw_MexId_LocalCkForPeer(0x23), gw_MexId_OnlineFighter(2));
        rv = 1;
    }
    if (rv == 0 && (gw_MexId_PeerExtForLocal(103) != 203 || gw_MexId_OnlineStage(107) != 0 ||
                    gw_MexId_LocalExtForPeer(229) != 129 || gw_MexId_CommonStageCount() != 29 ||
                    gw_MexId_CommonFighterCount() != 27)) {
        gw_test_fail("stage mapping / counts wrong (%d %d %d %d %d)", gw_MexId_PeerExtForLocal(103),
                     gw_MexId_OnlineStage(107), gw_MexId_LocalExtForPeer(229),
                     gw_MexId_CommonStageCount(), gw_MexId_CommonFighterCount());
        rv = 1;
    }
    /* not our message: left for the lobby */
    if (rv == 0 && mx_wire_feed(&mx_remote, (const uint8_t *) "S 1 2 3", 7) != 0) {
        gw_test_fail("a lobby message was taken as an identity chunk");
        rv = 1;
    }
    mx_local = saved;
    mx_remote = saved_peer;
    return rv;
}

static int test_mexid_global_diff(void) {
    char why[160];
    if (mx_global_diff("plco#1234,itco#abcd,mexflags#0001", "plco#1234,itco#abcd,mexflags#0001", why,
                       sizeof why) != 0) {
        gw_test_fail("same global data reported as different: %s", why);
        return 1;
    }
    if (mx_global_diff("plco#1234,itco#abcd,mexflags#0001", "plco#9999,itco#abcd,mexflags#0002", why,
                       sizeof why) != 2 || strstr(why, "PlCo.dat") == NULL || strstr(why, "feature flags") == NULL ||
        strstr(why, "ItCo") != NULL) {
        gw_test_fail("global diff not named right: \"%s\"", why);
        return 1;
    }
    return 0;
}

/* The real table on the current disc: the 26 retail fighters, every entry with an identity, and
 * the scene token of each fighter resolving back to it. Logs the table so two installs can be
 * compared by eye (grep "mexid:   "). */
static int test_mexid_disc_table(void) {
    mx_table *t;
    int i, fighters = 0, stages = 0;
    if (gw_iso_path() == NULL) {
        gw_log("mexid: no disc - skipping");
        return 0;
    }
    mx_local.built = 0;
    t = mx_tab();
    for (i = 0; i < t->n; ++i) {
        const mx_entry *e = &t->e[i];
        gw_log("mexid:   %-7s %3d %s %s", e->kind == GW_MEXID_FIGHTER ? "fighter" : "stage", e->local,
               e->hex, e->name);
        if (e->kind == GW_MEXID_FIGHTER) {
            ++fighters;
            if (gw_MexId_CkForHex(e->hex) != e->local) {
                gw_test_fail("fighter %s: token id:%s resolves to %d, not %d", e->name, e->hex,
                             gw_MexId_CkForHex(e->hex), e->local);
                return 1;
            }
        } else {
            ++stages;
        }
    }
    if (fighters < 26 || stages < 29) {
        gw_test_fail("only %d fighters / %d stages have identities (expected the 26 / 29 retail ones "
                     "at least)",
                     fighters, stages);
        return 1;
    }
    if (strncmp(gw_MexId_TokenForCk(2), "id:", 3) != 0 || strcmp(gw_MexId_TokenForCk(0x21), "ck:33") != 0) {
        gw_test_fail("scene tokens wrong: Fox \"%s\", ck 33 \"%s\"", gw_MexId_TokenForCk(2),
                     gw_MexId_TokenForCk(0x21));
        return 1;
    }
    (void) gw_MexId_GlobalHash();
    return 0;
}

/* ---- Geno slice 7: a native define is a fighter of the table ---------------------------------------- */

extern int gw_Geno_TestInstall(const char *text);
extern void gw_Geno_TestRestore(void);

/* one donor define as geno.json text; `walk` is the one attribute that makes two otherwise equal entries different */
static void mx_test_define(char *out, size_t cap, const char *key, const char *walk) {
    snprintf(out, cap,
             "{\"key\":\"%s\",\"name\":\"Test %s\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"}"
             ",\"attributes\":{\"walk_max_vel\":%s}", key, key, walk);
}

static void mx_test_install(const char *const *keys, const char *const *walks, int n) {
    char text[2048], one[400];
    int i;
    snprintf(text, sizeof text, "{\"geno\":7,\"fighters\":[");
    for (i = 0; i < n; ++i) {
        mx_test_define(one, sizeof one, keys[i], walks[i]);
        snprintf(text + strlen(text), sizeof text - strlen(text), "%s{\"define\":%s}", i ? "," : "", one);
    }
    strcat(text, "]}");
    (void) gw_Geno_TestInstall(text);
    mx_local.built = 0;
}

static int mx_test_find_define(const mx_table *t, const char *key) {
    int i;
    for (i = 0; i < t->n; ++i) {
        if (t->e[i].define && strcmp(t->e[i].key, key) == 0) return i;
    }
    return -1;
}

/* Identity of a define: stable, independent of its alias, sensitive to its content; it travels the wire with its key; a define the peer lacks
 * or holds another version of is not in common and the reason says which. Needs the disc (Mario's files are part of the identity). */
static int test_mexid_define_online(void) {
    static mx_table a, b, saved;
    static mx_peer saved_peer;
    const char *k1[] = { "alpha", "beta" }, *w1[] = { "1.8", "1.8" };
    const char *k2[] = { "beta", "alpha", "gamma", "aa1", "aa2" }, /* aliases follow the keys' order: two keys before "alpha" move it */
               *w2[] = { "1.9", "1.8", "1.8", "1.7", "1.6" };
    uint8_t buf[MX_WIRE_CAP];
    char why[200];
    int rv = 0, idx, len, ia, ib, ck_alpha_a, ck_alpha_b;
    if (gw_iso_path() == NULL) {
        gw_log("mexid: no disc - skipping the define identity test");
        return 0;
    }
    saved = mx_local;
    saved_peer = mx_remote;
    /* peer b: beta (another walk speed), alpha, gamma; ours a: alpha, beta */
    mx_test_install(k2, w2, 5);
    b = *mx_tab();
    mx_test_install(k1, w1, 2);
    a = *mx_tab();
    ia = mx_test_find_define(&a, "alpha");
    ib = mx_test_find_define(&b, "alpha");
    if (ia < 0 || ib < 0 || mx_test_find_define(&a, "beta") < 0 || mx_test_find_define(&b, "gamma") < 0) {
        gw_test_fail("a define has no entry in the identity table (alpha %d/%d)", ia, ib);
        rv = 1;
        goto done;
    }
    ck_alpha_a = a.e[ia].local;
    ck_alpha_b = b.e[ib].local;
    if (ck_alpha_a == ck_alpha_b) {
        gw_test_fail("the two installs gave 'alpha' the same alias (%d): the test cannot show alias independence", ck_alpha_a);
        rv = 1;
        goto done;
    }
    if (a.e[ia].hash != b.e[ib].hash) {
        gw_test_fail("the same define has two identities under two aliases: %s vs %s", a.e[ia].hex, b.e[ib].hex);
        rv = 1;
        goto done;
    }
    if (a.e[mx_test_find_define(&a, "beta")].hash == b.e[mx_test_find_define(&b, "beta")].hash) {
        gw_test_fail("a changed attribute did not change the identity");
        rv = 1;
        goto done;
    }
    if (a.e[ia].hash == a.e[mx_test_find_define(&a, "beta")].hash) {
        gw_test_fail("two different defines share an identity");
        rv = 1;
        goto done;
    }
    /* the peer's list reaches us through the real wire encoding, identity chunks and define messages */
    memset(&mx_remote, 0, sizeof mx_remote);
    for (idx = 0; (len = mx_wire_chunk(&b, idx, buf, sizeof buf)) > 0; ++idx) {
        if (!mx_wire_feed(&mx_remote, buf, len)) {
            gw_test_fail("identity chunk %d not recognised", idx);
            rv = 1;
            goto done;
        }
    }
    for (idx = 0; (len = mx_wire_define(&b, idx, buf, sizeof buf)) > 0; ++idx) {
        if (len > MX_WIRE_CAP || !mx_wire_feed(&mx_remote, buf, len)) {
            gw_test_fail("define message %d bad (%d bytes)", idx, len);
            rv = 1;
            goto done;
        }
    }
    mx_local = a;
    if (idx != 5 || mx_remote.nd != 5) {
        gw_test_fail("the peer's define messages: sent %d, received %d (expected 5)", idx, mx_remote.nd);
        rv = 1;
        goto done;
    }
    if (gw_MexId_OnlineFighter(ck_alpha_a) != 1 || gw_MexId_PeerCkForLocal(ck_alpha_a) != ck_alpha_b ||
        gw_MexId_LocalCkForPeer(ck_alpha_b) != ck_alpha_a) {
        gw_test_fail("alpha is not mapped %d <-> %d through the lists", ck_alpha_a, ck_alpha_b);
        rv = 1;
        goto done;
    }
    if (gw_MexId_FighterWhy(ck_alpha_a, why, sizeof why) != 1) {
        gw_test_fail("a define in common is refused: %s", why);
        rv = 1;
        goto done;
    }
    ib = mx_test_find_define(&a, "beta");
    if (gw_MexId_OnlineFighter(a.e[ib].local) != 0 || gw_MexId_FighterWhy(a.e[ib].local, why, sizeof why) != 0 ||
        strstr(why, "another version") == NULL || strstr(why, "Test beta") == NULL || strlen(why) > 79) {
        gw_test_fail("a define the peer holds in another version: \"%s\"", why);
        rv = 1;
        goto done;
    }
    /* the host's scene names a define by identity: the refusal says WHICH one and what differs (the peer announced its keys) */
    {
        char scene[160];
        snprintf(scene, sizeof scene, "mode=vs;p1=id:%s/c0/hu;p2=id:%s/c0/hu", b.e[mx_test_find_define(&b, "gamma")].hex,
                 b.e[mx_test_find_define(&b, "beta")].hex);
        if (!gw_MexId_PeerDefinesComplete() || gw_MexId_SceneCheck(scene, why, sizeof why) != 2 ||
            strstr(why, "Geno fighter gamma (not installed here)") == NULL ||
            strstr(why, "Geno fighter beta (you have another version of it)") == NULL) {
            gw_test_fail("the scene check does not name the defines: \"%s\"", why);
            rv = 1;
            goto done;
        }
    }
    /* we hold a define the peer lacks: swap roles (the peer's list is a's, ours is b's with gamma) */
    memset(&mx_remote, 0, sizeof mx_remote);
    for (idx = 0; (len = mx_wire_chunk(&a, idx, buf, sizeof buf)) > 0; ++idx) mx_wire_feed(&mx_remote, buf, len);
    for (idx = 0; (len = mx_wire_define(&a, idx, buf, sizeof buf)) > 0; ++idx) mx_wire_feed(&mx_remote, buf, len);
    mx_local = b;
    ib = mx_test_find_define(&b, "gamma");
    if (gw_MexId_FighterWhy(b.e[ib].local, why, sizeof why) != 0 || strstr(why, "doesn't have it") == NULL ||
        strstr(why, "Test gamma") == NULL) {
        gw_test_fail("a define the peer lacks: \"%s\"", why);
        rv = 1;
        goto done;
    }
    /* scene tokens name it by identity and resolve back */
    if (strncmp(gw_MexId_TokenForCk(b.e[ib].local), "id:", 3) != 0 || gw_MexId_CkForHex(b.e[ib].hex) != b.e[ib].local ||
        strcmp(gw_MexId_DefineKey(b.e[ib].local), "gamma") != 0 || gw_MexId_DefineKey(2)[0] != 0) {
        gw_test_fail("scene token / key of a define wrong");
        rv = 1;
        goto done;
    }
    /* before the peer's list is complete nothing is refused on a guess */
    memset(&mx_remote, 0, sizeof mx_remote);
    if (gw_MexId_FighterWhy(b.e[ib].local, why, sizeof why) != 1) {
        gw_test_fail("a define is refused before the peer's list arrived");
        rv = 1;
    }
done:
    gw_Geno_TestRestore();
    mx_local = saved;
    mx_local.built = 0; /* rebuilt from the real registry on the next use */
    mx_remote = saved_peer;
    return rv;
}

void gw_mexid_tests_register(void) {
    gw_test_register("mexid_define_online", test_mexid_define_online);
    gw_test_register("mexid_wire_intersection", test_mexid_wire_intersection);
    gw_test_register("mexid_global_diff", test_mexid_global_diff);
    gw_test_register("mexid_disc_table", test_mexid_disc_table);
}
