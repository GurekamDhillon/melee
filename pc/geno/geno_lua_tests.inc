/* geno_lua_tests.inc - slice 5, game half: fighter Lua through a define fighter (included by geno_tests.c; uses its t_* fixtures).
 * Capture mode (GenoGame_TestCapture) records a change of action instead of performing it, as the other v2 tests do. */

extern void* GenoGame_LuaBlockOf(Fighter* fp);
extern void GenoGame_TestCapture(int on, int anim_end);
extern int GenoGame_TestChanges(void);
extern u32 GenoGame_TestLastTarget(void);
extern MotionState* Geno_MotionRow(Fighter* fp, int msid);
extern int Geno_SpecialEnter(Fighter_GObj* gobj, int which);
extern u32 GenoDefine_StateDigest(Fighter* fp);
extern int GenoGame_LuaInfo(int port, int what);

#define TL_MODULE \
    "local M = {} " \
    "function M.enter(ctx) ctx.state.charge = 0 end " \
    "function M.frame(ctx) local s = ctx.state " \
    "if ctx.input.special_held and s.charge < 60 then s.charge = s.charge + 1 s.armed = s.charge >= 3 else ctx.go('Release') end end " \
    "function M.rel(ctx) ctx.velocity(1.6 + 0.03 * ctx.state.charge, 0) end " \
    "function M.relf(ctx) ctx.hitbox_damage(3, 6 + 0.15 * ctx.state.charge) end " \
    "function M.boom(ctx) ctx.state.charge = 77 error('boom') end " \
    "return M"

#define TL_HEAD(KEY) \
    "{\"geno\":10,\"fighters\":[{\"define\":{\"key\":\"" KEY "\",\"name\":\"Lua T\",\"base\":\"mario\",\"common\":\"melee.common.v1\"," \
    "\"resources\":\"retail:mario\"}"
#define TL_BLOCK ",\"lua\":{\"source\":\"" TL_MODULE "\",\"state\":{\"charge\":\"int\",\"armed\":\"bool\"}}"
#define TL_STATES \
    ",\"states\":[{\"name\":\"Charge\",\"behavior\":\"geno.ground\",\"lua\":{\"enter\":\"enter\",\"frame\":\"frame\"}}," \
    "{\"name\":\"Release\",\"behavior\":\"geno.ground\",\"lua\":{\"enter\":\"rel\",\"frame\":\"relf\"}}," \
    "{\"name\":\"Bad\",\"behavior\":\"geno.ground\",\"lua\":{\"frame\":\"boom\"}}]" \
    ",\"specials\":{\"n\":\"geno:Charge\",\"s\":\"geno:Bad\"}"

static int tl_setup(const char* json, TestGenoState** st)
{
    int kind;
    t_zero(t_rows, sizeof(t_rows));
    if (Geno_TestInstall(json) != 1 || Geno_DefineCount() != 1) {
        TestFail("fighter Lua: the test define was not installed");
        return 0;
    }
    kind = Geno_DefineCKAt(0) - 1;
    *st = t_setup();
    t_fp.kind = kind;
    t_fp.ground_or_air = GA_Ground;
    Geno_FighterReset(&t_fp);
    *st = GenoGame_StateOf(&t_fp);
    return 1;
}

