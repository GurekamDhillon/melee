/* Shared fill + state hash for the ftCo_SpecialS/Squat/SquatRv/SquatWait/StopCeil/StopWall/Throw/
 * Thrown drivers (included after tu_common.h). Mirrored field for field, draw for draw, by
 * tests/tu_s217/mod.rs. */
#include <sysdolphin/baselib/jobj.h>
static ftData S_FD, S_FD2;
static ftCo_DatAttrs S_DA, S_DA2;
static HSD_JObj S_J[8];
static FighterBone S_PARTS[8];
static HSD_JObj S_JO, S_JO2;
static WaitStruct* const S_WS = (WaitStruct*) 0x1234;
static const int S_KINDS[16] = { 4, 13, 14, 5, 31, 9, 19, 6, 20, 1, 0, 33, 34, 40, 2, 18 };

static u32 PI(void* p)
{
    int i;
    if (p == NULL) return 0;
    if (p == (void*) &GO) return 1;
    if (p == (void*) &GO2) return 2;
    if (p == (void*) &F) return 3;
    if (p == (void*) &F2) return 4;
    if (p == (void*) &S_JO) return 5;
    if (p == (void*) &S_JO2) return 6;
    if (p == (void*) &F.coll_data) return 7;
    if (p == (void*) &F2.coll_data) return 8;
    for (i = 0; i < 8; i++) if (p == (void*) &S_J[i]) return 10 + i;
    if (p == (void*) &F.xDF4[0]) return 20;
    if (p == (void*) &F.xDF4[1]) return 21;
    if (p == (void*) &F2.xDF4[0]) return 22;
    if (p == (void*) &F2.xDF4[1]) return 23;
    if (p == (void*) &F2.dmg.facing_dir_1) return 24;
    if (p == (void*) &F.dmg.facing_dir_1) return 25;
    if (p == (void*) &CD.x380) return 26;
    if (p == (void*) S_WS) return 27;
    return 99;
}
static u32 chk(int id) { return (SRV[10 + id % 6] >> (id % 7)) & 1; }

static void s_fill_one(Fighter* f, HSD_GObj* self, HSD_GObj* other)
{
    int i;
    u32* w = (u32*) &f->mv;
    f->kind = S_KINDS[rnd() % 16];
    f->x688 = rnd() & 1;
    if (rnd() % 3 == 0) f->facing_dir = rg(); else if (rnd() & 1) f->facing_dir = 1.0f; else f->facing_dir = -1.0f;
    f->gr_vel = rw();
    f->co_attrs.specials_ground_speed_retention = rg();
    f->co_attrs.walk_max_vel = rw();
    f->co_attrs.weight = rw();
    f->input.pressed_buttons = rnd() & 0x3FF;
    f->input.lstick[0].x = rg(); f->input.lstick[0].y = rg();
    f->input.lstick[1].x = rg(); f->input.lstick[1].y = rg();
    f->input.cstick[0].x = rg(); f->input.cstick[0].y = rg();
    f->active_timer.lstick.x = rnd() % 3;
    f->coll_data.env_flags = rnd();
    f->coll_data.ecb.top.y = rw(); f->coll_data.ecb.bottom.y = rw();
    f->coll_data.ecb.left.x = rw(); f->coll_data.ecb.left.y = rw();
    f->coll_data.ecb.right.x = rw(); f->coll_data.ecb.right.y = rw();
    f->cur_pos.x = rw(); f->cur_pos.y = rw(); f->cur_pos.z = rw();
    f->x68C_transNPos.x = rw(); f->x68C_transNPos.y = rw(); f->x68C_transNPos.z = rw();
    f->x6A4_transNOffset.x = rw(); f->x6A4_transNOffset.y = rw(); f->x6A4_transNOffset.z = rw();
    f->self_vel.x = rw(); f->self_vel.y = rw(); f->self_vel.z = rw();
    f->motion_id = (rnd() & 1) ? 219 + (int) (rnd() % 4) : (int) (rnd() % 400);
    f->cmd_vars[0] = rnd() & 1;
    f->cur_anim_frame = rg();
    f->grab_timer = (rnd() & 1) ? 0.0f : rw();
    f->player_id = rnd() % 4;
    f->is_sub_fighter = rnd() & 1;
    f->throw_flags = (rnd() & 0xFF) << 8;
    f->throw_flags_b0 = rnd() & 1; f->throw_flags_b1 = rnd() & 1; f->throw_flags_b2 = rnd() & 1; f->throw_flags_b3 = rnd() & 1;
    f->throw_flags_b4 = rnd() & 1; f->throw_flags_b5 = rnd() & 1; f->throw_flags_b6 = rnd() & 1; f->throw_flags_b7 = rnd() & 1;
    f->x221B_b5 = rnd() & 1; f->x221B_b7 = rnd() & 1;
    f->x2219_b2 = rnd() & 1; f->x2222_b0 = rnd() & 1; f->x2226_b2 = rnd() & 1;
    f->victim_gobj = (rnd() % 4) ? other : NULL;
    f->x1A5C = (rnd() & 1) ? other : NULL;
    f->ground_or_air = rnd() & 1;
    for (i = 0; i < 2; i++) {
        f->xDF4[i].damage = rw();
        f->xDF4[i].unk_count = rnd() % 5;
        f->xDF4[i].kb_angle = (int) (rnd() % 361);
        f->xDF4[i].element = rnd() % 5;
    }
    f->x1988 = rnd() % 3;
    f->x1A70.x = rw(); f->x1A70.y = rw(); f->x1A70.z = rw();
    f->x34_scale.y = rg();
    f->x2174.x = rw(); f->x2174.y = rw(); f->x2174.z = rw();
    f->dmg.x1848_kb_angle = (int) (rnd() % 361);
    f->dmg.facing_dir_1 = rg();
    for (i = 0; i < 8; i++) w[i] = (rnd() & 1) ? B(rg()) : rnd() % 3;
    f->gobj = self;
    f->parts = S_PARTS;
}

