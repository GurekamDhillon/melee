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
static MotionState t_rows[4];

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
    /* a Geno-state target (v2) changes nothing; the check stays */
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

void GenoTestRegisterAll(void)
{
    TestRegister("geno_ftcmd_escape", test_geno_ftcmd_escape);
    TestRegister("geno_ftcmd_loops", test_geno_ftcmd_loops);
    TestRegister("geno_vanilla_script_untouched", test_geno_vanilla_script_untouched);
    TestRegister("geno_state_resets", test_geno_state_resets);
    TestRegister("geno_attr_table", test_geno_attr_table);
    TestRegister("geno_multijump", test_geno_multijump);
    TestRegister("geno_state_savestate", test_geno_state_savestate);
    TestRegister("geno_v1_values", test_geno_v1_values);
    TestRegister("geno_v1_change_action", test_geno_v1_change_action);
    TestRegister("geno_v1_ground_edge", test_geno_v1_ground_edge);
    TestRegister("geno_v1_rehit", test_geno_v1_rehit);
    TestRegister("geno_v1_autolink", test_geno_v1_autolink);
    TestRegister("geno_v1_special_attrs", test_geno_v1_special_attrs);
    TestRegister("geno_v1_overlay", test_geno_v1_overlay);
    TestRegister("geno_v1_inert", test_geno_v1_inert);
}