/* enter -> frames while held count the charge -> release goes to Release, whose enter sets the dash and whose frame sets the damage */
static int test_geno_lua_charge(void)
{
    TestGenoState* st;
    GenoLuaBlock* lb;
    MotionState* row;
    int rc = 0, f;
    if (!tl_setup(TL_HEAD("lua-charge") TL_BLOCK TL_STATES "}]}", &st)) {
        Geno_TestRestore();
        return 1;
    }
    GenoGame_TestCapture(1, 0);
    lb = GenoGame_LuaBlockOf(&t_fp);
    lb->slot[0] = 41; /* stale charge from an earlier move: "enter" must clear it */
    if (!Geno_SpecialEnter(&t_gobj, GENO_SP_N) || GenoGame_TestChanges() != 1) {
        TestFail("fighter Lua: the neutral special did not enter the Charge state");
        rc = 1;
    }
    if (lb->slot[0] != 0 || lb->slot[1] != 0) {
        TestFail("fighter Lua: the Charge state's \"enter\" must run once and zero the charge");
        rc = 1;
    }
    row = Geno_MotionRow(&t_fp, GENO_MOTION_BASE + 0);
    if (row == NULL || row->anim_cb == NULL) {
        TestFail("fighter Lua: the Charge row has no anim callback");
        Geno_TestRestore();
        return 1;
    }
    t_fp.motion_id = GENO_MOTION_BASE + 0;
    t_fp.input.held_buttons[0] = HSD_PAD_B;
    for (f = 0; f < 5; f++) {
        row->anim_cb(&t_gobj);
    }
    if (lb->slot[0] != 5 || lb->slot[1] != 1 || GenoGame_TestChanges() != 1) {
        TestFail("fighter Lua: five held frames must count 5, arm, and not change action");
        rc = 1;
    }
    /* released: Release is entered (its enter sets the dash from the charge) */
    t_fp.input.held_buttons[0] = 0;
    row->anim_cb(&t_gobj);
    if (GenoGame_TestChanges() != 2 || GenoGame_TestLastTarget() != GENO_TARGET(GENO_TGT_GENO, 1) || lb->slot[0] != 5) {
        TestFail("fighter Lua: releasing must go to Release with the charge kept");
        rc = 1;
    }
    {
        f32 want = (f32) (1.6 + 0.03 * 5);
        f32 d = t_fp.self_vel.x - want;
        if (d < -0.0001f || d > 0.0001f || t_fp.gr_vel < want - 0.0001f || t_fp.gr_vel > want + 0.0001f) {
            TestFail("fighter Lua: the Release dash must be 1.6 + 0.03 x charge forward");
            rc = 1;
        }
    }
    row = Geno_MotionRow(&t_fp, GENO_MOTION_BASE + 1);
    t_fp.motion_id = GENO_MOTION_BASE + 1;
    row->anim_cb(&t_gobj);
    {
        f32 want = (f32) (6 + 0.15 * 5), d = t_fp.x914[0].damage - want;
        if (d < -0.0001f || d > 0.0001f || t_fp.x914[1].damage < want - 0.0001f || t_fp.x914[1].damage > want + 0.0001f) {
            TestFail("fighter Lua: Release frame must set hitboxes 0 and 1 to 6 + 0.15 x charge");
            rc = 1;
        }
        if (t_fp.x914[2].damage != 0.0f) {
            TestFail("fighter Lua: hitbox_damage(3, ..) must not touch hitbox 2");
            rc = 1;
        }
    }
    /* the cap */
    lb->slot[0] = 60;
    t_fp.motion_id = GENO_MOTION_BASE + 0;
    t_fp.input.held_buttons[0] = HSD_PAD_B;
    Geno_MotionRow(&t_fp, GENO_MOTION_BASE + 0)->anim_cb(&t_gobj);
    if (lb->slot[0] != 60 || GenoGame_TestLastTarget() != GENO_TARGET(GENO_TGT_GENO, 1)) {
        TestFail("fighter Lua: at 60 frames the charge ends");
        rc = 1;
    }
    /* a fresh spawn clears the block */
    Geno_FighterReset(&t_fp);
    if (lb->slot[0] != 0 || lb->faults != 0) {
        TestFail("fighter Lua: Geno_FighterReset must clear the Lua block");
        rc = 1;
    }
    GenoGame_TestCapture(0, 0);
    Geno_TestRestore();
    return rc;
}

/* a function that errors: its writes are dropped, the fault is counted (simulation state), the fighter goes to auto */
static int test_geno_lua_fault(void)
{
    TestGenoState* st;
    GenoLuaBlock* lb;
    MotionState* row;
    u32 d0, d1;
    int rc = 0;
    if (!tl_setup(TL_HEAD("lua-fault") TL_BLOCK TL_STATES "}]}", &st)) {
        Geno_TestRestore();
        return 1;
    }
    GenoGame_TestCapture(1, 0);
    lb = GenoGame_LuaBlockOf(&t_fp);
    lb->slot[0] = 3;
    t_fp.motion_id = GENO_MOTION_BASE + 2; /* Bad */
    row = Geno_MotionRow(&t_fp, GENO_MOTION_BASE + 2);
    d0 = GenoDefine_StateDigest(&t_fp);
    row->anim_cb(&t_gobj);
    if (lb->slot[0] != 3) {
        TestFail("fighter Lua: a faulting call must not keep its state writes");
        rc = 1;
    }
    if (lb->faults != 1 || lb->last_fault != GENO_LUA_FAULT_ERROR) {
        TestFail("fighter Lua: the fault must be counted (faults 1, last GENO_LUA_FAULT_ERROR)");
        rc = 1;
    }
    if (GenoGame_TestChanges() != 1 || GenoGame_TestLastTarget() != GENO_TARGET(GENO_TGT_MOTION, ftCo_MS_Wait)) {
        TestFail("fighter Lua: a fault must send a grounded fighter to Wait");
        rc = 1;
    }
    d1 = GenoDefine_StateDigest(&t_fp);
    if (d0 == d1) {
        TestFail("fighter Lua: the fault count must be in the digest");
        rc = 1;
    }
    GenoGame_TestCapture(0, 0);
    Geno_TestRestore();
    return rc;
}

