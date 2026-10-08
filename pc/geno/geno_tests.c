/* geno_tests.c - Geno's game-side tests (the escape interpreter, the state block, multi-jump,
 * attributes, the savestate round trip). Compiled through the game pipeline like
 * pc/tests/mex_tests.c, so TestRegister/TestFail resolve to gw_TestRegister/gw_TestFail and every
 * script word is stored in the guest's byte order, exactly as a Pl*.dat would hold it.
 * Registered from gw_tests_register_all via GenoTestRegisterAll. */

#include <Runtime/platform.h>

#include <math.h>

#include <melee/ft/fighter.h>
#include <melee/ft/ftaction.h>
#include <melee/ft/inlines.h>
#include <melee/ft/types.h>
#include <melee/it/types.h>
#include <melee/it/kinds/types.h>
#include <melee/lb/types.h>
#include <melee/pl/player.h>
#include <sysdolphin/baselib/controller.h>

#include "geno.h"
#include "geno_state.h"

extern void TestRegister(const char* name, int (*fn)(void));
extern void TestFail(const char* msg);

extern int GenoGame_AttrFind(const char* name);
extern int GenoGame_AttrOffset(int index);
extern int GenoGame_HookFind(const char* name);
extern void* GenoGame_StateOf(Fighter* fp);
extern void Geno_FighterReset(Fighter* fp);
extern void Geno_OnActionChange(Fighter_GObj* gobj);
extern void Geno_ApplyAttrs(Fighter* fp);
extern void Geno_MultiJump(Fighter* fp, int first_state, int* msid, float* vy);
extern int Geno_TestReloadLayout(const char* text);
extern int GenoGame_ScriptRange(u32 address, u32 bytes);
extern int GenoGame_ScriptHasIasa(const void* script);
extern int Geno_TestInstall(const char* text);
extern void Geno_TestRestore(void);
extern int snap_open(int k);
extern void snap_save(int frame);
extern int snap_load(int frame);

typedef GenoState TestGenoState; /* geno_state.h: the same layout geno_game.c uses */

static Fighter t_fp;
static HSD_GObj t_gobj;
static u32 t_script[96];
static Fighter_x2D0_t t_mj;
/* v1: a fake fighter file (ftData) with a special-attribute block and a subaction table, a fake
 * per-fighter attribute buffer and a motion-state row, so the v1 paths have something to touch */
static struct ftData t_ftdata;
static u32 t_ext_attr[16];
static u32 t_dat_attrs[16];
static Fighter_WaitAnimData t_subactions[8];
static MotionState t_rows[400]; /* v2: common rows the Geno states' "like" copies */

static void t_zero(void* p, int n)
{
    u8* b = p;
    while (n-- > 0) {
        *b++ = 0;
    }
}

/* A fighter object good enough for the script loops: player 5, sub-fighter (never a live slot in
 * --test), kind Kirby. */
static TestGenoState* t_setup(void)
{
    t_zero(&t_fp, sizeof(t_fp));
    t_zero(&t_gobj, sizeof(t_gobj));
    t_gobj.user_data = &t_fp;
    t_fp.gobj = &t_gobj;
    t_fp.kind = Ft_Kind_Kirby;
    t_fp.player_id = 5;
    t_fp.is_sub_fighter = 1;
    t_fp.frame_speed_mul = 1.0f;
    t_fp.ft_data = &t_ftdata;
    t_fp.dat_attrs = t_dat_attrs;
    t_fp.dat_attrs_backup = t_dat_attrs;
    t_fp.x18 = 341;
    t_fp.x1C_actionStateList = t_rows;
    t_fp.x20_actionStateList = t_rows;
    t_fp.facing_dir = 1.0f;
    t_ftdata.ext_attr = t_ext_attr;
    t_ftdata.xC = t_subactions;
    Geno_FighterReset(&t_fp);
    return GenoGame_StateOf(&t_fp);
}

static void t_run(u32* script, int loop)
{
    t_fp.x3E4_fighterCmdScript.u = (CmdUnion*) script;
    t_fp.x3E4_fighterCmdScript.timer = 0.0f;
    t_fp.x3E4_fighterCmdScript.loop_count = 0;
    if (loop == GENO_MODE_SKIP) {
        ftAction_8007349C(&t_gobj);
    } else if (loop == GENO_MODE_ANIM) {
        ftAction_80073354(&t_gobj);
    } else {
        ftAction_80073240(&t_gobj);
    }
}

static u32 t_fbits(float f)
{
    union {
        f32 f;
        u32 u;
    } x;
    x.f = f;
    return x.u;
}

#define LA(i) GENO_VAR(GENO_BANK_LA_INT, i)
#define RA(i) GENO_VAR(GENO_BANK_RA_INT, i)
#define LAF(i) GENO_VAR(GENO_BANK_LA_FLOAT, i)
#define SETCMDVAR(idx, v) ((19u << 26) | ((u32) (idx) << 24) | (u32) (v))

/* Builds the synthetic script; returns its length in words. Branch distances are in words. */
static int t_build(u32* s)
{
    int n = 0;
    /* LA0 = 5 */
    s[n++] = GENO_W0_VAR(GENO_SUB_SET, 2, LA(0), 0, 0);
    s[n++] = 5;
    /* if (LA0 == 5) { LA1 = 111; SetCmdVar[1] = 42 } else { LA1 = 222; SetCmdVar[1] = 99 } */
    s[n++] = GENO_W0_VAR(GENO_SUB_IF, 3, LA(0), 0, GENO_CMP_EQ);
    s[n++] = 5;
    s[n++] = 5; /* then-block is 5 words: SET(2) + SetCmdVar(1) + SKIP(2) */
    s[n++] = GENO_W0_VAR(GENO_SUB_SET, 2, LA(1), 0, 0);
    s[n++] = 111;
    s[n++] = SETCMDVAR(1, 42);
    s[n++] = GENO_W0(GENO_SUB_SKIP, 2, 0);
    s[n++] = 3; /* else-block is 3 words */
    s[n++] = GENO_W0_VAR(GENO_SUB_SET, 2, LA(1), 0, 0);
    s[n++] = 222;
    s[n++] = SETCMDVAR(1, 99);
    /* if (LA0 != 5) { LA2 = 1 } else { LA2 = 2 }  - the false branch */
    s[n++] = GENO_W0_VAR(GENO_SUB_IF, 3, LA(0), 0, GENO_CMP_NE);
    s[n++] = 5;
    s[n++] = 4;
    s[n++] = GENO_W0_VAR(GENO_SUB_SET, 2, LA(2), 0, 0);
    s[n++] = 1;
    s[n++] = GENO_W0(GENO_SUB_SKIP, 2, 0);
    s[n++] = 2;
    s[n++] = GENO_W0_VAR(GENO_SUB_SET, 2, LA(2), 0, 0);
    s[n++] = 2;
    /* LA0 += 10 -> 15; LA0 -= 1 -> 14; LA0 *= 3 -> 42 */
    s[n++] = GENO_W0_VAR(GENO_SUB_ADD, 2, LA(0), 0, 0);
    s[n++] = 10;
    s[n++] = GENO_W0_VAR(GENO_SUB_SUB, 2, LA(0), 0, 0);
    s[n++] = 1;
    s[n++] = GENO_W0_VAR(GENO_SUB_MUL, 2, LA(0), 0, 0);
    s[n++] = 3;
    /* LAF0 = 1.5; LAF0 *= 2.0 -> 3.0; if (LAF0 > 2.5) LA3 = 7 */
    s[n++] = GENO_W0_VAR(GENO_SUB_SET, 2, LAF(0), 0, 0);
    s[n++] = t_fbits(1.5f);
    s[n++] = GENO_W0_VAR(GENO_SUB_MUL, 2, LAF(0), 0, 0);
    s[n++] = t_fbits(2.0f);
    s[n++] = GENO_W0_VAR(GENO_SUB_IF, 3, LAF(0), 0, GENO_CMP_GT);
    s[n++] = t_fbits(2.5f);
    s[n++] = 2;
    s[n++] = GENO_W0_VAR(GENO_SUB_SET, 2, LA(3), 0, 0);
    s[n++] = 7;
    /* bits: LA4 |= 1<<3; LA4 |= 1<<0; LA4 &= ~1 -> 8; if bit 3: LA5 = 1 */
    s[n++] = GENO_W0_VAR(GENO_SUB_SETBIT, 2, LA(4), 0, 0);
    s[n++] = 3;
    s[n++] = GENO_W0_VAR(GENO_SUB_SETBIT, 2, LA(4), 0, 0);
    s[n++] = 0;
    s[n++] = GENO_W0_VAR(GENO_SUB_CLRBIT, 2, LA(4), 0, 0);
    s[n++] = 0;
    s[n++] = GENO_W0_VAR(GENO_SUB_IF, 3, LA(4), 0, GENO_CMP_BIT);
    s[n++] = 3;
    s[n++] = 2;
    s[n++] = GENO_W0_VAR(GENO_SUB_SET, 2, LA(5), 0, 0);
    s[n++] = 1;
    /* var operand: RA0 = LA0 (42); LA6 = LAF0 (3.0 -> 3) */
    s[n++] = GENO_W0_VAR(GENO_SUB_SET, 2, RA(0), 1, 0);
    s[n++] = LA(0);
    s[n++] = GENO_W0_VAR(GENO_SUB_SET, 2, LA(6), 1, 0);
    s[n++] = LAF(0);
    /* a native hook: geno.count_frames on LA7 */
    s[n++] = GENO_W0(GENO_SUB_CALL, 3, 0);
    s[n++] = GENO_HOOK_COUNT_FRAMES;
    s[n++] = 7;
    /* an unknown sub-command is skipped by its length: len 3, two junk words */
    s[n++] = GENO_W0(0x3E, 3, 0);
    s[n++] = 0xFFFFFFFF;
    s[n++] = 0xFFFFFFFF;
    /* a vanilla command after all of it still runs */
    s[n++] = SETCMDVAR(2, 7);
    s[n++] = 0; /* End */
    return n;
}

static int test_geno_ftcmd_escape(void)
{
    TestGenoState* st = t_setup();
    t_build(t_script);
    t_run(t_script, GENO_MODE_EXEC);
    if (t_fp.x3E4_fighterCmdScript.u != NULL) {
        TestFail("the script did not reach End");
        return 1;
    }
    if (st->la_i[0] != 42 || st->la_i[1] != 111 || st->la_i[2] != 2 || st->la_i[3] != 7 ||
        st->la_i[4] != 8 || st->la_i[5] != 1 || st->ra_i[0] != 42 || st->la_i[6] != 3 ||
        st->la_f[0] != 3.0f)
    {
        TestFail("variable results wrong (expected LA0..6 = 42,111,2,7,8,1,3; RA0 42; LAF0 3)");
        return 1;
    }
    if (t_fp.cmd_vars[1] != 42 || t_fp.cmd_vars[2] != 7) {
        TestFail("vanilla SetCmdVar inside/after Geno branches did not run as expected");
        return 1;
    }
    if (st->la_i[7] != 1 || st->hook_calls != 1) {
        TestFail("CALL did not run geno.count_frames exactly once");
        return 1;
    }
    if (!(st->flags & 1)) {
        TestFail("the script flag was not set");
        return 1;
    }
    return 0;
}

/* The fast-forward loop (ftAction_8007349C) runs control flow and variables but no hooks; the
 * first-frame loop (ftAction_80073354) runs everything. */
static int test_geno_ftcmd_loops(void)
{
    TestGenoState* st = t_setup();
    t_build(t_script);
    t_run(t_script, GENO_MODE_SKIP);
    if (st->la_i[0] != 42 || st->la_i[1] != 111 || st->la_i[7] != 0 || st->hook_calls != 0) {
        TestFail("skip loop: expected vars set and no hook call");
        return 1;
    }
    st = t_setup();
    t_run(t_script, GENO_MODE_ANIM);
    if (st->la_i[0] != 42 || st->la_i[7] != 1) {
        TestFail("anim-start loop: expected vars set and one hook call");
        return 1;
    }
    return 0;
}

/* A script with no escape runs exactly as before and leaves the block untouched. */
static int test_geno_vanilla_script_untouched(void)
{
    TestGenoState* st = t_setup();
    t_script[0] = SETCMDVAR(0, 5);
    t_script[1] = SETCMDVAR(3, 9);
    t_script[2] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    if (t_fp.cmd_vars[0] != 5 || t_fp.cmd_vars[3] != 9 || st->flags != 0) {
        TestFail("a plain script changed Geno state or did not run");
        return 1;
    }
    return 0;
}

static int test_geno_state_resets(void)
{
    TestGenoState* st = t_setup();
    u32 resets = st->resets;
    st->la_i[3] = 5;
    st->ra_i[3] = 6;
    st->ra_f[1] = 1.0f;
    st->flags |= 1; /* as if a script had run: RA clears on the next action */
    Geno_OnActionChange(&t_gobj);
    if (st->la_i[3] != 5 || st->ra_i[3] != 0 || st->ra_f[1] != 0.0f) {
        TestFail("action change must clear RA and keep LA");
        return 1;
    }
    Geno_FighterReset(&t_fp);
    if (st->la_i[3] != 0 || st->resets != resets + 1 || st->profile != -1) {
        TestFail("fighter reset must clear LA (and no profile without a geno.json)");
        return 1;
    }
    return 0;
}

extern int GenoGame_AttrIsInt(int index);
static int test_geno_attr_table(void)
{
    if (GenoGame_AttrOffset(GenoGame_AttrFind("max_jumps")) != 0x58 ||
        GenoGame_AttrOffset(GenoGame_AttrFind("gravity")) != 0x5C ||
        GenoGame_AttrOffset(GenoGame_AttrFind("weight")) != 0x88 ||
        GenoGame_AttrFind("nope") != -1)
    {
        TestFail("attribute name table does not match ftCo_DatAttrs");
        return 1;
    }
    /* v7: every entry lies inside the struct and is word aligned; the original 40 keep their indices;
       the new ones are appended; the profile arrays hold the whole table. */
    {
        int i, n = 0;
        for (i = 0; GenoGame_AttrOffset(i) >= 0; ++i) {
            int off = GenoGame_AttrOffset(i);
            if (off + 4 > (int) sizeof(ftCo_DatAttrs) || (off & 3) != 0) {
                TestFail("an attribute offset lies outside ftCo_DatAttrs or is unaligned");
                return 1;
            }
            ++n;
        }
        if (GenoGame_AttrFind("walk_accel_mul") != 0 || GenoGame_AttrFind("clank_animation_length") != 39 ||
            GenoGame_AttrFind("hit_spark_variant") != 40 || GenoGame_AttrFind("normal_landing_lag") != 47 ||
            GenoGame_AttrFind("landingairn_lag") != 48 || GenoGame_AttrOffset(47) != 0xE4 ||
            GenoGame_AttrIsInt(40) != 1 || GenoGame_AttrIsInt(47) != 0 || n < 60 || n > GENO_MAX_ATTRS) {
            TestFail("attribute table order or size is not the v7 table");
            return 1;
        }
    }
    if (GenoGame_HookFind("geno.count_frames") != GENO_HOOK_COUNT_FRAMES ||
        GenoGame_HookFind("geno.nope") != -1)
    {
        TestFail("hook name table");
        return 1;
    }
    return 0;
}

/* Multi-jump past Melee's table, and attribute overrides, with a profile installed for Kirby. */
static int test_geno_multijump(void)
{
    int msid;
    float vy;
    int i, rc = 0;
    if (Geno_TestInstall("{\"geno\":1,\"fighters\":[{\"attach\":\"kirby\","
                         "\"attributes\":{\"gravity\":0.05},"
                         "\"jumps\":{\"max\":9,\"air_vy\":[2.0,1.0]}}]}") != 1)
    {
        TestFail("could not install the test profile");
        Geno_TestRestore();
        return 1;
    }
    t_setup();
    t_zero(&t_mj, sizeof(t_mj));
    t_mj.x28 = 5;
    t_mj.x2C = 300;
    for (i = 0; i < 5; i++) {
        t_mj.x14[i] = 1.5f - 0.25f * i;
    }
    t_fp.x2D0 = &t_mj;
    t_fp.co_attrs.max_jumps = 6;
    t_fp.co_attrs.gravity = 0.1f;
    Geno_ApplyAttrs(&t_fp);
    if (t_fp.co_attrs.max_jumps != 9 || t_fp.co_attrs.gravity != 0.05f) {
        TestFail("attributes not overridden (max_jumps 9, gravity 0.05)");
        rc = 1;
    }
    /* first air jump: inside the table, state unchanged, vy from air_vy[0] */
    t_fp.x1968_jumpsUsed = 1;
    msid = 300;
    vy = 1.5f;
    Geno_MultiJump(&t_fp, 300, &msid, &vy);
    if (msid != 300 || vy != 2.0f) {
        TestFail("jump 1: expected state 300, vy 2.0");
        rc = 1;
    }
    /* seventh air jump: past the 5-row table -> last state, last air_vy */
    t_fp.x1968_jumpsUsed = 7;
    msid = 306;
    vy = 123.0f;
    Geno_MultiJump(&t_fp, 300, &msid, &vy);
    if (msid != 303 || vy != 1.0f) {
        TestFail("jump 7: expected state 303 (repeatable row), vy 1.0");
        rc = 1;
    }
    Geno_TestRestore();
    /* no profile: nothing changes */
    msid = 306;
    vy = 123.0f;
    Geno_MultiJump(&t_fp, 300, &msid, &vy);
    if (msid != 306 || vy != 123.0f) {
        TestFail("without a profile Geno_MultiJump must not touch anything");
        rc = 1;
    }
    return rc;
}

/* The last retail Kirby script never sets cmd_var0: only enter it on the final jump. */
static int test_geno_multijump_script_gate(void)
{
    int used, base, msid, rc = 0;
    float vy;
    Geno_TestInstall("{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\",\"jumps\":{\"max\":9}}]}");
    t_setup();
    t_zero(&t_mj, sizeof t_mj);
    t_mj.x28 = 5;
    t_fp.x2D0 = &t_mj;
    t_fp.co_attrs.max_jumps = 6;
    Geno_ApplyAttrs(&t_fp);
    t_mj.x14[3] = 1.25f;
    t_mj.x14[4] = 1.0f;
    for (base = 341; base <= 346; base += 5) {
        for (used = 1; used < 9; used++) {
            int row = used - 1 < 4 ? used - 1 : used == 8 ? 4 : 3;
            t_fp.x1968_jumpsUsed = used;
            msid = base + used - 1;
            vy = 123.0f;
            Geno_MultiJump(&t_fp, base, &msid, &vy);
            if (msid != base + row || vy != t_mj.x14[used > 5 ? 4 : used - 1]) {
                TestFail("multi-jump must keep a script with a next-jump gate until the last jump");
                rc = 1;
            }
        }
    }
    Geno_TestRestore();
    return rc;
}

static int test_geno_reload_state_callbacks(void)
{
    const char* a = "{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\",\"states\":["
        "{\"name\":\"TutorialB\",\"behavior\":\"geno.ground\",\"subaction\":3}]}]}";
    const char* b = "{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\",\"states\":["
        "{\"name\":\"TutorialB\",\"behavior\":\"geno.ground\",\"subaction\":3,\"iasa\":\"interrupt\"}]}]}";
    int rc = 0;
    Geno_TestInstall(a);
    if (Geno_TestReloadLayout(a) != 0 || Geno_TestReloadLayout(b) != 1) {
        TestFail("IASA callback edits must restart; unchanged rows must stay live");
        rc = 1;
    }
    Geno_TestRestore();
    return rc;
}

static int test_geno_effective_script_range(void)
{
    extern u32 Geno_ScriptPool[GENO_POOL_WORDS];
    int rc = 0;
    Geno_TestInstall("{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\","
        "\"subactions\":[{\"index\":3,\"words\":[\"0x04000004\",\"0x5c000000\"]}]}]}");
    t_setup();
    if (!GenoGame_ScriptRange((u32) &Geno_ScriptPool[0], 12) ||
        GenoGame_ScriptRange((u32) &Geno_ScriptPool[0], 16) ||
        GenoGame_ScriptRange((u32) &Geno_ScriptPool[0] - 4, 4) ||
        !GenoGame_ScriptHasIasa(t_subactions[3].xC)) {
        TestFail("effective overlay script must be readable only inside its installed slot");
        rc = 1;
    }
    t_script[0] = GENO_W0(GENO_SUB_CALL, 3, 0);
    t_script[1] = 0x5c000000; /* operand, not IASA */
    t_script[2] = 0;
    t_script[3] = 0;
    if (GenoGame_ScriptHasIasa(t_script)) {
        TestFail("IASA warning must decode command lengths rather than scan operands");
        rc = 1;
    }
    Geno_TestRestore();
    return rc;
}

/* Ordinary double-jump input uses the same declared count, including the final boundary. */
static int test_geno_ordinary_jumps(void)
{
    extern ftCommonData* p_ftCommonData;
    extern bool ft_did_jump(Fighter* fp, bool arg1);
    static ftCommonData common;
    ftCommonData* saved = p_ftCommonData;
    int used, rc = 0;
    Geno_TestInstall("{\"geno\":5,\"fighters\":[{\"attach\":\"mario\",\"jumps\":{\"max\":9}}]}");
    t_setup();
    t_fp.kind = Ft_Kind_Mario;
    t_fp.co_attrs.max_jumps = 2;
    Geno_ApplyAttrs(&t_fp);
    t_zero(&common, sizeof common);
    common.tap_jump_threshold = 1.0f;
    p_ftCommonData = &common;
    t_fp.input.pressed_buttons = HSD_PAD_X;
    for (used = 1; used < 9; used++) {
        t_fp.x1968_jumpsUsed = used;
        if (!ft_did_jump(&t_fp, false)) rc = 1;
    }
    t_fp.x1968_jumpsUsed = 9;
    if (ft_did_jump(&t_fp, false) || t_fp.co_attrs.max_jumps != 9) rc = 1;
    t_fp.x1968_jumpsUsed = 2;
    t_fp.input.pressed_buttons = 0;
    if (ft_did_jump(&t_fp, false)) rc = 1;
    p_ftCommonData = saved;
    Geno_TestRestore();
    if (rc) TestFail("ordinary jump input must allow eight air jumps and stop at max 9");
    return rc;
}

