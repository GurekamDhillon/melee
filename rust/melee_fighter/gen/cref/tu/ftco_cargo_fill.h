/* Shared fill + state hash for the CaptureWait/CaptureYoshi/Cargo TU drivers (ftCo_CaptureWaitKirby.c ..
 * ftCo_CargoThrow.c). Mirrored by tests/tu_ftco_cargo/mod.rs: same fill order, same hash. */
#include <sysdolphin/baselib/jobj.h>
#include <melee/ft/kinds/ftDonkey/types.h>
#include <melee/ft/kinds/ftCaptain/types.h>
static ftDonkeyAttributes DA;
static HSD_JObj JO;

static u32 PI(void* p)
{
    if (p == NULL) return 0;
    if (p == (void*) &GO) return 1;
    if (p == (void*) &GO2) return 2;
    if (p == (void*) &F) return 3;
    if (p == (void*) &F2) return 4;
    return 9;
}

static void cargo_fill(void)
{
    zero(&DA, sizeof DA);
    zero(&JO, sizeof JO);
    GO.hsd_obj = &JO;
    F.victim_gobj = &GO2;
    F.x2CC = &DA;
    DA.x4_motion_state = (s32) (rnd() % 400);
    DA.cargo_hold.x24_JUMP_STARTUP_LAG = rw();
    DA.cargo_hold.x28_LANDING_LAG = rw();
    F.ground_or_air = rnd() & 1;
    F.motion_id = (FtMotionId) (rnd() % 400);
    F.facing_dir = rg();
    F.facing_dir1 = rg();
    F.cur_anim_frame = rw();
    F.grab_timer = rw();
    F.input.lstick[0].x = rg();
    F.input.lstick[0].y = rg();
    F.input.pressed_buttons = rnd() & 0x3FF;
    F.active_timer.lstick.x = rnd() & 0xFF;
    F.active_timer.lstick.y = rnd() & 0xFF;
    F.mv.ca.specialhi.vel.x = rw();
    F.mv.ca.specialhi.vel.y = rw();
    F.mv.co.capturekirby.scale.x = rw();
    F.mv.co.capturekirby.scale.y = rw();
    F.mv.co.capturekirby.scale.z = rw();
    F.x221C_b6 = rnd() & 1;
    F2.facing_dir = rg();
    CD.tap_jump_threshold = rg();
    CD.tap_jump_window = (int) (rnd() % 300);
    CD.x464 = rg();
    CD.x468 = rw();
    CD.dash_smash_stick_threshold = rg();
    CD.dash_smash_window = (int) (rnd() % 300);
    CD.x44 = rw();
    CD.x98 = rg();
    CD.attackhi3_stick_threshold_y = rg();
    CD.xB0 = rg();
    JO.flags = rnd();
}

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.facing_dir));
    HM(B(F.grab_timer));
    HM(F.active_timer.lstick.x);
    HM(F.active_timer.lstick.y);
    HM(B(F.mv.ca.specialhi.vel.x));
    HM(B(F.mv.ca.specialhi.vel.y));
    HM((u32) F.mv.co.cargokneebend.x0);
    HM((u32) F.mv.co.cargokneebend.x4);
    HM((u32) F.mv.co.capturekirby.x18);
    HM(B(JO.scale.x));
    HM(B(JO.scale.y));
    HM(B(JO.scale.z));
    HM(F.accessory1_cb != NULL);
    HM(F.x2226_b2);
    HM(F.invisible);
    HM(F.x221B_b5);
    HM(F.x221B_b7);
    HM(PI(F.x1A5C));
    HM(PI(F.victim_gobj));
    HM(F.ground_or_air);
    return H;
}
