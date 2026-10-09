/* geno_combo_tests.inc - geno 11, the combo graph (docs/geno.md section 25): parse, routes, buffer, hitlag, priority, meter, Lua
 * gate, string depth, the digest, and a resimulation from a snapshot. Included by geno_tests.c (uses its t_* fixtures and capture
 * mode: a change of action is recorded and performed on motion_id, not run through Fighter_ChangeMotionState). */

extern void* GenoGame_ComboBlockOf(Fighter* fp);
extern void Geno_ComboInput(Fighter_GObj* gobj);
extern int Geno_ComboTick(Fighter_GObj* gobj);
extern void Geno_ComboContact(Fighter* fp, int kind);
extern int Geno_ComboParam(int p, int what);
extern int Geno_MeterParam(int p, int what);
extern int Geno_ComboWord(int p, int route, int word);

#define TC_HEAD(VER, KEY) \
    "{\"geno\":" VER ",\"fighters\":[{\"define\":{\"key\":\"" KEY "\",\"name\":\"Combo T\",\"base\":\"mario\",\"common\":\"melee.common.v1\"," \
    "\"resources\":\"retail:mario\"}"
#define TC_LUA ",\"lua\":{\"source\":\"local M = {} function M.f(ctx) end return M\",\"state\":{\"armed\":\"int\"}}"
#define TC_STATES ",\"states\":[{\"name\":\"Light2\",\"behavior\":\"geno.ground\"},{\"name\":\"Heavy\",\"behavior\":\"geno.ground\"}]"
#define TC_COMBO \
    ",\"combo\":{\"buffer\":3,\"max_links\":3,\"ki\":{\"max\":100,\"start\":20,\"on_hit\":5,\"on_block\":1,\"on_taken\":2},\"routes\":[" \
    "{\"name\":\"L1>L2\",\"from\":\"jab1\",\"to\":\"geno:Light2\",\"input\":\"light\",\"on\":\"hit\",\"window\":[5,20]}," \
    "{\"name\":\"L1>H\",\"from\":\"jab1\",\"to\":\"geno:Heavy\",\"input\":\"heavy\",\"dir\":\"forward\",\"on\":\"hit\",\"window\":[5,20],\"priority\":5}," \
    "{\"name\":\"L2>H\",\"from\":\"geno:Light2\",\"to\":\"geno:Heavy\",\"input\":\"heavy\",\"on\":\"hit\",\"ki_cost\":10,\"when\":{\"slot\":\"armed\",\"min\":1}}," \
    "{\"name\":\"H>L2\",\"from\":[\"geno:Heavy\"],\"to\":\"geno:Light2\",\"input\":\"light\",\"on\":\"contact\"}]," \
    "\"strings\":[{\"name\":\"L1 L2 H\",\"steps\":[\"jab1\",\"geno:Light2\",\"geno:Heavy\"]}]}"

static TestGenoState* tc_st;

static int tc_setup(const char* json)
{
    int kind;
    t_zero(t_rows, sizeof(t_rows));
    if (Geno_TestInstall(json) != 1 || Geno_DefineCount() != 1) {
        return 0;
    }
    kind = Geno_DefineCKAt(0) - 1;
    tc_st = t_setup();
    t_fp.kind = kind;
    t_fp.ground_or_air = GA_Ground;
    Geno_FighterReset(&t_fp);
    tc_st = GenoGame_StateOf(&t_fp);
    return 1;
}

/* One logic frame of input: the buttons pressed this frame and the main stick; ages the buffer like the engine (outside hitlag). */
static void tc_frame(HSD_Pad pressed, f32 sx, f32 sy, int hitlag)
{
    t_fp.input.pressed_buttons = pressed;
    t_fp.input.held_buttons[0] = pressed;
    t_fp.input.lstick[0].x = sx;
    t_fp.input.lstick[0].y = sy;
    t_fp.input.cstick[0].x = t_fp.input.cstick[0].y = 0.0f;
    t_fp.input.cstick[1].x = t_fp.input.cstick[1].y = 0.0f;
    t_fp.active_timer.lstick.x = sx != 0.0f ? 0 : 50;
    t_fp.active_timer.lstick.y = sy != 0.0f ? 0 : 50;
    t_fp.x2219_b5 = hitlag ? 1 : 0;
    Geno_ComboInput(&t_gobj);
    t_fp.x2219_b5 = 0;
}