static int test_geno_jump_limits(void)
{
    int rc = 0;
    Geno_TestInstall("{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\",\"jumps\":{\"max\":1e50}}]}");
    t_setup();
    Geno_ApplyAttrs(&t_fp);
    if (t_fp.co_attrs.max_jumps != 250) rc = 1;
    Geno_TestInstall("{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\",\"attributes\":{\"max_jumps\":300}}]}");
    t_setup();
    Geno_ApplyAttrs(&t_fp);
    if (t_fp.co_attrs.max_jumps != 250) rc = 1;
    Geno_TestInstall("{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\",\"jumps\":{\"max\":0}}]}");
    t_setup();
    Geno_ApplyAttrs(&t_fp);
    if (t_fp.co_attrs.max_jumps != 1) rc = 1;
    Geno_TestRestore();
    if (rc) TestFail("both max-jump keys must clamp safely to the u8-supported range");
    return rc;
}

/* Native decoder fixture: actual effective Geno row, pool outside MEM1, with ORIG tail. */
const void* GenoTest_LabScript(u32 original)
{
    t_subactions[3].xC = (CmdUnion*) original;
    Geno_TestInstall("{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\","
        "\"states\":[{\"name\":\"TutorialB\",\"subaction\":3,\"iasa\":\"interrupt\"}],"
        "\"subactions\":[{\"index\":3,\"words\":[\"0x04000005\","
        "\"0xed030000\",0,5,\"0x2c80100c\",\"0x04000000\",0,\"0xb4990000\",\"0x0a000000\","
        "\"0x5c000000\",\"0xec930000\",3,\"0x40000000\",\"0xed310000\"]}]}]}");
    t_setup();
    t_fp.x24 = t_subactions;
    return t_fp.x24[3].xC;
}

void GenoTest_LabScriptEnd(void)
{
    t_subactions[3].xC = NULL;
    Geno_TestRestore();
}

static int test_geno_iasa_script_validation(void)
{
    int rc = 0;
    Geno_TestInstall("{\"geno\":5,\"fighters\":[{\"attach\":\"kirby\",\"subactions\":["
        "{\"index\":3,\"words\":[\"0x14000000\",1]},"
        "{\"index\":4,\"words\":[\"0xec1f0000\",\"0x5c000000\"]},"
        "{\"index\":5,\"words\":[\"0xec230000\",\"0x5c000000\",0]},"
        "{\"index\":6,\"words\":[\"0xed030000\",0,1,0,\"0x5c000000\",0]}]}]}");
    t_setup();
    if (GenoGame_ScriptHasIasa(t_subactions[3].xC) ||
        GenoGame_ScriptHasIasa(t_subactions[4].xC) ||
        GenoGame_ScriptHasIasa(t_subactions[5].xC)) {
        TestFail("IASA load scan must reject bad pointers/lengths and ignore operand opcodes");
        rc = 1;
    }
    if (!GenoGame_ScriptHasIasa(t_subactions[6].xC)) {
        TestFail("IASA load warning must inspect a conditional arm beyond a fall-through End");
        rc = 1;
    }
    Geno_TestRestore();
    return rc;
}

/* The state block is game state: a savestate taken with some values brings them back. */
static int test_geno_state_savestate(void)
{
    TestGenoState* st = t_setup();
    st->la_i[10] = 1234;
    st->la_f[2] = 2.5f;
    st->ra_i[63] = -7;
    /* v1 state: checks, rehit timers, autolink flags, the action frame, a pending edge */
    st->nchecks = 2;
    st->checks[1].target = 0x10000005u;
    st->checks[1].cond[0].arg1 = 77;
    st->rehit_period[2] = 6;
    st->rehit_count[2] = 4;
    st->link_mode[3] = GENO_LINK_SPEED;
    st->action_time = 31;
    st->edge_target = 43;
    if (snap_open(1) != 0) {
        TestFail("gw_snap_open failed (no melee-pc.map beside the exe?)");
        return 1;
    }
    snap_save(0);
    st->la_i[10] = 99;
    st->la_f[2] = 0.0f;
    st->ra_i[63] = 0;
    st->nchecks = 0;
    st->checks[1].target = 0;
    st->checks[1].cond[0].arg1 = 0;
    st->rehit_period[2] = 0;
    st->rehit_count[2] = 0;
    st->link_mode[3] = 0;
    st->action_time = 0;
    st->edge_target = 0;
    if (snap_load(0) != 0) {
        TestFail("gw_snap_load failed");
        return 1;
    }
    if (st->la_i[10] != 1234 || st->la_f[2] != 2.5f || st->ra_i[63] != -7) {
        TestFail("the Geno state block was not restored by a savestate load");
        return 1;
    }
    if (st->nchecks != 2 || st->checks[1].target != 0x10000005u || st->checks[1].cond[0].arg1 != 77 ||
        st->rehit_period[2] != 6 || st->rehit_count[2] != 4 || st->link_mode[3] != GENO_LINK_SPEED ||
        st->action_time != 31 || st->edge_target != 43)
    {
        TestFail("the v1 state (checks, rehit, autolink, action frame, edge) was not restored");
        return 1;
    }
    return 0;
}

/* ---- v1 ------------------------------------------------------------------------------------- */

extern void GenoGame_TestCapture(int on, int anim_end);
extern int GenoGame_TestChanges(void);
extern u32 GenoGame_TestLastTarget(void);
extern int Geno_PreAnim(Fighter_GObj* gobj);
extern void Geno_CollBegin(Fighter_GObj* gobj);
extern void Geno_CollEnd(Fighter_GObj* gobj);
extern void Geno_GroundEdge(Fighter* fp, int landing);
extern int Geno_Autolink(Fighter* attacker, HitCapsule* hit, float* dir, float* angle, float* kb);
extern u32 Geno_ScriptPool[];

#define LAF_(i) GENO_VAR(GENO_BANK_LA_FLOAT, i)

/* Engine values: GET / PUT / IFV, DIV, RAND. */
static int test_geno_v1_values(void)
{
    TestGenoState* st = t_setup();
    int n = 0;
    u32* s = t_script;
    t_fp.ground_or_air = GA_Air;
    t_fp.self_vel.x = 1.5f;
    t_fp.facing_dir = -1.0f;
    t_fp.input.lstick[0].x = 0.5f;
    t_fp.cmd_vars[2] = 7;
    t_fp.motion_id = 123;
    t_dat_attrs[3] = t_fbits(2.5f);
    t_dat_attrs[4] = 9;
    s[n++] = GENO_W0_VAR(GENO_SUB_GET, 2, LAF_(1), 0, 0); /* LAF1 = VEL_X (1.5) */
    s[n++] = GENO_VAL_VEL_X;
    s[n++] = GENO_W0_VAR(GENO_SUB_GET, 2, LA(1), 0, 0); /* LA1 = AIR (1) */
    s[n++] = GENO_VAL_AIR;
    s[n++] = GENO_W0_VAR(GENO_SUB_GET, 2, LAF_(2), 0, 0); /* LAF2 = STICK_FWD (-0.5) */
    s[n++] = GENO_VAL_STICK_FWD;
    s[n++] = GENO_W0_VAR(GENO_SUB_GET, 2, LA(2), 0, 0); /* LA2 = CMD_VAR2 (7) */
    s[n++] = GENO_VAL_CMD_VAR0 + 2;
    s[n++] = GENO_W0_VAR(GENO_SUB_GET, 2, LAF_(3), 0, 0); /* LAF3 = special word 3 (2.5) */
    s[n++] = GENO_VAL_SPECIAL_F + 3;
    s[n++] = GENO_W0_VAR(GENO_SUB_GET, 2, LA(3), 0, 0); /* LA3 = special word 4 as int (9) */
    s[n++] = GENO_VAL_SPECIAL_I + 4;
    s[n++] = GENO_W0_VAR(GENO_SUB_GET, 2, LA(8), 0, 0); /* LA8 = MOTION (123) */
    s[n++] = GENO_VAL_MOTION;
    s[n++] = GENO_W0(GENO_SUB_PUT, 3, 0); /* VEL_Y = 3.0 */
    s[n++] = GENO_VAL_VEL_Y;
    s[n++] = t_fbits(3.0f);
    s[n++] = GENO_W0(GENO_SUB_PUT, 3, 0); /* FWD_VEL = 2.0 -> self_vel.x = -2 (facing left) */
    s[n++] = GENO_VAL_FWD_VEL;
    s[n++] = t_fbits(2.0f);
    s[n++] = GENO_W0(GENO_SUB_PUT, 3, 0x80); /* CMD_VAR0 = LA3 (9), B is a var */
    s[n++] = GENO_VAL_CMD_VAR0;
    s[n++] = LA(3);
    s[n++] = GENO_W0(GENO_SUB_PUT, 3, 0); /* FACING = 0: turn around -> +1 */
    s[n++] = GENO_VAL_FACING;
    s[n++] = t_fbits(0.0f);
    s[n++] = GENO_W0(GENO_SUB_PUT, 3, 0); /* MOTION is read-only: ignored */
    s[n++] = GENO_VAL_MOTION;
    s[n++] = 5;
    /* if (STICK_X > 0.25) LA4 = 1 */
    s[n++] = GENO_W0(GENO_SUB_IFV, 4, GENO_CMP_GT << 4);
    s[n++] = GENO_VAL_STICK_X;
    s[n++] = t_fbits(0.25f);
    s[n++] = 2;
    s[n++] = GENO_W0_VAR(GENO_SUB_SET, 2, LA(4), 0, 0);
    s[n++] = 1;
    /* if (AIR == 0) LA5 = 1   (false: in the air) */
    s[n++] = GENO_W0(GENO_SUB_IFV, 4, GENO_CMP_EQ << 4);
    s[n++] = GENO_VAL_AIR;
    s[n++] = 0;
    s[n++] = 2;
    s[n++] = GENO_W0_VAR(GENO_SUB_SET, 2, LA(5), 0, 0);
    s[n++] = 1;
    /* LA6 = 10; LA6 /= 3 -> 3; LA6 /= 0 -> unchanged; LAF4 = 5; LAF4 /= 2 -> 2.5 */
    s[n++] = GENO_W0_VAR(GENO_SUB_SET, 2, LA(6), 0, 0);
    s[n++] = 10;
    s[n++] = GENO_W0_VAR(GENO_SUB_DIV, 2, LA(6), 0, 0);
    s[n++] = 3;
    s[n++] = GENO_W0_VAR(GENO_SUB_DIV, 2, LA(6), 0, 0);
    s[n++] = 0;
    s[n++] = GENO_W0_VAR(GENO_SUB_SET, 2, LAF_(4), 0, 0);
    s[n++] = t_fbits(5.0f);
    s[n++] = GENO_W0_VAR(GENO_SUB_DIV, 2, LAF_(4), 0, 0);
    s[n++] = t_fbits(2.0f);
    /* LA7 = RAND 5 (0..4) */
    s[n++] = GENO_W0_VAR(GENO_SUB_RAND, 2, LA(7), 0, 0);
    s[n++] = 5;
    s[n++] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    if (t_fp.x3E4_fighterCmdScript.u != NULL) {
        TestFail("the v1 value script did not reach End");
        return 1;
    }
    if (st->la_f[1] != 1.5f || st->la_i[1] != 1 || st->la_f[2] != -0.5f || st->la_i[2] != 7 ||
        st->la_f[3] != 2.5f || st->la_i[3] != 9 || st->la_i[8] != 123)
    {
        TestFail("GET: expected VEL_X 1.5, AIR 1, STICK_FWD -0.5, CMD_VAR2 7, special 2.5 / 9, "
                 "MOTION 123");
        return 1;
    }
    if (t_fp.self_vel.y != 3.0f || t_fp.self_vel.x != -2.0f || t_fp.cmd_vars[0] != 9 ||
        t_fp.facing_dir != 1.0f || t_fp.motion_id != 123)
    {
        TestFail("PUT: expected VEL_Y 3, VEL_X -2 (forward 2 facing left), CMD_VAR0 9, facing +1, "
                 "motion untouched");
        return 1;
    }
    if (st->la_i[4] != 1 || st->la_i[5] != 0) {
        TestFail("IFV: stick > 0.25 should run, AIR == 0 should skip");
        return 1;
    }
    if (st->la_i[6] != 3 || st->la_f[4] != 2.5f || st->la_i[7] < 0 || st->la_i[7] > 4) {
        TestFail("DIV / RAND results wrong");
        return 1;
    }
    return 0;
}

/* Change action: persistent and ONCE checks, CHGAND, NOT, conditions, dedupe, the check limit,
 * the Geno-state stub, and the action change clearing every check. */
static int test_geno_v1_change_action(void)
{
    TestGenoState* st = t_setup();
    int n = 0, rc = 0, i;
    u32* s = t_script;
    GenoGame_TestCapture(1, 0);
    t_fp.motion_id = 50;
    t_fp.ground_or_air = GA_Air;
    /* CHG PRESSED(ATTACK) -> motion 60 */
    s[n++] = GENO_W0_CHG(GENO_SUB_CHG, 3, GENO_COND_PRESSED, 0, 0, 0);
    s[n++] = GENO_TARGET(GENO_TGT_MOTION, 60);
    s[n++] = GENO_BTN_ATTACK;
    /* CHG BIT(RA0, 3) AND FRAME >= 10 -> special 2 */
    s[n++] = GENO_W0_CHG(GENO_SUB_CHG, 4, GENO_COND_BIT, 0, 0, 0);
    s[n++] = GENO_TARGET(GENO_TGT_SPECIAL, 2);
    s[n++] = RA(0);
    s[n++] = 3;
    s[n++] = GENO_W0_CHG(GENO_SUB_CHGAND, 2, GENO_COND_FRAME, 0, 0, 0);
    s[n++] = 10;
    /* CHG ANIM_END -> Wait (common entry) */
    s[n++] = GENO_W0_CHG(GENO_SUB_CHG, 2, GENO_COND_ANIM_END, 0, 0, 0);
    s[n++] = GENO_TARGET(GENO_TGT_MOTION, 14);
    /* CHG NOT ALWAYS, ONCE -> never; dropped after one test */
    s[n++] = GENO_W0_CHG(GENO_SUB_CHG, 2, GENO_COND_ALWAYS, 0, 0, GENO_CHG_NOT | GENO_CHG_ONCE);
    s[n++] = GENO_TARGET(GENO_TGT_MOTION, 70);
    s[n++] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    if (st->nchecks != 4 || st->checks[1].ncond != 2) {
        TestFail("expected 4 checks registered, the second with 2 conditions");
        rc = 1;
    }
    /* running the same script again (a loop) registers nothing new */
    t_run(t_script, GENO_MODE_EXEC);
    if (st->nchecks != 4 || st->checks[1].ncond != 2) {
        TestFail("a re-run CHG / CHGAND must not register twice");
        rc = 1;
    }
    /* frame 1: nothing true; the ONCE check is gone, the others stay */
    t_fp.cur_anim_frame = 5.0f;
    if (Geno_PreAnim(&t_gobj) != 0 || st->nchecks != 3 || st->action_time != 1) {
        TestFail("frame 1: no change expected, the ONCE check dropped, action frame 1");
        rc = 1;
    }
    /* bit set but frame 5 < 10: the AND fails */
    st->ra_i[0] = 1 << 3;
    if (Geno_PreAnim(&t_gobj) != 0) {
        TestFail("CHGAND FRAME >= 10 must hold the change back at frame 5");
        rc = 1;
    }
    t_fp.cur_anim_frame = 10.0f;
    if (Geno_PreAnim(&t_gobj) != 1 || GenoGame_TestLastTarget() != GENO_TARGET(GENO_TGT_SPECIAL, 2) ||
        t_fp.motion_id != 343 || st->nchecks != 0 || st->ra_i[0] != 0 || st->action_time != 0)
    {
        TestFail("bit + frame: expected special 2 (motion 343), checks and RA cleared");
        rc = 1;
    }
    /* button pressed */
    n = 0;
    s[n++] = GENO_W0_CHG(GENO_SUB_CHG, 3, GENO_COND_PRESSED, 0, 0, 0);
    s[n++] = GENO_TARGET(GENO_TGT_MOTION, 60);
    s[n++] = GENO_BTN_ATTACK | GENO_BTN_SPECIAL;
    s[n++] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    t_fp.input.pressed_buttons = HSD_PAD_B;
    if (Geno_PreAnim(&t_gobj) != 1 || t_fp.motion_id != 60) {
        TestFail("PRESSED(ATTACK|SPECIAL) with B pressed should change to 60");
        rc = 1;
    }
    t_fp.input.pressed_buttons = 0;
    /* anim end */
    n = 0;
    s[n++] = GENO_W0_CHG(GENO_SUB_CHG, 2, GENO_COND_ANIM_END, 0, 0, 0);
    s[n++] = GENO_TARGET(GENO_TGT_MOTION, 14);
    s[n++] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    if (Geno_PreAnim(&t_gobj) != 0) {
        TestFail("ANIM_END must wait for the animation's end");
        rc = 1;
    }
    GenoGame_TestCapture(1, 1);
    if (Geno_PreAnim(&t_gobj) != 1 || t_fp.motion_id != 14) {
        TestFail("ANIM_END at the end should change to 14");
        rc = 1;
    }
    /* VALUE ACTION_FRAME >= 1, ONCE: true at its one test */
    GenoGame_TestCapture(1, 0);
    n = 0;
    s[n++] = GENO_W0_CHG(GENO_SUB_CHG, 4, GENO_COND_VALUE, 0, GENO_CMP_GE, GENO_CHG_ONCE);
    s[n++] = GENO_TARGET(GENO_TGT_MOTION, 80);
    s[n++] = GENO_VAL_ACTION_FRAME;
    s[n++] = 1;
    s[n++] = 0;
    t_run(t_script, GENO_MODE_SKIP); /* the fast-forward pass registers no ONCE check */
    if (st->nchecks != 0) {
        TestFail("a ONCE check must not be registered by the fast-forward pass");
        rc = 1;
    }
    t_run(t_script, GENO_MODE_EXEC);
    if (Geno_PreAnim(&t_gobj) != 1 || t_fp.motion_id != 80) {
        TestFail("VALUE ACTION_FRAME >= 1 ONCE should change to 80");
        rc = 1;
    }
    /* VAR: LAF0 > 1.5 (float compare) */
    n = 0;
    s[n++] = GENO_W0_CHG(GENO_SUB_CHG, 4, GENO_COND_VAR, 0, GENO_CMP_GT, 0);
    s[n++] = GENO_TARGET(GENO_TGT_MOTION, 81);
    s[n++] = LAF(0);
    s[n++] = t_fbits(1.5f);
    s[n++] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    st->la_f[0] = 1.0f;
    if (Geno_PreAnim(&t_gobj) != 0) {
        TestFail("VAR 1.0 > 1.5 must be false");
        rc = 1;
    }
    st->la_f[0] = 2.0f;
    if (Geno_PreAnim(&t_gobj) != 1 || t_fp.motion_id != 81) {
        TestFail("VAR 2.0 > 1.5 should change to 81");
        rc = 1;
    }
    /* a Geno-state target (v2) changes nothing and the check stays; a later check still wins */
    n = 0;
    s[n++] = GENO_W0_CHG(GENO_SUB_CHG, 2, GENO_COND_ALWAYS, 0, 0, 0);
    s[n++] = GENO_TARGET(GENO_TGT_GENO, 1);
    s[n++] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    i = GenoGame_TestChanges();
    if (Geno_PreAnim(&t_gobj) != 0 || GenoGame_TestChanges() != i || st->nchecks != 1) {
        TestFail("a Geno-state target must be ignored in v1");
        rc = 1;
    }
    s[0] = GENO_W0_CHG(GENO_SUB_CHG, 2, GENO_COND_ALWAYS, 0, 0, 0);
    s[1] = GENO_TARGET(GENO_TGT_MOTION, 35); /* the fallback, registered after it */
    s[2] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    if (Geno_PreAnim(&t_gobj) != 1 || t_fp.motion_id != 35) {
        TestFail("a fallback check after a Geno-state check must still fire in v1");
        rc = 1;
    }
    n = 0;
    s[n++] = GENO_W0_CHG(GENO_SUB_CHG, 2, GENO_COND_ALWAYS, 0, 0, 0);
    s[n++] = GENO_TARGET(GENO_TGT_GENO, 1);
    s[n++] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    /* CHGCLR, then the limit: 9 distinct checks -> 8 kept */
    t_script[0] = GENO_W0(GENO_SUB_CHGCLR, 1, 0);
    t_script[1] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    if (st->nchecks != 0) {
        TestFail("CHGCLR must drop every check");
        rc = 1;
    }
    n = 0;
    for (i = 0; i < 9; i++) {
        s[n++] = GENO_W0_CHG(GENO_SUB_CHG, 3, GENO_COND_HELD, 0, 0, 0);
        s[n++] = GENO_TARGET(GENO_TGT_MOTION, 90 + i);
        s[n++] = GENO_BTN_TAUNT;
    }
    s[n++] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    if (st->nchecks != GENO_MAX_CHECKS) {
        TestFail("the 9th check must be dropped (8 max)");
        rc = 1;
    }
    GenoGame_TestCapture(0, 0);
    return rc;
}

/* Landing / take-off edges inside the collision callback: script GROUND / AIR checks, geno.json
 * on_land and its hooks; nothing outside the callback. */