/* the digest: only a define that declares a layout folds the block; the snapshot restores the block */
static int test_geno_lua_digest(void)
{
    TestGenoState* st;
    GenoLuaBlock* lb;
    u32 d0, d1, d2;
    int rc = 0;
    /* a define WITHOUT a lua block: the block is not part of its digest */
    if (!tl_setup(TL_HEAD("lua-plain") "}]}", &st)) {
        Geno_TestRestore();
        return 1;
    }
    lb = GenoGame_LuaBlockOf(&t_fp);
    d0 = GenoDefine_StateDigest(&t_fp);
    lb->slot[0] = 12345;
    lb->faults = 9;
    if (d0 == 0 || GenoDefine_StateDigest(&t_fp) != d0) {
        TestFail("fighter Lua: a define with no lua block must keep its digest whatever the block holds");
        rc = 1;
    }
    Geno_TestRestore();
    /* a define with one */
    if (!tl_setup(TL_HEAD("lua-digest") TL_BLOCK TL_STATES "}]}", &st)) {
        Geno_TestRestore();
        return 1;
    }
    lb = GenoGame_LuaBlockOf(&t_fp);
    d0 = GenoDefine_StateDigest(&t_fp);
    lb->slot[0] = 7;
    d1 = GenoDefine_StateDigest(&t_fp);
    if (d0 == 0 || d1 == d0) {
        TestFail("fighter Lua: a slot must move the digest of a define that declares a layout");
        rc = 1;
    }
    lb->slot[0] = 0;
    if (GenoDefine_StateDigest(&t_fp) != d0) {
        TestFail("fighter Lua: the digest must return when the slot does");
        rc = 1;
    }
    lb->slot[0] = 7;
    if (snap_open(1) != 0) {
        TestFail("fighter Lua: the snapshot could not open");
        rc = 1;
    } else {
        snap_save(0);
        lb->slot[0] = 99;
        lb->slot[1] = 1;
        lb->faults = 4;
        d2 = GenoDefine_StateDigest(&t_fp);
        if (d2 == d1) {
            TestFail("fighter Lua: the mutation did not move the digest");
            rc = 1;
        }
        if (snap_load(0) != 0 || lb->slot[0] != 7 || lb->slot[1] != 0 || lb->faults != 0 || GenoDefine_StateDigest(&t_fp) != d1) {
            TestFail("fighter Lua: a savestate must restore the typed state and the digest");
            rc = 1;
        }
    }
    /* a retail-kind fighter (the Kirby fixture): digest stays 0 whatever its block holds */
    t_setup();
    GenoLuaBlock* kb = GenoGame_LuaBlockOf(&t_fp);
    kb->slot[0] = 5;
    if (GenoDefine_StateDigest(&t_fp) != 0) {
        TestFail("fighter Lua: a non-define fighter's digest must stay 0");
        rc = 1;
    }
    Geno_TestRestore();
    return rc;
}

/* slice 5 increment 2: a counter answered in Lua. The entry of the answer state reads what the hit left in the fighter (hit_damage,
 * countered) and scales the strike by it; a second state turns the fighter. Both read GenoState values that are already in the hash. */
#define TC_MODULE \
    "local M = {} " \
    "function M.answer(ctx) local s = ctx.state " \
    "s.power = math.min(30, math.max(9, 1.5 * ctx.self.hit_damage)) s.seen = ctx.self.countered s.from = ctx.self.hit_from end " \
    "function M.strike(ctx) ctx.hitbox_damage(3, ctx.state.power) end " \
    "function M.spin(ctx) ctx.turn() ctx.velocity(2, 0) end " \
    "return M"
