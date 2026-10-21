/* Shared fill + state hash for the Ottotto/Pass/Passive/PassiveCeil/PassiveStand/PassiveWall/
 * Rebound/Run TU drivers. Mirrored by tests/tu_ftco_misc8/mod.rs: same fill order, same hash.
 * Included after tu_common.h. */
#include <sysdolphin/baselib/jobj.h>
static ftData FD;
static FtSFX SFX;
static HSD_JObj JO;
static FighterBone PARTS[2];

static u32 PI(void* p)
{
    if (p == NULL) return 0;
    if (p == (void*) &GO) return 1;
    if (p == (void*) &GO2) return 2;
    if (p == (void*) &F) return 3;
    if (p == (void*) &F.x60C) return 4;
    if (p == (void*) &JO) return 5;
    return 9;
}

static void m8_fill(void)
{
    int i;
    u32* mv = (u32*) &F.mv;
    zero(&FD, sizeof FD); zero(&SFX, sizeof SFX); zero(&JO, sizeof JO); zero(PARTS, sizeof PARTS);
    F.ft_data = &FD; FD.x4C_sfx = &SFX; F.parts = PARTS; PARTS[0].joint = &JO;
    SFX.x18 = (int) rnd(); SFX.x24 = (int) rnd();
    CD.x478 = rw(); CD.x47C = rw(); CD.x464 = rg(); CD.x468 = (float) (rnd() % 300); CD.x46C = rw();
    CD.x470 = rw(); CD.x254 = rg(); CD.x3D8 = rg(); CD.x3DC = rw(); CD.x760 = (int) (rnd() % 3);
    CD.x764 = (int) rnd(); CD.x250 = (float) (rnd() % 10); CD.tap_jump_threshold = rg();
    CD.passive_wall_vel_y_base = rg(); CD.x58_someLStickXThreshold = rg(); CD.x430 = rw();
    CD.x410 = (int) rnd(); CD.x440 = rg(); CD.run_accel_taper_gain = rg();
    CD.run_dash_turn_friction_multiplier = rg();
    F.coll_data.env_flags = (int) rnd(); F.is_sandbag = rnd() & 1; F.coll_data.floor.index = (int) (rnd() % 8);
    F.facing_dir = rg(); F.cur_pos.x = rw(); F.cur_pos.y = rw(); F.cur_pos.z = rg();
    F.self_vel.x = rg(); F.self_vel.y = rw(); F.self_vel.z = rg(); F.gr_vel = rw();
    F.input.lstick[0].x = rg(); F.input.lstick[0].y = rg();
    F.active_timer.lstick.x = rnd() % 256; F.active_timer.lstick.y = rnd() % 256;
    F.input.held_buttons[0] = rnd(); F.x618_player_id = rnd() % 4; F.player_id = rnd() % 6;
    F.ground_or_air = rnd() & 1; F.dmg.x191C = rw(); F.dmg.facing_dir = rg();
    F.co_attrs.clank_animation_length = rw(); F.co_attrs.passiveceil_vel_x = rg();
    F.co_attrs.passivewall_vel_x = rg(); F.co_attrs.wall_jump_horizontal_velocity = rg();
    F.co_attrs.wall_jump_vertical_velocity = rw(); F.co_attrs.gravity = rg();
    F.co_attrs.terminal_velocity = rw(); F.co_attrs.aerial_friction = rg();
    F.co_attrs.run_animation_scaling = rw(); F.co_attrs.dash_accel_mul = rg();
    F.co_attrs.dash_accel_base = rg();
    if (F.co_attrs.dash_accel_base != F.co_attrs.dash_accel_base) F.co_attrs.dash_accel_base = 0.5f; /* NaN+NaN picks an operand by codegen order */
    F.co_attrs.dash_max_velocity = rw(); F.co_attrs.ground_friction = rg();
    F.coll_data.ecb.top.y = rw(); F.coll_data.ecb.left.x = rg(); F.coll_data.ecb.left.y = rw();
    F.coll_data.ecb.right.x = rg(); F.coll_data.ecb.right.y = rw();
    F.x68C_transNPos.y = rw(); F.x68C_transNPos.z = rg();
    F.x67E = rnd() % 20; F.x1969_walljumpUsed = rnd() & 1; F.cmd_vars[0] = rnd() & 1;
    F.motion_id = (rnd() & 1) ? ftCo_MS_PassiveWall : (FtMotionId) (rnd() % 400);
    F.throw_flags_b0 = rnd() & 1; F.throw_flags_b3 = rnd() & 1;
    F.cur_anim_frame = rw(); F.frame_speed_mul = rg();
    /* motion vars: word 0/1 are floats or small ints, 2/3 small ints */
    for (i = 0; i < 2; i++) {
        if (rnd() & 1) { float v = rg(); mv[i] = B(v); } else { mv[i] = rnd() % 3; }
    }
    mv[2] = rnd() & 1; mv[3] = rnd() % 4;
}

static u32 state_hash(void)
{
    int i;
    u32* mv = (u32*) &F.mv;
    H = 0x811C9DC5u;
    HM(B(F.self_vel.x)); HM(B(F.self_vel.y)); HM(B(F.self_vel.z)); HM(B(F.gr_vel));
    HM(B(F.cur_pos.x)); HM(B(F.cur_pos.y)); HM(B(F.cur_pos.z)); HM(B(F.facing_dir));
    HM(F.active_timer.lstick.x); HM(F.active_timer.lstick.y); HM(F.cmd_vars[0]);
    HM(F.throw_flags_b0); HM(F.throw_flags_b3);
    for (i = 0; i < 4; i++) HM(mv[i]);
    return H;
}

#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(PI(g)); }
#define BSTUB(name, id) bool name(Fighter_GObj* g) { T(id); T(PI(g)); return SR(id); }
#define CMS(id) void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o) \
    { T(id); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