static int test_geno_v1_ground_edge(void)
{
    TestGenoState* st = t_setup();
    int rc = 0;
    u32* s = t_script;
    GenoGame_TestCapture(1, 0);
    t_fp.motion_id = 50;
    s[0] = GENO_W0_CHG(GENO_SUB_CHG, 2, GENO_COND_GROUND, 0, 0, 0);
    s[1] = GENO_TARGET(GENO_TGT_MOTION, 43);
    s[2] = GENO_W0_CHG(GENO_SUB_CHG, 2, GENO_COND_AIR, 0, 0, 0);
    s[3] = GENO_TARGET(GENO_TGT_MOTION, 30);
    s[4] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    /* a landing outside the collision callback (a state's entry, a hit) picks nothing */
    t_fp.ground_or_air = GA_Ground;
    Geno_GroundEdge(&t_fp, 1);
    if (st->edge_pending || GenoGame_TestChanges() != 0) {
        TestFail("an edge outside the collision callback must not pick a target");
        rc = 1;
    }
    Geno_CollBegin(&t_gobj);
    Geno_GroundEdge(&t_fp, 1);
    t_fp.motion_id = 42; /* as if the state's own landing ran inside the callback */
    Geno_CollEnd(&t_gobj);
    if (GenoGame_TestLastTarget() != GENO_TARGET(GENO_TGT_MOTION, 43) || t_fp.motion_id != 43 ||
        st->in_coll != 0)
    {
        TestFail("landing in the callback: expected the GROUND check's target 43 after it");
        rc = 1;
    }
    /* take-off */
    t_run(t_script, GENO_MODE_EXEC);
    t_fp.ground_or_air = GA_Air;
    Geno_CollBegin(&t_gobj);
    Geno_GroundEdge(&t_fp, 0);
    Geno_CollEnd(&t_gobj);
    if (t_fp.motion_id != 30) {
        TestFail("leaving the ground in the callback: expected the AIR check's target 30");
        rc = 1;
    }
    /* geno.json on_land + on_land hooks */
    if (Geno_TestInstall("{\"geno\":1,\"fighters\":[{\"attach\":\"kirby\","
                         "\"on_land\":[{\"from\":\"special:3\",\"to\":42,\"keep_frame\":true},"
                         "{\"from\":66,\"to\":\"special:5\"}],"
                         "\"hooks\":{\"on_land\":[\"geno.count_frames:9\"]}}]}") != 1)
    {
        TestFail("could not install the on_land profile");
        Geno_TestRestore();
        GenoGame_TestCapture(0, 0);
        return 1;
    }
    st = t_setup();
    GenoGame_TestCapture(1, 0);
    t_fp.motion_id = 344; /* special 3 */
    t_fp.ground_or_air = GA_Ground;
    Geno_CollBegin(&t_gobj);
    Geno_GroundEdge(&t_fp, 1);
    Geno_CollEnd(&t_gobj);
    if (GenoGame_TestLastTarget() != (GENO_TARGET(GENO_TGT_MOTION, 42) | GENO_TGT_KEEP_FRAME) ||
        st->la_i[9] != 1)
    {
        TestFail("on_land special:3 -> 42 keep_frame, and the on_land hook once");
        rc = 1;
    }
    t_fp.motion_id = 66;
    Geno_CollBegin(&t_gobj);
    Geno_GroundEdge(&t_fp, 1);
    Geno_CollEnd(&t_gobj);
    if (t_fp.motion_id != 346 || st->la_i[9] != 2) {
        TestFail("on_land 66 -> special:5 (motion 346)");
        rc = 1;
    }
    t_fp.motion_id = 67; /* no entry: nothing */
    Geno_CollBegin(&t_gobj);
    Geno_GroundEdge(&t_fp, 1);
    Geno_CollEnd(&t_gobj);
    if (t_fp.motion_id != 67) {
        TestFail("a landing from a motion with no on_land entry must change nothing");
        rc = 1;
    }
    Geno_TestRestore();
    GenoGame_TestCapture(0, 0);
    return rc;
}

/* REHIT clears the hit lists of its hitboxes every N frames (and only theirs). */
static int test_geno_v1_rehit(void)
{
    TestGenoState* st = t_setup();
    int rc = 0, f;
    t_fp.x914[1].victims_1[0].victim = &t_gobj;
    t_fp.x914[1].x44 = 1;
    t_fp.x914[0].victims_1[0].victim = &t_gobj;
    t_fp.x914[0].x44 = 1;
    t_script[0] = GENO_W0(GENO_SUB_REHIT, 2, 0x02 << 8); /* hitbox 1 */
    t_script[1] = 3;
    t_script[2] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    for (f = 1; f <= 2; f++) {
        Geno_PreAnim(&t_gobj);
    }
    if (t_fp.x914[1].x44 != 1) {
        TestFail("rehit 3: the hit list must survive frames 1-2");
        rc = 1;
    }
    Geno_PreAnim(&t_gobj);
    if (t_fp.x914[1].x44 != 0 || t_fp.x914[1].victims_1[0].victim != NULL) {
        TestFail("rehit 3: the hit list must be cleared on frame 3");
        rc = 1;
    }
    if (t_fp.x914[0].x44 != 1) {
        TestFail("rehit must leave the other hitboxes' lists alone");
        rc = 1;
    }
    t_fp.x914[1].x44 = 1;
    Geno_PreAnim(&t_gobj);
    Geno_PreAnim(&t_gobj);
    if (t_fp.x914[1].x44 != 1) {
        TestFail("rehit 3: cleared again too early");
        rc = 1;
    }
    Geno_PreAnim(&t_gobj);
    if (t_fp.x914[1].x44 != 0) {
        TestFail("rehit 3: not cleared again on frame 6");
        rc = 1;
    }
    Geno_OnActionChange(&t_gobj);
    if (st->rehit_period[1] != 0) {
        TestFail("an action change must stop the rehit timer");
        rc = 1;
    }
    return rc;
}

/* LINK (autolink 365): the launch follows the attacker's momentum; other hitboxes and slow
 * attackers keep Melee's angle; a fighter Geno does not touch is never changed. */
static int test_geno_v1_autolink(void)
{
    TestGenoState* st;
    int rc = 0;
    float dir = 1.0f, angle = 361.0f, kb = 10.0f;
    t_setup();
    if (Geno_Autolink(&t_fp, &t_fp.x914[0], &dir, &angle, &kb) != 0 || angle != 361.0f) {
        TestFail("an untouched fighter must keep Melee's angle");
        rc = 1;
    }
    st = t_setup();
    t_script[0] = GENO_W0(GENO_SUB_LINK, 2, 0x01 << 8);
    t_script[1] = GENO_LINK_DIRECTION;
    t_script[2] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    t_fp.ground_or_air = GA_Air;
    t_fp.self_vel.x = -1.0f;
    t_fp.self_vel.y = 1.0f;
    if (Geno_Autolink(&t_fp, &t_fp.x914[0], &dir, &angle, &kb) != 1 || dir != 1.0f ||
        angle != 45.0f || kb != 10.0f)
    {
        TestFail("up-left momentum: expected dir +1 (launch left), angle 45, kb unchanged");
        rc = 1;
    }
    dir = 1.0f;
    angle = 361.0f;
    if (Geno_Autolink(&t_fp, &t_fp.x914[1], &dir, &angle, &kb) != 0 || angle != 361.0f) {
        TestFail("a hitbox without LINK must keep its angle");
        rc = 1;
    }
    t_fp.self_vel.x = 2.0f;
    t_fp.self_vel.y = -2.0f;
    if (Geno_Autolink(&t_fp, &t_fp.x914[0], &dir, &angle, &kb) != 1 || dir != -1.0f ||
        angle != 315.0f)
    {
        TestFail("down-right momentum: expected dir -1 (launch right), angle 315");
        rc = 1;
    }
    t_fp.ground_or_air = GA_Ground;
    t_fp.gr_vel = -1.5f;
    if (Geno_Autolink(&t_fp, &t_fp.x914[0], &dir, &angle, &kb) != 1 || dir != 1.0f ||
        angle != 0.0f)
    {
        TestFail("on the ground moving left: expected dir +1, angle 0");
        rc = 1;
    }
    t_fp.gr_vel = 0.01f;
    angle = 361.0f;
    if (Geno_Autolink(&t_fp, &t_fp.x914[0], &dir, &angle, &kb) != 0 || angle != 361.0f) {
        TestFail("a nearly still attacker keeps the hitbox's own angle");
        rc = 1;
    }
    Geno_OnActionChange(&t_gobj);
    if (st->link_mode[0] != 0) {
        TestFail("an action change must clear LINK");
        rc = 1;
    }
    return rc;
}

/* special_attributes: written into the file's block and the fighter's buffer, readable by GET. */
static int test_geno_v1_special_attrs(void)
{
    TestGenoState* st;
    int rc = 0;
    if (Geno_TestInstall("{\"geno\":1,\"fighters\":[{\"attach\":\"kirby\","
                         "\"special_attributes\":[{\"index\":2,\"float\":1.25},"
                         "{\"offset\":\"0x10\",\"int\":77},{\"index\":999,\"int\":1}]}]}") != 1)
    {
        TestFail("could not install the special_attributes profile");
        Geno_TestRestore();
        return 1;
    }
    t_zero(t_ext_attr, sizeof(t_ext_attr));
    t_zero(t_dat_attrs, sizeof(t_dat_attrs));
    st = t_setup();
    t_fp.co_attrs.max_jumps = 2;
    Geno_ApplyAttrs(&t_fp);
    if (t_ext_attr[2] != t_fbits(1.25f) || t_ext_attr[4] != 77 || t_dat_attrs[2] != t_fbits(1.25f) ||
        t_dat_attrs[4] != 77 || t_ext_attr[3] != 0 || t_fp.co_attrs.max_jumps != 2)
    {
        TestFail("special words 2 (1.25) and 4 (77) not written to the file block and the buffer");
        rc = 1;
    }
    t_script[0] = GENO_W0_VAR(GENO_SUB_GET, 2, LAF(5), 0, 0);
    t_script[1] = GENO_VAL_SPECIAL_F + 2;
    t_script[2] = GENO_W0_VAR(GENO_SUB_GET, 2, LA(5), 0, 0);
    t_script[3] = GENO_VAL_SPECIAL_I + 4;
    t_script[4] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    if (st->la_f[5] != 1.25f || st->la_i[5] != 77) {
        TestFail("GET of the overridden special words");
        rc = 1;
    }
    Geno_TestRestore();
    return rc;
}

/* subactions (script overlays): the table row points at the pool, ORIG continues with the
 * original script, a second spawn keeps the original. */
static int test_geno_v1_overlay(void)
{
    TestGenoState* st;
    static u32 orig[4];
    int rc = 0;
    char json[400];
    const char* head = "{\"geno\":1,\"fighters\":[{\"attach\":\"kirby\",\"subactions\":[{\"index\":3,"
                       "\"words\":[\"0xEC120100\",5,\"0xED310000\"]}]}]}";
    int k;
    for (k = 0; head[k] != '\0' && k < (int) sizeof(json) - 1; k++) {
        json[k] = head[k];
    }
    json[k] = '\0';
    /* 0xEC120100 = SET (sub 1, len 2) LA1 = 5; 0xED310000 = ORIG (sub 0x13, len 1) */
    if (GENO_W0_VAR(GENO_SUB_SET, 2, LA(1), 0, 0) != 0xEC120100u ||
        GENO_W0(GENO_SUB_ORIG, 1, 0) != 0xED310000u)
    {
        TestFail("the overlay test's hand-encoded words are wrong");
        return 1;
    }
    orig[0] = SETCMDVAR(1, 42);
    orig[1] = 0;
    t_zero(t_subactions, sizeof(t_subactions));
    t_subactions[3].xC = (CmdUnion*) orig;
    if (Geno_TestInstall(json) != 1) {
        TestFail("could not install the overlay profile");
        Geno_TestRestore();
        return 1;
    }
    st = t_setup();
    if ((u32*) t_subactions[3].xC == orig || ((u32*) t_subactions[3].xC)[0] != 0xEC120100u ||
        ((u32*) t_subactions[3].xC)[1] != 5 || ((u32*) t_subactions[3].xC)[3] != 0)
    {
        TestFail("subaction 3 does not point at the overlay words (with an End appended)");
        rc = 1;
    }
    t_run((u32*) t_subactions[3].xC, GENO_MODE_EXEC);
    if (st->la_i[1] != 5 || t_fp.cmd_vars[1] != 42 || t_fp.x3E4_fighterCmdScript.u != NULL) {
        TestFail("the overlay ran, then ORIG must run the original script to its End");
        rc = 1;
    }
    t_setup(); /* a respawn: the row already points at the overlay; ORIG must still find orig */
    t_run((u32*) t_subactions[3].xC, GENO_MODE_EXEC);
    if (t_fp.cmd_vars[1] != 42) {
        TestFail("after a second spawn ORIG lost the original script");
        rc = 1;
    }
    if (t_subactions[2].xC != NULL) {
        TestFail("other subactions must be untouched");
        rc = 1;
    }
    Geno_TestRestore();
    t_zero(t_subactions, sizeof(t_subactions));
    return rc;
}

/* A fighter with no profile whose scripts never used the escape: every v1 entry point is a no-op. */
static int test_geno_v1_inert(void)
{
    TestGenoState* st = t_setup();
    float dir = 1.0f, angle = 50.0f, kb = 3.0f;
    st->flags = 0;
    if (Geno_PreAnim(&t_gobj) != 0 || st->action_time != 0) {
        TestFail("PreAnim touched an inert fighter");
        return 1;
    }
    Geno_CollBegin(&t_gobj);
    Geno_GroundEdge(&t_fp, 1);
    Geno_CollEnd(&t_gobj);
    if (st->in_coll != 0 || st->edge_pending != 0 ||
        Geno_Autolink(&t_fp, &t_fp.x914[0], &dir, &angle, &kb) != 0 || angle != 50.0f)
    {
        TestFail("collision / edge / autolink touched an inert fighter");
        return 1;
    }
    return 0;
}

/* ---- v2 ------------------------------------------------------------------------------------- */

extern MotionState* Geno_MotionRow(Fighter* fp, int msid);
extern int Geno_SpecialEnter(Fighter_GObj* gobj, int which);

static void t_cam(Fighter_GObj* gobj) {}
static void t_input(Fighter_GObj* gobj) {}

#define T_GS 0     /* GlideStart */
#define T_GL 1     /* Glide */
#define T_GA 2     /* GlideAttack */
#define T_GLAND 3  /* GlideLanding */
#define T_GEND 4   /* GlideEnd */
#define T_TOR 5    /* Tornado */
#define T_DRILL 6  /* Drill */
#define T_DEND 7   /* DrillEnd */
#define T_PLAIN 8  /* a generic air state */
#define T_MS(s) (GENO_MOTION_BASE + (s))

static const char t_v2_json[] =
    "{\"geno\":2,\"fighters\":[{\"attach\":\"kirby\",\"states\":["
    "{\"name\":\"GlideStart\",\"behavior\":\"geno.glide.start\",\"subaction\":10},"
    "{\"name\":\"Glide\",\"behavior\":\"geno.glide\",\"subaction\":\"motion:65\"},"
    "{\"name\":\"GlideAttack\",\"behavior\":\"geno.glide.attack\",\"subaction\":12,\"move_id\":9},"
    "{\"name\":\"GlideLanding\",\"behavior\":\"geno.glide.landing\",\"subaction\":13},"
    "{\"name\":\"GlideEnd\",\"behavior\":\"geno.glide.end\",\"subaction\":14},"
    "{\"name\":\"Tornado\",\"behavior\":\"geno.tornado\",\"subaction\":15},"
    "{\"name\":\"Drill\",\"behavior\":\"geno.drill\",\"subaction\":16},"
    "{\"name\":\"DrillEnd\",\"behavior\":\"geno.drill.end\",\"subaction\":17},"
    "{\"name\":\"Plain\",\"behavior\":\"geno.air\",\"subaction\":18,\"phys\":\"none\","
    "\"iasa\":\"like\",\"like\":\"motion:29\",\"next\":\"geno:Glide\",\"flags\":\"0x55\"}],"
    "\"glide\":{\"hold_frames\":16},\"drill\":{\"angle_max\":60},"
    "\"specials\":{\"n\":\"geno:Tornado\",\"s\":\"geno:Drill\"}}]}";

static TestGenoState* t_v2_setup(void)
{
    t_rows[ftCo_MS_Fall].anim_id = 29;
    t_rows[ftCo_MS_Fall].x4_flags = 0x1234;
    t_rows[ftCo_MS_Fall].cam_cb = t_cam;
    t_rows[ftCo_MS_Fall].input_cb = t_input;
    t_rows[65].anim_id = 777;
    if (Geno_TestInstall(t_v2_json) != 1) {
        return NULL;
    }
    return t_setup();
}

/* Rows: subaction (index or another motion's), like row copy, flags, move id, callbacks. */
static int test_geno_v2_states(void)
{
    TestGenoState* st = t_v2_setup();
    MotionState* r;
    int rc = 0;
    if (st == NULL) {
        TestFail("could not install the v2 profile");
        Geno_TestRestore();
        return 1;
    }
    r = Geno_MotionRow(&t_fp, T_MS(T_GS));
    if (r == NULL || r->anim_id != 10 || r->x4_flags != 0x1234 || r->cam_cb != t_cam ||
        r->phys_cb == NULL || r->anim_cb == NULL || r->coll_cb == NULL)
    {
        TestFail("GlideStart row: subaction 10, Fall's flags and camera, Geno callbacks");
        rc = 1;
    }
    r = Geno_MotionRow(&t_fp, T_MS(T_GL));
    if (r->anim_id != 777) {
        TestFail("Glide row: \"subaction\": \"motion:65\" must play motion 65's animation (777)");
        rc = 1;
    }
    r = Geno_MotionRow(&t_fp, T_MS(T_GA));
    if (r->move_id != 9) {
        TestFail("GlideAttack row: move_id 9");
        rc = 1;
    }
    r = Geno_MotionRow(&t_fp, T_MS(T_PLAIN));
    if (r->input_cb != t_input || r->x4_flags != 0x55 || r->phys_cb == NULL) {
        TestFail("Plain row: iasa \"like\" keeps Fall's input callback, flags 0x55, phys \"none\"");
        rc = 1;
    }
    /* an undeclared state: the Fall row, never NULL */
    if (Geno_MotionRow(&t_fp, T_MS(12)) != &t_rows[ftCo_MS_Fall]) {
        TestFail("an undeclared Geno motion must fall back to the Fall row");
        rc = 1;
    }
    Geno_TestRestore();
    return rc;
}

/* Script CHG -> GENO(n): performed now (v1 skipped it); an undeclared state still falls through to
 * the translator's fallback check; the GENO_STATE value; specials bound to states. */
static int test_geno_v2_change_to_state(void)
{
    TestGenoState* st = t_v2_setup();
    u32* s = t_script;
    int n = 0, rc = 0;
    if (st == NULL) {
        TestFail("could not install the v2 profile");
        Geno_TestRestore();
        return 1;
    }
    GenoGame_TestCapture(1, 1);
    t_fp.motion_id = 330;
    t_fp.ground_or_air = GA_Air;
    /* CHG ANIM_END -> GENO(12) (undeclared) ; CHG ANIM_END -> GENO(0) ; CHG ANIM_END -> FallSpecial */
    s[n++] = GENO_W0_CHG(GENO_SUB_CHG, 2, GENO_COND_ANIM_END, 0, 0, 0);
    s[n++] = GENO_TARGET(GENO_TGT_GENO, 12);
    s[n++] = GENO_W0_CHG(GENO_SUB_CHG, 2, GENO_COND_ANIM_END, 0, 0, 0);
    s[n++] = GENO_TARGET(GENO_TGT_GENO, T_GS);
    s[n++] = GENO_W0_CHG(GENO_SUB_CHGAND, 1, GENO_COND_AIR, 0, 0, 0);
    s[n++] = GENO_W0_CHG(GENO_SUB_CHG, 2, GENO_COND_ANIM_END, 0, 0, 0);
    s[n++] = GENO_TARGET(GENO_TGT_MOTION, ftCo_MS_FallSpecial);
    s[n++] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    if (Geno_PreAnim(&t_gobj) != 1 || t_fp.motion_id != T_MS(T_GS) ||
        GenoGame_TestLastTarget() != GENO_TARGET(GENO_TGT_GENO, T_GS))
    {
        TestFail("CHG ANIM_END & AIR -> GENO(GlideStart) must enter the Geno state");
        rc = 1;
    }
    t_script[0] = GENO_W0_VAR(GENO_SUB_GET, 2, LA(3), 0, 0);
    t_script[1] = GENO_VAL_GENO_STATE;
    t_script[2] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    if (st->la_i[3] != T_GS) {
        TestFail("GET GENO_STATE in GlideStart must read 0");
        rc = 1;
    }
    /* specials: n -> Tornado, s -> Drill, hi unbound */
    t_fp.motion_id = 14;
    if (Geno_SpecialEnter(&t_gobj, GENO_SP_N) != 1 || t_fp.motion_id != T_MS(T_TOR)) {
        TestFail("specials.n -> Tornado");
        rc = 1;
    }
    if (Geno_SpecialEnter(&t_gobj, GENO_SP_AIR_S) != 1 || t_fp.motion_id != T_MS(T_DRILL)) {
        TestFail("specials.air_s defaults to specials.s -> Drill");
        rc = 1;
    }
    /* a stale Allow Interrupt from the previous state must not carry into the entered state */
    t_fp.motion_id = 14;
    t_fp.allow_interrupt = true;
    if (Geno_SpecialEnter(&t_gobj, GENO_SP_N) != 1 || t_fp.allow_interrupt) {
        TestFail("entering a Geno state must clear Allow Interrupt (a whiffed counter's IASA crouched out of the next one)");
        rc = 1;
    }
    if (Geno_SpecialEnter(&t_gobj, GENO_SP_HI) != 0) {
        TestFail("an unbound special must be left to the fighter's own code");
        rc = 1;
    }
    Geno_TestRestore();
    t_setup(); /* no profile */
    if (Geno_SpecialEnter(&t_gobj, GENO_SP_N) != 0) {
        TestFail("a fighter with no profile: Geno_SpecialEnter must return 0");
        rc = 1;
    }
    GenoGame_TestCapture(0, 0);
    return rc;
}

/* The jump-hold entry: 16 frames of jump held in an air jump -> GlideStart; a release cancels it
 * for that jump; the next air jump counts afresh. */
static int test_geno_v2_glide_entry(void)
{
    TestGenoState* st = t_v2_setup();
    int f, rc = 0;
    if (st == NULL) {
        TestFail("could not install the v2 profile");
        Geno_TestRestore();
        return 1;
    }
    GenoGame_TestCapture(1, 0);
    t_fp.ground_or_air = GA_Air;
    t_fp.motion_id = ftCo_MS_JumpAerialF;
    Geno_OnActionChange(&t_gobj);
    t_fp.input.held_buttons[0] = HSD_PAD_X;
    for (f = 1; f < 16; f++) {
        if (Geno_PreAnim(&t_gobj) != 0) {
            TestFail("glide entered before 16 frames of hold");
            rc = 1;
            break;
        }
    }
    if (Geno_PreAnim(&t_gobj) != 1 || t_fp.motion_id != T_MS(T_GS)) {
        TestFail("16th frame of jump held in JumpAerialF must enter GlideStart");
        rc = 1;
    }
    /* a release cancels the hold for that jump */
    t_fp.motion_id = ftCo_MS_JumpAerialB;
    Geno_OnActionChange(&t_gobj);
    for (f = 0; f < 5; f++) {
        Geno_PreAnim(&t_gobj);
    }
    t_fp.input.held_buttons[0] = 0;
    Geno_PreAnim(&t_gobj);
    t_fp.input.held_buttons[0] = HSD_PAD_Y;
    for (f = 0; f < 30; f++) {
        if (Geno_PreAnim(&t_gobj) != 0) {
            TestFail("after a release, holding again in the same jump must not glide");
            rc = 1;
            break;
        }
    }
    /* not in an air jump (Fall): nothing */
    t_fp.motion_id = ftCo_MS_Fall;
    Geno_OnActionChange(&t_gobj);
    for (f = 0; f < 30; f++) {
        if (Geno_PreAnim(&t_gobj) != 0) {
            TestFail("holding jump in Fall must not glide");
            rc = 1;
            break;
        }
    }
    t_fp.input.held_buttons[0] = 0;
    GenoGame_TestCapture(0, 0);
    Geno_TestRestore();
    return rc;
}

