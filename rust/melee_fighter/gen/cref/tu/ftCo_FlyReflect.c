/* TU: src/melee/ft/kinds/ftCommon/ftCo_FlyReflect.c */
/* C oracle for src/tu/ftCo_FlyReflect.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
/* The TU's stack-padding no-ops `GET_FIGHTER(0)` read address 0 (harmless on GC, a fault on x86):
 * make GET_FIGHTER null-tolerant before the TU is included. */
#include <melee/ft/inlines.h>
#undef GET_FIGHTER
#define GET_FIGHTER(gobj) ((Fighter*) ((gobj) ? HSD_GObjGetUserData(gobj) : NULL))
#include TU_SRC
#include "tu_common.h"

enum_t ftCo_DownBound_SfxIds[4] = { 9, 10, 11, 12 };
static float ATAN;
static void TV(Vec3* v) { T(B(v->x)); T(B(v->y)); T(B(v->z)); }
static u32 PI(void* p)
{
    if (p == NULL) return 0;
    if (p == (void*) &GO) return 1;
    if (p == (void*) &F) return 3;
    if (p == (void*) &F.x60C) return 4;
    return 9;
}

bool ftCo_800C23A0(Fighter_GObj* g) { T(1); T(PI(g)); return SR(1); }
bool ftCo_800C1D38(Fighter_GObj* g) { T(2); T(PI(g)); return SR(2); }
void ftKb_SpecialN_800F1F1C(Fighter_GObj* g, Vec3* v) { T(3); T(PI(g)); TV(v); v->z = (float) (SRV[7] % 5); }
void efAsync_Spawn(HSD_GObj* g, void* q, u32 kind, u32 id, HSD_JObj* j, ...)
{
    __builtin_va_list ap;
    Vec3* v;
    float* p;
    __builtin_va_start(ap, j);
    v = __builtin_va_arg(ap, Vec3*);
    p = __builtin_va_arg(ap, float*);
    __builtin_va_end(ap);
    T(4); T(PI(g)); T(PI(q)); T(kind); T(id); T(j == NULL); TV(v); T(B(*p));
}
void Camera_RequestQuake(CmQuakeKind k, Vec* v) { T(5); T(k); TV(v); }
float atan2f(float y, float x) { T(6); T(B(y)); T(B(x)); return ATAN; }
Vec3* lbVector_Add_xy(Vec3* a, Vec3* b) { T(7); TV(a); TV(b); a->x += b->x; a->y += b->y; return a; }
void lbVector_Mirror(Vec3* a, Vec3* b) { T(8); TV(a); TV(b); a->x = -a->x; a->y = a->y + b->y; }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(9); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
bool ft_80081F2C(Fighter_GObj* g) { T(10); T(PI(g)); return SR(3); }
bool ft_80082084(Fighter_GObj* g) { T(11); T(PI(g)); return SR(4); }
void ftCommon_8007EBAC(Fighter* f, u32 a, u32 b) { T(12); T(PI(f)); T(a); T(b); }
void ftColl_8007B760(Fighter_GObj* g, int v) { T(13); T(PI(g)); T(v); }
void ftCo_80097630(Fighter* f, enum_t* ids, float th) { T(14); T(PI(f)); T(ids == ftCo_DownBound_SfxIds); T(ids[0]); T(B(th)); }
void ft_80084DB0(Fighter_GObj* g) { T(15); T(PI(g)); }
void ftCo_80090184(Fighter_GObj* g) { T(16); T(PI(g)); }
void ftCo_DamageFly_Anim(Fighter_GObj* g) { T(17); T(PI(g)); }
void ftCo_DamageFly_IASA(Fighter_GObj* g) { T(18); T(PI(g)); }

static void fr_fill(void)
{
    CD.x1B0 = rg(); CD.x1B8 = (int) rnd(); CD.x1BC = rw(); CD.x1C0 = (float) (rnd() % 20);
    ATAN = rw();
    F.x8c_kb_vel.x = rw(); F.x8c_kb_vel.y = rw(); F.x8c_kb_vel.z = rg();
    F.self_vel.x = rw(); F.self_vel.y = rw(); F.self_vel.z = rg();
    F.cur_pos.x = rw(); F.cur_pos.y = rw(); F.cur_pos.z = rg();
    F.facing_dir = rg();
    F.x68C_transNPos.y = rw(); F.x68C_transNPos.z = rg();
    F.co_attrs.weight = rg();
    F.coll_data.env_flags = (int) rnd();
    F.coll_data.ecb.top.y = rw(); F.coll_data.ecb.left.x = rg(); F.coll_data.ecb.left.y = rw();
    F.coll_data.ecb.right.x = rg(); F.coll_data.ecb.right.y = rw();
    F.coll_data.right_facing_wall.normal.x = rg(); F.coll_data.right_facing_wall.normal.y = rg();
    F.coll_data.left_facing_wall.normal.x = rg(); F.coll_data.left_facing_wall.normal.y = rg();
    F.coll_data.ceiling.normal.x = rg(); F.coll_data.ceiling.normal.y = rg();
    F.mv.co.damage.x18 = rnd() % 4; F.mv.co.damage.x19 = rnd() % 5;
    F.motion_id = (rnd() & 1) ? ftCo_MS_FlyReflectWall : ftCo_MS_FlyReflectCeil;
}

static u32 fr_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.self_vel.x)); HM(B(F.self_vel.y)); HM(B(F.self_vel.z));
    HM(B(F.x8c_kb_vel.x)); HM(B(F.x8c_kb_vel.y)); HM(B(F.x8c_kb_vel.z));
    HM(B(F.cur_pos.x)); HM(B(F.cur_pos.y)); HM(B(F.cur_pos.z)); HM(B(F.facing_dir));
    HM(F.mv.co.damage.x18); HM(F.mv.co.damage.x19);
    return H;
}

#define NOPS 9
#define NCASE 4000

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0;
            tu_case(op, k);
            fr_fill();
            switch (op) {
            case 0: ret = ftCo_800C15F4(&GO); break;
            case 1: ret = ftCo_800C1718(&GO); break;
            case 2: ret = ftCo_800C17CC(&GO); break;
            case 3: {
                Vec3 n, o;
                n.x = rg(); n.y = rg(); n.z = rg();
                o.x = rw(); o.y = rw(); o.z = rg();
                ftCo_800C18A8(&GO, (rnd() & 1) ? ftCo_MS_FlyReflectWall : ftCo_MS_FlyReflectCeil, &n, &o);
                break;
            }
            case 4: ftCo_FlyReflect_Anim(&GO); break;
            case 5: ftCo_FlyReflect_IASA(&GO); break;
            case 6: ftCo_FlyReflect_Phys(&GO); break;
            default: ftCo_FlyReflect_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, fr_hash(), TR);
        }
    }
    return 0;
}
