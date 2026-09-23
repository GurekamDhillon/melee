/* geno_tests.c - Geno's game-side tests (the escape interpreter, the state block, multi-jump,
 * attributes, the savestate round trip). Compiled through the game pipeline like
 * pc/tests/mex_tests.c, so TestRegister/TestFail resolve to gw_TestRegister/gw_TestFail and every
 * script word is stored in the guest's byte order, exactly as a Pl*.dat would hold it.
 * Registered from gw_tests_register_all via GenoTestRegisterAll. */

#include <Runtime/platform.h>

#include <melee/ft/fighter.h>
#include <melee/ft/ftaction.h>
#include <melee/ft/inlines.h>
#include <melee/ft/types.h>
#include <melee/lb/types.h>

#include "geno.h"

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
extern int Geno_TestInstall(const char* text);
extern void Geno_TestRestore(void);
extern int snap_open(int k);
extern void snap_save(int frame);
extern int snap_load(int frame);

/* mirrors the head of geno_game.c's GenoState (tests read the banks through it) */
typedef struct {
    s32 profile, kind;
    u32 flags, resets;
    s32 la_i[GENO_VARS_PER_BANK];
    s32 ra_i[GENO_VARS_PER_BANK];
    f32 la_f[GENO_VARS_PER_BANK];
    f32 ra_f[GENO_VARS_PER_BANK];
    u32 hook_calls, extra_jumps;
} TestGenoState;

static Fighter t_fp;
static HSD_GObj t_gobj;
static u32 t_script[96];
static Fighter_x2D0_t t_mj;

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
    if (msid != 304 || vy != 1.0f) {
        TestFail("jump 7: expected state 304 (last row), vy 1.0");
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

/* The state block is game state: a savestate taken with some values brings them back. */
static int test_geno_state_savestate(void)
{
    TestGenoState* st = t_setup();
    st->la_i[10] = 1234;
    st->la_f[2] = 2.5f;
    st->ra_i[63] = -7;
    if (snap_open(1) != 0) {
        TestFail("gw_snap_open failed (no melee-pc.map beside the exe?)");
        return 1;
    }
    snap_save(0);
    st->la_i[10] = 99;
    st->la_f[2] = 0.0f;
    st->ra_i[63] = 0;
    if (snap_load(0) != 0) {
        TestFail("gw_snap_load failed");
        return 1;
    }
    if (st->la_i[10] != 1234 || st->la_f[2] != 2.5f || st->ra_i[63] != -7) {
        TestFail("the Geno state block was not restored by a savestate load");
        return 1;
    }
    return 0;
}

void GenoTestRegisterAll(void)
{
    TestRegister("geno_ftcmd_escape", test_geno_ftcmd_escape);
    TestRegister("geno_ftcmd_loops", test_geno_ftcmd_loops);
    TestRegister("geno_vanilla_script_untouched", test_geno_vanilla_script_untouched);
    TestRegister("geno_state_resets", test_geno_state_resets);
    TestRegister("geno_attr_table", test_geno_attr_table);
    TestRegister("geno_multijump", test_geno_multijump);
    TestRegister("geno_state_savestate", test_geno_state_savestate);
}
