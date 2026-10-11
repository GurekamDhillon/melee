/* Exercises the real Peach entry points patched by m-ex's Fighter OnFloat/
 * onFloat - Down+XY.asm (@8011BA54) and onFloat - JumpPeak.asm (@8011BAD8).
 * Synthetic inputs need no fighter archive, heaps, animation or renderer. */
#include <melee/ft/fighter.h>
#include <melee/ft/kinds/ftPeach/ftpeachfloat.h>
#include <string.h>

/* Registration-only mirror of GW_MEX_EVENT_ON_FLOAT in pc/platform/gw.h. */
enum { MEX_EVENT_ON_FLOAT = 36 };
static int mex_float_calls;
static int mex_float_result;
static void* mex_float_gobj;

static int mex_float_predicate(void* gobj)
{
    mex_float_calls++;
    mex_float_gobj = gobj;
    return mex_float_result;
}

static int test_mex_onfloat_inputs(void)
{
    static Fighter fp;
    static HSD_GObj gobj;
    static ftCommonData common;
    ftCommonData* saved_common = p_ftCommonData;
    int failed = 0;

    memset(&fp, 0, sizeof(fp));
    memset(&gobj, 0, sizeof(gobj));
    memset(&common, 0, sizeof(common));
    common.x88 = 0.7F;
    common.tap_jump_threshold = 0.7F;
    p_ftCommonData = &common;
    gobj.user_data = &fp;
    fp.kind = Ft_Kind_Mex0;
    mex_float_calls = 0;
    mex_float_result = 1;
    mex_float_gobj = NULL;

#define FLOAT_CHECK(cond, msg) \
    do { if (!(cond)) { TestFail(msg); failed = 1; goto done; } } while (0)
    FLOAT_CHECK(Mex_PredicateRegister(MEX_EVENT_ON_FLOAT, fp.kind,
                                      mex_float_predicate) == 1,
                "onFloat predicate registration rejected");

    /* Down+XY: both inputs are required; an ascending fighter may float. */
    fp.input.lstick[0].y = -common.x88;
    FLOAT_CHECK(!ftPe_8011BA54(&gobj) && mex_float_calls == 0,
                "onFloat ran without X/Y held");
    fp.input.held_buttons[0] = HSD_PAD_X;
    fp.input.lstick[0].y = 0;
    FLOAT_CHECK(!ftPe_8011BA54(&gobj) && mex_float_calls == 0,
                "onFloat ran without stick down");
    fp.input.lstick[0].y = -common.x88;
    fp.self_vel.y = 1;
    FLOAT_CHECK(ftPe_8011BA54(&gobj) == 1 && mex_float_calls == 1 &&
                    mex_float_gobj == &gobj,
                "Down+XY did not return the onFloat result for the same GObj");
    mex_float_result = 0;
    FLOAT_CHECK(!ftPe_8011BA54(&gobj) && mex_float_calls == 2,
                "Down+XY treated callback presence as success");

    /* JumpPeak: descending/zero velocity and stick up OR X/Y, not AND. */
    mex_float_result = 1;
    fp.input.lstick[0].y = 0;
    FLOAT_CHECK(!ftPe_8011BAD8(&gobj) && mex_float_calls == 2,
                "JumpPeak called onFloat while ascending");
    fp.self_vel.y = 0;
    fp.input.held_buttons[0] = 0;
    FLOAT_CHECK(!ftPe_8011BAD8(&gobj) && mex_float_calls == 2,
                "JumpPeak called onFloat without jump input");
    fp.input.lstick[0].y = common.tap_jump_threshold;
    FLOAT_CHECK(ftPe_8011BAD8(&gobj) == 1 && mex_float_calls == 3,
                "JumpPeak did not float at zero velocity with stick up");
    fp.self_vel.y = -1;
    fp.input.lstick[0].y = 0;
    fp.input.held_buttons[0] = HSD_PAD_Y;
    FLOAT_CHECK(ftPe_8011BAD8(&gobj) == 1 && mex_float_calls == 4,
                "JumpPeak did not float while falling with Y held");
    mex_float_result = 0;
    FLOAT_CHECK(!ftPe_8011BAD8(&gobj) && mex_float_calls == 5,
                "JumpPeak discarded the callback's false result");

    /* Another kind has no hook. Peach keeps her retail has_float gate even if
     * somebody registers this event for her; m-ex's asm checks Peach first. */
    fp.kind = Ft_Kind_Mex0 + 1;
    fp.input.lstick[0].y = -common.x88;
    FLOAT_CHECK(!ftPe_8011BA54(&gobj) && !ftPe_8011BAD8(&gobj) &&
                    mex_float_calls == 5,
                "onFloat leaked into a kind with no override");
    Mex_PredicateRegister(MEX_EVENT_ON_FLOAT, Ft_Kind_Peach, mex_float_predicate);
    fp.kind = Ft_Kind_Peach;
    fp.u.pe.has_float = false;
    FLOAT_CHECK(!ftPe_8011BA54(&gobj) && !ftPe_8011BAD8(&gobj) &&
                    mex_float_calls == 5,
                "Peach bypassed the retail has_float gate");

    fp.kind = Ft_Kind_Mex0;
    Mex_PredicateRegister(MEX_EVENT_ON_FLOAT, fp.kind, NULL);
    FLOAT_CHECK(!ftPe_8011BA54(&gobj) && !ftPe_8011BAD8(&gobj) &&
                    mex_float_calls == 5,
                "cleared onFloat hook still fired");
done:
    Mex_PredicateRegister(MEX_EVENT_ON_FLOAT, Ft_Kind_Mex0, NULL);
    Mex_PredicateRegister(MEX_EVENT_ON_FLOAT, Ft_Kind_Peach, NULL);
    p_ftCommonData = saved_common;
#undef FLOAT_CHECK
    return failed;
}
