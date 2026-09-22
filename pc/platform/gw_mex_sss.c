/* gw_mex_sss.c - m-ex stage-select icon table. See gw_mex_sss.h for the layout and the evidence.
 *
 * WHAT WAS DUMPED, NOT ASSUMED (scratch dumper over Akaneia.iso and SSBM ACE Build v2.0.0.iso):
 *   - mexData +0x04 -> menu, menu +0x08 -> the SSS icon table. Stride 0x20 exactly: rows 0..28
 *     reproduce the port's compiled-in mnStageSel_803F06D0[0..28] field for field, e.g. row 1 is
 *     w2 = 0x02010C00 (x8=2, x9=1, xA=0x0C, byte 3 = 0), floats 3.1/2.7/1.0/1.0, and +0x1C = 11
 *     where the port's row 1 has stkind = 0x0B.
 *   - metadata +0x18 = sss_icon_count: 67 on Akaneia, 163 on ACE.
 *   - Row 61 on Akaneia has +0x1C = 0x125 = external 293 = Meta Crystal, the confirming entry.
 *   - A blank slot is w2 = 0x01000000 with +0x1C = 0 (type byte 1 = "locked"); the "Random" icon
 *     has type 3 and preview id 0xFF, and Akaneia has TWO of them (rows 29 and 60), so the
 *     random icon is NOT at a fixed index and must be found by its type byte.
 *   - The icon MODELS are not here: MnSlMap.usd/.dat carries a `mexMapData` public symbol (both
 *     discs) whose +0x0C positions model has exactly sss_icon_count sibling joints - 67 on
 *     Akaneia, 163 on ACE. Game code reads that itself.
 *
 * This module caches nothing. Every call re-resolves through mexData, so it cannot hold a
 * pointer into a MEM1 region the test harness has since restored (docs/HANDOFF.md section 6).
 */
#include "gw.h"
#include "gw_test.h"
#include "gw_mex_grfunction.h"
#include "gw_mex_sss.h"

#include <stdint.h>

/* From gw_mex_ftfunction_runtime.c (the shared m-ex runtime environment). */
extern void gw_Mex_RuntimeInit(void);
extern uint32_t gw_Mex_MexData(uint32_t *base, uint32_t *size);
extern int gw_DVDFileExists(const char *path);

#define SSS_OFF_METADATA 0x00u
#define SSS_OFF_MENU 0x04u
#define SSS_MENU_OFF_SSS 0x08u
/* MexMetaData: u8 major, u8 minor, s16 flags, then s32 counts; sss_icon_count is count 5. */
#define SSS_META_OFF_ICON_COUNT (4u + 5u * 4u)
#define SSS_ROW_STRIDE 0x20u

/* Resolve {count, table} together - they are only ever meaningful as a pair. Returns 0 and
 * leaves *out_table 0 on a vanilla disc or on any implausible reading. */
static int sss_resolve(uint32_t *out_table) {
    uint32_t root, base, size, meta, menu, tbl;
    int n;

    *out_table = 0u;
    /* An icon names an EXTERNAL stage id. Without the m-ex stage tables that id resolves to
     * nothing, so an expanded SSS would hand the loader a stage that does not exist. Tie the two
     * together: this is also what keeps ACE's 163 icons off while ST_MEX_EXT_MAX is 313. */
    if (gw_Mex_GrExternalCount() <= 0) {
        return 0;
    }
    gw_Mex_RuntimeInit();
    root = gw_Mex_MexData(&base, &size);
    if (root == 0u || size == 0u) {
        return 0;
    }
#define SSS_IN(a, len) ((a) >= base && (uint64_t) (a) + (len) <= (uint64_t) base + size)
    if (!SSS_IN(root + SSS_OFF_MENU, 4u)) {
        return 0;
    }
    meta = gw_r32((const void *) (uintptr_t) (root + SSS_OFF_METADATA));
    menu = gw_r32((const void *) (uintptr_t) (root + SSS_OFF_MENU));
    if (!SSS_IN(meta, SSS_META_OFF_ICON_COUNT + 4u) || !SSS_IN(menu, SSS_MENU_OFF_SSS + 4u)) {
        return 0;
    }
    n = (int) gw_r32((const void *) (uintptr_t) (meta + SSS_META_OFF_ICON_COUNT));
    tbl = gw_r32((const void *) (uintptr_t) (menu + SSS_MENU_OFF_SSS));
    if (n <= 0 || n > GW_MEX_SSS_MAX) {
        gw_log("mex sss: implausible icon count %d (the port's table holds %d rows) - the "
               "expanded stage select stays off",
               n, GW_MEX_SSS_MAX);
        return 0;
    }
    /* Prove the stride by requiring the whole table to lie inside MxDt.dat. A wrong stride or a
     * wrong root word fails here rather than reading a neighbouring structure as icons. */
    if (!SSS_IN(tbl, (uint32_t) n * SSS_ROW_STRIDE)) {
        gw_log("mex sss: icon table 0x%08X does not hold %d x %u bytes inside MxDt.dat - the "
               "expanded stage select stays off",
               tbl, n, SSS_ROW_STRIDE);
        return 0;
    }
#undef SSS_IN
    *out_table = tbl;
    return n;
}

