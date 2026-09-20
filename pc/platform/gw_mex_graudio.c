/* gw_mex_graudio.c - m-ex custom STAGES: per-stage audio (Arch_Map_Audio) and line-type rows
 * (Arch_Map_LineTypeData). See gw_mex_graudio.h for the shape and for what was dumped.
 *
 * Reimplemented in original C from m-ex's published behaviour (`StageAudio References/
 * GetStageSSMID.asm` and `SetEcho.asm` read bytes 0 and 1 of the Audio row; `Header.s` gives the
 * Arch_Map field order). None of m-ex's sources are vendored.
 *
 * This file deliberately does NOT share gw_mex_grfunction.c's cached table pointers. Two reasons:
 * that file is owned elsewhere, and its `gr_tables()` refuses outright on a disc whose internal
 * stage count exceeds the port's stage_datas[] (ACE's 155 today). Stage AUDIO has no such
 * dependency - the audio table is filled from mexData into an array this port sizes itself - so
 * it must keep working on a disc where the grFunction path has stood itself down.
 */
#include "gw.h"
#include "gw_test.h"
#include "gw_mex_graudio.h"

#include <string.h>

/* mexData root words (m-ex Header.s). */
#define GRA_OFF_METADATA 0x00u
#define GRA_OFF_MAP 0x28u /* Arch_Map */

/* MexMetaData: u8 major, u8 minor, u16 flags, then the s32 counts. */
#define GRA_META_INT_STAGES (4u + 3u * 4u)

/* Arch_Map fields this file uses. */
#define GRA_MAP_AUDIO 0x04u
#define GRA_MAP_LINETYPE 0x08u

#define GRA_AUDIO_STRIDE 3u
#define GRA_LINETYPE_STRIDE 8u

/* The first m-ex-ADDED internal stage id: 0..70 are the vanilla stages the port compiles in. */
#define GRA_FIRST_NEW 71
#define GRA_VANILLA_ROWS 71

/* A loose upper bound whose only job is to stop a misread word sizing a loop. Akaneia has 96
 * internal stages and ACE 155; m-ex's own external id field is an s32, so there is no natural
 * ceiling to quote here. */
#define GRA_MAX_STAGES 256

/* From gw_mex_ftfunction_runtime.c (the shared PPC runtime environment). */
extern void gw_Mex_RuntimeInit(void);
extern uint32_t gw_Mex_MexData(uint32_t *base, uint32_t *size);

static uint32_t gra_root, gra_base, gra_size, gra_audio, gra_linetype;
static int gra_count;
static int gra_ready; /* 1 = usable, -1 = tried and unusable for THIS mexData */

static int gra_in(uint32_t a, uint32_t len) {
    return gra_size != 0u && a >= gra_base && (uint64_t) a + len <= (uint64_t) gra_base + gra_size;
}

static uint32_t gra_rd(uint32_t a) {
    return gra_in(a, 4u) ? gw_r32((const void *) (uintptr_t) a) : 0u;
}

/* Resolve the two tables. Re-resolves by itself when mexData has moved or been dropped - the
 * archive lives in MEM1 but these statics do not, and gw_test.c restores a MEM1 snapshot between
 * tests without restoring platform state. Keying the cache on the live (root, base, size) means
 * this file needs no invalidation hook of its own. */
