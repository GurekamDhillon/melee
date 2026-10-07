/* atlas-erase: the plan table of Erase Data (pc/platform/gw_ui_erase.h): the ordered list of state-changing calls of each of the six retail operations
 * (src/melee/mn/mndatadel.c, the five case arms of fn_8024F318 and mnDataDel_8024EA6C), with every animation line removed. Tested with FAKE callbacks only:
 * nothing here touches a real save. */
#include "atlas_check.h"
#include "../platform/gw_ui_erase.h"

static int LOG[64], NLOG;
static void rec(int id) { if (NLOG < 64) LOG[NLOG++] = id; }

/* What Step 1 read out of the retail code, in the order the retail code makes the calls (the test restates it, so a change to the table must be a change here too). */
static const int RETAIL[6][8] = {
    { AT_ER_GM_8016505C, AT_ER_MAINLIB_8015F464, AT_ER_GM_801729EC, AT_ER_SAVE, AT_ER_MAINLIB_8015DB80, -1 },                                  /* case 0 */
    { AT_ER_GM_801647D0, AT_ER_MAINLIB_8015F490, AT_ER_STAGE_RELOCK, AT_ER_GM_801729EC, AT_ER_SAVE, -1 },                                      /* case 1: mnDataDel_8024E940 */
    { AT_ER_MAINLIB_8015EEC8, AT_ER_GM_801729EC, AT_ER_SAVE, -1 },                                                                             /* case 2 */
    { AT_ER_MAINLIB_8015F150, AT_ER_MAINLIB_8015F260, AT_ER_GM_801729EC, AT_ER_SAVE, -1 },                                                     /* case 3 */
    { AT_ER_TOY_80311960, AT_ER_MAINLIB_8015F4BC, AT_ER_GM_80174238, AT_ER_GM_801729EC, AT_ER_SAVE, -1 },                                      /* case 4 */
    { AT_ER_LANG_SAVE, AT_ER_MAINLIB_8015FBA4, AT_ER_GM_801A3EF4, AT_ER_LANG_RESTORE, AT_ER_GM_801603B0, AT_ER_DEFLICKER_APPLY, AT_ER_GM_801729EC, AT_ER_SAVE },   /* everything */
};

static void erase_plan_order(void)
{
    int c, k;
    CHECK(at_erase_categories() == 6);
    for (c = 0; c < at_erase_categories(); c++) {
        int n = at_erase_calls(c), saves = 0;
        CHECK(n >= 3);
        for (k = 0; k < n; k++) {
            int id = at_erase_call(c, k), j;
            if (id == AT_ER_SAVE) saves++;
            CHECK(id >= 0 && id < AT_ER_COUNT);
            for (j = 0; j < k; j++) CHECK(at_erase_call(c, j) != id);                  /* no call twice in one operation */
            CHECK(RETAIL[c][k] == id);                                                  /* the retail order, call by call */
        }
        CHECK(saves == 1);                                                              /* the card is saved exactly once per operation */
        if (c != 5) CHECK(RETAIL[c][n] == -1);                                          /* and the retail list is no longer than the table */
        CHECK(at_erase_call(c, n) == -1 && at_erase_call(c, -1) == -1);                 /* out of range: -1, never a read past the table */
    }
    CHECK(at_erase_call(0, 4) == AT_ER_MAINLIB_8015DB80);                                /* category 0 ends AFTER its save: the legacy order wins over a tidy invariant */
    CHECK(at_erase_calls(-1) == 0 && at_erase_calls(6) == 0 && at_erase_call(9, 0) == -1);
}
static void labels_are_honest(void)
{
    int c, d;
    CHECK_STR(at_erase_label(5), "Everything");                                         /* the sixth operation resets every row (retail animates all six as erased) */
    CHECK_STR(at_erase_label(0), "Unlocked Fighters"); CHECK_STR(at_erase_label(1), "Unlocked Stages"); CHECK_STR(at_erase_label(3), "Fighters and Name Tags"); CHECK_STR(at_erase_label(4), "Trophies");
    CHECK_STR(at_erase_label(2), "Data set 3");                                         /* named only where the calls make it certain: the fighter counters and the save region of case 2 have no name in the decomp */
    (void) c;
    for (c = 0; c < 6; c++) {
        CHECK(strlen(at_erase_label(c)) < 24);
        for (d = c + 1; d < 6; d++) CHECK(strcmp(at_erase_label(c), at_erase_label(d)) != 0);
    }
    CHECK_STR(at_erase_label(-1), "") ; CHECK_STR(at_erase_label(6), "");
}
static void erase_cancel_calls_nothing(void)                                            /* Review Focus 6 */
{
    int c;
    for (c = 0; c < at_erase_categories(); c++) {
        NLOG = 0;
        CHECK(at_erase_run(c, 0 /* cancelled */, rec) == 0 && NLOG == 0);
    }
    NLOG = 0; CHECK(at_erase_run(-1, 1, rec) == 0 && NLOG == 0);                         /* a bad category does nothing */
    CHECK(at_erase_run(99, 1, rec) == 0 && NLOG == 0);
    CHECK(at_erase_run(0, 1, NULL) == 0);                                               /* no callback: nothing to run, no crash */
    CHECK(at_erase_run(0, 2, rec) == 0 && NLOG == 0);                                   /* only 1 means confirmed: a stray 2 is not a yes */
}
static void erase_confirmed_runs_the_plan_once(void)
{
    int c, k;
    for (c = 0; c < at_erase_categories(); c++) {
        NLOG = 0;
        CHECK(at_erase_run(c, 1, rec) == at_erase_calls(c) && NLOG == at_erase_calls(c));
        for (k = 0; k < NLOG; k++) CHECK(LOG[k] == at_erase_call(c, k));
    }
}
static void categories_do_not_share_a_state_change(void)                                 /* the categories the player picks do not erase each other's data */
{
    int c, d, k, j;
    static const int common[] = { AT_ER_GM_801729EC, AT_ER_SAVE };                       /* the bookkeeping every operation ends with */
    for (c = 0; c < 6; c++) for (d = c + 1; d < 6; d++) for (k = 0; k < at_erase_calls(c); k++) for (j = 0; j < at_erase_calls(d); j++) {
        int a = at_erase_call(c, k), b = at_erase_call(d, j), shared = 0, i;
        for (i = 0; i < 2; i++) if (a == common[i]) shared = 1;
        if (!shared) CHECK(a != b);
    }
}
int main(void) { erase_plan_order(); labels_are_honest(); erase_cancel_calls_nothing(); erase_confirmed_runs_the_plan_once(); categories_do_not_share_a_state_change(); ATLAS_DONE("atlas erase"); }