/* Glide physics: the stick pitches within the limits; diving gains speed, climbing loses it;
 * A -> GlideAttack, shield -> GlideEnd. Also the state survives a savestate. */
static int test_geno_v2_glide(void)
{
    TestGenoState* st = t_v2_setup();
    MotionState* gl;
    int f, rc = 0;
    f32 s0, s1;
    if (st == NULL) {
        TestFail("could not install the v2 profile");
        Geno_TestRestore();
        return 1;
    }
    GenoGame_TestCapture(1, 0);
    t_fp.ground_or_air = GA_Air;
    t_fp.self_vel.x = 1.0f;
    t_fp.self_vel.y = 0.5f;
    t_fp.co_attrs.gravity = 0.08f;
    t_fp.co_attrs.terminal_velocity = 2.0f;
    t_fp.motion_id = ftCo_MS_JumpAerialF;
    t_script[0] = GENO_W0_CHG(GENO_SUB_CHG, 2, GENO_COND_ALWAYS, 0, 0, GENO_CHG_ONCE);
    t_script[1] = GENO_TARGET(GENO_TGT_GENO, T_GS);
    t_script[2] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    Geno_PreAnim(&t_gobj);
    if (t_fp.motion_id != T_MS(T_GS)) {
        TestFail("could not enter GlideStart");
        GenoGame_TestCapture(0, 0);
        Geno_TestRestore();
        return 1;
    }
    Geno_MotionRow(&t_fp, T_MS(T_GS))->phys_cb(&t_gobj);
    /* GlideStart's animation ends -> Glide */
    GenoGame_TestCapture(1, 1);
    Geno_MotionRow(&t_fp, T_MS(T_GS))->anim_cb(&t_gobj);
    GenoGame_TestCapture(1, 0);
    if (t_fp.motion_id != T_MS(T_GL)) {
        TestFail("GlideStart's anim end must go to Glide");
        rc = 1;
    }
    gl = Geno_MotionRow(&t_fp, T_MS(T_GL));
    /* dive: stick down */
    t_fp.input.lstick[0].y = -1.0f;
    s0 = st->move_f[1];
    for (f = 0; f < 90; f++) {
        gl->phys_cb(&t_gobj);
    }
    s1 = st->move_f[1];
    if (st->move_f[0] > -69.0f || st->move_f[0] < -70.001f) {
        TestFail("a held stick down must pitch the nose to the lower limit (-70)");
        rc = 1;
    }
    if (!(s1 > s0) || !(t_fp.self_vel.y < 0.0f) || !(t_fp.self_vel.x > 0.0f)) {
        TestFail("diving must gain speed and move down-forward");
        rc = 1;
    }
    /* climb: stick up */
    t_fp.input.lstick[0].y = 1.0f;
    for (f = 0; f < 40; f++) {
        gl->phys_cb(&t_gobj);
    }
    if (!(st->move_f[1] < s1) || st->move_f[0] <= 0.0f) {
        TestFail("climbing must lose speed");
        rc = 1;
    }
    /* savestate round trip of the move vars */
    if (snap_open(1) == 0) {
        f32 a = st->move_f[0];
        snap_save(0);
        st->move_f[0] = 123.0f;
        st->hold_frames = 77;
        snap_load(0);
        if (st->move_f[0] != a || st->hold_frames == 77) {
            TestFail("glide angle / hold count not restored by a savestate load");
            rc = 1;
        }
    }
    /* A -> GlideAttack */
    t_fp.input.pressed_buttons = HSD_PAD_A;
    gl->input_cb(&t_gobj);
    if (t_fp.motion_id != T_MS(T_GA)) {
        TestFail("A in Glide must go to GlideAttack");
        rc = 1;
    }
    /* shield -> GlideEnd */
    t_fp.motion_id = T_MS(T_GL);
    t_fp.input.pressed_buttons = HSD_PAD_R;
    gl->input_cb(&t_gobj);
    if (t_fp.motion_id != T_MS(T_GEND)) {
        TestFail("shield in Glide must go to GlideEnd");
        rc = 1;
    }
    t_fp.input.pressed_buttons = 0;
    /* a fresh glide held nose-up climbs, slows and stalls; the stall ends the glide (GlideEnd) */
    t_fp.self_vel.y = 0.0f;
    t_script[0] = GENO_W0_CHG(GENO_SUB_CHG, 2, GENO_COND_ALWAYS, 0, 0, GENO_CHG_ONCE);
    t_script[1] = GENO_TARGET(GENO_TGT_GENO, T_GL);
    t_script[2] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    Geno_PreAnim(&t_gobj);
    if (t_fp.motion_id != T_MS(T_GL) || st->move_f[1] != 1.7f) {
        TestFail("entering Glide must start at the glide speed 1.7");
        rc = 1;
    }
    t_fp.input.lstick[0].y = 1.0f;
    for (f = 0; f < 200 && !st->move_i[2]; f++) {
        gl->phys_cb(&t_gobj);
    }
    gl->anim_cb(&t_gobj);
    if (f >= 200 || t_fp.motion_id != T_MS(T_GEND) || st->move_i[1] != 2) {
        TestFail("a nose-up glide must stall and end in GlideEnd (reason 2)");
        rc = 1;
    }
    t_fp.input.lstick[0].y = 0.0f;
    GenoGame_TestCapture(0, 0);
    Geno_TestRestore();
    return rc;
}

/* Mach Tornado: B taps lift, no taps sink; the stick drifts. Drill Rush: the stick steers within
 * the limits; a bounce request goes to DrillEnd moving back. */
static int test_geno_v2_specials(void)
{
    TestGenoState* st = t_v2_setup();
    MotionState* r;
    int f, rc = 0;
    f32 y0;
    if (st == NULL) {
        TestFail("could not install the v2 profile");
        Geno_TestRestore();
        return 1;
    }
    GenoGame_TestCapture(1, 0);
    t_fp.ground_or_air = GA_Air;
    t_fp.facing_dir = 1.0f;
    t_fp.self_vel.x = 0.0f;
    t_fp.self_vel.y = 0.0f;
    t_fp.co_attrs.gravity = 0.08f;
    t_fp.co_attrs.terminal_velocity = 2.0f;
    t_fp.motion_id = ftCo_MS_Fall;
    Geno_SpecialEnter(&t_gobj, GENO_SP_AIR_N);
    r = Geno_MotionRow(&t_fp, T_MS(T_TOR));
    /* no taps: sinks (after the entry lift) to the tornado's fall speed */
    for (f = 0; f < 30; f++) {
        t_fp.input.pressed_buttons = 0;
        r->phys_cb(&t_gobj);
    }
    if (!(t_fp.self_vel.y < 0.0f) || t_fp.self_vel.y < -0.5001f) {
        TestFail("tornado without taps must sink, no faster than max_fall 0.5");
        rc = 1;
    }
    /* B mashed: a lift every 10 frames (the cooldown), each +1.0, rise capped at 1.4 */
    y0 = -10.0f;
    for (f = 0; f < 20; f++) {
        t_fp.input.pressed_buttons = HSD_PAD_B;
        r->phys_cb(&t_gobj);
        if (t_fp.self_vel.y > y0) {
            y0 = t_fp.self_vel.y;
        }
    }
    if (!(y0 > 0.0f) || y0 > 1.4001f || st->move_i[3] != 2) {
        TestFail("tornado mashing B: 2 lifts in 20 frames, rising, capped at 1.4");
        rc = 1;
    }
    /* drift */
    t_fp.input.pressed_buttons = 0;
    t_fp.input.lstick[0].x = 1.0f;
    for (f = 0; f < 20; f++) {
        r->phys_cb(&t_gobj);
    }
    if (!(t_fp.self_vel.x > 0.0f)) {
        TestFail("tornado must drift with the stick");
        rc = 1;
    }
    t_fp.input.lstick[0].x = 0.0f;
    /* drill */
    Geno_SpecialEnter(&t_gobj, GENO_SP_AIR_S);
    r = Geno_MotionRow(&t_fp, T_MS(T_DRILL));
    t_fp.input.lstick[0].y = 1.0f;
    for (f = 0; f < 60; f++) {
        r->phys_cb(&t_gobj);
    }
    if (!(st->move_f[0] > 0.0f) || !(t_fp.self_vel.y > 0.0f) || !(t_fp.self_vel.x > 0.0f)) {
        TestFail("drill steered up must travel up-forward");
        rc = 1;
    }
    {
        f32 amax = st->move_f[0];
        r->phys_cb(&t_gobj);
        if (st->move_f[0] != amax) {
            TestFail("drill steering must stop at its limit");
            rc = 1;
        }
    }
    t_fp.input.lstick[0].y = 0.0f;
    st->move_i[2] = 1;
    st->move_i[3] = 2;
    r->anim_cb(&t_gobj);
    if (t_fp.motion_id != T_MS(T_DEND) || !(t_fp.self_vel.x < 0.0f)) {
        TestFail("a drill bounce must go to DrillEnd moving back");
        rc = 1;
    }
    GenoGame_TestCapture(0, 0);
    Geno_TestRestore();
    return rc;
}

/* ---- v3 ------------------------------------------------------------------------------------- */

extern int GenoGame_TestLedgeMode(Fighter* fp);
extern f32 GenoGame_TestStartFrame(Fighter* fp, int s);

#define T3_UP 0   /* root motion, no lift-off, front ledge */
#define T3_LOOP 1 /* root motion, lift-off, both ledges, gravity 0.5, origin */
#define T3_GS 2   /* GlideStart */
#define T3_GL 3   /* Glide (posed) */
#define T3_GEND 4 /* GlideEnd */

static const char t_v3_json[] =
    "{\"geno\":3,\"fighters\":[{\"attach\":\"kirby\",\"states\":["
    "{\"name\":\"Up\",\"behavior\":\"geno.anim_motion\",\"subaction\":20,\"liftoff\":false,"
    "\"ledge\":\"front\",\"land\":\"geno:GlideEnd\"},"
    "{\"name\":\"Loop\",\"behavior\":\"geno.anim_motion\",\"subaction\":21,\"ledge\":\"both\","
    "\"gravity\":0.5,\"origin\":true,\"landing_lag\":30},"
    "{\"name\":\"GlideStart\",\"behavior\":\"geno.glide.start\",\"subaction\":10},"
    "{\"name\":\"Glide\",\"behavior\":\"geno.glide\",\"subaction\":11},"
    "{\"name\":\"GlideEnd\",\"behavior\":\"geno.glide.end\",\"subaction\":14,\"next\":\"auto\"}],"
    "\"glide\":{\"pose_center\":90}}]}";

static int t_near(f32 a, f32 b)
{
    return a - b < 0.001f && b - a < 0.001f;
}

/* geno.anim_motion: the clip's TransN delta moves the fighter in the air (both facings) and on the
 * ground; lift-off; gravity on top; the frame-0 origin; ledge defaults and PUT LEDGE; state rows. */
static int test_geno_v3_anim_motion(void)
{
    TestGenoState* st;
    MotionState* up;
    MotionState* loop;
    int rc = 0;
    t_rows[ftCo_MS_Fall].anim_id = 29;
    if (Geno_TestInstall(t_v3_json) != 1 || (st = t_setup()) == NULL) {
        TestFail("could not install the v3 profile");
        Geno_TestRestore();
        return 1;
    }
    GenoGame_TestCapture(1, 0);
    up = Geno_MotionRow(&t_fp, T_MS(T3_UP));
    loop = Geno_MotionRow(&t_fp, T_MS(T3_LOOP));
    if (up->anim_id != 20 || up->phys_cb == NULL || up->coll_cb == NULL || loop->anim_id != 21) {
        TestFail("anim_motion rows: subactions 20 / 21 with the root-motion callbacks");
        rc = 1;
    }
    t_fp.co_attrs.gravity = 0.1f;
    t_fp.co_attrs.terminal_velocity = 2.0f;
    t_fp.x594_b0 = 1;
    /* air, facing right: self_vel = (fwd, up) */
    t_fp.motion_id = T_MS(T3_UP);
    Geno_OnActionChange(&t_gobj);
    t_fp.ground_or_air = GA_Air;
    t_fp.facing_dir = 1.0f;
    t_fp.x6A4_transNOffset.z = 3.0f;
    t_fp.x6A4_transNOffset.y = 2.0f;
    up->phys_cb(&t_gobj);
    if (t_fp.self_vel.x != 3.0f || t_fp.self_vel.y != 2.0f) {
        TestFail("air root motion facing right must give self_vel (3, 2)");
        rc = 1;
    }
    /* facing left: the forward motion is mirrored */
    t_fp.facing_dir = -1.0f;
    up->phys_cb(&t_gobj);
    if (t_fp.self_vel.x != -3.0f || t_fp.self_vel.y != 2.0f) {
        TestFail("air root motion facing left must give self_vel (-3, 2)");
        rc = 1;
    }
    /* ground without lift-off: forward only, stays grounded */
    t_fp.ground_or_air = GA_Ground;
    t_fp.facing_dir = 1.0f;
    up->phys_cb(&t_gobj);
    if (t_fp.ground_or_air != GA_Ground || t_fp.gr_vel != 3.0f) {
        TestFail("ground root motion (no lift-off) must set gr_vel 3 and stay grounded");
        rc = 1;
    }
    /* ledge: the state default (front), then a script's PUT LEDGE 0 */
    if (GenoGame_TestLedgeMode(&t_fp) != 1) {
        TestFail("state \"ledge\": \"front\" must default the ledge grab to 1");
        rc = 1;
    }
    t_script[0] = GENO_W0(GENO_SUB_PUT, 3, 0);
    t_script[1] = GENO_VAL_LEDGE;
    t_script[2] = 0;
    t_script[3] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    if (GenoGame_TestLedgeMode(&t_fp) != 0 || st->ledge != 0) {
        TestFail("PUT LEDGE 0 must disallow the ledge grab for this action");
        rc = 1;
    }
    /* Loop: an action change resets the per-action ledge; lift-off, origin and gravity */
    t_fp.motion_id = T_MS(T3_LOOP);
    Geno_OnActionChange(&t_gobj);
    if (st->ledge != -1 || GenoGame_TestLedgeMode(&t_fp) != 2) {
        TestFail("a new action must reset PUT LEDGE; state \"ledge\": \"both\" -> 2");
        rc = 1;
    }
    t_fp.ground_or_air = GA_Ground;
    t_fp.x68C_transNPos.z = 5.0f;
    t_fp.x68C_transNPos.y = 1.0f;
    t_fp.x6A4_transNOffset.z = 1.0f;
    t_fp.x6A4_transNOffset.y = 2.0f;
    loop->phys_cb(&t_gobj);
    /* origin: (1 + 5, 2 + 1), gravity 0.1 x 0.5 = 0.05 */
    if (t_fp.ground_or_air != GA_Air || !t_near(t_fp.self_vel.x, 6.0f) ||
        !t_near(t_fp.self_vel.y, 2.95f))
    {
        TestFail("lift-off with origin + gravity: airborne, self_vel (6, 2.95)");
        rc = 1;
    }
    loop->phys_cb(&t_gobj);
    if (!t_near(t_fp.self_vel.x, 1.0f) || !t_near(t_fp.self_vel.y, 1.9f)) {
        TestFail("second frame: no origin, gravity accumulated: self_vel (1, 1.9)");
        rc = 1;
    }
    /* savestate: the per-action gravity and ledge survive a round trip */
    if (snap_open(1) == 0) {
        f32 g = st->motion_vy;
        snap_save(0);
        st->motion_vy = 55.0f;
        st->ledge = 2;
        snap_load(0);
        if (st->motion_vy != g || st->ledge != -1) {
            TestFail("anim_motion gravity / ledge not restored by a savestate load");
            rc = 1;
        }
    }
    /* no root motion on the row (flag clear), no gravity: hold still in the air */
    t_fp.motion_id = T_MS(T3_UP);
    Geno_OnActionChange(&t_gobj);
    t_fp.x594_b0 = 0;
    t_fp.ground_or_air = GA_Air;
    t_fp.self_vel.x = 4.0f;
    t_fp.self_vel.y = -1.0f;
    up->phys_cb(&t_gobj);
    if (t_fp.self_vel.x != 0.0f || t_fp.self_vel.y != 0.0f) {
        TestFail("an anim_motion state without root motion and gravity must hold still");
        rc = 1;
    }
    GenoGame_TestCapture(0, 0);
    Geno_TestRestore();
    return rc;
}

/* HIDDEN hides the fighter (Melee's FighterVis flag), survives Geno-to-Geno changes and is cleared
 * by a non-Geno action. The glide starts at its pose frame (the entry pop fix). A glide entered
 * straight from another action ends helpless; one from GlideStart does not. */
static int test_geno_v3_hidden_glide(void)
{
    TestGenoState* st;
    int rc = 0;
    if (Geno_TestInstall(t_v3_json) != 1 || (st = t_setup()) == NULL) {
        TestFail("could not install the v3 profile");
        Geno_TestRestore();
        return 1;
    }
    GenoGame_TestCapture(1, 0);
    t_fp.motion_id = T_MS(T3_UP);
    Geno_OnActionChange(&t_gobj);
    t_script[0] = GENO_W0(GENO_SUB_PUT, 3, 0);
    t_script[1] = GENO_VAL_HIDDEN;
    t_script[2] = 1;
    t_script[3] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    if (!t_fp.x221E_b5 || st->hidden != 1) {
        TestFail("PUT HIDDEN 1 must hide the fighter");
        rc = 1;
    }
    t_fp.motion_id = T_MS(T3_LOOP);
    Geno_OnActionChange(&t_gobj);
    if (st->hidden != 1) {
        TestFail("HIDDEN must survive a change to another Geno state");
        rc = 1;
    }
    t_fp.motion_id = ftCo_MS_Fall;
    Geno_OnActionChange(&t_gobj);
    if (st->hidden != 0) {
        TestFail("a non-Geno action must clear HIDDEN");
        rc = 1;
    }
    /* the glide pose: pose_center 90 -> Glide starts its clip at frame 90 (not 0) */
    if (GenoGame_TestStartFrame(&t_fp, T3_GL) != 90.0f ||
        GenoGame_TestStartFrame(&t_fp, T3_UP) != 0.0f)
    {
        TestFail("Glide must start at pose_center (90); other states at frame 0");
        rc = 1;
    }
    /* a Glide entered straight from the up-B state is helpless; its GlideEnd -> FallSpecial */
    t_fp.ground_or_air = GA_Air;
    t_fp.motion_id = T_MS(T3_LOOP);
    Geno_OnActionChange(&t_gobj);
    t_script[0] = GENO_W0_CHG(GENO_SUB_CHG, 2, GENO_COND_ALWAYS, 0, 0, GENO_CHG_ONCE);
    t_script[1] = GENO_TARGET(GENO_TGT_GENO, T3_GL);
    t_script[2] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    Geno_PreAnim(&t_gobj);
    if (t_fp.motion_id != T_MS(T3_GL) || st->move_i[4] != 1 || st->enter_from != T_MS(T3_LOOP)) {
        TestFail("a Glide entered from the up-B state must be marked helpless");
        rc = 1;
    }
    t_fp.input.pressed_buttons = HSD_PAD_R;
    Geno_MotionRow(&t_fp, T_MS(T3_GL))->input_cb(&t_gobj);
    t_fp.input.pressed_buttons = 0;
    if (t_fp.motion_id != T_MS(T3_GEND)) {
        TestFail("shield in the Glide must go to GlideEnd");
        rc = 1;
    }
    GenoGame_TestCapture(1, 1);
    Geno_MotionRow(&t_fp, T_MS(T3_GEND))->anim_cb(&t_gobj);
    if (GenoGame_TestLastTarget() != GENO_TARGET(GENO_TGT_MOTION, ftCo_MS_FallSpecial)) {
        TestFail("GlideEnd after a script-entered glide must end in FallSpecial");
        rc = 1;
    }
    /* from GlideStart: not helpless */
    GenoGame_TestCapture(1, 0);
    t_fp.motion_id = T_MS(T3_GS);
    Geno_OnActionChange(&t_gobj);
    t_script[0] = GENO_W0_CHG(GENO_SUB_CHG, 2, GENO_COND_ALWAYS, 0, 0, GENO_CHG_ONCE);
    t_script[1] = GENO_TARGET(GENO_TGT_GENO, T3_GL);
    t_script[2] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    Geno_PreAnim(&t_gobj);
    if (t_fp.motion_id != T_MS(T3_GL) || st->move_i[4] != 0) {
        TestFail("a Glide from GlideStart must not be helpless");
        rc = 1;
    }
    GenoGame_TestCapture(0, 0);
    Geno_TestRestore();
    return rc;
}

/* GENO_MAX_STATES (v3: 48): a profile with 40 states builds every row. */
static char t_v3_many[4096];

static int test_geno_v3_many_states(void)
{
    static const char head[] = "{\"geno\":3,\"fighters\":[{\"attach\":\"kirby\",\"states\":[";
    static const char a[] = "{\"name\":\"S";
    static const char b[] = "\",\"behavior\":\"geno.air\",\"subaction\":";
    static const char tail[] = "]}]}";
    int n = 0, i, rc = 0;
    const char* c;
    for (c = head; *c; c++) {
        t_v3_many[n++] = *c;
    }
    for (i = 0; i < 40; i++) {
        for (c = a; *c; c++) {
            t_v3_many[n++] = *c;
        }
        t_v3_many[n++] = (char) ('0' + i / 10);
        t_v3_many[n++] = (char) ('0' + i % 10);
        for (c = b; *c; c++) {
            t_v3_many[n++] = *c;
        }
        t_v3_many[n++] = (char) ('0' + (100 + i) / 100);
        t_v3_many[n++] = (char) ('0' + ((100 + i) / 10) % 10);
        t_v3_many[n++] = (char) ('0' + (100 + i) % 10);
        t_v3_many[n++] = '}';
        if (i < 39) {
            t_v3_many[n++] = ',';
        }
    }
    for (c = tail; *c; c++) {
        t_v3_many[n++] = *c;
    }
    t_v3_many[n] = 0;
    if (GENO_MAX_STATES < 40 || Geno_TestInstall(t_v3_many) != 1 || t_setup() == NULL) {
        TestFail("could not install a 40-state profile");
        Geno_TestRestore();
        return 1;
    }
    if (Geno_MotionRow(&t_fp, T_MS(39))->anim_id != 139 ||
        Geno_MotionRow(&t_fp, T_MS(16))->anim_id != 116)
    {
        TestFail("a 40-state profile must build rows 16..39 (subactions 116 / 139)");
        rc = 1;
    }
    Geno_TestRestore();
    return rc;
}

