/* Shared fighter fill + state hash for the ftCo_DownDamage/DownSpot/DownStand/Escape/EscapeAir/
 * Fall/FallAerial/FallSpecial drivers (included after tu_common.h). Mirrored field for field,
 * draw for draw, by tests/tu_ftco_dfe/mod.rs. */
static u8 DF_DAT[256];
static u8 DF_SKEL[4];
static const int DF_MS[7] = { ftCo_MS_DownBoundU, ftCo_MS_DownWaitU, ftCo_MS_DownDamageU,
                              ftCo_MS_DownBoundD, ftCo_MS_DownWaitD, ftCo_MS_DownDamageD, 0 };

static void df_fill(void)
{
    int i;
    u32* w = (u32*) &F.mv;
    CD.x428 = rnd() % 300; CD.x244 = rg(); CD.x20_radians = rg(); CD.x31C = rg();
    CD.x320 = rnd() % 8; CD.x324 = rnd() % 3; CD.x314 = rg(); CD.x318 = rnd() % 8;
    CD.walk_fast_stick_threshold = rg(); CD.x334 = rnd() % 40;
    CD.escapeair_deadzone.x = rg(); CD.escapeair_deadzone.y = rg(); CD.escapeair_force = rw();
    CD.x340 = rg(); CD.x344 = rw(); CD.escapeair_decay = rg(); CD.x444 = rg(); CD.x448 = rg();
    CD.x25C = rg();
    for (i = 0; i < 7; i++) {
        if (rnd() & 1) w[i] = B(rw()); else w[i] = rnd() % 64;
    }
    F.motion_id = DF_MS[rnd() % 7];
    F.x2224_b2 = rnd() & 1;
    F.dmg.x1838_percentTemp = rw();
    if (rnd() % 3 == 0) F.facing_dir = rg(); else if (rnd() & 1) F.facing_dir = 1.0f; else F.facing_dir = -1.0f;
    F.ground_or_air = rnd() & 1;
    F.coll_data.env_flags = rnd();
    F.coll_data.floor.index = (rnd() & 1) ? -1 : (int) (rnd() % 5);
    F.coll_data.floor.flags = rnd() & 0x100;
    F.input.lstick[0].x = rg(); F.input.lstick[0].y = rg();
    F.input.cstick[0].x = rg(); F.input.cstick[0].y = rg();
    F.input.pressed_buttons = (rnd() & 0x6F) | ((rnd() & 1) ? HSD_PAD_LR : 0);
    F.input.held_buttons[0] = (rnd() & 1) ? HSD_PAD_LR : 0;
    F.active_timer.lstick.x = rnd() % 8; F.active_timer.lstick.y = rnd() % 8;
    switch (rnd() % 3) {
    case 0: F.kind = Ft_Kind_Samus; break;
    case 1: F.kind = Ft_Kind_Yoshi; break;
    default: F.kind = rnd() % 27; break;
    }
    F.throw_flags = (rnd() & 0xFF) << 8; F.throw_flags_b3 = rnd() & 1;
    F.cmd_vars[0] = rnd() & 1;
    F.x5F4_arr[0].idx = rnd() % 3;
    F.dat_attrs = DF_DAT;
    F.self_vel.x = rw(); F.self_vel.y = rw(); F.self_vel.z = rw();
    F.co_attrs.air_drift_max = rw(); F.co_attrs.gravity = rg(); F.co_attrs.terminal_velocity = rg();
    F.co_attrs.fast_fall_velocity = rg(); F.co_attrs.air_drift_stick_mul = rg();
    F.co_attrs.aerial_drift_base = rg(); F.co_attrs.aerial_friction = rg();
    F.fall_fast = rnd() & 1; F.is_sandbag = rnd() & 1;
    F.cur_anim_frame = rg();
    F.x8AC_animSkeleton = (HSD_JObj*) DF_SKEL;
    F.x8_spawnNum = rnd() % 6;
    F.cur_pos.y = rw(); F.x2EC = rw();
    F.x618_player_id = rnd() % 4;
    F.gr_vel = rw();
    F.gobj = &GO;
}

static void df_hash(void)
{
    int i;
    u32* w = (u32*) &F.mv;
    for (i = 0; i < 7; i++) HM(w[i]);
    HM(B(F.facing_dir));
    HM(F.throw_flags >> 8);
    HM(F.throw_flags_b0); HM(F.throw_flags_b1); HM(F.throw_flags_b2); HM(F.throw_flags_b3);
    HM(F.throw_flags_b4); HM(F.throw_flags_b5); HM(F.throw_flags_b6); HM(F.throw_flags_b7);
    HM(F.cmd_vars[0]);
    HM(B(F.self_vel.x)); HM(B(F.self_vel.y)); HM(B(F.self_vel.z));
    HM(B(F.gr_vel)); HM(F.x221D_b5);
}

/* Any-NaN: a NaN's sign/payload depends on x86 operand order the C build does not fix. */
static u32 BN(float f) { return f != f ? 0x7FC00000u : B(f); }

/* Recording stubs: T(id) T(gobj is GO) [, return a draw from SRV]. */
static u32 chk(int id) { return (SRV[10 + id % 6] >> (id % 7)) & 1; }
#define DS_V(name, id) void name(Fighter_GObj* g) { T(id); T(g == &GO); }
#define DS_CHK(name, id) bool name(Fighter_GObj* g) { T(id); T(g == &GO); return chk(id); }
#define DS_FV(name, id) void name(Fighter* f) { T(id); T(f == &F); }
