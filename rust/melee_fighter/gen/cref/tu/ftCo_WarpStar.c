/* TU: src/melee/ft/kinds/ftCommon/ftCo_WarpStar.c */
/* C oracle for src/tu/ftCo_WarpStar.rs: the whole TU included verbatim; every callee out of it is
 * a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static HSD_JObj JO;
static float BZ;

static u32 PI(void* p)
{
    if (p == NULL) return 0;
    if (p == (void*) &GO) return 1;
    if (p == (void*) &F) return 3;
    if (p == (void*) &IGO) return 5;
    if (p == (void*) &JO) return 6;
    if (p == (void*) &F.mv.co.warpstar.ecb) return 7;
    if (p == (void*) &F.coll_data) return 8;
    if (p == (void*) &F.cur_pos) return 9;
    if (p == (void*) &F.x60C) return 10;
    if (p == (void*) &GO2) return 11;
    return 99;
}

void it_80294430(Item_GObj* i, float a, float b) { T(1); T(PI(i)); T(B(a)); T(B(b)); }
HSD_AnimJoint* it_80294364(Item_GObj* i) { T(2); T(PI(i)); return (HSD_AnimJoint*) &GO2; }
void it_802944AC(Item_GObj* i, ftCollisionBox* e) { T(3); T(PI(i)); T(PI(e)); }
void it_802946B0(Item_GObj* i) { T(4); T(PI(i)); }
void it_802947CC(Item_GObj* i, Vec3* p) { T(5); T(PI(i)); T(PI(p)); }
void Item_8026ABD8(Item_GObj* i, Vec3* p, float v) { T(6); T(PI(i)); T(B(p->x)); T(B(p->y)); T(B(p->z)); T(B(v)); }
void ftCommon_8007D5D4(Fighter* f) { T(7); T(PI(f)); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(8); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
void ftColl_8007B6A0(Fighter_GObj* g) { T(9); T(PI(g)); }
void ftColl_8007B6EC(Fighter_GObj* g) { T(10); T(PI(g)); }
void ftCommon_8007E2FC(Fighter_GObj* g) { T(11); T(PI(g)); }
void HSD_JObjAddAnim(HSD_JObj* j, HSD_AnimJoint* a, HSD_MatAnimJoint* m, HSD_ShapeAnimJoint* s) { T(12); T(PI(j)); T(PI(a)); T(PI(m)); T(PI(s)); }
void ftAnim_80070734(HSD_JObj* j, float fr) { T(13); T(PI(j)); T(B(fr)); }
void HSD_JObjAnim(HSD_JObj* j) { T(14); T(PI(j)); }
void HSD_JObjSetMtxDirtySub(HSD_JObj* j) { T(15); T(PI(j)); }
float Stage_GetBlastZoneTopOffset(void) { T(16); return BZ; }
void ftCamera_800762F4(HSD_GObj* g) { T(17); T(PI(g)); }
void mpColl_80043680(CollData* c, Vec3* p) { T(18); T(PI(c)); T(PI(p)); }
void ftCommon_Fall(Fighter* f, float a, float b) { T(19); T(PI(f)); T(B(a)); T(B(b)); }
void ftCommon_CalcSelfAccel_AccelToVelClampedFrom(Fighter* f, float v, float a, float t, float fr)
{ T(20); T(PI(f)); T(B(v)); T(B(a)); T(B(t)); T(B(fr)); }
void ft_80083E64(Fighter_GObj* g, ftCollisionBox* e, HSD_GObjEvent cb) { T(21); T(PI(g)); T(PI(e)); T(cb != NULL); if (SR(21)) cb(g); }
float atan2f(float y, float x) { T(22); T(B(y)); T(B(x)); return rg(); }
void efAsync_Spawn(HSD_GObj* g, void* q, u32 kind, u32 id, HSD_JObj* j, ...)
{
    __builtin_va_list ap;
    Vec3* pos;
    float* param;
    __builtin_va_start(ap, j);
    pos = __builtin_va_arg(ap, Vec3*);
    param = __builtin_va_arg(ap, float*);
    __builtin_va_end(ap);
    T(23); T(PI(g)); T(PI(q)); T(kind); T(id); T(PI(j)); T(B(pos->x)); T(B(pos->y)); T(B(pos->z)); T(B(*param));
}
void Camera_RequestQuake(CmQuakeKind k, Vec* p) { T(24); T(k); T(B(p->x)); T(B(p->y)); T(B(p->z)); }
void ftCommon_8007EBAC(Fighter* f, u32 a, u32 b) { T(25); T(PI(f)); T(a); T(b); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.facing_dir)); HM(B(F.cur_pos.x)); HM(B(F.cur_pos.y)); HM(B(F.cur_pos.z));
    HM(B(F.self_vel.x)); HM(B(F.self_vel.y)); HM(B(F.self_vel.z));
    HM(B(F.mv.co.warpstar.cur_pos.x)); HM(B(F.mv.co.warpstar.cur_pos.y)); HM(B(F.mv.co.warpstar.cur_pos.z));
    HM(B(F.mv.co.warpstar.self_vel.x)); HM(B(F.mv.co.warpstar.self_vel.y)); HM(B(F.mv.co.warpstar.self_vel.z));
    HM(B(F.mv.co.warpstar.facing_dir)); HM((u32) F.mv.co.warpstar.x1C);
    HM(F.take_dmg_cb != NULL); HM(F.death2_cb != NULL);
    HM(F.x2219_b2); HM(F.x2222_b7); HM(F.x221E_b2);
    HM(B(JO.rotate.x)); HM(B(JO.rotate.y)); HM(B(JO.rotate.z)); HM(B(JO.rotate.w));
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 12; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&JO, sizeof JO);
            GO.hsd_obj = &JO;
            JO.flags = rnd() & 0x02000040;
            JO.translate.x = rw(); JO.translate.y = rw(); JO.translate.z = rw();
            BZ = rw();
            F.item_gobj = (Item_GObj*) &IGO;
            F.facing_dir = rg();
            F.cur_pos.x = rw(); F.cur_pos.y = rw(); F.cur_pos.z = rw();
            F.self_vel.x = rw(); F.self_vel.y = rw(); F.self_vel.z = rw();
            F.x2C4.y = rw();
            F.input.lstick[0].x = rg();
            F.co_attrs.warp_star_hitbox_scale = rw();
            F.co_attrs.ground_to_air_jump_momentum_multiplier = rg();
            F.co_attrs.jump_h_initial_velocity = rw();
            F.co_attrs.jump_v_initial_velocity = rw();
            F.co_attrs.jump_h_max_velocity = rw();
            F.coll_data.ecb.bottom.y = rw();
            F.coll_data.floor.normal.x = rg(); F.coll_data.floor.normal.y = rg();
            F.mv.co.warpstar.facing_dir = rg();
            F.mv.co.warpstar.x1C = (int) (rnd() % 4);
            F.mv.co.warpstar.cur_pos.x = rw();
            F.mv.co.warpstar.self_vel.x = rw(); F.mv.co.warpstar.self_vel.y = rw(); F.mv.co.warpstar.self_vel.z = rw();
            CD.x694 = rg(); CD.x698 = rg(); CD.x6A8 = rg(); CD.x438 = rg(); CD.x78 = rg();
            CD.warpstarfall_drift_scaling = rg(); CD.warpstarfall_drift_flat = rg(); CD.warpstarfall_drift_max = rw();
            if (CD.warpstarfall_drift_flat != CD.warpstarfall_drift_flat) CD.warpstarfall_drift_flat = 0.5f; /* NaN+NaN picks an operand by codegen order */
            switch (op) {
            case 0: ftCo_800C4724(&GO); break;
            case 1: ftCo_800C4858(&GO); break;
            case 2: ftCo_800C48AC(&GO); break;
            case 3: ftCo_WarpStarJump_Anim(&GO); break;
            case 4: ftCo_WarpStarJump_Phys(&GO); break;
            case 5: ftCo_WarpStarJump_Cam(&GO); ftCo_WarpStarFall_Cam(&GO); ftCo_WarpStarJump_IASA(&GO);
                ftCo_WarpStarJump_Coll(&GO); ftCo_WarpStarFall_Anim(&GO); ftCo_WarpStarFall_IASA(&GO); break;
            case 6: ftCo_800C4A38(&GO); break;
            case 7: ftCo_WarpStarFall_Phys(&GO); break;
            case 8: ftCo_WarpStarFall_Coll(&GO); break;
            case 9: case 10: case 11: ftCo_800C4C60(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