/* ---- v3: Dimensional Cape --------------------------------------------------------------------- */

extern int GenoGame_TestPairSwap(Fighter* fp, int bhv);

#define TC_START 0
#define TC_START_AIR 1
#define TC_N 2 /* attack states: N, N air, F, F air, B, B air = 2..7 */
#define TC_END 8
#define TC_END_AIR 9

static const char t_cape_json[] =
    "{\"geno\":3,\"fighters\":[{\"attach\":\"kirby\",\"states\":["
    "{\"name\":\"CapeStart\",\"behavior\":\"geno.cape\",\"subaction\":30},"
    "{\"name\":\"CapeStartAir\",\"behavior\":\"geno.cape\",\"subaction\":31},"
    "{\"name\":\"CapeN\",\"behavior\":\"geno.cape.attack\",\"subaction\":32,\"facing\":\"entry\"},"
    "{\"name\":\"CapeNAir\",\"behavior\":\"geno.cape.attack\",\"subaction\":33,\"facing\":\"entry\"},"
    "{\"name\":\"CapeF\",\"behavior\":\"geno.cape.attack\",\"subaction\":34},"
    "{\"name\":\"CapeFAir\",\"behavior\":\"geno.cape.attack\",\"subaction\":35},"
    "{\"name\":\"CapeB\",\"behavior\":\"geno.cape.attack\",\"subaction\":36},"
    "{\"name\":\"CapeBAir\",\"behavior\":\"geno.cape.attack\",\"subaction\":37},"
    "{\"name\":\"CapeEnd\",\"behavior\":\"geno.cape.end\",\"subaction\":38},"
    "{\"name\":\"CapeEndAir\",\"behavior\":\"geno.cape.end\",\"subaction\":39}],"
    "\"cape\":{\"w00\":0.5,\"w01\":0.4,\"w02\":0.5,\"w03\":2.5,\"w04\":0.5,\"w05\":2.5},"
    "\"specials\":{\"lw\":\"geno:CapeStart\",\"air_lw\":\"geno:CapeStartAir\"}}]}";

/* The start keeps half the momentum without gravity, the vanish steers on both axes (accel 0.5 to
 * stick x 2.5), B held at frame 26 picks the slash (stick back -> the "F" states), no button the
 * plain reappear; ground / air partners swap with the frame kept (the move's counter survives);
 * the root motion of a reappear keeps the entry facing through a Reverse Direction. */
static int test_geno_v3_cape(void)
{
    TestGenoState* st;
    MotionState* r;
    int f, rc = 0;
    if (Geno_TestInstall(t_cape_json) != 1 || (st = t_setup()) == NULL) {
        TestFail("could not install the cape profile");
        Geno_TestRestore();
        return 1;
    }
    GenoGame_TestCapture(1, 0);
    t_fp.ground_or_air = GA_Air;
    t_fp.facing_dir = 1.0f;
    t_fp.self_vel.x = 2.0f;
    t_fp.self_vel.y = -1.0f;
    t_fp.motion_id = ftCo_MS_Fall;
    if (Geno_SpecialEnter(&t_gobj, GENO_SP_AIR_LW) != 1 || t_fp.motion_id != T_MS(TC_START_AIR)) {
        TestFail("specials.air_lw -> CapeStartAir");
        GenoGame_TestCapture(0, 0);
        Geno_TestRestore();
        return 1;
    }
    if (t_fp.self_vel.x != 1.0f || !t_near(t_fp.self_vel.y, -0.4f)) {
        TestFail("cape start must keep the momentum at 0.5 x / 0.4 y");
        rc = 1;
    }
    r = Geno_MotionRow(&t_fp, T_MS(TC_START_AIR));
    /* frames 0-11: the kept momentum, no gravity */
    t_fp.input.lstick[0].x = -1.0f;
    t_fp.input.lstick[0].y = 1.0f;
    for (f = 0; f < 12; f++) {
        r->phys_cb(&t_gobj);
    }
    if (t_fp.self_vel.x != 1.0f || !t_near(t_fp.self_vel.y, -0.4f)) {
        TestFail("cape frames 0-11 must hold the kept momentum (no gravity, no steering)");
        rc = 1;
    }
    /* frame 12: steering, 0.5 a frame toward (-2.5, 2.5) */
    r->phys_cb(&t_gobj);
    if (!t_near(t_fp.self_vel.x, 0.5f) || !t_near(t_fp.self_vel.y, 0.1f)) {
        TestFail("cape frame 12 must steer by 0.5 toward the stick (vel 0.5, 0.1)");
        rc = 1;
    }
    for (f = 0; f < 10; f++) {
        r->phys_cb(&t_gobj);
    }
    if (!t_near(t_fp.self_vel.x, -2.5f) || !t_near(t_fp.self_vel.y, 2.5f)) {
        TestFail("cape steering must reach stick x 2.5 on both axes");
        rc = 1;
    }
    /* before frame 26 nothing is decided */
    t_fp.input.held_buttons[0] = HSD_PAD_B;
    r->anim_cb(&t_gobj);
    if (t_fp.motion_id != T_MS(TC_START_AIR)) {
        TestFail("cape must not reappear before frame 26");
        rc = 1;
    }
    /* a landing swaps to the ground start with the frame kept (the counter goes on) */
    t_fp.ground_or_air = GA_Ground;
    if (GenoGame_TestPairSwap(&t_fp, GENO_BHV_CAPE) != 1 || t_fp.motion_id != T_MS(TC_START) ||
        !st->enter_keep || st->move_i[0] != 23)
    {
        TestFail("a landed cape start must swap to CapeStart, frame and counter kept");
        rc = 1;
    }
    t_fp.ground_or_air = GA_Air;
    GenoGame_TestPairSwap(&t_fp, GENO_BHV_CAPE);
    r = Geno_MotionRow(&t_fp, T_MS(TC_START_AIR));
    for (f = 0; f < 3; f++) {
        r->phys_cb(&t_gobj);
    }
    /* frame 26, B held, stick back (x -1 facing right) -> the "F" state, air */
    r->anim_cb(&t_gobj);
    if (t_fp.motion_id != T_MS(TC_N + 3) || st->move_i[1] != 12) {
        TestFail("B held + stick back at frame 26 (air) must reappear in CapeFAir");
        rc = 1;
    }
    /* again, nothing held, on the ground -> the plain reappear */
    t_fp.motion_id = ftCo_MS_Wait;
    t_fp.ground_or_air = GA_Ground;
    Geno_SpecialEnter(&t_gobj, GENO_SP_LW);
    r = Geno_MotionRow(&t_fp, T_MS(TC_START));
    t_fp.input.held_buttons[0] = 0;
    t_fp.input.lstick[0].x = 0.0f;
    t_fp.input.lstick[0].y = 0.0f;
    for (f = 0; f < 26; f++) {
        r->phys_cb(&t_gobj);
    }
    r->anim_cb(&t_gobj);
    if (t_fp.motion_id != T_MS(TC_END) || st->move_i[1] != 0) {
        TestFail("nothing held at frame 26 on the ground must reappear in CapeEnd");
        rc = 1;
    }
    /* neutral with A held -> N; its root motion keeps the entry facing through a turn */
    t_fp.motion_id = ftCo_MS_Fall;
    t_fp.ground_or_air = GA_Air;
    Geno_SpecialEnter(&t_gobj, GENO_SP_AIR_LW);
    r = Geno_MotionRow(&t_fp, T_MS(TC_START_AIR));
    t_fp.input.held_buttons[0] = HSD_PAD_A;
    for (f = 0; f < 26; f++) {
        r->phys_cb(&t_gobj);
    }
    r->anim_cb(&t_gobj);
    t_fp.input.held_buttons[0] = 0;
    if (t_fp.motion_id != T_MS(TC_N + 1)) {
        TestFail("A held with a neutral stick (air) must reappear in CapeNAir");
        rc = 1;
    }
    r = Geno_MotionRow(&t_fp, T_MS(TC_N + 1));
    t_fp.x594_b0 = 1;
    t_fp.x6A4_transNOffset.z = 2.0f;
    t_fp.x6A4_transNOffset.y = 0.0f;
    r->phys_cb(&t_gobj);
    t_fp.facing_dir = -1.0f; /* Brawl's Reverse Direction at frame 1 */
    r->phys_cb(&t_gobj);
    if (t_fp.self_vel.x != 2.0f) {
        TestFail("facing \"entry\": the reappear's travel must keep the entry facing after a turn");
        rc = 1;
    }
    t_fp.x594_b0 = 0;
    t_fp.facing_dir = 1.0f;
    GenoGame_TestCapture(0, 0);
    Geno_TestRestore();
    return rc;
}

/* ---- v3: Drill Rush (Brawl's code) ------------------------------------------------------------- */

#define TD_START 0
#define TD_START_AIR 1
#define TD_RUSH 2
#define TD_END 3
#define TD_END_AIR 4

static const char t_drill_json[] =
    "{\"geno\":3,\"fighters\":[{\"attach\":\"kirby\",\"states\":["
    "{\"name\":\"DrillStart\",\"behavior\":\"geno.drill.start\",\"subaction\":40},"
    "{\"name\":\"DrillStartAir\",\"behavior\":\"geno.drill.start\",\"subaction\":41},"
    "{\"name\":\"Drill\",\"behavior\":\"geno.drill\",\"subaction\":42},"
    "{\"name\":\"DrillEndGround\",\"behavior\":\"geno.drill.end\",\"subaction\":43,\"next\":\"auto\"},"
    "{\"name\":\"DrillEnd\",\"behavior\":\"geno.drill.end\",\"subaction\":44,\"next\":\"helpless\"}],"
    "\"drill\":{\"w00\":0.5,\"w01\":1.0,\"w02\":-0.08,\"w03\":3.0,\"w04\":10},"
    "\"specials\":{\"s\":\"geno:DrillStart\",\"air_s\":\"geno:DrillStartAir\"}}]}";

/* The start's ground / air versions swap with the frame kept and the speeds untouched; the rush's
 * pitch has no limit (3 deg a frame, 45 frames -> 135); a hit does not end the rush; the rush ends
 * in the end state of its situation; the air end pops back and up and is helpless. */
static int test_geno_v3_drill(void)
{
    TestGenoState* st;
    MotionState* r;
    int f, rc = 0;
    if (Geno_TestInstall(t_drill_json) != 1 || (st = t_setup()) == NULL) {
        TestFail("could not install the drill profile");
        Geno_TestRestore();
        return 1;
    }
    GenoGame_TestCapture(1, 0);
    t_fp.ground_or_air = GA_Air;
    t_fp.facing_dir = 1.0f;
    t_fp.self_vel.x = 2.0f;
    t_fp.self_vel.y = -1.0f;
    t_fp.motion_id = ftCo_MS_Fall;
    Geno_SpecialEnter(&t_gobj, GENO_SP_AIR_S);
    if (t_fp.motion_id != T_MS(TD_START_AIR) || t_fp.self_vel.x != 1.0f || t_fp.self_vel.y != 1.0f) {
        TestFail("air drill start: vx x 0.5, vy = 1.0");
        rc = 1;
    }
    t_fp.ground_or_air = GA_Ground;
    t_fp.gr_vel = 0.7f;
    if (GenoGame_TestPairSwap(&t_fp, GENO_BHV_DRILL_START) != 1 || t_fp.motion_id != T_MS(TD_START) ||
        t_fp.gr_vel != 0.7f)
    {
        TestFail("a landed drill start must swap to the ground start, speeds untouched");
        rc = 1;
    }
    /* the rush */
    t_fp.ground_or_air = GA_Air;
    t_fp.motion_id = ftCo_MS_Fall;
    Geno_SpecialEnter(&t_gobj, GENO_SP_AIR_S);
    t_script[0] = GENO_W0_CHG(GENO_SUB_CHG, 2, GENO_COND_ALWAYS, 0, 0, GENO_CHG_ONCE);
    t_script[1] = GENO_TARGET(GENO_TGT_GENO, TD_RUSH);
    t_script[2] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    Geno_PreAnim(&t_gobj);
    r = Geno_MotionRow(&t_fp, T_MS(TD_RUSH));
    if (t_fp.motion_id != T_MS(TD_RUSH)) {
        TestFail("could not enter the rush");
        GenoGame_TestCapture(0, 0);
        Geno_TestRestore();
        return 1;
    }
    t_fp.input.lstick[0].y = 1.0f;
    for (f = 0; f < 45; f++) {
        r->phys_cb(&t_gobj);
    }
    if (!t_near(st->move_f[0], 135.0f) || !(t_fp.self_vel.x < 0.0f) || !(t_fp.self_vel.y > 0.0f)) {
        TestFail("45 frames of stick up must pitch the rush to 135 deg (no limit): back and up");
        rc = 1;
    }
    t_fp.input.lstick[0].y = 0.0f;
    /* a hit does not end the rush (no bounce by default) */
    t_fp.deal_dmg_cb(&t_gobj);
    r->anim_cb(&t_gobj);
    if (t_fp.motion_id != T_MS(TD_RUSH) || st->move_i[1] != 1) {
        TestFail("a hit must be recorded but must not end the rush");
        rc = 1;
    }
    /* the clip ends in the air -> the air end: pop back / up; then helpless */
    GenoGame_TestCapture(1, 1);
    r->anim_cb(&t_gobj);
    if (t_fp.motion_id != T_MS(TD_END_AIR) || t_fp.self_vel.x != -1.0f || !t_near(t_fp.self_vel.y, 2.1f)) {
        TestFail("the rush ending in the air must go to the air end with the pop (-1, 2.1)");
        rc = 1;
    }
    Geno_MotionRow(&t_fp, T_MS(TD_END_AIR))->anim_cb(&t_gobj);
    if (GenoGame_TestLastTarget() != GENO_TARGET(GENO_TGT_MOTION, ftCo_MS_FallSpecial)) {
        TestFail("the air end must end in FallSpecial even after a hit");
        rc = 1;
    }
    /* a rush ending on the floor -> the ground end */
    GenoGame_TestCapture(1, 0);
    t_fp.motion_id = T_MS(TD_RUSH);
    Geno_OnActionChange(&t_gobj);
    t_fp.ground_or_air = GA_Ground;
    GenoGame_TestCapture(1, 1);
    r->anim_cb(&t_gobj);
    if (t_fp.motion_id != T_MS(TD_END)) {
        TestFail("the rush ending on the floor must go to the ground end");
        rc = 1;
    }
    GenoGame_TestCapture(0, 0);
    Geno_TestRestore();
    return rc;
}

/* ---- v4: the model follows the move ------------------------------------------------------------ */

extern f32 GenoGame_TestModelRotX(void);
extern f32 GenoGame_TestJointRate(void);
extern void GenoGame_TestPoseReset(f32 v);

#define T_D2R 0.017453292f

static const char t_drill_flat_json[] =
    "{\"geno\":3,\"fighters\":[{\"attach\":\"kirby\",\"states\":["
    "{\"name\":\"DrillStart\",\"behavior\":\"geno.drill.start\",\"subaction\":40},"
    "{\"name\":\"DrillStartAir\",\"behavior\":\"geno.drill.start\",\"subaction\":41},"
    "{\"name\":\"Drill\",\"behavior\":\"geno.drill\",\"subaction\":42},"
    "{\"name\":\"DrillEndGround\",\"behavior\":\"geno.drill.end\",\"subaction\":43,\"next\":\"auto\"},"
    "{\"name\":\"DrillEnd\",\"behavior\":\"geno.drill.end\",\"subaction\":44,\"next\":\"helpless\"}],"
    "\"drill\":{\"w03\":3.0,\"w04\":10,\"pitch_model\":0},"
    "\"specials\":{\"s\":\"geno:DrillStart\",\"air_s\":\"geno:DrillStartAir\"}}]}";

/* into the rush from the air start (the script's CHG, as MK's DrillStart does) */
static MotionState* t_drill_rush(void)
{
    t_fp.ground_or_air = GA_Air;
    t_fp.motion_id = ftCo_MS_Fall;
    Geno_SpecialEnter(&t_gobj, GENO_SP_AIR_S);
    t_script[0] = GENO_W0_CHG(GENO_SUB_CHG, 2, GENO_COND_ALWAYS, 0, 0, GENO_CHG_ONCE);
    t_script[1] = GENO_TARGET(GENO_TGT_GENO, TD_RUSH);
    t_script[2] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    Geno_PreAnim(&t_gobj);
    return t_fp.motion_id == T_MS(TD_RUSH) ? Geno_MotionRow(&t_fp, T_MS(TD_RUSH)) : NULL;
}

/* Drill Rush (Brawl SpecialSRush / SpecialSEnd: posture rot.x): the model's TopN rotation X is
 * -pitch every frame of the rush (nose up for either facing, the travel along the nose) and of the
 * end (easing with the pitch); pitch_model 0 keeps the model level. */
static int test_geno_v4_drill_pose(void)
{
    TestGenoState* st;
    MotionState* r;
    int f, rc = 0;
    if (Geno_TestInstall(t_drill_json) != 1 || (st = t_setup()) == NULL) {
        TestFail("could not install the drill profile");
        Geno_TestRestore();
        return 1;
    }
    GenoGame_TestCapture(1, 0);
    t_fp.facing_dir = 1.0f;
    GenoGame_TestPoseReset(99.0f);
    if ((r = t_drill_rush()) == NULL || GenoGame_TestModelRotX() != 0.0f) {
        TestFail("entering the rush must set the model level (rot x 0)");
        rc = 1;
    }
    if (r == NULL) {
        GenoGame_TestCapture(0, 0);
        Geno_TestRestore();
        return 1;
    }
    /* facing right, stick up: +3 deg a frame; the model and the travel turn together */
    t_fp.input.lstick[0].y = 1.0f;
    for (f = 1; f <= 10; f++) {
        f32 want, trav;
        r->phys_cb(&t_gobj);
        want = 3.0f * f;
        trav = atan2f(t_fp.self_vel.y, t_fp.self_vel.x * t_fp.facing_dir) / T_D2R;
        if (!t_near(st->move_f[0], want) || !t_near(GenoGame_TestModelRotX(), -want * T_D2R) ||
            !(trav - want < 0.01f && want - trav < 0.01f))
        {
            TestFail("steering up (facing right): rot x = -pitch, travel angle = pitch, every frame");
            rc = 1;
            break;
        }
    }
    /* facing left (a turn mid-rush, as a Reverse Direction would): still nose up, travel to the left */
    t_fp.facing_dir = -1.0f;
    for (f = 0; f < 5; f++) {
        r->phys_cb(&t_gobj);
    }
    if (!t_near(st->move_f[0], 45.0f) || !t_near(GenoGame_TestModelRotX(), -45.0f * T_D2R) ||
        !(t_fp.self_vel.x < 0.0f) || !(t_fp.self_vel.y > 0.0f))
    {
        TestFail("facing left: rot x = -45 deg (nose up), travel up-left");
        rc = 1;
    }
    /* stick down: the pitch and the model come back down past level */
    t_fp.input.lstick[0].y = -1.0f;
    for (f = 0; f < 20; f++) {
        r->phys_cb(&t_gobj);
    }
    if (!t_near(st->move_f[0], -15.0f) || !t_near(GenoGame_TestModelRotX(), 15.0f * T_D2R) ||
        !(t_fp.self_vel.y < 0.0f))
    {
        TestFail("steering down to -15 deg: rot x = +15 deg (nose down), travel down");
        rc = 1;
    }
    /* the end keeps the pitch on entry and eases it x (1 - 2/10) a frame, the model with it */
    t_fp.input.lstick[0].y = 0.0f;
    t_fp.facing_dir = 1.0f;
    GenoGame_TestCapture(1, 1);
    r->anim_cb(&t_gobj);
    if (t_fp.motion_id != T_MS(TD_END_AIR) || !t_near(GenoGame_TestModelRotX(), 15.0f * T_D2R)) {
        TestFail("the end must start with the rush's pitch on the model");
        rc = 1;
    }
    GenoGame_TestCapture(1, 0);
    Geno_MotionRow(&t_fp, T_MS(TD_END_AIR))->phys_cb(&t_gobj);
    if (!t_near(st->move_f[0], -12.0f) || !t_near(GenoGame_TestModelRotX(), 12.0f * T_D2R)) {
        TestFail("the end eases the pitch (-15 -> -12) and the model follows");
        rc = 1;
    }
    Geno_TestRestore();
    /* pitch_model 0: the travel turns, the model does not */
    if (Geno_TestInstall(t_drill_flat_json) != 1 || (st = t_setup()) == NULL) {
        TestFail("could not install the flat drill profile");
        GenoGame_TestCapture(0, 0);
        Geno_TestRestore();
        return 1;
    }
    GenoGame_TestCapture(1, 0);
    t_fp.facing_dir = 1.0f;
    if ((r = t_drill_rush()) != NULL) {
        t_fp.input.lstick[0].y = 1.0f;
        for (f = 0; f < 10; f++) {
            r->phys_cb(&t_gobj);
        }
        t_fp.input.lstick[0].y = 0.0f;
    }
    if (r == NULL || !t_near(st->move_f[0], 30.0f) || GenoGame_TestModelRotX() != 0.0f) {
        TestFail("pitch_model 0: the rush pitches (30 deg) with the model level");
        rc = 1;
    }
    GenoGame_TestCapture(0, 0);
    Geno_TestRestore();
    return rc;
}

static const char t_spin_json[] =
    "{\"geno\":3,\"fighters\":[{\"attach\":\"kirby\",\"states\":["
    "{\"name\":\"Tornado\",\"behavior\":\"geno.tornado\",\"subaction\":15}],"
    "\"tornado\":{\"spin_anim\":1,\"spin_period\":360},"
    "\"specials\":{\"n\":\"geno:Tornado\"}}]}";

/* Mach Tornado (Brawl: the spin rate IS SpecialNSpin's frame speed): the joints play the clip at the
 * spin rate from the entry (80) on, every frame: -1.5 a frame, +16 per lift, 0..80; spin_anim 0
 * (v3) leaves the animation alone. */