int gw_Mex_SssIconCount(void) {
    uint32_t tbl;
    return sss_resolve(&tbl);
}

void *gw_Mex_SssTable(void) {
    uint32_t tbl;
    return sss_resolve(&tbl) != 0 ? (void *) (uintptr_t) tbl : NULL;
}

/* ---- tests ------------------------------------------------------------------------------- */

/* Row field reads for the tests only. Game code never uses these - it copies the rows wholesale
 * through gwtool, which byte-swaps for it. */
static uint32_t sss_word(uint32_t tbl, int i, int w) {
    return gw_r32((const void *) (uintptr_t) (tbl + (uint32_t) i * SSS_ROW_STRIDE +
                                              (uint32_t) w * 4u));
}

static int test_mex_sss_table(void) {
    uint32_t tbl;
    int n, i, blanks = 0, randoms = 0, reachable = 0;

    if (gw_iso_path() == NULL || (n = sss_resolve(&tbl)) == 0) {
        gw_log("mex sss: no m-ex stage-select table on this disc - skipping");
        return 0;
    }
    if (n < 30) {
        gw_test_fail("sss_icon_count is %d - fewer icons than retail's 30", n);
        return 1;
    }
    /* Rows 0..28 must reproduce the port's own retail rows, or every index below names a
     * different stage. Row 1 is the cheapest complete check: all four packed bytes and the id. */
    if (sss_word(tbl, 1, 2) != 0x02010C00u || (int32_t) sss_word(tbl, 1, 7) != 11) {
        gw_test_fail("sss row 1 is w2=0x%08X id=%d, expected 0x02010C00 / 11 - the table is not "
                     "the port's own retail rows at stride 0x%X",
                     sss_word(tbl, 1, 2), (int32_t) sss_word(tbl, 1, 7), SSS_ROW_STRIDE);
        return 1;
    }
    for (i = 0; i < n; i++) {
        uint32_t w2 = sss_word(tbl, i, 2);
        int32_t ext = (int32_t) sss_word(tbl, i, 7);
        int type = (int) ((w2 >> 24) & 0xFFu);
        /* The whole reason the id moved out of the packed word: byte 3 must be free. */
        if ((w2 & 0xFFu) != 0u) {
            gw_test_fail("sss row %d has w2 byte 3 = 0x%02X; the external id is supposed to live "
                         "in word 7, so the row layout is not what this port assumes",
                         i, (unsigned) (w2 & 0xFFu));
            return 1;
        }
        if (ext < 0 || ext >= gw_Mex_GrExternalCount()) {
            gw_test_fail("sss row %d names external stage %d, outside the %d-entry stage map", i,
                         ext, gw_Mex_GrExternalCount());
            return 1;
        }
        if (type == 3) {
            randoms++;
        } else if (type <= 1 || ext == 0) {
            blanks++;
        } else if (gw_Mex_GrKindForExt(ext) >= 0) {
            reachable++;
        }
    }
    if (randoms == 0) {
        gw_test_fail("no Random icon (type 3) in %d rows", n);
        return 1;
    }
    if (reachable <= 29) {
        gw_test_fail("only %d selectable icons resolve to a stage - the expanded screen would "
                     "show no more than retail",
                     reachable);
        return 1;
    }
    gw_log("test mex_sss_table: %d icons - %d selectable, %d blank, %d random", n, reachable,
           blanks, randoms);
    return 0;
}

/* The entry this whole task exists for: a custom stage that a player can actually point at. */
static int test_mex_sss_meta_crystal(void) {
    uint32_t tbl;
    int n, i;

    if (gw_iso_path() == NULL || (n = sss_resolve(&tbl)) == 0) {
        gw_log("mex sss: no m-ex stage-select table on this disc - skipping");
        return 0;
    }
    for (i = 0; i < n; i++) {
        if ((int32_t) sss_word(tbl, i, 7) == 293) {
            int k = gw_Mex_GrKindForExt(293);
            if (((sss_word(tbl, i, 2) >> 24) & 0xFFu) < 2u) {
                gw_test_fail("sss row %d is Meta Crystal but its type byte is %u (not shown)", i,
                             (unsigned) ((sss_word(tbl, i, 2) >> 24) & 0xFFu));
                return 1;
            }
            if (k >= 0 && gw_Mex_GrFile(k) == NULL && !gw_DVDFileExists("/GrOMc.dat")) {
                /* the table names it, but its stage mod is not mounted (gw_mods.h) */
                gw_log("test mex_sss_meta_crystal: /GrOMc.dat is not mounted - skipping");
                return 0;
            }
            if (k < 0 || gw_Mex_GrFile(k) == NULL) {
                gw_test_fail("sss row %d names external 293, which maps to internal %d with no "
                             "file on this disc",
                             i, k);
                return 1;
            }
            gw_log("test mex_sss_meta_crystal: icon %d = external 293 = internal %d (%s)", i, k,
                   gw_Mex_GrFile(k));
            return 0;
        }
    }
    gw_test_fail("no stage-select icon names external stage 293 (Meta Crystal) in %d rows", n);
    return 1;
}

void gw_mex_sss_tests_register(void) {
    gw_test_register("mex_sss_table", test_mex_sss_table);
    gw_test_register("mex_sss_meta_crystal", test_mex_sss_meta_crystal);
}