static int gra_tables(void) {
    uint32_t root, base = 0u, size = 0u, meta, map;

    gw_Mex_RuntimeInit();
    root = gw_Mex_MexData(&base, &size);
    if (root != gra_root || base != gra_base || size != gra_size) {
        gra_root = root;
        gra_base = base;
        gra_size = size;
        gra_audio = gra_linetype = 0u;
        gra_count = 0;
        gra_ready = 0;
    }
    if (gra_ready != 0) {
        return gra_ready > 0;
    }
    gra_ready = -1;
    if (root == 0u) {
        return 0; /* a vanilla disc: not an error, there is simply nothing to add */
    }
    meta = gra_rd(root + GRA_OFF_METADATA);
    map = gra_rd(root + GRA_OFF_MAP);
    if (!gra_in(meta, 0x3Cu) || !gra_in(map, 0x18u)) {
        gw_log("graudio: mexData has no usable Arch_Map (metadata 0x%08X map 0x%08X) - stage "
               "audio stays vanilla",
               meta, map);
        return 0;
    }
    gra_count = (int) gra_rd(meta + GRA_META_INT_STAGES);
    if (gra_count <= GRA_VANILLA_ROWS || gra_count > GRA_MAX_STAGES) {
        gw_log("graudio: implausible internal_stage_count %d - stage audio stays vanilla",
               gra_count);
        gra_count = 0;
        return 0;
    }
    gra_audio = gra_rd(map + GRA_MAP_AUDIO);
    gra_linetype = gra_rd(map + GRA_MAP_LINETYPE);
    /* Prove each table really spans `count` entries at the stride before trusting one row of it.
     * A wrong stride here does not error, it returns plausible-looking numbers - the same trap
     * that made StageNames read the string pool as pointers. */
    if (!gra_in(gra_audio, (uint32_t) gra_count * GRA_AUDIO_STRIDE)) {
        gw_log("graudio: Arch_Map_Audio 0x%08X does not hold %d x %u bytes - stage audio stays "
               "vanilla",
               gra_audio, gra_count, GRA_AUDIO_STRIDE);
        gra_audio = 0u;
    }
    if (!gra_in(gra_linetype, (uint32_t) gra_count * GRA_LINETYPE_STRIDE)) {
        gw_log("graudio: Arch_Map_LineTypeData 0x%08X does not hold %d x %u bytes - added stages "
               "keep the fallback line types",
               gra_linetype, gra_count, GRA_LINETYPE_STRIDE);
        gra_linetype = 0u;
    }
    if (gra_audio == 0u && gra_linetype == 0u) {
        gra_count = 0;
        return 0;
    }
    gra_ready = 1;
    gw_log("graudio: mexData stage audio ready: %d internal stages (Audio 0x%08X, LineTypeData "
           "0x%08X)",
           gra_count, gra_audio, gra_linetype);
    return 1;
}

int gw_Mex_GrAudioCount(void) { return gra_tables() ? gra_count : 0; }

int gw_Mex_GrAudioByte(int grkind, int which) {
    uint32_t a;
    if (!gra_tables() || gra_audio == 0u || grkind < 0 || grkind >= gra_count || which < 0 ||
        which > 2)
    {
        return -1;
    }
    a = gra_audio + (uint32_t) grkind * GRA_AUDIO_STRIDE + (uint32_t) which;
    if (!gra_in(a, 1u)) {
        return -1;
    }
    return (int) gw_r8((const void *) (uintptr_t) a);
}

int gw_Mex_GrLineTypeRow(int grkind) {
    static int8_t cache[GRA_MAX_STAGES];
    static uint32_t cache_for; /* the gra_linetype the cache was built against */
    uint32_t want;
    int j;

    if (grkind < 0) {
        return -1;
    }
    if (grkind < GRA_VANILLA_ROWS) {
        return grkind; /* the port compiles this row in already */
    }
    if (!gra_tables() || gra_linetype == 0u || grkind >= gra_count) {
        return -1;
    }
    if (cache_for != gra_linetype) {
        cache_for = gra_linetype;
        memset(cache, 0, sizeof cache);
    }
    if (cache[grkind] != 0) {
        return cache[grkind] > 0 ? cache[grkind] - 1 : -1;
    }

    /* The row's pointer is an absolute VANILLA guest address, never relocated, and on both discs
     * it is always one of the 71 vanilla rows' pointers. Find WHICH vanilla row by matching
     * against the table's own rows 0..70 - data-driven, so no guest address is hard-coded here
     * and a build that moved its .data would be caught rather than silently mis-indexed. */
    want = gra_rd(gra_linetype + (uint32_t) grkind * GRA_LINETYPE_STRIDE + 4u);
    if (want != 0u) {
        for (j = 0; j < GRA_VANILLA_ROWS; j++) {
            if (gra_rd(gra_linetype + (uint32_t) j * GRA_LINETYPE_STRIDE + 4u) == want) {
                cache[grkind] = (int8_t) (j + 1);
                gw_log("graudio: stage %d takes vanilla stage %d's line types (0x%08X)", grkind,
                       j, want);
                return j;
            }
        }
    }
    cache[grkind] = -1;
    gw_log("graudio: stage %d's Arch_Map_LineTypeData pointer 0x%08X is not one of the 71 "
           "vanilla rows - the caller keeps its fallback",
           grkind, want);
    return -1;
}

/* ---- tests -------------------------------------------------------------------------------
 * Headless under --test, against whatever disc was passed. A vanilla disc has no mexData and
 * they skip; on an m-ex disc they assert the dumped layout, because a silent layout drift here
 * is a stage that is simply mute, with nothing in the log to say why. */