static void tc_enter_motion(int motion)
{
    t_fp.motion_id = motion;
    Geno_OnActionChange(&t_gobj);
}

static int test_geno_combo_parse(void)
{
    int rc = 0;
    if (Geno_TestInstall(TC_HEAD("10", "combo-v10") TC_LUA TC_STATES TC_COMBO "}]}") != 0) {
        TestFail("combo: a \"combo\" block in a geno 10 file must refuse the entry");
        rc = 1;
    }
    if (Geno_TestInstall(TC_HEAD("11", "combo-bad") TC_STATES ",\"combo\":{\"routes\":[{\"from\":\"jab1\",\"to\":\"geno:Nope\",\"input\":\"light\"}]}}]}") != 0) {
        TestFail("combo: a route to an undeclared state must refuse the entry");
        rc = 1;
    }
    if (Geno_TestInstall(TC_HEAD("11", "combo-bad2") TC_STATES ",\"combo\":{\"routes\":[{\"from\":\"jab1\",\"to\":\"geno:Light2\",\"input\":\"kick\"}]}}]}") != 0) {
        TestFail("combo: an unknown input class must refuse the entry");
        rc = 1;
    }
    if (Geno_TestInstall(TC_HEAD("11", "combo-bad3") TC_STATES ",\"combo\":{\"routes\":[{\"from\":\"jab1\",\"to\":\"geno:Light2\",\"input\":\"light\",\"when\":{\"slot\":\"nope\"}}]}}]}") != 0) {
        TestFail("combo: a Lua gate naming no lua.state slot must refuse the entry");
        rc = 1;
    }
    if (Geno_TestInstall("{\"geno\":11,\"fighters\":[{\"attach\":\"mario\",\"combo\":{\"routes\":[]}}]}") != 0) {
        TestFail("combo: an attach entry must not carry a combo graph");
        rc = 1;
    }
    if (!tc_setup(TC_HEAD("11", "combo-ok") TC_LUA TC_STATES TC_COMBO "}]}")) {
        TestFail("combo: the test define was not installed");
        Geno_TestRestore();
        return 1;
    }
    if (Geno_ComboParam(tc_st->profile, GENO_CP_ROUTES) != 4 || Geno_ComboParam(tc_st->profile, GENO_CP_MAX_LINKS) != 3 ||
        Geno_ComboParam(tc_st->profile, GENO_CP_KI_START) != 2000)
    {
        TestFail("combo: 4 routes, max_links 3, ki start 20.00");
        rc = 1;
    }
    /* priority 5 sorts L1>H first, file order otherwise */
    if (Geno_ComboWord(tc_st->profile, 0, GENO_CR_PRIORITY) != 5 || Geno_ComboWord(tc_st->profile, 1, GENO_CR_TO) != (int) GENO_TARGET(GENO_TGT_GENO, 0) ||
        Geno_ComboWord(tc_st->profile, 0, GENO_CR_FROM) != (int) GENO_TARGET(GENO_TGT_MOTION, 44) ||
        Geno_ComboWord(tc_st->profile, 1, GENO_CR_BUFFER) != 3 || Geno_ComboWord(tc_st->profile, 2, GENO_CR_KI_COST) != 1000)
    {
        TestFail("combo: parsed words (sort by priority, from jab1 = motion 44, default buffer, ki cost in hundredths)");
        rc = 1;
    }
    if (((GenoComboBlock*) GenoGame_ComboBlockOf(&t_fp))->ki != 2000) {
        TestFail("combo: the meter starts at ki.start at the reset");
        rc = 1;
    }
    Geno_TestRestore();
    /* "jump_short" is a route target (the launcher jump-cancel): parsed as the combo action GENO_CA_JUMP_SHORT, beside "jump" */
    if (!tc_setup(TC_HEAD("11", "combo-hop") TC_STATES ",\"combo\":{\"routes\":[{\"from\":\"jab1\",\"to\":\"jump_short\",\"input\":\"jump\"},"
                  "{\"from\":\"jab2\",\"to\":\"jump\",\"input\":\"jump\"}]}}]}")) {
        TestFail("combo: a jump_short / jump route define was not installed");
        Geno_TestRestore();
        t_setup();
        return 1;
    }
    if (Geno_ComboWord(tc_st->profile, 0, GENO_CR_TO) != (int) GENO_TARGET(GENO_COMBO_TGT_ACT, GENO_CA_JUMP_SHORT) ||
        Geno_ComboWord(tc_st->profile, 1, GENO_CR_TO) != (int) GENO_TARGET(GENO_COMBO_TGT_ACT, GENO_CA_JUMP))
    {
        TestFail("combo: \"jump_short\" must parse to GENO_CA_JUMP_SHORT and \"jump\" to GENO_CA_JUMP");
        rc = 1;
    }
    /* "meter": a HUD bar for the ki value; needs a combo block with ki; refused when malformed */
    if (Geno_TestInstall(TC_HEAD("11", "meter-bad1") TC_STATES ",\"meter\":{\"name\":\"KI\"}}]}") != 0) {
        TestFail("combo: a meter without a combo block must refuse the entry");
        rc = 1;
    }
    if (Geno_TestInstall(TC_HEAD("11", "meter-bad2") TC_STATES ",\"combo\":{\"ki\":{\"max\":100},\"routes\":[]},\"meter\":{\"color\":\"red\"}}]}") != 0) {
        TestFail("combo: a meter colour that is not #RRGGBB must refuse the entry");
        rc = 1;
    }
    if (!tc_setup(TC_HEAD("11", "meter-ok") TC_STATES ",\"combo\":{\"ki\":{\"max\":80},\"routes\":[]},"
                  "\"meter\":{\"name\":\"CHI\",\"color\":\"#10A0FF\",\"position\":[100,420],\"size\":[90,6]}}]}")) {
        TestFail("combo: a define with a meter was not installed");
        Geno_TestRestore();
        t_setup();
        return 1;
    }
    {
        int pr = tc_st->profile;
        if (!Geno_MeterParam(pr, 0) || Geno_MeterParam(pr, 7) != 'C' || Geno_MeterParam(pr, 8) != 'H' || Geno_MeterParam(pr, 9) != 'I' ||
            Geno_MeterParam(pr, 10) != 0 || (unsigned) Geno_MeterParam(pr, 1) != 0x10A0FFFFu || Geno_MeterParam(pr, 2) != 10000 ||
            Geno_MeterParam(pr, 3) != 42000 || Geno_MeterParam(pr, 4) != 9000 || Geno_MeterParam(pr, 5) != 600 || Geno_MeterParam(pr, 6) != 8000)
        {
            TestFail("combo: the meter's name CHI, colour #10A0FF, position 100,420, size 90x6 and max (combo.ki.max 80 = 8000 hundredths)");
            rc = 1;
        }
    }
    Geno_TestRestore();
    t_setup();
    return rc;
}

