/* Game-side tests: link the real gwtool-retargeted objects and run in guest memory.
 * Compiled through the game pipeline, so `TestRegister`/`TestFail` resolve to gw_TestRegister/
 * gw_TestFail after gwtool's symbol prefixing. Registered from gw_MexTestRegisterAll. */

#include <melee/ft/forward.h>
#include <melee/gm/forward.h>
#include <melee/gm/gm_1A3F.h>

extern void TestRegister(const char *name, int (*fn)(void));
extern void TestFail(const char *msg);

#define MEX_SPECIAL_COUNT 7

static int test_external_special_range(void) {
    if ((int)ChKind_Max != 0x21 || (int)Ft_Kind_Max != 0x21) {
        TestFail("ChKind_Max and Ft_Kind_Max should both be 0x21");
        return 1;
    }
    if ((int)ChKind_Max - (int)CKind_Playable_Count != MEX_SPECIAL_COUNT) {
        TestFail("external special range is not 7 wide");
        return 1;
    }
    return 0;
}

/* m-ex's external-ID shift computes SpecialStart = FtExtNum - 7 and remaps each special to
 * SpecialStart + (extID - CKind_Playable_Count). Under vanilla counts that must be the identity,
 * which is the claim the port's char-ID work rests on. */
static int test_special_id_remap_is_identity(void) {
    const int start = (int)ChKind_Max - MEX_SPECIAL_COUNT;
    int ext;
    for (ext = (int)CKind_Playable_Count; ext < (int)ChKind_Max; ++ext) {
        const int remapped = start + (ext - (int)CKind_Playable_Count);
        if (remapped != ext) {
            TestFail("special remap is not identity under vanilla counts");
            return 1;
        }
    }
    return 0;
}

/* Mirrors struct lbHeap_HeapDesc in src/melee/lb/lbheap.c, which is not in a header. Guarding the
 * binary layout is the point: m-ex's heap expansion adds a slot and moves the terminator, and the
 * loop bounds in lbHeap_80015F3C read this table directly. */
struct mex_heap_desc {
    unsigned int idx;
    unsigned int type;
    unsigned int prev_idx;
    unsigned int size;
};
extern struct mex_heap_desc lbHeap_803BA380[];

static int test_heap_table_shape(void) {
    if (lbHeap_803BA380[4].idx != 6) {
        TestFail("heap table slot 4 should be the added heap id 6");
        return 1;
    }
    if (lbHeap_803BA380[4].size != 0x20) {
        TestFail("added heap 6 size should be 0x20");
        return 1;
    }
    if (lbHeap_803BA380[5].idx != 7) {
        TestFail("heap descriptor terminator should be 7 on TARGET_PC");
        return 1;
    }
    return 0;
}

static int test_1p_mode_classification(void) {
    if (!gm_Is1PMode(GM_CLASSIC)) {
        TestFail("gm_Is1PMode(CLASSIC) should be true");
        return 1;
    }
    if (!gm_Is1PMode(GM_ADVENTURE)) {
        TestFail("gm_Is1PMode(ADVENTURE) should be true");
        return 1;
    }
    if (!gm_Is1PMode(GM_TRAINING)) {
        TestFail("gm_Is1PMode(TRAINING) should be true");
        return 1;
    }
    if (!gm_Is1PMode(GM_TARGET_TEST)) {
        TestFail("gm_Is1PMode(TARGET_TEST) should be true");
        return 1;
    }
    return 0;
}

static int test_vs_mode_is_not_1p(void) {
    if (gm_Is1PMode(GM_VS)) {
        TestFail("gm_Is1PMode(VS) should be false");
        return 1;
    }
    if (gm_Is1PMode(GM_TOURNAMENT)) {
        TestFail("gm_Is1PMode(TOURNAMENT) should be false");
        return 1;
    }
    return 0;
}

void MexTestRegisterAll(void) {
    TestRegister("gm_Is1PMode_1p_modes", test_1p_mode_classification);
    TestRegister("gm_Is1PMode_vs_modes", test_vs_mode_is_not_1p);
    TestRegister("heap_table_shape", test_heap_table_shape);
    TestRegister("charid_special_range_is_7", test_external_special_range);
    TestRegister("charid_remap_identity_vanilla", test_special_id_remap_is_identity);
}
