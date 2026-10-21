/* Shared fill + state hash for the RunBrake/RunDirect/ShieldBreak-Shouldered/SpecialAir TU drivers
 * (game_tus.txt lines 209-216). Mirrored by tests/tu_ftco_s209/mod.rs: same fill order, same hash.
 * Every callee out of a TU is a recording stub; a bool-returning stub returns SR(id). */
#include <sysdolphin/baselib/jobj.h>
#include <melee/ft/ftdata.h>
static HSD_JObj JO[4];
static FighterBone BONES[4];
static ftData FD;
static struct ftData_x8 FD8;

static u32 PI(void* p)
{
    int i;
    if (p == NULL) return 0;
    if (p == (void*) &GO) return 1;
    if (p == (void*) &GO2) return 2;
    if (p == (void*) &F) return 3;
    if (p == (void*) &F2) return 4;
    for (i = 0; i < 4; i++) {
        if (p == (void*) &JO[i]) return 10 + i;
    }
    if (p == (void*) &F.xDF4[1]) return 20;
    if (p == (void*) &F2.xDF4[1]) return 21;
    if (p == (void*) &F.dmg.facing_dir_1) return 22;
    if (p == (void*) &F2.dmg.facing_dir_1) return 23;
    if (p == (void*) &F.x60C) return 24;
    return 9;
}

static void hit_fill(HitCapsule* h)
{
    h->unk_count = rnd() & 7;
    h->damage = rw();
    h->kb_angle = (s32) (rnd() % 361);
    h->element = rnd() % 16;
}

/* mv word 0/1: written as raw bits so every union view (runbrake, shouldered, specials) is random. */
static void mv_fill(Fighter* fp)
{
    union { float f; u32 u; } c;
    u32* w = (u32*) &fp->mv;
    switch (rnd() % 4) {
    case 0: w[0] = 0; break;
    case 1: w[0] = rnd() & 3; break;
    default: c.f = rw(); w[0] = c.u; break;
    }
    switch (rnd() % 4) {
    case 0: w[1] = 0; break;
    case 1: w[1] = rnd() & 3; break;
    default: c.f = rw(); w[1] = c.u; break;
    }
}

static void s209_fill(void)
{
    int i;
    zero(JO, sizeof JO);
    zero(BONES, sizeof BONES);
    zero(&FD, sizeof FD);
    zero(&FD8, sizeof FD8);
    for (i = 0; i < 4; i++) BONES[i].joint = &JO[i];
    FD.x8 = &FD8;
    FD8.x11 = (u8) (rnd() % 4);
    F.ft_data = &FD;
    F.parts = BONES;
    F.victim_gobj = &GO2;
    F2.victim_gobj = &GO;
    F2.parts = BONES;
    F.kind = (FighterKind) (rnd() % 33);
    F.ground_or_air = rnd() & 1;
    switch (rnd() % 4) {
    case 0: F.motion_id = ftCo_MS_ShieldBreakDownU; break;
    case 1: F.motion_id = ftCo_MS_ShoulderedWait; break;
    default: F.motion_id = (FtMotionId) (rnd() % 400); break;
    }
    switch (rnd() % 4) {
    case 0: F.facing_dir = 1.0f; break;
    case 1: F.facing_dir = -1.0f; break;
    default: F.facing_dir = rg(); break;
    }
    F.gr_vel = rw();
    F.grab_timer = rg();
    F.input.lstick[0].x = rg();
    F.input.lstick[0].y = rg();
    F.input.pressed_buttons = rnd() & 0x3FF;
    F.active_duration.lstick.x = rnd() & 0xFF;
    F.cmd_vars[0] = rnd() & 1;
    F.cmd_vars[1] = rnd() & 1;
    mv_fill(&F);
    F.co_attrs.ground_friction = rw();
    F.co_attrs.max_run_brake_frames = rw();
    F.co_attrs.shield_break_initial_velocity = rw();
    F.x197C = (rnd() & 1) ? &GO2 : NULL;
    F.x2222_b6 = rnd() & 1;
    F.x2228_b7 = rnd() & 1;
    F.dmg.x1830_percent = rw();
    F.self_vel.x = rw();
    F.self_vel.y = rw();
    hit_fill(&F.xDF4[1]);
    F2.facing_dir = rg();
    F2.ground_or_air = rnd() & 1;
    mv_fill(&F2);
    hit_fill(&F2.xDF4[1]);
    CD.x58_someLStickXThreshold = rg();
    CD.x42C = rw();
    CD.run_dash_turn_friction_multiplier = rw();
    CD.x410 = (s32) (rnd() % 100);
    CD.x4A0 = rw();
    CD.x4A4 = rw();
    CD.x4A8 = rw();
    CD.x4AC = rw();
    CD.x3B0 = rw();
    CD.shouldered_anim_rate = rw();
    CD.x218 = rg();
    CD.x21C = rg();
    CD.x220 = rg();
    CD.x224 = (s32) (rnd() % 300);
}

static void hm_dmg(Fighter* fp)
{
    HM(B(fp->dmg.kb_applied));
    HM((u32) fp->dmg.x1848_kb_angle);
    HM(B(fp->dmg.facing_dir_1));
    HM((u32) fp->dmg.x184c_damaged_hurtbox);
    HM(B(fp->dmg.x1854_collpos.x));
    HM(B(fp->dmg.x1854_collpos.y));
    HM(B(fp->dmg.x1854_collpos.z));
    HM(fp->dmg.x1860_element);
}

static u32 state_hash(void)
{
    u32* w = (u32*) &F.mv;
    H = 0x811C9DC5u;
    HM(F.cmd_vars[0]);
    HM(F.cmd_vars[1]);
    HM(w[0]);
    HM(w[1]);
    HM(B(F.facing_dir));
    HM(B(F.self_vel.x));
    HM(B(F.self_vel.y));
    HM(F.x2222_b3);
    HM(F.x2227_b5);
    HM(F.accessory1_cb != NULL);
    hm_dmg(&F);
    hm_dmg(&F2);
    return H;
}
