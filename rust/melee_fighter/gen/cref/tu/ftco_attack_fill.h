/* Shared fighter fill + state hash for the ftCo_AppealS/Attack1/Attack100/AttackAir/AttackDash/
 * AttackHi3/AttackHi4/AttackLw3 drivers (included after tu_common.h). Mirrored field for field,
 * draw for draw, by tests/tu_ftco_attack/mod.rs. */
static MotionState AF_MS1[400], AF_MS2[400];
static ftData AF_FD;
static struct ftData_x8 AF_FDX8;
static FighterBone AF_PARTS[8];
static u8 AF_JOBJ[8][4];
static HSD_GObj IGO2;
static const int AF_KINDS[16] = { 0, 4, 6, 8, 9, 12, 18, 19, 20, 21, 23, 24, 25, 33, 34, 2 };

static void af_init(void)
{
    int i;
    for (i = 0; i < 400; i++) { AF_MS1[i].anim_id = i; AF_MS2[i].anim_id = 1000 + i; }
    for (i = 0; i < 8; i++) AF_PARTS[i].joint = (HSD_JObj*) AF_JOBJ[i];
    AF_FD.x8 = &AF_FDX8;
}
static u32 af_jobj(HSD_JObj* j) { int i; for (i = 0; i < 8; i++) if (j == (HSD_JObj*) AF_JOBJ[i]) return i + 1; return j == NULL ? 0 : 99; }
static u32 af_item(void* p) { return p == NULL ? 0 : p == (void*) &IGO ? 1 : p == (void*) &IGO2 ? 2 : 99; }

static void af_fill(void)
{
    CD.attackhi3_stick_threshold_y = rg(); CD.x20_radians = rg(); CD.xCC = rg(); CD.xD0 = rw();
    CD.x21C = rg(); CD.x1C = rnd() % 8; CD.xB0 = rg(); CD.xDC = rg(); CD.xE0 = rg();
    CD.x50 = rw(); CD.x68 = rw(); CD.x414 = (int) (rnd() % 5) - 1;
    F.input.pressed_buttons = rnd() & 0x30F;
    F.input.released_buttons = rnd() & 0x100;
    F.input.held_buttons[0] = (rnd() & 1) ? HSD_PAD_LR : 0;
    F.input.lstick[0].x = rg(); F.input.lstick[0].y = rg();
    F.input.cstick[0].x = rg(); F.input.cstick[0].y = rg();
    F.active_timer.lstick.y = rnd() % 8;
    F.item_gobj = (rnd() & 1) ? (Item_GObj*) &IGO : NULL;
    F.kind = AF_KINDS[rnd() % 16];
    F.allow_interrupt = rnd() & 1; F.x2218_b1 = rnd() & 1; F.x2218_b2 = rnd() & 1; F.x2224_b1 = rnd() & 1;
    { float a = rg(); F.hitlag_mul = a + (float) (rnd() % 3); }
    F.unk_msid = 43 + rnd() % 4;
    F.cmd_vars[0] = rnd() & 1;
    F.mv.co.attack100.x0 = rnd() & 1; F.mv.co.attack100.x4 = rnd() & 1;
    F.throw_flags = (rnd() & 0xFF) << 8; F.throw_flags_b3 = rnd() & 1; F.throw_flags_b0 = rnd() & 1;
    F.x1A54 = rnd() % 6; F.co_attrs.rapid_jab_window = rnd() % 6;
    F.co_attrs.jab_2_input_window = rw(); F.co_attrs.jab_3_input_window = rw();
    F.x209C = rnd() % 3; F.x683 = rnd() % 8; F.x686 = rnd() & 1; F.x687 = rnd() & 1; F.x689 = rnd() & 1;
    F.x68B = rnd() % 8;
    F.cur_anim_frame = rg(); F.frame_speed_mul = rg();
    if (rnd() % 3 == 0) F.facing_dir = rg(); else if (rnd() & 1) F.facing_dir = 1.0f; else F.facing_dir = -1.0f;
    F.co_attrs.ground_friction = rw();
    F.player_id = rnd() % 4; F.is_sub_fighter = rnd() & 1;
    F.x18 = (rnd() & 1) ? 200 : 300;
    F.x1C_actionStateList = AF_MS1; F.x20_actionStateList = AF_MS2;
    F.ft_data = &AF_FD; AF_FDX8.x10 = rnd() % 4;
    F.parts = AF_PARTS;
    F.gobj = &GO;
}

static void af_hash(void)
{
    HM(F.allow_interrupt); HM(F.x2218_b1); HM(F.x2218_b2); HM(F.x2224_b1);
    HM(B(F.hitlag_mul)); HM(F.unk_msid); HM(F.cmd_vars[0]);
    HM(F.mv.co.attack100.x0); HM(F.mv.co.attack100.x4);
    HM(F.throw_flags >> 8);
    HM(F.throw_flags_b0); HM(F.throw_flags_b1); HM(F.throw_flags_b2); HM(F.throw_flags_b3);
    HM(F.throw_flags_b4); HM(F.throw_flags_b5); HM(F.throw_flags_b6); HM(F.throw_flags_b7);
    HM(F.x1A54); HM(F.x209C); HM(B(F.facing_dir));
    HM(af_item(F.item_gobj)); HM(af_item(F.x1978));
}

/* Recording stubs: T(id) T(gobj is GO) [, return a draw from SRV]. */
static u32 chk(int id) { return (SRV[10 + id % 6] >> (id % 7)) & 1; }
static float angle_of(u32 v) { return (float) ((int) (v % 41) - 20) / 20.0f; }
#define STUB_V(name, id) void name(Fighter_GObj* g) { T(id); T(g == &GO); }
#define STUB_CHK(name, id) bool name(Fighter_GObj* g) { T(id); T(g == &GO); return chk(id); }
