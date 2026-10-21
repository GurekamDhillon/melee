/* TU: src/melee/ft/kinds/ftCommon/ftCo_DamageIce.c */
/* C oracle for src/tu/ftCo_DamageIce.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Matrix stubs write simple per-element values (one float op each, so no
 * contraction differs) and record what they read. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static FighterBone PARTS[8];
static HSD_JObj JO[2];
static float ATAN, RANDF;
static void TV(Vec3* v) { T(B(v->x)); T(B(v->y)); T(B(v->z)); }
static void TM(Mtx m) { int r, c; for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) T(B(m[r][c])); }
static u32 J(HSD_JObj* j) { return j == NULL ? 0 : j == &JO[0] ? 1 : j == &JO[1] ? 2 : 3; }

bool ftCo_80098928(Fighter_GObj* g) { T(1); return SR(1); }
bool ftCo_8009872C(Fighter_GObj* g) { T(2); return SR(2); }
void ftCo_80097D40(Fighter_GObj* g) { T(3); T(g == &GO); }
Fighter_Part ftParts_GetBoneIndex(Fighter* f, Fighter_Part p) { T(4); T(f == &F); T(p); return (Fighter_Part) (SRV[5] % 2 + 3); }
void lb_8000B1CC(HSD_JObj* j, Vec3* off, Vec3* out) { T(5); T(J(j)); T(off == NULL); out->x = j->translate.x; out->y = j->translate.y; out->z = j->rotate.x; }
Vec3* lbVector_Sub(Vec3* a, Vec3* b) { T(6); TV(a); TV(b); a->x -= b->x; a->y -= b->y; a->z -= b->z; return a; }
void ftCommon_UnlockECB(Fighter* f) { T(7); T(f == &F); }
bool ft_80082638(Fighter_GObj* g, ftCollisionBox* b) { T(8); T(g == &GO); T(b == &F.mv.co.damageice.ice_coll); return SR(3); }
bool ft_80082888(Fighter_GObj* g, ftCollisionBox* b) { T(9); T(g == &GO); T(b == &F.mv.co.damageice.ice_coll); return SR(4); }
void efAsync_Spawn(HSD_GObj* g, void* q, u32 kind, u32 id, HSD_JObj* j, ...)
{
    __builtin_va_list ap;
    __builtin_va_start(ap, j);
    T(10); T(g == &GO); T(q == &F.x60C); T(kind); T(id); T(J(j));
    if (kind == 3) {
        float* p = __builtin_va_arg(ap, float*);
        T(B(*p));
    } else {
        Vec3* v = __builtin_va_arg(ap, Vec3*);
        float* p = __builtin_va_arg(ap, float*);
        TV(v); T(B(*p));
    }
    __builtin_va_end(ap);
}
void ftCo_8009750C(Fighter_GObj* g) { T(11); T(g == &GO); }
void ftCo_800DD168(Fighter_GObj* g) { T(12); T(g == &GO); }
void ftCommon_8007DB58(Fighter_GObj* g) { T(13); T(g == &GO); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(14); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void ftCommon_InitGrab(Fighter* f, bool a, float t) { T(15); T(f == &F); T(a); T(B(t)); }
void PSMTXTrans(Mtx m, float x, float y, float z)
{ int r, c; T(16); T(B(x)); T(B(y)); T(B(z)); for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) m[r][c] = (float) (r * 4 + c) + y; m[2][3] = z; }
void HSD_MkRotationMtx(Mtx m, Vec3* v)
{ int r, c; T(17); TV(v); for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) m[r][c] = v->x - (float) (r * 4 + c); m[1][3] = v->z; }
void HSD_MtxQuat(Mtx m, Quaternion* q)
{ int r, c; T(18); T(B(q->x)); T(B(q->y)); T(B(q->z)); T(B(q->w)); for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) m[r][c] = q->w + (float) (r - c); }
void PSMTXTranspose(Mtx s, Mtx d) { int r, c; T(19); TM(s); for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) d[r][c] = s[r][c] * 0.5f; }
void PSMTXConcat(Mtx a, Mtx b, Mtx ab) { int r, c; T(20); TM(a); TM(b); for (r = 0; r < 3; r++) for (c = 0; c < 4; c++) ab[r][c] = a[r][c] - b[r][c]; }
void HSD_MtxGetTranslate(Mtx m, Vec3* v) { T(21); TM(m); v->x = m[0][3]; v->y = m[1][3]; v->z = m[2][3]; }
void HSD_JObjSetMtxDirtySub(HSD_JObj* j) { T(22); T(J(j)); }
void ftCo_8009E140(Fighter* f, bool b) { T(23); T(f == &F); T(b); }
void ftCommon_8007F824(Fighter_GObj* g) { T(24); T(g == &GO); }
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(25); T(f == &F); T(v); }
f32 HSD_Randf(void) { T(26); return RANDF; }
void ftColl_8007B0C0(Fighter_GObj* g, HurtCapsuleState s) { T(27); T(g == &GO); T(s); }
void ftColl_HurtboxInit(Fighter* f, FighterHurtCapsule* h, ftHurtboxInit* i)
{ T(28); T(f == &F); T(h == F.hurt_capsules); T(i->bone_idx); T(i->height); T(i->is_grabbable); TV(&i->a_offset); TV(&i->b_offset); T(B(i->scale)); }
void ftCommon_8007EBAC(Fighter* f, u32 a, u32 b) { T(29); T(f == &F); T(a); T(b); }
void ftCo_800886D8(Fighter* f, int id, u8 v, u8 p) { T(30); T(f == &F); T(id); T(v); T(p); }
bool ftCommon_GrabMash(Fighter* f, float v) { T(31); T(f == &F); T(B(v)); return SR(5); }
void ftCommon_CalcSelfAccel_DeaccelAir(Fighter* f) { T(32); T(f == &F); }
void ftCommon_Fall(Fighter* f, float a, float b) { T(33); T(f == &F); T(B(a)); T(B(b)); }
void ft_80084F3C(Fighter_GObj* g) { T(34); T(g == &GO); }
void ftCommon_8007D5D4(Fighter* f) { T(35); T(f == &F); }
bool ft_800824A0(Fighter_GObj* g, ftCollisionBox* b) { T(36); T(g == &GO); T(b == &F.mv.co.damageice.ice_coll); return SR(6); }
void ftKb_SpecialN_800F1F1C(Fighter_GObj* g, Vec3* v) { T(37); T(g == &GO); TV(v); v->z = (float) (SRV[7] % 5); }
void ftCommon_8007D7FC(Fighter* f) { T(38); T(f == &F); }
float atan2f(float y, float x) { T(39); T(B(y)); T(B(x)); return ATAN; }
void Camera_RequestQuake(CmQuakeKind k, Vec* v) { T(40); T(k); TV(v); }
void ft_PlaySFX(Fighter* f, enum_t id, u8 a, u8 b) { T(41); T(f == &F); T(id); T(a); T(b); }
void ftCo_80090780(Fighter_GObj* g) { T(42); T(g == &GO); }
void lbVector_Mirror(Vec3* a, Vec3* b) { T(43); TV(a); TV(b); a->x = -a->x; a->y = a->y + b->y; }
GroundOrAir ft_80081D0C(Fighter_GObj* g) { T(44); return SR(7); }
bool ftCo_800C5240(Fighter_GObj* g) { T(45); return SR(8); }
void ftCo_800C5A98(Fighter_GObj* g) { T(46); T(g == &GO); }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(47); T(g == &GO); }
void ft_80084EEC(Fighter_GObj* g) { T(48); T(g == &GO); }
bool ftCo_80096CC8(Fighter_GObj* g, int l) { T(49); T(g == &GO); T(l); return SR(9); }
void ft_80082B1C(Fighter_GObj* g) { T(50); T(g == &GO); }
void ft_800831CC(Fighter_GObj* g, bool (*a)(Fighter_GObj*, int), void (*b)(Fighter_GObj*))
{ T(51); T(g == &GO); T(a(g, 7)); b(g); }

static u32 cbid(void* cb)
{
    return cb == NULL ? 0 : cb == (void*) ftCo_DamageIce_OnHit ? 1 : cb == (void*) ftCo_DamageIce_OnHit2 ? 2 : 3;
}

static u32 state_hash(void)
{
    int i;
    H = 0x811C9DC5u;
    HM(F.x2219_b0); HM(F.x2227_b6); HM(F.x2222_b3);
    HM(B(F.self_vel.x)); HM(B(F.self_vel.y)); HM(B(F.self_vel.z));
    HM(B(F.x8c_kb_vel.x)); HM(B(F.x8c_kb_vel.y)); HM(B(F.x8c_kb_vel.z));
    HM(B(F.gr_vel)); HM(B(F.xF0_ground_kb_vel)); HM(B(F.grab_timer));
    HM(B(F.cur_pos.x)); HM(B(F.cur_pos.y)); HM(B(F.cur_pos.z));
    HM(F.mv.co.damageice.wall_hit_dir); HM(B(F.mv.co.damageice.rot_speed));
    HM(B(F.mv.co.damageice.ice_coll.top)); HM(B(F.mv.co.damageice.ice_coll.bottom));
    HM(B(F.mv.co.damageice.ice_coll.left.x)); HM(B(F.mv.co.damageice.ice_coll.left.y));
    HM(B(F.mv.co.damageice.ice_coll.right.x)); HM(B(F.mv.co.damageice.ice_coll.right.y));
    HM(cbid(F.take_dmg_cb)); HM(cbid(F.take_dmg_2_cb));
    for (i = 0; i < 2; i++) {
        HM(B(JO[i].translate.y)); HM(B(JO[i].translate.z)); HM(B(JO[i].rotate.x));
    }
    return H;
}

int main(void)
{
    static const u32 jf[5] = { 0, JOBJ_MTX_DIRTY, JOBJ_USE_QUATERNION, JOBJ_MTX_INDEP_SRT, JOBJ_USER_DEF_MTX };
    int op, k, i;
    tu_init();
    for (op = 0; op < 20; op++) {
        for (k = 0; k < 3000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(PARTS, sizeof PARTS); zero(JO, sizeof JO);
            for (i = 0; i < 8; i++) PARTS[i].joint = &JO[i & 1];
            for (i = 0; i < 2; i++) {
                JO[i].flags = jf[rnd() % 5] | jf[rnd() % 5];
                JO[i].translate.x = rw(); JO[i].translate.y = rw(); JO[i].translate.z = rw();
                JO[i].rotate.x = rw(); JO[i].rotate.y = rw(); JO[i].rotate.z = rw(); JO[i].rotate.w = rw();
            }
            ATAN = rg(); RANDF = rg();
            CD.x790_damageice_unk = rw(); CD.x794_damageice_unk = rw(); CD.x798_damageice_unk = rg();
            CD.damageice_rot_speed_min = rw(); CD.damageice_rot_speed_max = rw();
            CD.damageice_dmg_time_reduction_mult = rw(); CD.damageice_ice_size = rw();
            CD.damageice_gravity_mult = rw(); CD.damageice_min_speed = rw();
            CD.damageice_speed_mult_on_break = rw(); CD.damageicejump_escape_time = rw();
            F.parts = PARTS;
            F.gobj = &GO;
            F.x34_scale.y = rw();
            F.co_attrs.damageice_ice_size = rw(); F.co_attrs.x150_damageice_unk = rw();
            F.co_attrs.x154_damageice_unk = rw(); F.co_attrs.damageicejump_vel_y = rw();
            F.co_attrs.damageicejump_vel_x_mult = rw(); F.co_attrs.gravity = rw();
            F.co_attrs.terminal_velocity = rw();
            F.ground_or_air = rnd() & 1;
            F.self_vel.x = rw(); F.self_vel.y = rw(); F.self_vel.z = rw();
            F.x8c_kb_vel.x = rw(); F.x8c_kb_vel.y = rw(); F.x8c_kb_vel.z = rw();
            F.gr_vel = rw(); F.xF0_ground_kb_vel = rw();
            F.dmg.x1838_percentTemp = rw();
            F.dmg.x1860_element = (rnd() & 1) ? HitElement_Fire : rnd() % 16;
            F.grab_timer = rw();
            F.x2224_b2 = rnd() & 1;
            F.mv.co.damageice.rot_speed = rw();
            F.mv.co.damageice.wall_hit_dir = rnd() % 4;
            F.cur_anim_frame = (float) (rnd() % 7);
            F.coll_data.env_flags = rnd() & (Collide_RightWallHug | Collide_LeftWallHug | Collide_CeilingHug);
            F.coll_data.ecb.left.x = rw(); F.coll_data.ecb.left.y = rw();
            F.coll_data.ecb.right.x = rw(); F.coll_data.ecb.right.y = rw();
            F.coll_data.ecb.top.y = rw();
            F.coll_data.right_facing_wall.normal.x = rg(); F.coll_data.right_facing_wall.normal.y = rg();
            F.coll_data.left_facing_wall.normal.x = rg(); F.coll_data.left_facing_wall.normal.y = rg();
            F.coll_data.ceiling.normal.x = rg(); F.coll_data.ceiling.normal.y = rg();
            F.cur_pos.x = rw(); F.cur_pos.y = rw(); F.cur_pos.z = rw();
            F.x68C_transNPos.y = rw(); F.x68C_transNPos.z = rw();
            F.facing_dir = rg();
            F.input.lstick[0].x = rg();
            if (op == 16) F.mv.co.damageicejump.escape_timer = (float) ((int) (rnd() % 5) - 1);
            switch (op) {
            case 0: ftCo_80090984(&GO); break;
            case 1: ftCo_800909D0(&F); break;
            case 2: ftCo_80090AC0(&F); break;
            case 3: ftCo_DamageIce_OnHit(&GO); break;
            case 4: ftCo_DamageIce_Init(&GO); break;
            case 5: ftCo_DamageIce_HitWhileFrozen(&GO); break;
            case 6: ftCo_DamageIce_OnHit2(&GO); break;
            case 7: ftCo_DamageIce_Anim(&GO); break;
            case 8: ftCo_DamageIce_IASA(&GO); break;
            case 9: ftCo_DamageIce_Phys(&GO); break;
            case 10: ftCo_DamageIce_Coll(&GO); break;
            case 11: ftCo_DamageIce_InAirUpdate(&GO); break;
            case 12: {
                Vec3 n, v;
                n.x = rg(); n.y = rg(); n.z = rg(); v.x = rw(); v.y = rw(); v.z = rw();
                ftCo_DamageIce_Collide(&GO, &n, &v);
                break;
            }
            case 13: ftCo_80091854(&GO); break;
            case 14: ftCo_DamageIceJump_IASA(&GO); break;
            case 15: ftCo_DamageIceJump_Phys(&GO); break;
            case 16: ftCo_DamageIceJump_Anim(&GO); break;
            case 17: ftCo_DamageIceJump_Coll(&GO); break;
            case 18: ret = B(ftCo_Speed(&F)); break;
            case 19: { float p = rw(); ftCo_SpawnEffect_x415(&GO, &JO[1], &p); break; }
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