static int test_geno_combo_routes(void)
{
    GenoComboBlock* cb;
    GenoLuaBlock* lb;
    int rc = 0, i;
    if (!tc_setup(TC_HEAD("11", "combo-routes") TC_LUA TC_STATES TC_COMBO "}]}")) {
        TestFail("combo: the test define was not installed");
        Geno_TestRestore();
        return 1;
    }
    GenoGame_TestCapture(1, 0);
    cb = GenoGame_ComboBlockOf(&t_fp);
    lb = GenoGame_LuaBlockOf(&t_fp);
    /* the press that starts jab1 is consumed by the action change: it never cancels jab1 itself */
    tc_frame(HSD_PAD_A, 0, 0, 0);
    tc_enter_motion(44);
    tc_st->action_time = 6;
    Geno_ComboContact(&t_fp, 0);
    if (Geno_ComboTick(&t_gobj) != 0 || GenoGame_TestChanges() != 0) {
        TestFail("combo: the press that began the move must not feed its routes");
        rc = 1;
    }
    if (cb->ki != 2500) {
        TestFail("combo: a hit landed adds ki.on_hit (20 + 5)");
        rc = 1;
    }
    /* whiff: no hit -> an on-hit route does not fire */
    tc_enter_motion(44);
    tc_st->action_time = 6;
    tc_frame(HSD_PAD_A, 0, 0, 0);
    if (Geno_ComboTick(&t_gobj) != 0) {
        TestFail("combo: an on-hit route fired on a whiff");
        rc = 1;
    }
    /* before the window: buffered, then taken when the window opens (within the buffer) */
    tc_enter_motion(44);
    tc_st->action_time = 3;
    Geno_ComboContact(&t_fp, 0);
    tc_frame(HSD_PAD_A, 0, 0, 0);
    if (Geno_ComboTick(&t_gobj) != 0) {
        TestFail("combo: a route fired before its window");
        rc = 1;
    }
    /* hitlag: 10 frames of hitlag do not age the press */
    for (i = 0; i < 10; i++) {
        tc_frame(0, 0, 0, 1);
    }
    if (cb->age[GENO_CI_LIGHT] != 0) {
        TestFail("combo: hitlag frames must not age a buffered press");
        rc = 1;
    }
    tc_frame(0, 0, 0, 0);
    tc_frame(0, 0, 0, 0); /* age 2 */
    tc_st->action_time = 5;
    if (Geno_ComboTick(&t_gobj) != 1 || GenoGame_TestLastTarget() != GENO_TARGET(GENO_TGT_GENO, 0) || cb->links != 1 ||
        cb->age[GENO_CI_LIGHT] != 255 || t_fp.motion_id != GENO_MOTION_BASE + 0)
    {
        TestFail("combo: a buffered light press (age 2 <= buffer 3) must take L1>L2 when the window opens, consume the press, link 1");
        rc = 1;
    }
    /* the buffer runs out */
    tc_enter_motion(44);
    cb->links = 0;
    Geno_ComboContact(&t_fp, 0);
    tc_frame(HSD_PAD_A, 0, 0, 0);
    for (i = 0; i < 4; i++) {
        tc_frame(0, 0, 0, 0);
    }
    tc_st->action_time = 8;
    if (Geno_ComboTick(&t_gobj) != 0) {
        TestFail("combo: a press older than the buffer (4 > 3) fired");
        rc = 1;
    }
    /* priority and direction: a forward smash input takes the heavy route (priority 5), a light one the light route */
    tc_enter_motion(44);
    Geno_ComboContact(&t_fp, 0);
    tc_st->action_time = 8;
    tc_frame(HSD_PAD_A, 1.0f, 0, 0);
    if (cb->age[GENO_CI_HEAVY] != 0 || cb->dir[GENO_CI_HEAVY] != GENO_CD_FORWARD || cb->age[GENO_CI_LIGHT] != 255) {
        TestFail("combo: A with a fresh forward flick is a heavy forward press, not a light one");
        rc = 1;
    }
    if (Geno_ComboTick(&t_gobj) != 1 || GenoGame_TestLastTarget() != GENO_TARGET(GENO_TGT_GENO, 1)) {
        TestFail("combo: the heavy forward press must take L1>H");
        rc = 1;
    }
    /* a back flick is heavy back: L1>H wants forward -> nothing */
    tc_enter_motion(44);
    Geno_ComboContact(&t_fp, 0);
    tc_st->action_time = 8;
    tc_frame(HSD_PAD_A, -1.0f, 0, 0);
    if (Geno_ComboTick(&t_gobj) != 0) {
        TestFail("combo: a heavy back press must not take a forward route");
        rc = 1;
    }
    /* the Lua gate and the meter: L2>H needs armed >= 1 and 10 ki */
    tc_enter_motion(GENO_MOTION_BASE + 0);
    cb->links = 1;
    cb->motion = GENO_MOTION_BASE + 0;
    Geno_ComboContact(&t_fp, 0);
    tc_st->action_time = 2;
    lb->slot[0] = 0;
    tc_frame(HSD_PAD_A, 1.0f, 0, 0);
    if (Geno_ComboTick(&t_gobj) != 0) {
        TestFail("combo: the Lua gate (armed 0 < 1) must hold the route");
        rc = 1;
    }
    lb->slot[0] = 1;
    cb->ki = 900;
    if (Geno_ComboTick(&t_gobj) != 0) {
        TestFail("combo: 9 ki < cost 10 must hold the route");
        rc = 1;
    }
    cb->ki = 3000;
    if (Geno_ComboTick(&t_gobj) != 1 || cb->ki != 2000 || cb->links != 2) {
        TestFail("combo: with the gate and 30 ki the route fires, pays 10, link 2");
        rc = 1;
    }
    /* block counts as contact for H>L2 ("contact"), but max_links 3 stops a fourth link */
    Geno_ComboContact(&t_fp, 1);
    if (cb->blocked != 1 || cb->hit != 0) {
        TestFail("combo: a shield hit marks blocked only");
        rc = 1;
    }
    tc_st->action_time = 2;
    tc_frame(HSD_PAD_A, 0, 0, 0);
    if (Geno_ComboTick(&t_gobj) != 1 || cb->links != 3) {
        TestFail("combo: H>L2 on contact (block) must fire, link 3");
        rc = 1;
    }
    Geno_ComboContact(&t_fp, 0);
    lb->slot[0] = 1;
    tc_frame(HSD_PAD_A, 1.0f, 0, 0);
    if (Geno_ComboTick(&t_gobj) != 0) {
        TestFail("combo: max_links 3 must stop a fourth link");
        rc = 1;
    }
    /* any other action ends the string */
    tc_enter_motion(14);
    if (cb->links != 0) {
        TestFail("combo: leaving the last link's motion must end the string");
        rc = 1;
    }
    GenoGame_TestCapture(0, 0);
    Geno_TestRestore();
    t_setup();
    return rc;
}