static int test_geno_v4_tornado_spin(void)
{
    TestGenoState* st;
    MotionState* r;
    int f, rc = 0, lifts = 0;
    f32 want = 80.0f;
    if (Geno_TestInstall(t_spin_json) != 1 || (st = t_setup()) == NULL) {
        TestFail("could not install the spin profile");
        Geno_TestRestore();
        return 1;
    }
    GenoGame_TestCapture(1, 0);
    GenoGame_TestPoseReset(-1.0f);
    t_fp.ground_or_air = GA_Air;
    t_fp.facing_dir = 1.0f;
    t_fp.self_vel.x = 0.0f;
    t_fp.self_vel.y = 0.0f;
    t_fp.co_attrs.gravity = 0.08f;
    t_fp.co_attrs.terminal_velocity = 2.0f;
    t_fp.motion_id = ftCo_MS_Fall;
    Geno_SpecialEnter(&t_gobj, GENO_SP_AIR_N);
    r = Geno_MotionRow(&t_fp, T_MS(0));
    if (t_fp.motion_id != T_MS(0) || GenoGame_TestJointRate() != 80.0f) {
        TestFail("the tornado must start its spin clip at the start rate (80)");
        rc = 1;
    }
    /* B tapped at frames 20 and 24 (the second is inside the 10-frame cooldown: it waits), then
       none: the rate follows Brawl's formula frame by frame */
    for (f = 1; f <= 60 && rc == 0; f++) {
        int tap = f == 20 || f == 24;
        t_fp.input.pressed_buttons = tap ? HSD_PAD_B : 0;
        r->phys_cb(&t_gobj);
        want -= 1.5f;
        if (f == 20 || f == 30) {
            want += 16.0f; /* the lift at 20; the tap at 24 is armed and lifts at the cooldown (30) */
            lifts++;
        }
        want = want < 0.0f ? 0.0f : want > 80.0f ? 80.0f : want;
        if (!t_near(st->move_f[0], want) || !t_near(GenoGame_TestJointRate(), want)) {
            TestFail("the spin clip's rate must be the spin rate (80 - 1.5 a frame, +16 per lift)");
            rc = 1;
        }
    }
    if (rc == 0 && st->move_i[3] != lifts) {
        TestFail("two lifts expected (frames 20 and 30)");
        rc = 1;
    }
    t_fp.input.pressed_buttons = 0;
    GenoGame_TestCapture(0, 0);
    Geno_TestRestore();
    /* spin_anim 0 (the v2 profile): the animation is left alone */
    if ((st = t_v2_setup()) == NULL) {
        TestFail("could not install the v2 profile");
        Geno_TestRestore();
        return 1;
    }
    GenoGame_TestCapture(1, 0);
    GenoGame_TestPoseReset(-1.0f);
    t_fp.ground_or_air = GA_Air;
    t_fp.motion_id = ftCo_MS_Fall;
    Geno_SpecialEnter(&t_gobj, GENO_SP_AIR_N);
    Geno_MotionRow(&t_fp, T_MS(T_TOR))->phys_cb(&t_gobj);
    if (GenoGame_TestJointRate() != -1.0f) {
        TestFail("spin_anim 0 must not touch the animation rate");
        rc = 1;
    }
    GenoGame_TestCapture(0, 0);
    Geno_TestRestore();
    return rc;
}

/* Geno Lab: a player slot whose fighter the scene has freed (the player table keeps the pointer
 * after a match; the Lab crashed in ScriptGame_FighterI from Script_FramePost on the CSS) must
 * read as "no fighter", never dereference it. */
extern StaticPlayer player_slots[];
extern int ScriptGame_FighterI(int slot, int field);
extern float ScriptGame_FighterF(int slot, int field);

static int test_geno_lab_stale_fighter(void)
{
    static StaticPlayer saved;
    int rc = 0;
    t_setup();
    saved = player_slots[5];
    player_slots[5].transformed[0] = 0;
    player_slots[5].player_entity[0] = &t_gobj; /* a fighter gobj that is not in the live list */
    if (ScriptGame_FighterI(5, 0) != 0 || ScriptGame_FighterI(5, 3) != -1 ||
        ScriptGame_FighterF(5, 0) != 0.0f)
    {
        TestFail("a slot whose fighter is not in the live fighter list must read as absent");
        rc = 1;
    }
    player_slots[5] = saved;
    return rc;
}

/* gd.player(p).team: the retail team index, readable for a slot whose fighter has not spawned, -1 for an empty slot.
 * Field 9 is SCRIPT_I_TEAM (script_game.c; SI_TEAM in gw_script.c). The Envoy mod reads it to tell a CPU teammate
 * (a 1P team stage) from an opponent. */
static int test_geno_player_team_read(void)
{
    static StaticPlayer saved1, saved2;
    int rc = 0;
    t_setup();
    saved1 = player_slots[1];
    saved2 = player_slots[2];
    player_slots[1].pkind = 1; /* CPU */
    player_slots[1].team = 0;
    player_slots[2].pkind = 1;
    player_slots[2].team = 2;
    if (ScriptGame_FighterI(1, 9) != 0 || ScriptGame_FighterI(2, 9) != 2) {
        TestFail("the team read must return Player_GetTeam for a present slot");
        rc = 1;
    }
    player_slots[2].pkind = 3; /* none */
    if (ScriptGame_FighterI(2, 9) != -1 || ScriptGame_FighterI(6, 9) != -1) {
        TestFail("an empty or out-of-range slot must read team -1");
        rc = 1;
    }
    player_slots[1] = saved1;
    player_slots[2] = saved2;
    return rc;
}

/* ---- LAB, the Lab's own game mode (geno_lab_mode.c) ------------------------------------------ */
#include <melee/gm/gm_1A3F.h>
#include <melee/gm/gmfrontend.h>
#include <melee/gm/gmscdata.h>
#include <melee/gm/gmvsmelee.h>
#include <melee/gm/gmvs.h>
#include <melee/gm/gmscenelaunch.h>
#include <melee/it/itspawn.h>
#include "geno_lab_mode.h"

extern void SceneLaunch_LoadForTest(const char* text);
extern int SceneLaunch_BootGameMode(void);
extern int SceneLaunch_EntryStateId(void);
extern int SceneLaunch_PlayerCKind(int slot);
extern int SceneLaunch_PlayerSlotType(int slot);
extern int SceneLaunch_StageExternal(void);

/* its own mode id, past every vanilla one and the port's GM_FRONTEND, in the mode table with
   CSS -> SSS -> match (+ the loading screen), the match being VS's scene */
static int test_geno_lab_mode_table(void)
{
    GameMode* m;
    GameModeState* s;
    int found = 0;
    if (GM_LAB <= GM_COUNT || GM_LAB == GM_FRONTEND) {
        TestFail("GM_LAB must be past GM_COUNT and not GM_FRONTEND");
        return 1;
    }
    for (m = gm_GetAllGameModes(); m->kind != GM_COUNT; m++) {
        if (m->kind == GM_LAB) {
            found++;
            if (m->states != gm_Mode_Lab_States || m->on_load != gm_Mode_Lab_OnLoad) {
                TestFail("the GM_LAB row does not point at LAB's states / on_load");
                return 1;
            }
        }
    }
    if (found != 1) {
        TestFail("GM_LAB must be in the mode table exactly once");
        return 1;
    }
    s = gm_Mode_Lab_States;
    if (s[0].info.scene_kind != GS_CSS || s[1].info.scene_kind != GS_SSS ||
        s[2].info.scene_kind != GS_VS || s[2].info.enter_data != &gmVsMelee_StartData ||
        s[3].info.scene_kind != GS_FRONTEND || s[4].id != (u8) GM_GAMEMODESTATE_TERMINATE)
    {
        TestFail("LAB's states must be CSS, SSS, the VS match and the loading screen");
        return 1;
    }
    return 0;
}

/* the rules: no timer, no stocks (a KO respawns: time mode never eliminates), no items, Melee's
   pause off - applied over whatever VS's saved rules said (stock mode, 4 stocks, 8 minutes) */
static int test_scene_item_frequency_rules(void)
{
    StartMeleeRules* live = gm_GetStartMeleeRules();
    StartMeleeRules saved = *live;
    struct GamePrefs prefs = { 0 };
    int rc = 0;
    live->item_freq = 3;
    live->x20 = ~0ULL; /* Allowed kinds remain separate from frequency. */
    prefs.item_freq = 3;
    prefs.item_mask = ~0ULL;
    SceneLaunch_LoadForTest("mode=vs;items=off");
    SceneLaunch_ApplyItemFrequency(live, &prefs);
    if (SceneLaunch_ItemFreq() != -2 || live->item_freq != -1 ||
        prefs.item_freq != 0xFF || gm_8016AE80() != -1 ||
        live->x20 != ~0ULL || prefs.item_mask != ~0ULL) {
        TestFail("items=off must disable native/saved frequency without changing allowed kinds");
        rc = 1;
    } else {
        /* The actual random-item initializer must exit safely with all kinds allowed. */
        it_8026D018();
        if (it_8026D324(It_Kind_BombHei)) {
            TestFail("items=off left native random-item eligibility enabled");
            rc = 1;
        }
    }
    SceneLaunch_LoadForTest("mode=vs;items=0");
    SceneLaunch_ApplyItemFrequency(live, &prefs);
    if (SceneLaunch_ItemFreq() != 0 || live->item_freq != 0 ||
        prefs.item_freq != 0 || gm_8016AE80() != 0) {
        TestFail("items=0 is frequency index zero, not disabled");
        rc = 1;
    }
    SceneLaunch_LoadForTest(NULL);
    live->item_freq = 2;
    prefs.item_freq = 2;
    SceneLaunch_ApplyItemFrequency(live, &prefs);
    if (live->item_freq != 2 || prefs.item_freq != 2) {
        TestFail("unspecified item frequency must preserve existing rules");
        rc = 1;
    }
    *live = saved;
    return rc;
}

static int test_geno_lab_rules(void)
{
    static StartMeleeData d;
    int i;
    d.rules.match_kind = MatchKind_Stock;
    d.rules.is_stock = true;
    d.rules.timer_enabled = true;
    d.rules.time_limit = 480;
    d.rules.item_freq = 2;
    d.rules.x4_2 = true;
    for (i = 0; i < Gm_Player_NumMax; i++) {
        d.players[i].stocks = 4;
    }
    GenoLab_ApplyRules(&d);
    if (d.rules.match_kind != MatchKind_Time || d.rules.is_stock || d.rules.timer_enabled ||
        d.rules.time_limit != 0 || d.rules.x4_2)
    {
        TestFail("LAB rules: time mode with the clock off, no stock game over");
        return 1;
    }
    if (d.rules.item_freq != -1 || !d.rules.disable_pausing) {
        TestFail("LAB rules: items off, Melee's pause off");
        return 1;
    }
    /* A scene may opt in to items (gd.give_item needs the match's item data); off stays off. */
    SceneLaunch_LoadForTest("mode=lab;items=3");
    d.rules.item_freq = -1;
    GenoLab_ApplyRules(&d);
    if (d.rules.item_freq != 3) { SceneLaunch_LoadForTest(NULL); TestFail("LAB rules: items=3 must reach the match"); return 1; }
    SceneLaunch_LoadForTest("mode=lab;items=off");
    d.rules.item_freq = 2;
    GenoLab_ApplyRules(&d);
    if (d.rules.item_freq != -1) { SceneLaunch_LoadForTest(NULL); TestFail("LAB rules: items=off stays off"); return 1; }
    SceneLaunch_LoadForTest(NULL);
    for (i = 0; i < Gm_Player_NumMax; i++) {
        if (d.players[i].stocks != 0) {
            TestFail("LAB rules: no stocks on any player");
            return 1;
        }
    }
    return 0;
}

/* MELEE_SCENE=mode=lab;p1=...;p2=.../cpu;stage=fd boots LAB's match with those fighters */
static int test_geno_lab_scene(void)
{
    int rc = 0;
    SceneLaunch_LoadForTest("mode=lab;p1=fox;p2=falco/cpu;stage=fd");
    if (SceneLaunch_BootGameMode() != GM_LAB || SceneLaunch_EntryStateId() != 2 ||
        SceneLaunch_PlayerCKind(0) < 0 || SceneLaunch_PlayerCKind(1) < 0 ||
        SceneLaunch_PlayerSlotType(1) != Gm_PKind_Cpu || SceneLaunch_StageExternal() != 32)
    {
        TestFail("mode=lab did not give LAB's match, Fox + CPU Falco on FD");
        rc = 1;
    }
    SceneLaunch_LoadForTest("mode=lab;at=css");
    if (rc == 0 && (SceneLaunch_BootGameMode() != GM_LAB || SceneLaunch_EntryStateId() != 0)) {
        TestFail("mode=lab;at=css did not open LAB's character select");
        rc = 1;
    }
    SceneLaunch_LoadForTest(NULL);
    return rc;
}

/* the menu flow: LAB's select states always run the kit's screens (GS_FRONTEND) - whatever
   MELEE_NATIVE_CSS says - while Training's default stays the native screen; and leaving is
   refused outside a LAB match (the pause menu cannot end some other mode's match) */
static int test_geno_lab_select_flow(void)
{
    GameModeState st = gm_Mode_Lab_States[0];
    gmFrontend_ModeSelect(&st, 0, "LAB");
    if (st.info.scene_kind != GS_FRONTEND) {
        TestFail("LAB's character select must run the kit's screen");
        return 1;
    }
    if (GenoLab_ModeActive() || GenoLab_Leave(GENO_LAB_TO_MENU) != 0 ||
        GenoLab_NextAfterMatch() != GENO_LAB_TO_CSS)
    {
        TestFail("outside LAB: not active, leave refused, next stays the character select");
        return 1;
    }
    return 0;
}

/* Stage E: the rollback visualiser names a SyncTest mismatch inside a GenoState / a Fighter */
extern int GenoGame_StateSize(void);
extern const char* GenoGame_StateFieldName(int off);
extern int GenoGame_StateFieldBase(int off);
extern const char* ScriptGame_LabFighterFieldName(int off);
extern int ScriptGame_LabFighterFieldBase(int off);
static int t_streq(const char* a, const char* b)
{
    if (a == NULL || b == NULL) {
        return 0;
    }
    while (*a != 0 && *a == *b) {
        ++a;
        ++b;
    }
    return *a == *b;
}
static int test_geno_lab_mismatch_fields(void)
{
    int mi = (int) __builtin_offsetof(GenoState, move_i);
    int pos = (int) __builtin_offsetof(Fighter, cur_pos);
    int pct = (int) __builtin_offsetof(Fighter, dmg);
    if (GenoGame_StateSize() != (int) sizeof(GenoState)) {
        TestFail("GenoGame_StateSize is not sizeof(GenoState)");
        return 1;
    }
    if (!t_streq(GenoGame_StateFieldName(mi + 8), "move_i") || GenoGame_StateFieldBase(mi + 8) != mi) {
        TestFail("GenoState +move_i[2] must name move_i at its base");
        return 1;
    }
    if (!t_streq(GenoGame_StateFieldName((int) __builtin_offsetof(GenoState, motion_vy)), "motion_vy")) {
        TestFail("GenoState motion_vy must be named");
        return 1;
    }
    if (!t_streq(ScriptGame_LabFighterFieldName(pos + 4), "cur_pos") || ScriptGame_LabFighterFieldBase(pos + 4) != pos) {
        TestFail("Fighter +cur_pos.y must name cur_pos");
        return 1;
    }
    if (!t_streq(ScriptGame_LabFighterFieldName(pct + 4), "dmg")) {
        TestFail("Fighter dmg.x1830_percent must name dmg");
        return 1;
    }
    return 0;
}

/* ---- v5: articles, on_hit, counter windows (docs/geno.md section 19) ------------------------- */

extern int Geno_ArticleCount(int p);
extern int Geno_ArticleParam(int p, int a, int id);
extern int Geno_ArticleHitCount(int p, int a);
extern int Geno_ArticleHitParam(int p, int a, int h, int id);
extern int Geno_StateCounter(int p, int s, int field);
extern int Geno_HookCount(int p, int ev);
extern void* Geno_ArticleDesc(int kind);
extern void* Geno_ArticleLogic(int kind);
extern int Geno_OnHit(Fighter_GObj* gobj);
extern void GenoGame_ArticleStep(int p, int a, int frame, f32* vx, f32* vy, f32 px, f32 py,
                                 int has_target, f32 tx, f32 ty);

static const char t_v5_json[] =
    "{\"geno\":4,\"fighters\":[{\"attach\":\"kirby\",\"states\":["
    "{\"name\":\"Counter\",\"behavior\":\"geno.ground\",\"subaction\":20,"
    "\"counter\":{\"from\":3,\"to\":10,\"target\":\"geno:CounterHit\"}},"
    "{\"name\":\"CounterHit\",\"behavior\":\"geno.ground\",\"subaction\":21},"
    "{\"name\":\"Flag\",\"behavior\":\"geno.ground\",\"subaction\":22,"
    "\"counter\":{\"negate\":false}}],"
    "\"specials\":{\"lw\":\"geno:Counter\"},"
    "\"hooks\":{\"on_hit\":[\"geno.count_frames:7\"]},"
    "\"articles\":["
    "{\"name\":\"Shot\",\"lifetime\":40,\"velocity\":[2.5,0],\"spawn\":[6,8],"
    "\"hitboxes\":[{\"damage\":7,\"size\":2.5,\"angle\":45,\"kbg\":60,\"bkb\":20,\"element\":1,"
    "\"start\":2,\"end\":30}]},"
    "{\"name\":\"Lob\",\"velocity\":[1,2],\"gravity\":0.1,\"max_fall\":1.5,\"max_speed\":3,"
    "\"homing\":{\"turn\":5,\"delay\":3},\"despawn\":{\"stage\":false},\"max_live\":2,"
    "\"hitboxes\":[{\"damage\":3,\"hits\":{\"reflect\":false}},{\"damage\":4,\"offset\":[1,2,3]}]}"
    "]}]}";

static f32 t_bitsf(int b)
{
    union {
        u32 u;
        f32 f;
    } x;
    x.u = (u32) b;
    return x.f;
}

/* The registry side: articles (defaults, hitboxes, flags), counter windows, the on_hit hook list,
 * the article item kinds and their descriptors. */
static int test_geno_v5_registry(void)
{
    int rc = 0;
    Article* d;
    if (Geno_TestInstall(t_v5_json) != 1) {
        TestFail("could not install the v5 profile");
        Geno_TestRestore();
        return 1;
    }
    if (Geno_ArticleCount(0) != 2 || Geno_ArticleHitCount(0, 0) != 1 || Geno_ArticleHitCount(0, 1) != 2) {
        TestFail("two articles, with 1 and 2 hitboxes");
        rc = 1;
    }
    if (t_bitsf(Geno_ArticleParam(0, 0, GENO_AP_LIFETIME)) != 40.0f ||
        t_bitsf(Geno_ArticleParam(0, 0, GENO_AP_VEL_FWD)) != 2.5f ||
        t_bitsf(Geno_ArticleParam(0, 0, GENO_AP_SPAWN_UP)) != 8.0f ||
        t_bitsf(Geno_ArticleParam(0, 1, GENO_AP_LIFETIME)) != 60.0f ||
        t_bitsf(Geno_ArticleParam(0, 1, GENO_AP_SCALE)) != 1.0f ||
        Geno_ArticleParam(0, 0, GENO_AP_MAX_LIVE) != 4 || Geno_ArticleParam(0, 1, GENO_AP_MAX_LIVE) != 2)
    {
        TestFail("article parameters: lifetime 40 / default 60, velocity, spawn, scale 1, max_live");
        rc = 1;
    }
    if (Geno_ArticleParam(0, 0, GENO_AP_DESPAWN) != (int) GENO_ART_DESPAWN_DEFAULT ||
        Geno_ArticleParam(0, 1, GENO_AP_DESPAWN) != (int) (GENO_ART_DESPAWN_DEFAULT & ~GENO_ART_DESPAWN_STAGE))
    {
        TestFail("despawn mask: default hit|shield|stage|clank; \"stage\": false clears one bit");
        rc = 1;
    }
    if (t_bitsf(Geno_ArticleHitParam(0, 0, 0, GENO_AH_DAMAGE)) != 7.0f ||
        Geno_ArticleHitParam(0, 0, 0, GENO_AH_ANGLE) != 45 || Geno_ArticleHitParam(0, 0, 0, GENO_AH_KBG) != 60 ||
        Geno_ArticleHitParam(0, 0, 0, GENO_AH_BKB) != 20 || Geno_ArticleHitParam(0, 0, 0, GENO_AH_START) != 2 ||
        Geno_ArticleHitParam(0, 0, 0, GENO_AH_END) != 30 || Geno_ArticleHitParam(0, 1, 0, GENO_AH_ANGLE) != 361 ||
        Geno_ArticleHitParam(0, 1, 0, GENO_AH_FLAGS) != (int) (GENO_AHF_DEFAULT & ~GENO_AHF_REFLECT) ||
        t_bitsf(Geno_ArticleHitParam(0, 1, 1, GENO_AH_OFF_Z)) != 3.0f)
    {
        TestFail("article hitboxes: numbers, defaults (angle 361), the hits flags, offsets");
        rc = 1;
    }
    if (Geno_StateCounter(0, 0, GENO_CTR_FROM) != 3 || Geno_StateCounter(0, 0, GENO_CTR_TO) != 10 ||
        Geno_StateCounter(0, 0, GENO_CTR_TARGET) != (int) GENO_TARGET(GENO_TGT_GENO, 1) ||
        Geno_StateCounter(0, 0, GENO_CTR_NEGATE) != 1 || Geno_StateCounter(0, 1, GENO_CTR_FROM) != -1 ||
        Geno_StateCounter(0, 2, GENO_CTR_NEGATE) != 0 || Geno_StateCounter(0, 2, GENO_CTR_TARGET) != -1)
    {
        TestFail("counter windows: frames 3-10 -> geno:CounterHit, negate default 1; none on CounterHit");
        rc = 1;
    }
    if (Geno_HookCount(0, GENO_EV_HIT) != 1) {
        TestFail("hooks.on_hit must bind one hook");
        rc = 1;
    }
    d = Geno_ArticleDesc(GENO_ART_KIND_BASE + 1);
    if (d == NULL || d->x10_modelDesc == NULL || d->x10_modelDesc->x0_joint != NULL ||
        d->x0_common_attr == NULL || d->x0_common_attr->x60_scale != 1.0f || d->x8_hurtbones != NULL ||
        Geno_ArticleLogic(GENO_ART_KIND_BASE + 1) == NULL)
    {
        TestFail("article kind 0x1001: a descriptor (no model: invisible, scale 1, no hurtbox)");
        rc = 1;
    }
    if (Geno_ArticleDesc(GENO_ART_KIND_BASE + 2) != NULL || Geno_ArticleDesc(GENO_ART_KIND_BASE + 8) != NULL ||
        Geno_ArticleDesc(237) != NULL)
    {
        TestFail("an undeclared article kind (or an m-ex kind) must have no Geno descriptor");
        rc = 1;
    }
    Geno_TestRestore();
    return rc;
}