#define TC_BLOCK ",\"lua\":{\"source\":\"" TC_MODULE "\",\"state\":{\"power\":\"float\",\"seen\":\"int\",\"from\":\"int\"}}"
#define TC_STATES \
    ",\"states\":[{\"name\":\"Parry\",\"behavior\":\"geno.ground\",\"counter\":{\"from\":2,\"to\":9,\"target\":\"geno:Answer\"}}," \
    "{\"name\":\"Answer\",\"behavior\":\"geno.ground\",\"lua\":{\"enter\":\"answer\",\"frame\":\"strike\"}}," \
    "{\"name\":\"Spin\",\"behavior\":\"geno.ground\",\"lua\":{\"enter\":\"spin\"}}]" \
    ",\"specials\":{\"lw\":\"geno:Parry\",\"s\":\"geno:Spin\"}"

static int test_geno_lua_counter(void)
{
    TestGenoState* st;
    GenoLuaBlock* lb;
    int rc = 0;
    f32 want;
    if (!tl_setup(TL_HEAD("lua-counter") TC_BLOCK TC_STATES "}]}", &st)) {
        Geno_TestRestore();
        return 1;
    }
    GenoGame_TestCapture(1, 0);
    lb = GenoGame_LuaBlockOf(&t_fp);
    /* a hit of 12 landed in the window: the answer's enter sees hit_damage 12 and countered 1 (Geno_OnHit counts it) */
    st->hit_damage = 12.0f;
    st->hit_port = -1; /* no fighter attacker: hit_from is 0 */
    if (!Geno_SpecialEnter(&t_gobj, GENO_SP_LW)) {
        TestFail("fighter Lua counter: the down special did not enter Parry");
        rc = 1;
    }
    t_fp.motion_id = GENO_MOTION_BASE + 0;
    st->action_time = 4;
    t_fp.dmg.x1838_percentTemp = 12.0f;
    Geno_OnHit(&t_gobj);
    want = 18.0f; /* 1.5 x 12, inside 9..30 */
    if (lb->slot[1] != 1 || lb->slot[2] != 0 || GenoGame_TestLastTarget() != GENO_TARGET(GENO_TGT_GENO, 1)) {
        TestFail("fighter Lua counter: the answer must be entered with countered 1 and hit_from 0");
        rc = 1;
    }
    {
        union { f32 f; s32 i; } u;
        u.i = lb->slot[0];
        if (u.f < want - 0.0001f || u.f > want + 0.0001f) {
            TestFail("fighter Lua counter: power must be 1.5 x the countered hit (18)");
            rc = 1;
        }
    }
    /* clamps: a 2% hit answers with 9, a 40% hit with 30 */
    st->hit_damage = 2.0f;
    Geno_MotionRow(&t_fp, GENO_MOTION_BASE + 1);
    t_fp.motion_id = GENO_MOTION_BASE + 0;
    st->action_time = 4;
    t_fp.dmg.x1838_percentTemp = 2.0f;
    Geno_OnHit(&t_gobj);
    {
        union { f32 f; s32 i; } u;
        u.i = lb->slot[0];
        if (u.f != 9.0f) {
            TestFail("fighter Lua counter: a 2 percent hit must answer with the floor 9");
            rc = 1;
        }
    }
    st->action_time = 4;
    t_fp.motion_id = GENO_MOTION_BASE + 0;
    t_fp.dmg.x1838_percentTemp = 40.0f;
    Geno_OnHit(&t_gobj);
    {
        union { f32 f; s32 i; } u;
        u.i = lb->slot[0];
        if (u.f != 30.0f) {
            TestFail("fighter Lua counter: a 40 percent hit must answer with the cap 30");
            rc = 1;
        }
    }
    /* the strike frame sets the hitboxes to the stored power */
    t_fp.motion_id = GENO_MOTION_BASE + 1;
    Geno_MotionRow(&t_fp, GENO_MOTION_BASE + 1)->anim_cb(&t_gobj);
    if (t_fp.x914[0].damage != 30.0f || t_fp.x914[1].damage != 30.0f || t_fp.x914[2].damage != 0.0f) {
        TestFail("fighter Lua counter: the strike frame must set hitboxes 0 and 1 to power 30");
        rc = 1;
    }
    /* turn: facing flips, and a velocity after it is relative to the new facing */
    t_fp.facing_dir = 1.0f;
    t_fp.ground_or_air = GA_Ground;
    if (!Geno_SpecialEnter(&t_gobj, GENO_SP_S) || t_fp.facing_dir != -1.0f) {
        TestFail("fighter Lua counter: ctx.turn must flip the facing");
        rc = 1;
    }
    if (t_fp.gr_vel != -2.0f) {
        TestFail("fighter Lua counter: a velocity queued after turn is relative to the new facing");
        rc = 1;
    }
    GenoGame_TestCapture(0, 0);
    Geno_TestRestore();
    return rc;
}