static void s_init(void)
{
    int i;
    for (i = 0; i < 8; i++) S_PARTS[i].joint = &S_J[i];
}

static void s_fill(void)
{
    int i;
    zero(&S_FD, sizeof S_FD); zero(&S_FD2, sizeof S_FD2); zero(&S_DA, sizeof S_DA); zero(&S_DA2, sizeof S_DA2);
    zero(S_J, sizeof S_J); zero(&S_JO, sizeof S_JO); zero(&S_JO2, sizeof S_JO2);
    GO.hsd_obj = &S_JO; GO2.hsd_obj = &S_JO2;
    S_JO.flags = rnd(); S_JO2.flags = rnd();
    for (i = 0; i < 8; i++) S_J[i].flags = rnd();
    S_FD.x0 = &S_DA; S_FD2.x0 = &S_DA2; S_FD.x28 = S_WS;
    F.ft_data = &S_FD; F2.ft_data = &S_FD2;
    S_DA.weight = rw(); S_DA2.weight = rw();
    S_DA.weight_independent_throws_mask = rnd() & 15;
    s_fill_one(&F, &GO, &GO2);
    s_fill_one(&F2, &GO2, &GO);
    CD.x218 = rg(); CD.x220 = rg(); CD.x90 = rg(); CD.x94 = rg(); CD.x5F0 = rnd();
    CD.x98 = rg(); CD.attackhi3_stick_threshold_y = rg(); CD.xB0 = rg();
    CD.x348 = (int) (rnd() % 9); CD.x37C = rw(); CD.x10C = rw();
    CD.grab_timer_decrement = rw(); CD.x3C8 = rw();
}

static void s_hash_one(Fighter* f)
{
    int i;
    u32* w = (u32*) &f->mv;
    HM(B(f->facing_dir)); HM(B(f->gr_vel));
    HM(B(f->self_vel.x)); HM(B(f->self_vel.y)); HM(B(f->self_vel.z));
    HM(B(f->cur_pos.x)); HM(B(f->cur_pos.y)); HM(B(f->cur_pos.z));
    HM(f->throw_flags >> 8);
    HM(f->throw_flags_b0); HM(f->throw_flags_b1); HM(f->throw_flags_b2); HM(f->throw_flags_b3);
    HM(f->throw_flags_b4); HM(f->throw_flags_b5); HM(f->throw_flags_b6); HM(f->throw_flags_b7);
    HM(f->x221B_b5); HM(f->x221B_b7); HM(f->x2219_b2); HM(f->x2226_b2);
    HM(f->cmd_vars[0]); HM(B(f->grab_timer));
    HM(PI(f->victim_gobj)); HM(PI(f->x1A5C));
    HM(f->accessory1_cb != NULL); HM(f->x21EC != NULL);
    for (i = 0; i < 4; i++) HM(w[i]);
    HM(w[6]); HM(w[7]);
    HM(PI(f->mv.co.fighterthrow.victim));
    HM(B(f->dmg.kb_applied)); HM((u32) f->dmg.x1848_kb_angle); HM(B(f->dmg.facing_dir_1));
    HM((u32) f->dmg.x184c_damaged_hurtbox);
    HM(B(f->dmg.x1854_collpos.x)); HM(B(f->dmg.x1854_collpos.y)); HM(B(f->dmg.x1854_collpos.z));
    HM(f->dmg.x1860_element);
    HM(B(f->coll_data.last_pos.x)); HM(B(f->coll_data.last_pos.y)); HM(B(f->coll_data.last_pos.z));
    HM(B(f->coll_data.cur_pos.x)); HM(B(f->coll_data.cur_pos.y)); HM(B(f->coll_data.cur_pos.z));
}

static u32 state_hash(void)
{
    int i;
    H = 0x811C9DC5u;
    s_hash_one(&F);
    s_hash_one(&F2);
    HM(B(S_JO.translate.x)); HM(B(S_JO.translate.y)); HM(B(S_JO.translate.z));
    HM(B(S_JO2.translate.x)); HM(B(S_JO2.translate.y)); HM(B(S_JO2.translate.z));
    for (i = 0; i < 8; i++) { HM(B(S_J[i].translate.x)); HM(B(S_J[i].translate.y)); HM(B(S_J[i].translate.z)); }
    return H;
}

/* Recording stubs. */
#define STUB_V(name, id) void name(Fighter_GObj* g) { T(id); T(PI(g)); }
#define STUB_CHK(name, id) bool name(Fighter_GObj* g) { T(id); T(PI(g)); return chk(id); }
#define CMS_STUB void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o) \
    { T(9); T(PI(g)); T((u32) m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