/* Reflection scales the travel state as well as the next velocity written to the item. */
static int test_geno_v5_article_reflection(void)
{
    static Item item;
    static HSD_GObj gobj;
    ItemLogicTable* logic;
    GenoArtVars* vars;
    int rc = 0;
    if (Geno_TestInstall(t_v5_json) != 1) {
        TestFail("could not install the v5 profile");
        return 1;
    }
    t_zero(&item, sizeof(item));
    t_zero(&gobj, sizeof(gobj));
    gobj.user_data = &item;
    vars = (GenoArtVars*) &item.xDD4_itemVar;
    vars->vx = 2.0f;
    vars->vy = -4.0f;
    vars->frame = 1;
    /* Same facing avoids a visual JObj update in this headless item. */
    item.facing_dir = item.xC68 = -1.0f;
    item.xC70 = 1.5f;
    logic = Geno_ArticleLogic(GENO_ART_KIND_BASE);
    if (logic == NULL || logic->reflected(&gobj)) {
        TestFail("a reflected Geno article must survive");
        rc = 1;
    } else {
        logic->states[0].physics_updated(&gobj);
        if (vars->vx != -3.0f || vars->vy != -6.0f ||
            item.x40_vel.x != -3.0f || item.x40_vel.y != -6.0f) {
            TestFail("reflector speed multiplier must survive the following article physics step");
            rc = 1;
        }
    }
    Geno_TestRestore();
    return rc;
}

/* The travel: a straight shot keeps its velocity; gravity to max_fall; homing turns at most
 * `turn` degrees a frame toward the target, only after `delay`; max_speed caps the speed. */
static int test_geno_v5_article_motion(void)
{
    int rc = 0, f;
    f32 vx = 2.5f, vy = 0.0f, px = 0.0f;
    if (Geno_TestInstall(t_v5_json) != 1) {
        TestFail("could not install the v5 profile");
        Geno_TestRestore();
        return 1;
    }
    for (f = 1; f <= 40; f++) {
        GenoGame_ArticleStep(0, 0, f, &vx, &vy, px, 0.0f, 1, 0.0f, 50.0f);
        px += vx;
    }
    if (vx != 2.5f || vy != 0.0f || !t_near(px, 100.0f)) {
        TestFail("a straight article must fly 2.5 a frame for 40 frames (100 units), no homing");
        rc = 1;
    }
    /* Lob: velocity (1, 2), gravity 0.1, max fall 1.5, homing 5 deg after frame 3, target far below */
    vx = 1.0f;
    vy = 2.0f;
    GenoGame_ArticleStep(0, 1, 1, &vx, &vy, 0.0f, 0.0f, 1, 100.0f, -100.0f);
    if (!t_near(vx, 1.0f) || !t_near(vy, 1.9f)) {
        TestFail("before the homing delay only gravity applies (vy 2 -> 1.9)");
        rc = 1;
    }
    {
        f32 a0 = atan2f(vy, vx), a1;
        GenoGame_ArticleStep(0, 1, 4, &vx, &vy, 0.0f, 0.0f, 1, 100.0f, -100.0f);
        vy += 0.1f; /* undo this frame's gravity to measure the turn alone */
        a1 = atan2f(vy, vx);
        if (!t_near((a0 - a1) * 57.29578f, 5.0f)) {
            TestFail("homing must turn exactly 5 degrees toward the target");
            rc = 1;
        }
    }
    for (f = 5; f < 200; f++) {
        GenoGame_ArticleStep(0, 1, f, &vx, &vy, 0.0f, 1000.0f, 0, 0.0f, 0.0f);
    }
    if (!t_near(vy, -1.5f) || sqrtf(vx * vx + vy * vy) > 3.0001f) {
        TestFail("gravity must stop at max_fall 1.5 and the speed stay under max_speed 3");
        rc = 1;
    }
    Geno_TestRestore();
    return rc;
}

/* on_hit: the hit's context (HIT_* values), the on_hit hooks, the counter window (frames 3-10:
 * negated, -> CounterHit), outside the window the hit stands, a flag-only window, the spawn hook
 * from a script CALL, and no profile = nothing. */
static int test_geno_v5_on_hit(void)
{
    TestGenoState* st;
    int rc = 0;
    t_rows[ftCo_MS_Fall].anim_id = 29;
    if (Geno_TestInstall(t_v5_json) != 1 || (st = t_setup()) == NULL) {
        TestFail("could not install the v5 profile");
        Geno_TestRestore();
        return 1;
    }
    GenoGame_TestCapture(1, 0);
    t_fp.ground_or_air = GA_Ground;
    t_fp.motion_id = ftCo_MS_Wait;
    if (Geno_SpecialEnter(&t_gobj, GENO_SP_LW) != 1 || t_fp.motion_id != T_MS(0)) {
        TestFail("specials.lw -> Counter");
        rc = 1;
    }
    /* frame 2: before the window - the hit stands, on_hit runs */
    st->action_time = 2;
    t_fp.dmg.kb_applied = 40.0f;
    t_fp.dmg.x1838_percentTemp = 12.0f;
    t_fp.dmg.x18c4_source_ply = 1;
    if (Geno_OnHit(&t_gobj) != 0 || t_fp.dmg.kb_applied != 40.0f || t_fp.dmg.x1838_percentTemp != 12.0f ||
        t_fp.motion_id != T_MS(0) || st->hit_count != 1 || st->la_i[7] != 1 || st->hit_counter != 0 ||
        st->hit_port != 1 || st->hit_damage != 12.0f)
    {
        TestFail("a hit before the window: taken, on_hit hook ran, HIT_* set, still in Counter");
        rc = 1;
    }
    /* frame 5: in the window - negated, -> CounterHit, hook ran again */
    st->action_time = 5;
    t_fp.dmg.kb_applied = 40.0f;
    t_fp.dmg.x1838_percentTemp = 9.0f;
    if (Geno_OnHit(&t_gobj) != 1 || t_fp.dmg.kb_applied != 0.0f || t_fp.dmg.x1838_percentTemp != 0.0f ||
        t_fp.motion_id != T_MS(1) || st->hit_count != 2 || st->la_i[7] != 2 || st->hit_counter != 1 ||
        st->counters != 1 || st->hit_damage != 9.0f)
    {
        TestFail("a hit in the window: countered (no damage, no knockback), -> CounterHit, HIT_COUNTER 1");
        rc = 1;
    }
    /* CounterHit has no window: the next hit stands */
    st->action_time = 5;
    t_fp.dmg.kb_applied = 40.0f;
    t_fp.dmg.x1838_percentTemp = 9.0f;
    if (Geno_OnHit(&t_gobj) != 0 || t_fp.dmg.x1838_percentTemp != 9.0f || st->hit_counter != 0) {
        TestFail("a hit in a state without a window stands");
        rc = 1;
    }
    /* no hit this frame: nothing */
    t_fp.dmg.kb_applied = 0.0f;
    t_fp.dmg.x1838_percentTemp = 0.0f;
    if (Geno_OnHit(&t_gobj) != 0 || st->hit_count != 3) {
        TestFail("no hit, no on_hit");
        rc = 1;
    }
    /* a flag-only window (negate false, no target): flagged, the hit stands, the state stays */
    t_fp.motion_id = T_MS(2);
    st->action_time = 1;
    t_fp.dmg.kb_applied = 30.0f;
    t_fp.dmg.x1838_percentTemp = 5.0f;
    if (Geno_OnHit(&t_gobj) != 1 || t_fp.dmg.x1838_percentTemp != 5.0f || t_fp.motion_id != T_MS(2) ||
        st->hit_counter != 1)
    {
        TestFail("\"negate\": false: the hit is flagged (HIT_COUNTER 1) and still taken");
        rc = 1;
    }
    /* the HIT_* values through GET */
    t_script[0] = GENO_W0_VAR(GENO_SUB_GET, 2, LA(3), 0, 0);
    t_script[1] = GENO_VAL_HIT_COUNT;
    t_script[2] = GENO_W0_VAR(GENO_SUB_GET, 2, LA(4), 0, 0);
    t_script[3] = GENO_VAL_HIT_COUNTER;
    /* the spawn hook from a script: CALL geno.article.spawn 0 (counted: the test has no item heap) */
    t_script[4] = GENO_W0(GENO_SUB_CALL, 3, 0);
    t_script[5] = GENO_HOOK_ARTICLE_SPAWN;
    t_script[6] = 0;
    t_script[7] = 0;
    t_run(t_script, GENO_MODE_EXEC);
    if (st->la_i[3] != 4 || st->la_i[4] != 1 || st->art_spawned != 1) {
        TestFail("GET HIT_COUNT / HIT_COUNTER, and CALL geno.article.spawn from a script");
        rc = 1;
    }
    GenoGame_TestCapture(0, 0);
    Geno_TestRestore();
    t_setup(); /* no profile */
    t_fp.dmg.kb_applied = 40.0f;
    t_fp.dmg.x1838_percentTemp = 12.0f;
    if (Geno_OnHit(&t_gobj) != 0 || t_fp.dmg.x1838_percentTemp != 12.0f ||
        ((TestGenoState*) GenoGame_StateOf(&t_fp))->hit_count != 0)
    {
        TestFail("a fighter with no profile: on_hit is inert");
        rc = 1;
    }
    t_fp.dmg.kb_applied = 0.0f;
    t_fp.dmg.x1838_percentTemp = 0.0f;
    return rc;
}

/* ---- v5.1: slots, spawn variants, children, brakes, specials select ------------------------- */

extern int Geno_SpecialSelect(int p, int which, int i);

static const char t_v51_json[] =
    "{\"geno\":4,\"fighters\":[{\"attach\":\"kirby\",\"states\":["
    "{\"name\":\"Fire\",\"behavior\":\"geno.ground\",\"subaction\":20},"
    "{\"name\":\"Ice\",\"behavior\":\"geno.ground\",\"subaction\":21},"
    "{\"name\":\"Thunder\",\"behavior\":\"geno.ground\",\"subaction\":22}],"
    "\"specials\":{\"n\":{\"select\":\"la_i:0\",\"targets\":[\"geno:Fire\",\"geno:Ice\",\"geno:Thunder\"]}},"
    "\"articles\":["
    "{\"name\":\"Ice\",\"velocity\":[3.6,0],\"accel\":-0.18,\"min_speed\":0.2,\"angle\":4,"
    "\"spawns\":[[7.6,6.8],[10,2]],"
    "\"hitboxes\":[{\"damage\":2.4,\"start\":1,\"end\":3},{\"damage\":2.4,\"slot\":0,\"start\":4,\"end\":6}]},"
    "{\"name\":\"Cloud\",\"lifetime\":30,\"bone\":2,\"effect\":1147,"
    "\"children\":[{\"article\":\"Bolt\",\"frame\":3,\"every\":3,\"count\":4,\"spawn\":1}]},"
    "{\"name\":\"Bolt\",\"velocity\":[0,-3.8],\"show_model\":true}"
    "]}]}";

static int test_geno_v51(void)
{
    TestGenoState* st;
    int rc = 0, f;
    f32 vx = 3.6f, vy = 0.0f;
    if (Geno_TestInstall(t_v51_json) != 1 || (st = t_setup()) == NULL) {
        TestFail("could not install the v5.1 profile");
        Geno_TestRestore();
        return 1;
    }
    if (Geno_ArticleHitParam(0, 0, 0, GENO_AH_SLOT) != 0 || Geno_ArticleHitParam(0, 0, 1, GENO_AH_SLOT) != 0 ||
        Geno_ArticleParam(0, 0, GENO_AP_SPAWN_N) != 2 || t_bitsf(Geno_ArticleParam(0, 0, GENO_AP_SPAWN_V + 3)) != 2.0f ||
        t_bitsf(Geno_ArticleParam(0, 0, GENO_AP_ANGLE)) != 4.0f || Geno_ArticleParam(0, 0, GENO_AP_BONE) != -1 ||
        Geno_ArticleParam(0, 1, GENO_AP_BONE) != 2 || Geno_ArticleParam(0, 1, GENO_AP_EFFECT) != 1147)
    {
        TestFail("v5.1 article keys: slots (both entries in slot 0), spawns, angle, bone, effect");
        rc = 1;
    }
    if (Geno_ArticleParam(0, 2, GENO_AP_SHOW_MODEL) != 1 || Geno_ArticleParam(0, 0, GENO_AP_SHOW_MODEL) != 0 ||
        Geno_ArticleParam(0, 1, GENO_AP_SHOW_MODEL) != 0)
    {
        TestFail("v5.7 show_model: on for Bolt only, default off");
        rc = 1;
    }
    if (Geno_ArticleParam(0, 1, GENO_AP_CHILD) != 2 || Geno_ArticleParam(0, 1, GENO_AP_CHILD + 1) != 3 ||
        Geno_ArticleParam(0, 1, GENO_AP_CHILD + 2) != 3 || Geno_ArticleParam(0, 1, GENO_AP_CHILD + 3) != 4 ||
        Geno_ArticleParam(0, 1, GENO_AP_CHILD + 4) != 1 || Geno_ArticleParam(0, 1, GENO_AP_CHILD + 5) != -1 ||
        Geno_ArticleParam(0, 0, GENO_AP_CHILD) != -1)
    {
        TestFail("children: Cloud -> Bolt (by name) at 3, every 3, 4 times, variant 1; none elsewhere");
        rc = 1;
    }
    /* the brake: 3.6 - 0.18 a frame, stopping at 0.2 */
    for (f = 1; f <= 5; f++) {
        GenoGame_ArticleStep(0, 0, f, &vx, &vy, 0.0f, 0.0f, 0, 0.0f, 0.0f);
    }
    if (!t_near(vx, 2.7f)) {
        TestFail("brake: 5 frames of -0.18 from 3.6 = 2.7");
        rc = 1;
    }
    for (f = 6; f <= 60; f++) {
        GenoGame_ArticleStep(0, 0, f, &vx, &vy, 0.0f, 0.0f, 0, 0.0f, 0.0f);
    }
    if (!t_near(vx, 0.2f)) {
        TestFail("brake must stop at min_speed 0.2");
        rc = 1;
    }
    /* specials select: LA0 picks Fire / Ice / Thunder; out of range = the fighter's own */
    if (Geno_SpecialSelect(0, GENO_SP_N, -1) != GENO_VAR(GENO_BANK_LA_INT, 0) ||
        Geno_SpecialSelect(0, GENO_SP_AIR_N, -2) != 3)
    {
        TestFail("select: la_i:0, three targets, air_n inherits it");
        rc = 1;
    }
    GenoGame_TestCapture(1, 0);
    t_fp.ground_or_air = GA_Ground;
    for (f = 0; f < 3; f++) {
        st->la_i[0] = f;
        t_fp.motion_id = ftCo_MS_Wait;
        if (Geno_SpecialEnter(&t_gobj, GENO_SP_N) != 1 || t_fp.motion_id != T_MS(f)) {
            TestFail("select: LA0 = n must enter state n");
            rc = 1;
        }
    }
    st->la_i[0] = 3;
    t_fp.motion_id = ftCo_MS_Wait;
    if (Geno_SpecialEnter(&t_gobj, GENO_SP_N) != 0) {
        TestFail("select: LA0 out of range must leave the special to the fighter");
        rc = 1;
    }
    GenoGame_TestCapture(0, 0);
    Geno_TestRestore();
    return rc;
}

/* v5.4: the directed dash's pure cores (Sonic Blade as Ultimate's status code aims it) */
extern int GenoGame_DashStick(f32 sx, f32 sy, f32 thr, f32* cx, f32* cy);
extern void GenoGame_DashAim(int has_target, f32 dx, f32 dy, int has_stick, f32 cx, f32 cy, f32 facing,
                             f32 vert_deg, f32 up_min, f32 up_max, f32* ux, f32* uy, f32* face, int* kind);
extern void GenoGame_Brake(f32* vx, f32* vy, f32 brake, f32 max);
extern int GenoGame_HookFind(const char* name);

static int test_geno_v54_dash(void)
{
    f32 cx = 9.0f, cy = 9.0f, ux, uy, face, vx, vy;
    int kind, rc = 0;
    /* the stick: a vector-length dead zone, then the raw polar angle, the whole circle */
    if (GenoGame_DashStick(0.15f, 0.15f, 0.25f, &cx, &cy) != 0 || cx != 9.0f) {
        TestFail("dash stick: inside the dead zone writes nothing");
        rc = 1;
    }
    if (!GenoGame_DashStick(0.0f, -1.0f, 0.25f, &cx, &cy) || !t_near(cx, 0.0f) || !t_near(cy, -1.0f)) {
        TestFail("dash stick: straight down is straight down");
        rc = 1;
    }
    if (!GenoGame_DashStick(-0.5f, 0.5f, 0.25f, &cx, &cy) || !t_near(cx, -0.70710678f) || !t_near(cy, 0.70710678f)) {
        TestFail("dash stick: up-back is 135 degrees, unclamped");
        rc = 1;
    }
    /* stick straight up, facing right: heading up, facing kept (within 20 of vertical), up clip */
    GenoGame_DashAim(0, 0.0f, 0.0f, 1, 0.0f, 1.0f, 1.0f, 20.0f, 40.0f, 140.0f, &ux, &uy, &face, &kind);
    if (!t_near(ux, 0.0f) || !t_near(uy, 1.0f) || face != 1.0f || kind != 1) {
        TestFail("dash aim: straight up keeps the facing");
        rc = 1;
    }
    /* stick back and down (225 deg): turns, down clip, on the ground too (no ground rule) */
    GenoGame_DashAim(0, 0.0f, 0.0f, 1, -0.70710678f, -0.70710678f, 1.0f, 20.0f, 40.0f, 140.0f, &ux, &uy, &face, &kind);
    if (!t_near(ux, -0.70710678f) || !t_near(uy, -0.70710678f) || face != -1.0f || kind != 2) {
        TestFail("dash aim: 225 degrees turns and aims down");
        rc = 1;
    }
    /* a target wins over the stick; the heading is exactly at it, any angle */
    GenoGame_DashAim(1, -30.0f, -40.0f, 1, 1.0f, 0.0f, 1.0f, 20.0f, 40.0f, 140.0f, &ux, &uy, &face, &kind);
    if (!t_near(ux, -0.6f) || !t_near(uy, -0.8f) || face != -1.0f || kind != 2) {
        TestFail("dash aim: the target overrides the stick");
        rc = 1;
    }
    /* a target exactly on top, or nothing at all: straight ahead along the facing */
    GenoGame_DashAim(1, 0.0f, 0.0f, 1, 0.0f, 1.0f, -1.0f, 20.0f, 40.0f, 140.0f, &ux, &uy, &face, &kind);
    if (!t_near(ux, -1.0f) || !t_near(uy, 0.0f) || face != -1.0f || kind != 0) {
        TestFail("dash aim: a zero offset dashes along the facing");
        rc = 1;
    }
    GenoGame_DashAim(0, 0.0f, 0.0f, 0, 0.0f, 1.0f, 1.0f, 20.0f, 40.0f, 140.0f, &ux, &uy, &face, &kind);
    if (!t_near(ux, 1.0f) || !t_near(uy, 0.0f) || kind != 0) {
        TestFail("dash aim: no target and no stick is straight ahead");
        rc = 1;
    }
    /* the brake: cap 2.0 first, then 0.34 along the vector; never past rest */
    vx = 3.0f;
    vy = 4.0f;
    GenoGame_Brake(&vx, &vy, 0.34f, 2.0f);
    if (!t_near(vx, 0.996f) || !t_near(vy, 1.328f)) {
        TestFail("brake: capped to 2.0 then 0.34 off along the vector");
        rc = 1;
    }
    vx = -0.1f;
    vy = 0.0f;
    GenoGame_Brake(&vx, &vy, 0.24f, 0.0f);
    if (vx != 0.0f || vy != 0.0f) {
        TestFail("brake: stops at rest, never reverses");
        rc = 1;
    }
    if (GenoGame_HookFind("geno.dash.search") != 8 || GenoGame_HookFind("geno.dash.aim") != 9 ||
        GenoGame_HookFind("geno.brake") != 10)
    {
        TestFail("dash hooks: stable ids 8, 9, 10");
        rc = 1;
    }
    return rc;
}

/* v5.2: geno.lockon's aim (Sonic Blade) and the "air_drift" phys name / hook 6 by name */
extern void GenoGame_LockonAim(int has, f32 dx, f32 dy, f32 sx, f32 sy, f32 facing, int grounded,
                               f32 max_deg, f32 stick_deg, f32* fwd, f32* up, f32* face);
extern int GenoGame_HookFind(const char* name);