/* Rows 0..70 of Arch_Map_Audio are byte-identical to the port's compiled s32_arr_803BB6B0 on
 * both Akaneia and ACE. That table is a static inside a game TU and not reachable from here, so
 * spot-check three rows whose values are distinctive enough to catch a stride or index-space
 * slip: a plain row, the one vanilla row with a non-default echo, and the one whose two echo
 * bytes differ. */
static const struct {
    int grkind;
    int ssm, echo, echo2;
} gra_vanilla_spot[] = {
    { 2, 0x22, 0x01, 0x01 },  /* Princess Peach's Castle */
    { 7, 0x37, 0x18, 0x18 },  /* Temple - bank 55 is "none", echo is not default */
    { 15, 0x2C, 0x01, 0x88 }, /* the row whose echo2 differs from echo */
};

static int test_graudio_table(void) {
    int n = gw_Mex_GrAudioCount();
    size_t k;
    int i, above63 = 0;

    if (gw_iso_path() == NULL || n == 0) {
        gw_log("graudio: no m-ex stage audio on this disc - skipping");
        return 0;
    }
    for (k = 0; k < sizeof gra_vanilla_spot / sizeof gra_vanilla_spot[0]; k++) {
        int g = gra_vanilla_spot[k].grkind;
        int a = gw_Mex_GrAudioByte(g, 0), b = gw_Mex_GrAudioByte(g, 1),
            c = gw_Mex_GrAudioByte(g, 2);
        if (a != gra_vanilla_spot[k].ssm || b != gra_vanilla_spot[k].echo ||
            c != gra_vanilla_spot[k].echo2)
        {
            gw_test_fail("Arch_Map_Audio[%d] = {%d,%d,%d}, expected {%d,%d,%d} - the table's "
                         "stride or index space has drifted",
                         g, a, b, c, gra_vanilla_spot[k].ssm, gra_vanilla_spot[k].echo,
                         gra_vanilla_spot[k].echo2);
            return 1;
        }
    }
    /* Out of range must be -1, never a half-validated number: the caller puts byte 0 straight
     * into a bank request. */
    if (gw_Mex_GrAudioByte(-1, 0) != -1 || gw_Mex_GrAudioByte(n, 0) != -1 ||
        gw_Mex_GrAudioByte(0, 3) != -1)
    {
        gw_test_fail("out-of-range Arch_Map_Audio reads must return -1");
        return 1;
    }
    /* Every added row must be a bank the port can name at all. Akaneia's added ids run 55..77
     * and ACE's 38..100; both discs have some past 63, which is the whole reason the by-index
     * request path exists. */
    for (i = GRA_FIRST_NEW; i < n; i++) {
        int ssm = gw_Mex_GrAudioByte(i, 0);
        if (ssm < 0 || ssm > 255) {
            gw_test_fail("Arch_Map_Audio[%d] bank id %d is not readable", i, ssm);
            return 1;
        }
        if (ssm > 63) {
            above63++;
        }
    }
    gw_log("graudio: %d internal stages, %d added rows with a bank index past the u64 mask", n,
           above63);
    return 0;
}

static int test_graudio_linetype_rows(void) {
    int n = gw_Mex_GrAudioCount();
    int i;

    if (gw_iso_path() == NULL || n == 0) {
        return 0;
    }
    if (gw_Mex_GrLineTypeRow(0) != 0 || gw_Mex_GrLineTypeRow(GRA_FIRST_NEW - 1) != GRA_FIRST_NEW - 1)
    {
        gw_test_fail("a vanilla stage's line-type row must be itself");
        return 1;
    }
    if (gw_Mex_GrLineTypeRow(-1) != -1 || gw_Mex_GrLineTypeRow(n) != -1) {
        gw_test_fail("out-of-range line-type rows must be -1");
        return 1;
    }
    /* The contract mpLib depends on: every added stage resolves to a row that EXISTS in the
     * port's compiled mpLib_803BF248[0x47]. If m-ex ever authors line-type data of its own this
     * fails here rather than by indexing past that table at runtime. */
    for (i = GRA_FIRST_NEW; i < n; i++) {
        int row = gw_Mex_GrLineTypeRow(i);
        if (row < 0 || row >= GRA_VANILLA_ROWS) {
            gw_test_fail("added stage %d resolves to line-type row %d, which is not one of the "
                         "%d vanilla rows",
                         i, row, GRA_VANILLA_ROWS);
            return 1;
        }
    }
    return 0;
}

void gw_mex_graudio_tests_register(void) {
    gw_test_register("graudio_table", test_graudio_table);
    gw_test_register("graudio_linetype_rows", test_graudio_linetype_rows);
}
