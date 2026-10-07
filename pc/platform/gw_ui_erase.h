/* gw_ui_erase.h - the plan table of Erase Data: for each of the six retail operations (src/melee/mn/mndatadel.c: the five case arms of fn_8024F318 and
 * mnDataDel_8024EA6C), the ordered list of STATE-CHANGING calls, with every animation line removed. Header only and libc-free, so the game side (mndatadel.c,
 * compiled for PowerPC) and the native test include the same text and the list cannot differ between them.
 *
 * WHY: the retail code animates the retail screen's objects through mnDataDel_804D6C68->user_data on every path, and that is NULL outside that screen: calling
 * mnDataDel_8024E940() or mnDataDel_8024EA6C() from the Atlas screen would dereference NULL. The Atlas screen runs THIS table through mnDataDel_RunCall instead.
 * tools/port/test_erase_plan.py pins the table to the retail handlers' call order, so a change to either is seen.
 *
 * Only a confirmed run calls anything (at_erase_run(.., confirmed == 1, ..)): a cancel, a bad category and a missing callback run nothing.
 *
 * NAMES: a category is named only where its calls make it certain. Operation 5 resets every row at once (the retail code marks all six rows erased), so it is
 * "Everything". Operations 0 to 4 clear regions of the save through memzero and per-fighter calls whose names the decomp does not give: they are "Data set N",
 * which is a question for the owner (the retail words are the disc's text, which this repository does not hold). */
#ifndef GW_UI_ERASE_H
#define GW_UI_ERASE_H

/* one id per distinct call (a composite is one id where the retail code makes one decision) */
enum {
    AT_ER_GM_8016505C,
    AT_ER_MAINLIB_8015F464,
    AT_ER_GM_801729EC,
    AT_ER_SAVE,                    /* lbCardGame_SaveChanges */
    AT_ER_MAINLIB_8015DB80,
    AT_ER_GM_801647D0,
    AT_ER_MAINLIB_8015F490,
    AT_ER_STAGE_RELOCK,            /* the loop in mnDataDel_8024E940: when no stage is both unlocked and flagged, gm_801641E4(0, 1) */
    AT_ER_MAINLIB_8015EEC8,
    AT_ER_MAINLIB_8015F150,
    AT_ER_MAINLIB_8015F260,
    AT_ER_TOY_80311960,
    AT_ER_MAINLIB_8015F4BC,
    AT_ER_GM_80174238,
    AT_ER_LANG_SAVE,               /* mnDataDel_8024EA6C reads the saved language BEFORE the reset ... */
    AT_ER_MAINLIB_8015FBA4,
    AT_ER_GM_801A3EF4,
    AT_ER_LANG_RESTORE,            /* ... and puts it back after gm_801A3EF4 (the player's language survives an erase) */
    AT_ER_GM_801603B0,
    AT_ER_DEFLICKER_APPLY,         /* gmMainLib_8015F588(gmMainLib_8015F4E8()) */
    AT_ER_COUNT
};

typedef void (*AtEraseRec)(int id);

static const signed char at_erase_plan[6][9] = {
    { AT_ER_GM_8016505C, AT_ER_MAINLIB_8015F464, AT_ER_GM_801729EC, AT_ER_SAVE, AT_ER_MAINLIB_8015DB80, -1 },
    { AT_ER_GM_801647D0, AT_ER_MAINLIB_8015F490, AT_ER_STAGE_RELOCK, AT_ER_GM_801729EC, AT_ER_SAVE, -1 },
    { AT_ER_MAINLIB_8015EEC8, AT_ER_GM_801729EC, AT_ER_SAVE, -1 },
    { AT_ER_MAINLIB_8015F150, AT_ER_MAINLIB_8015F260, AT_ER_GM_801729EC, AT_ER_SAVE, -1 },
    { AT_ER_TOY_80311960, AT_ER_MAINLIB_8015F4BC, AT_ER_GM_80174238, AT_ER_GM_801729EC, AT_ER_SAVE, -1 },
    { AT_ER_LANG_SAVE, AT_ER_MAINLIB_8015FBA4, AT_ER_GM_801A3EF4, AT_ER_LANG_RESTORE, AT_ER_GM_801603B0, AT_ER_DEFLICKER_APPLY, AT_ER_GM_801729EC, AT_ER_SAVE, -1 },
};

static int at_erase_categories(void) { return 6; }

static int at_erase_calls(int category)
{
    int n = 0;
    if (category < 0 || category >= 6) return 0;
    while (at_erase_plan[category][n] >= 0) n++;
    return n;
}

/* the id of call k of a category, or -1 */
static int at_erase_call(int category, int k)
{
    if (k < 0 || k >= at_erase_calls(category)) return -1;
    return at_erase_plan[category][k];
}

static const char *at_erase_label(int category)
{
    static const char *const names[6] = { "Data set 1", "Data set 2", "Data set 3", "Data set 4", "Data set 5", "Everything" };
    return (category >= 0 && category < 6) ? names[category] : "";
}

/* Runs a category's calls in order through rec(id), but only when confirmed is exactly 1; returns how many it ran. */
static int at_erase_run(int category, int confirmed, AtEraseRec rec)
{
    int n, k;
    if (confirmed != 1 || rec == 0 || category < 0 || category >= 6) return 0;
    n = at_erase_calls(category);
    for (k = 0; k < n; k++) rec(at_erase_plan[category][k]);
    return n;
}

#endif