static int test_geno_v52_lockon(void)
{
    f32 fw, up, face;
    int rc = 0;
    /* a target 30 ahead, 30 up, facing right: 45 deg, clamped to 30 */
    GenoGame_LockonAim(1, 30.0f, 30.0f, 0.0f, 0.0f, 1.0f, 0, 30.0f, 20.0f, &fw, &up, &face);
    if (!t_near(fw, 0.8660254f) || !t_near(up, 0.5f) || face != 1.0f) {
        TestFail("lockon: 45 deg up clamps to 30");
        rc = 1;
    }
    /* a target behind, level: turn, aim straight */
    GenoGame_LockonAim(1, -20.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0, 30.0f, 20.0f, &fw, &up, &face);
    if (!t_near(fw, 1.0f) || !t_near(up, 0.0f) || face != -1.0f) {
        TestFail("lockon: a target behind turns the fighter");
        rc = 1;
    }
    /* no target: the stick (down in the air = -20; down on the ground = level; neutral = level) */
    GenoGame_LockonAim(0, 0.0f, 0.0f, 0.0f, -0.8f, -1.0f, 0, 30.0f, 20.0f, &fw, &up, &face);
    if (!t_near(up, -0.34202f) || face != -1.0f) {
        TestFail("lockon: no target, stick down in the air = -20 deg, facing kept");
        rc = 1;
    }
    GenoGame_LockonAim(0, 0.0f, 0.0f, 0.0f, -0.8f, 1.0f, 1, 30.0f, 20.0f, &fw, &up, &face);
    if (!t_near(up, 0.0f) || !t_near(fw, 1.0f)) {
        TestFail("lockon: never aims down on the ground");
        rc = 1;
    }
    GenoGame_LockonAim(0, 0.0f, 0.0f, 0.0f, 0.1f, 1.0f, 0, 30.0f, 20.0f, &fw, &up, &face);
    if (!t_near(up, 0.0f)) {
        TestFail("lockon: stick inside 0.25 = level");
        rc = 1;
    }
    /* ANIM_RATE is writable (v5.2): 0.5 taken, 0 refused */
    {
        u32* s = t_script;
        int n = 0;
        t_setup();
        t_fp.frame_speed_mul = 1.0f;
        s[n++] = GENO_W0(GENO_SUB_PUT, 3, 0);
        s[n++] = GENO_VAL_ANIM_RATE;
        s[n++] = t_fbits(0.5f);
        s[n++] = GENO_W0(GENO_SUB_PUT, 3, 0);
        s[n++] = GENO_VAL_ANIM_RATE;
        s[n++] = t_fbits(0.0f);
        s[n++] = 0;
        t_run(t_script, GENO_MODE_EXEC);
        if (t_fp.frame_speed_mul != 0.5f) {
            TestFail("PUT ANIM_RATE 0.5 sets frame_speed_mul; 0 is refused");
            rc = 1;
        }
        t_fp.frame_speed_mul = 1.0f;
    }
    /* HBDMG (v5.3): hitboxes 0 and 3 = LAF5 (13.5), hitbox 1 = 2.0 immediate, 2 untouched */
    {
        u32* s = t_script;
        int n = 0;
        TestGenoState* st = t_setup();
        t_fp.x914[0].damage = t_fp.x914[1].damage = t_fp.x914[2].damage = t_fp.x914[3].damage = 9.0f;
        st->la_f[5] = 13.5f;
        s[n++] = GENO_W0(GENO_SUB_HBDMG, 2, (0x09 << 8) | 0x80);
        s[n++] = LAF_(5);
        s[n++] = GENO_W0(GENO_SUB_HBDMG, 2, 0x02 << 8);
        s[n++] = t_fbits(2.0f);
        s[n++] = 0;
        t_run(t_script, GENO_MODE_EXEC);
        if (t_fp.x914[0].damage != 13.5f || t_fp.x914[3].damage != 13.5f || t_fp.x914[1].damage != 2.0f ||
            t_fp.x914[2].damage != 9.0f)
        {
            TestFail("HBDMG: mask 9 = LAF5, mask 2 = 2.0, hitbox 2 kept");
            rc = 1;
        }
    }
    /* HBSTUN (v5.5): hitboxes 0 and 2 = +12 hitstun frames (immediate), hitbox 1 = LAI7 (5); a hit by hitbox 0
       gives the victim 12 for that frame only, taken once; hitbox 3 gives nothing; an action change clears it */
    {
        extern void Geno_HitStunBonus(Fighter * atk, int idx, Fighter * vic);
        extern int Geno_TakeStunBonus(Fighter * vic);
        u32* s = t_script;
        int n = 0, b0, b0again, b1, b3, bmax, bcleared;
        TestGenoState* st = t_setup();
        st->la_i[7] = 5;
        s[n++] = GENO_W0(GENO_SUB_HBSTUN, 2, 0x05 << 8);
        s[n++] = 12;
        s[n++] = GENO_W0(GENO_SUB_HBSTUN, 2, (0x02 << 8) | 0x80);
        s[n++] = GENO_VAR(GENO_BANK_LA_INT, 7);
        s[n++] = 0;
        t_run(t_script, GENO_MODE_EXEC);
        if (st->stun_add[0] != 12) { TestFail("HBSTUN: stun_add[0] != 12"); rc = 1; }
        if (st->stun_add[1] != 5) { TestFail("HBSTUN: stun_add[1] != 5"); rc = 1; }
        if (st->stun_add[2] != 12) { TestFail("HBSTUN: stun_add[2] != 12"); rc = 1; }
        if (st->stun_add[3] != 0) { TestFail("HBSTUN: stun_add[3] != 0"); rc = 1; }
        Geno_HitStunBonus(&t_fp, 0, &t_fp);
        b0 = Geno_TakeStunBonus(&t_fp);
        b0again = Geno_TakeStunBonus(&t_fp);
        Geno_HitStunBonus(&t_fp, 3, &t_fp);
        b3 = Geno_TakeStunBonus(&t_fp);
        Geno_HitStunBonus(&t_fp, 1, &t_fp);
        Geno_HitStunBonus(&t_fp, 2, &t_fp); /* two hits in one frame: the largest */
        bmax = Geno_TakeStunBonus(&t_fp);
        Geno_HitStunBonus(&t_fp, 1, &t_fp);
        b1 = Geno_TakeStunBonus(&t_fp);
        Geno_OnActionChange(&t_gobj);
        Geno_HitStunBonus(&t_fp, 0, &t_fp);
        bcleared = Geno_TakeStunBonus(&t_fp);
        if (b0 != 12) { TestFail("HBSTUN: hitbox 0 gives +12"); rc = 1; }
        if (b0again != 0) { TestFail("HBSTUN: the bonus is taken once"); rc = 1; }
        if (b3 != 0) { TestFail("HBSTUN: hitbox 3 gives nothing"); rc = 1; }
        if (bmax != 12) { TestFail("HBSTUN: two hits in a frame give the largest"); rc = 1; }
        if (b1 != 5) { TestFail("HBSTUN: hitbox 1 from LAI7 gives 5"); rc = 1; }
        if (bcleared != 0) { TestFail("HBSTUN: an action change clears the hitbox bonuses"); rc = 1; }
    }
    /* HBFLAGS (v5.5): hitbox 0 = NO_HITLAG|FLINCHLESS|ZERO_DAMAGE (7), hitbox 1 = FORCE_REACTION (8); a hit by
       hitbox 0 drops the knockback and the hitlag (this frame only), hitbox 1 lifts no_kb; an action change clears */
    {
        extern int Geno_HitFlags(Fighter * atk, int idx, Fighter * vic);
        extern int Geno_HitReact(Fighter * fp);
        extern int Geno_NoHitlag(Fighter * fp);
        u32* s = t_script;
        int n = 0, f0, lifted;
        t_setup();
        s[n++] = GENO_W0(GENO_SUB_HBFLAGS, 2, 0x01 << 8);
        s[n++] = 7;
        s[n++] = GENO_W0(GENO_SUB_HBFLAGS, 2, 0x02 << 8);
        s[n++] = 8;
        s[n++] = 0;
        t_run(t_script, GENO_MODE_EXEC);
        f0 = Geno_HitFlags(&t_fp, 0, &t_fp);
        t_fp.dmg.kb_applied = 50.0f;
        Geno_HitReact(&t_fp);
        if (f0 != 7) { TestFail("HBFLAGS: hitbox 0 carries 7"); rc = 1; }
        if (t_fp.dmg.kb_applied != 0.0f) { TestFail("HBFLAGS: FLINCHLESS drops the knockback"); rc = 1; }
        if (!Geno_NoHitlag(&t_fp)) { TestFail("HBFLAGS: NO_HITLAG reported"); rc = 1; }
        t_setup();
        s[0] = GENO_W0(GENO_SUB_HBFLAGS, 2, 0x02 << 8);
        s[1] = 8;
        s[2] = 0;
        t_run(t_script, GENO_MODE_EXEC);
        Geno_HitFlags(&t_fp, 1, &t_fp);
        t_fp.no_kb = 1;
        t_fp.dmg.kb_applied = 50.0f;
        lifted = Geno_HitReact(&t_fp);
        if (lifted != 1 || t_fp.no_kb != 0 || t_fp.dmg.kb_applied != 50.0f) {
            TestFail("HBFLAGS: FORCE_REACTION lifts no_kb, keeps the knockback");
            rc = 1;
        }
        t_fp.no_kb = 0;
        if (Geno_HitFlags(&t_fp, 3, &t_fp) != 0) { TestFail("HBFLAGS: hitbox 3 has none"); rc = 1; }
        Geno_OnActionChange(&t_gobj);
        if (Geno_HitFlags(&t_fp, 1, &t_fp) != 0) { TestFail("HBFLAGS: an action change clears them"); rc = 1; }
    }
    /* ATTACK_CONNECTED (v5.3): set by a won hit (Geno_Autolink's site), moved to _PREV and cleared
       at the action change, writable */
    {
        u32* s = t_script;
        int n = 0;
        f32 dir = 1.0f, ang = 45.0f, kb = 50.0f;
        TestGenoState* st = t_setup();
        s[0] = GENO_W0(0x00, 1, 0); /* a NOP escape: the block is live (scripted) */
        s[1] = 0;
        t_run(t_script, GENO_MODE_EXEC);
        Geno_OnActionChange(&t_gobj);
        Geno_Autolink(&t_fp, &t_fp.x914[0], &dir, &ang, &kb);
        s[n++] = GENO_W0_VAR(GENO_SUB_GET, 2, LA(20), 0, 0);
        s[n++] = GENO_VAL_ATTACK_CONNECTED;
        s[n++] = 0;
        t_run(t_script, GENO_MODE_EXEC);
        if (st->la_i[20] != 1) {
            TestFail("ATTACK_CONNECTED is 1 after a won hit");
            rc = 1;
        }
        Geno_OnActionChange(&t_gobj);
        n = 0;
        s[n++] = GENO_W0_VAR(GENO_SUB_GET, 2, LA(21), 0, 0);
        s[n++] = GENO_VAL_ATTACK_CONNECTED;
        s[n++] = GENO_W0_VAR(GENO_SUB_GET, 2, LA(22), 0, 0);
        s[n++] = GENO_VAL_ATTACK_CONNECTED_PREV;
        s[n++] = GENO_W0(GENO_SUB_PUT, 3, 0);
        s[n++] = GENO_VAL_ATTACK_CONNECTED;
        s[n++] = 1;
        s[n++] = GENO_W0_VAR(GENO_SUB_GET, 2, LA(23), 0, 0);
        s[n++] = GENO_VAL_ATTACK_CONNECTED;
        s[n++] = 0;
        t_run(t_script, GENO_MODE_EXEC);
        if (st->la_i[21] != 0 || st->la_i[22] != 1 || st->la_i[23] != 1) {
            TestFail("action change: ATTACK_CONNECTED 0, _PREV 1; PUT 1 writes it");
            rc = 1;
        }
    }
    /* aim_stick (v5.3): polar stick heading, threshold, clamps, turn, ground */
    {
        extern int GenoGame_AimStick(f32 sx, f32 sy, f32 facing, int grounded, f32 thr, f32 max_up,
                                     f32 max_down, int allow_turn, f32* fwd, f32* up, f32* face);
        f32 fw = 9.0f, up = 9.0f, fc = 1.0f;
        if (GenoGame_AimStick(0.1f, 0.1f, 1.0f, 0, 0.25f, 60.0f, 60.0f, 1, &fw, &up, &fc) != 0 || fw != 9.0f) {
            TestFail("aim_stick: below the threshold writes nothing");
            rc = 1;
        }
        GenoGame_AimStick(0.5f, 0.5f, 1.0f, 0, 0.25f, 60.0f, 60.0f, 1, &fw, &up, &fc);
        if (!t_near(fw, 0.7071068f) || !t_near(up, 0.7071068f) || fc != 1.0f) {
            TestFail("aim_stick: up-forward diagonal = 45 deg");
            rc = 1;
        }
        GenoGame_AimStick(0.0f, 1.0f, 1.0f, 0, 0.25f, 60.0f, 60.0f, 1, &fw, &up, &fc);
        if (!t_near(up, 0.8660254f)) {
            TestFail("aim_stick: straight up clamps to max_up 60");
            rc = 1;
        }
        GenoGame_AimStick(-0.8f, -0.3f, 1.0f, 0, 0.25f, 60.0f, 60.0f, 1, &fw, &up, &fc);
        if (fc != -1.0f || !t_near(up, sinf(atan2f(-0.3f, 0.8f)))) {
            TestFail("aim_stick: back-down turns and aims down");
            rc = 1;
        }
        GenoGame_AimStick(0.8f, -0.8f, 1.0f, 1, 0.25f, 60.0f, 60.0f, 1, &fw, &up, &fc);
        if (!t_near(up, 0.0f)) {
            TestFail("aim_stick: never down on the ground");
            rc = 1;
        }
        if (GenoGame_HookFind("geno.aim_stick") != GENO_HOOK_AIM_STICK || GENO_HOOK_AIM_STICK != 7) {
            TestFail("hook 7 is geno.aim_stick");
            rc = 1;
        }
    }
    if (GenoGame_HookFind("geno.lockon") != GENO_HOOK_LOCKON || GENO_HOOK_LOCKON != 6) {
        TestFail("hook 6 is geno.lockon");
        rc = 1;
    }
    return rc;
}

/* ---- debug movement (geno_lab_mode.c): the fly callbacks on a test fighter ------------------ */
extern void GenoFly_TestArm(Fighter* fp);
extern int GenoFly_Fighter(Fighter* fp);
extern void GenoFly_SetSpeed(int speed_bits);
extern float GenoFly_Speed(void);
extern void GenoFly_SetSolid(int solid);
extern bool ftCo_800D3158(Fighter_GObj* gobj);

static int test_geno_fly(void)
{
    extern int GenoFly_CursorTest(void);
    if (GenoFly_CursorTest()) {
        TestFail("debug attack cursor: native rearm, victim caches, convergence or hitlag failed");
        return 1;
    }
    float old = GenoFly_Speed();
    int rc = 0;
    t_setup();
    if (GenoFly_Fighter(&t_fp)) {
        TestFail("an unarmed fighter reads as flying");
        return 1;
    }
    GenoFly_TestArm(&t_fp);
    GenoFly_SetSpeed((int) t_fbits(2.0f));
    GenoFly_SetSolid(0);
    t_fp.input.lstick[0].x = 0.5f;
    t_fp.input.lstick[0].y = -1.0f;
    t_fp.x8c_kb_vel.x = 7.0f;
    t_fp.facing_dir = -1.0f;
    t_fp.dmg.x1910 = 30;
    t_fp.phys_cb(&t_gobj);
    if (!GenoFly_Fighter(&t_fp) || t_fp.self_vel.x != 1.0f || t_fp.self_vel.y != -2.0f ||
        t_fp.x8c_kb_vel.x != 0.0f || t_fp.facing_dir != 1.0f || t_fp.x1988 != 2 || t_fp.dmg.x1910 != 0)
    {
        TestFail("fly phys: velocity = stick x speed, knockback cleared, facing the stick, intangible");
        rc = 1;
    }
    t_fp.input.held_buttons[0] = HSD_PAD_A;
    t_fp.phys_cb(&t_gobj);
    if (t_fp.self_vel.x != 0.25f || t_fp.self_vel.y != -0.5f) {
        TestFail("fly phys: A held is a quarter speed");
        rc = 1;
    }
    t_fp.input.held_buttons[0] = HSD_PAD_B;
    t_fp.phys_cb(&t_gobj);
    if (t_fp.self_vel.x != 4.0f || t_fp.self_vel.y != -8.0f) {
        TestFail("fly phys: B held is 4x speed");
        rc = 1;
    }
    t_fp.input.held_buttons[0] = 0;
    t_fp.input.lstick[0].x = 0.05f;
    t_fp.input.lstick[0].y = -0.05f;
    t_fp.phys_cb(&t_gobj);
    if (t_fp.self_vel.x != 0.0f || t_fp.self_vel.y != 0.0f) {
        TestFail("fly phys: a resting stick's drift does not move");
        rc = 1;
    }
    GenoFly_SetSolid(1);
    t_fp.x1988 = 2; /* intangible from before: solid mid-flight turns the hurtboxes back on */
    t_fp.phys_cb(&t_gobj);
    GenoFly_SetSolid(0);
    if (t_fp.x1988 != 0) {
        TestFail("fly phys: solid keeps the hurtboxes");
        rc = 1;
    }
    t_fp.cur_pos.x = 5.0f;
    t_fp.cur_pos.y = 6.0f;
    t_fp.coll_data.prev_pos.x = -300.0f;
    t_fp.coll_data.env_flags = 0x7;
    t_fp.coll_cb(&t_gobj);
    if (t_fp.coll_data.cur_pos.x != 5.0f || t_fp.coll_data.cur_pos.y != 6.0f ||
        t_fp.coll_data.prev_pos.x != 5.0f || t_fp.coll_data.last_pos.y != 6.0f || t_fp.coll_data.env_flags != 0)
    {
        TestFail("fly coll: the collision data follows the fighter, no floor / wall / ceiling");
        rc = 1;
    }
    t_fp.cur_pos.x = 1.0e6f; /* past any blast zone */
    t_fp.cur_pos.y = -1.0e6f;
    if (ftCo_800D3158(&t_gobj)) {
        TestFail("a flying fighter past the blast zones was KO'd");
        rc = 1;
    }
    GenoFly_SetSpeed((int) t_fbits(1000.0f));
    if (GenoFly_Speed() != 200.0f) {
        TestFail("fly speed clamps to 200");
        rc = 1;
    }
    GenoFly_SetSpeed((int) t_fbits(old));
    t_setup();
    return rc;
}

#include "geno_define_snapshot_tests.inc"
#include "geno_lua_tests.inc"
#include "geno_guard_tests.inc"

/* Slice 4d: the independence census. A base "none" define's ftData must hold no pointer of the donor's that it does not
 * list as shared (docs/geno.md 22.4). The census function is pure: it names the pointer fields two ftData share. */
extern unsigned GenoDefine_FtDataSharedMask(const struct ftData* own, const struct ftData* donor);
extern const char* GenoDefine_FtDataFieldName(int i);
static int test_geno_none_independence(void)
{
    static struct ftData a, b;
    static int marker[4];
    unsigned m;
    int rc = 0;
    memset(&a, 0, sizeof a); memset(&b, 0, sizeof b);
    if (GenoDefine_FtDataSharedMask(&a, &b) != 0u) { TestFail("census: empty structs share nothing (NULL is not shared)"); rc = 1; }
    a.x20 = (ftData_x20*) &marker[0]; b.x20 = (ftData_x20*) &marker[0];
    a.x3C = (struct UnkFloat6_Camera*) &marker[1]; b.x3C = (struct UnkFloat6_Camera*) &marker[2];
    a.x5C = (HSD_Joint*) &marker[3]; b.x5C = (HSD_Joint*) &marker[3];
    m = GenoDefine_FtDataSharedMask(&a, &b);
    if (m != ((1u << 8) | (1u << 23))) { TestFail("census: exactly x20 and x5C are the shared pointers"); rc = 1; }
    if (strcmp(GenoDefine_FtDataFieldName(8), "x20") != 0 || strcmp(GenoDefine_FtDataFieldName(23), "x5C") != 0) {
        TestFail("census: field names"); rc = 1;
    }
    return rc;
}
void GenoTestRegisterAll(void)
{
    TestRegister("geno_none_guard_contact", test_geno_none_guard_contact);
    TestRegister("geno_lua_charge", test_geno_lua_charge);
    TestRegister("geno_lua_fault", test_geno_lua_fault);
    TestRegister("geno_lua_digest", test_geno_lua_digest);
    TestRegister("geno_lua_counter", test_geno_lua_counter);
    TestRegister("geno_define_snapshot", test_geno_define_snapshot);
    TestRegister("geno_none_independence", test_geno_none_independence);
    TestRegister("geno_define_hit_tag", test_geno_define_hit_tag);
    TestRegister("geno_define_state_hash", test_geno_define_state_hash);
    TestRegister("geno_define_article_digest", test_geno_define_article_digest);
    TestRegister("rb_game_hash", test_rb_game_hash);
    TestRegister("geno_fly", test_geno_fly);
    TestRegister("geno_ftcmd_escape", test_geno_ftcmd_escape);
    TestRegister("geno_ftcmd_loops", test_geno_ftcmd_loops);
    TestRegister("geno_vanilla_script_untouched", test_geno_vanilla_script_untouched);
    TestRegister("geno_state_resets", test_geno_state_resets);
    TestRegister("geno_attr_table", test_geno_attr_table);
    TestRegister("geno_multijump", test_geno_multijump);
    TestRegister("geno_multijump_script_gate", test_geno_multijump_script_gate);
    TestRegister("geno_ordinary_jumps", test_geno_ordinary_jumps);
    TestRegister("geno_jump_limits", test_geno_jump_limits);
    TestRegister("geno_reload_state_callbacks", test_geno_reload_state_callbacks);
    TestRegister("geno_effective_script_range", test_geno_effective_script_range);
    TestRegister("geno_iasa_script_validation", test_geno_iasa_script_validation);
    TestRegister("geno_state_savestate", test_geno_state_savestate);
    TestRegister("geno_v1_values", test_geno_v1_values);
    TestRegister("geno_v1_change_action", test_geno_v1_change_action);
    TestRegister("geno_v1_ground_edge", test_geno_v1_ground_edge);
    TestRegister("geno_v1_rehit", test_geno_v1_rehit);
    TestRegister("geno_v1_autolink", test_geno_v1_autolink);
    TestRegister("geno_v1_special_attrs", test_geno_v1_special_attrs);
    TestRegister("geno_v1_overlay", test_geno_v1_overlay);
    TestRegister("geno_v1_inert", test_geno_v1_inert);
    TestRegister("geno_v2_states", test_geno_v2_states);
    TestRegister("geno_v2_change_to_state", test_geno_v2_change_to_state);
    TestRegister("geno_v2_glide_entry", test_geno_v2_glide_entry);
    TestRegister("geno_v2_glide", test_geno_v2_glide);
    TestRegister("geno_v2_specials", test_geno_v2_specials);
    TestRegister("geno_v3_anim_motion", test_geno_v3_anim_motion);
    TestRegister("geno_v3_hidden_glide", test_geno_v3_hidden_glide);
    TestRegister("geno_v3_many_states", test_geno_v3_many_states);
    TestRegister("geno_v3_cape", test_geno_v3_cape);
    TestRegister("geno_v3_drill", test_geno_v3_drill);
    TestRegister("geno_lab_stale_fighter", test_geno_lab_stale_fighter);
    TestRegister("geno_player_team_read", test_geno_player_team_read);
    TestRegister("geno_v4_drill_pose", test_geno_v4_drill_pose);
    TestRegister("geno_v4_tornado_spin", test_geno_v4_tornado_spin);
    TestRegister("geno_lab_mode_table", test_geno_lab_mode_table);
    TestRegister("geno_lab_rules", test_geno_lab_rules);
    TestRegister("scene_item_frequency_rules", test_scene_item_frequency_rules);
    TestRegister("geno_lab_scene", test_geno_lab_scene);
    TestRegister("geno_lab_select_flow", test_geno_lab_select_flow);
    TestRegister("geno_lab_mismatch_fields", test_geno_lab_mismatch_fields);
    TestRegister("geno_v5_registry", test_geno_v5_registry);
    TestRegister("geno_v5_article_motion", test_geno_v5_article_motion);
    TestRegister("geno_v5_article_reflection", test_geno_v5_article_reflection);
    TestRegister("geno_v5_on_hit", test_geno_v5_on_hit);
    TestRegister("geno_v51", test_geno_v51);
    TestRegister("geno_v52_lockon", test_geno_v52_lockon);
    TestRegister("geno_v54_dash", test_geno_v54_dash);
}
