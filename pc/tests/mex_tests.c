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
    /* ChKind_Max is the retail "none" sentinel and must keep its value (it is stored and compared
     * everywhere); the m-ex character kinds follow it. Fighter kinds, retail + the m-ex slots, must
     * stay below 64: the animation code compares fp->kind with a 6-bit field (x597_bits). */
    if ((int)ChKind_Max != 0x21 || (int)ChKind_Mex0 != 0x22 || (int)Ft_Kind_Mex0 != 0x21 ||
        (int)Ft_Kind_Max > 64)
    {
        TestFail("kind layout: expected ChKind_Max 0x21, ChKind_Mex0 0x22, Ft_Kind_Mex0 0x21, "
                 "Ft_Kind_Max <= 64");
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

/* ---- m-ex Tier C hook surface (OnFrame) ------------------------------------------------
 * Exercises the native re-expression of the Fighter OnFrame table-slot override. Registration
 * writes a native fn into a flat per-(event,kind) array; the dispatch runs the override if set,
 * else the vanilla entry. Clearing (NULL) must restore vanilla. */

extern int Mex_HookRegister(int event, int kind, void (*fn)(void*));
extern void Mex_OnFrameDispatch(int kind, void* gobj, void* vanilla);

/* Mirrors GW_MEX_EVENT_ON_FRAME in pc/platform/gw.h. The game call sites dispatch through the
 * per-event Mex_*Dispatch wrappers, so this id is only needed here to register. */
enum { MEX_EVENT_ON_FRAME = 3 };

static int mex_onframe_hook_calls;
static int mex_onframe_vanilla_calls;

static void mex_onframe_hook(void* gobj)
{
    (void) gobj;
    mex_onframe_hook_calls++;
}

static void mex_onframe_vanilla(void* gobj)
{
    (void) gobj;
    mex_onframe_vanilla_calls++;
}

static int test_mex_onframe_hook_register_and_clear(void) {
    if (Mex_HookRegister(999, 0, mex_onframe_hook) != 0) {
        TestFail("Mex_HookRegister should reject an out-of-range event");
        return 1;
    }

    /* No override -> vanilla entry runs. */
    mex_onframe_hook_calls = 0;
    mex_onframe_vanilla_calls = 0;
    Mex_OnFrameDispatch(Ft_Kind_Fox, NULL, (void*) mex_onframe_vanilla);
    if (mex_onframe_hook_calls != 0 || mex_onframe_vanilla_calls != 1) {
        TestFail("OnFrame dispatch without an override should run the vanilla entry");
        return 1;
    }

    /* Registered override replaces the vanilla entry. */
    if (Mex_HookRegister(MEX_EVENT_ON_FRAME, Ft_Kind_Fox, mex_onframe_hook) != 1) {
        TestFail("Mex_HookRegister rejected a valid (event, kind)");
        return 1;
    }
    mex_onframe_hook_calls = 0;
    mex_onframe_vanilla_calls = 0;
    Mex_OnFrameDispatch(Ft_Kind_Fox, NULL, (void*) mex_onframe_vanilla);
    if (mex_onframe_hook_calls != 1 || mex_onframe_vanilla_calls != 0) {
        TestFail("registered OnFrame hook should run instead of the vanilla entry");
        return 1;
    }

    /* Clearing (NULL) restores the vanilla entry. */
    if (Mex_HookRegister(MEX_EVENT_ON_FRAME, Ft_Kind_Fox, NULL) != 1) {
        TestFail("Mex_HookRegister should accept a NULL fn to clear the slot");
        return 1;
    }
    mex_onframe_hook_calls = 0;
    mex_onframe_vanilla_calls = 0;
    Mex_OnFrameDispatch(Ft_Kind_Fox, NULL, (void*) mex_onframe_vanilla);
    if (mex_onframe_hook_calls != 0 || mex_onframe_vanilla_calls != 1) {
        TestFail("clearing the OnFrame slot should restore the vanilla entry");
        return 1;
    }
    return 0;
}

/* ---- m-ex Tier C predicate surface (Category 2, e.g. OnFloat) --------------------------
 * A predicate returns a value rather than merely running: dispatch returns the override's
 * result, else the vanilla predicate's, else 0. Register and clear must both work. */

extern int Mex_PredicateRegister(int event, int kind, int (*fn)(void*));
extern int Mex_GObjPredDispatch(int event, int kind, void* gobj, void* vanilla);

static int mex_pred_hook(void* gobj)
{
    (void) gobj;
    return 1;
}

static int mex_pred_vanilla(void* gobj)
{
    (void) gobj;
    return 2;
}

static int test_mex_predicate_register_and_clear(void) {
    if (Mex_PredicateRegister(999, 0, mex_pred_hook) != 0) {
        TestFail("Mex_PredicateRegister should reject an out-of-range event");
        return 1;
    }
    if (Mex_GObjPredDispatch(MEX_EVENT_ON_FRAME, Ft_Kind_Fox, NULL, NULL) != 0) {
        TestFail("predicate dispatch with neither override nor vanilla should return 0");
        return 1;
    }
    if (Mex_GObjPredDispatch(MEX_EVENT_ON_FRAME, Ft_Kind_Fox, NULL,
                             (void*) mex_pred_vanilla) != 2) {
        TestFail("predicate dispatch without an override should return the vanilla result");
        return 1;
    }
    if (Mex_PredicateRegister(MEX_EVENT_ON_FRAME, Ft_Kind_Fox, mex_pred_hook) != 1) {
        TestFail("Mex_PredicateRegister rejected a valid (event, kind)");
        return 1;
    }
    if (Mex_GObjPredDispatch(MEX_EVENT_ON_FRAME, Ft_Kind_Fox, NULL,
                             (void*) mex_pred_vanilla) != 1) {
        TestFail("registered predicate should return its own result, not vanilla's");
        return 1;
    }
    if (Mex_PredicateRegister(MEX_EVENT_ON_FRAME, Ft_Kind_Fox, NULL) != 1) {
        TestFail("Mex_PredicateRegister should accept NULL to clear the slot");
        return 1;
    }
    if (Mex_GObjPredDispatch(MEX_EVENT_ON_FRAME, Ft_Kind_Fox, NULL,
                             (void*) mex_pred_vanilla) != 2) {
        TestFail("clearing the predicate slot should restore the vanilla result");
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
    TestRegister("mex_onframe_hook_register_and_clear",
                 test_mex_onframe_hook_register_and_clear);
    TestRegister("mex_predicate_register_and_clear",
                 test_mex_predicate_register_and_clear);
}