/* Every word of the block is in the digest of a combo define; a define without a combo block never folds it; a resimulation from a
 * snapshot of the blocks with the same inputs gives the same bytes and the same targets. */
static int test_geno_combo_digest(void)
{
    static GenoComboBlock snap_cb;
    static TestGenoState snap_st;
    GenoComboBlock* cb;
    u32 d0, d1, targets[2][6];
    int rc = 0, i, pass;
    if (!tc_setup(TC_HEAD("11", "combo-digest") TC_LUA TC_STATES TC_COMBO "}]}")) {
        TestFail("combo: the test define was not installed");
        Geno_TestRestore();
        return 1;
    }
    cb = GenoGame_ComboBlockOf(&t_fp);
    d0 = GenoDefine_StateDigest(&t_fp);
    for (i = 0; i < (int) (sizeof(GenoComboBlock) / 4); i++) {
        u32* w = (u32*) cb + i;
        u32 keep = *w;
        *w ^= 0x10u;
        if (GenoDefine_StateDigest(&t_fp) == d0) {
            TestFail("combo: a word of GenoComboBlock is not in the digest");
            rc = 1;
        }
        *w = keep;
    }
    if (GenoDefine_StateDigest(&t_fp) != d0) {
        TestFail("combo: the digest must come back with the block");
        rc = 1;
    }
    /* resimulation: snapshot, run a sequence, restore, run it again */
    GenoGame_TestCapture(1, 0);
    tc_enter_motion(44);
    memcpy(&snap_cb, cb, sizeof snap_cb);
    memcpy(&snap_st, tc_st, sizeof snap_st);
    for (pass = 0; pass < 2; pass++) {
        memcpy(cb, &snap_cb, sizeof snap_cb);
        memcpy(tc_st, &snap_st, sizeof snap_st);
        t_fp.motion_id = 44;
        for (i = 0; i < 6; i++) {
            tc_frame(i == 1 ? HSD_PAD_A : 0, i == 3 ? 1.0f : 0, 0, i == 2);
            if (i == 0) {
                Geno_ComboContact(&t_fp, 0);
            }
            tc_st->action_time = 5 + i;
            targets[pass][i] = Geno_ComboTick(&t_gobj) ? GenoGame_TestLastTarget() : 0;
        }
        if (pass == 0) {
            d1 = GenoDefine_StateDigest(&t_fp);
        } else if (GenoDefine_StateDigest(&t_fp) != d1 || memcmp(targets[0], targets[1], sizeof targets[0]) != 0) {
            TestFail("combo: a resimulation from the snapshot must give the same block and the same links");
            rc = 1;
        }
    }
    if (targets[0][1] != GENO_TARGET(GENO_TGT_GENO, 0)) {
        TestFail("combo: the resimulated sequence must take L1>L2 on the press frame");
        rc = 1;
    }
    GenoGame_TestCapture(0, 0);
    Geno_TestRestore();
    /* a define without a combo block: the block is not folded */
    if (!tc_setup(TC_HEAD("11", "combo-none") TC_STATES "}]}")) {
        TestFail("combo: the plain define was not installed");
        Geno_TestRestore();
        return 1;
    }
    cb = GenoGame_ComboBlockOf(&t_fp);
    d0 = GenoDefine_StateDigest(&t_fp);
    cb->ki = 1234;
    cb->links = 3;
    if (GenoDefine_StateDigest(&t_fp) != d0) {
        TestFail("combo: a define without a combo block must keep its digest");
        rc = 1;
    }
    Geno_TestRestore();
    t_setup();
    return rc;
}
