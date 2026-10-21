/* TU: src/melee/ft/kinds/ftCommon/ftCo_CatchPull.c */
/* C oracle for src/tu/ftCo_CatchPull.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static FighterBone PARTS[8], PARTS2[8];
static unsigned char JB[64];
static struct ftData FD;
static struct ftData_x8 X8;
static ftYoshiAttributes YA;
static Item IT;
static Vec3 POS[2];
static int npos;

void ftAnim_SetAnimRate(Fighter_GObj* g, float r) { T(1); T(g == &GO); T(B(r)); }
void ftAnim_8006EBA4(Fighter_GObj* g) { T(2); T(g == &GO); }
void it_802A7840(HSD_GObj* g) { T(3); T(g == &IGO); }
void it_802BAA94(Item_GObj* g) { T(4); T(g == &IGO); }
int Geno_MotionAnimRow(Fighter* fp, int m) { T(5); T(fp == &F); T(m); return SR(4) ? 0 : -1; }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(6); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void fn_800DA1D8(Fighter_GObj* g) { T(7); T(g == &GO); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(8); T(g == &GO); return SR(1); }
void ftCommon_CalcGroundAccel_Deaccel(Fighter* fp, float f) { T(9); T(fp == &F); T(B(f)); }
void ftCommon_SetSelfMovementFromGroundedMovement(Fighter_GObj* g) { T(10); T(g == &GO); }
void ft_800841B8(Fighter_GObj* g, HSD_GObjEvent cb) { T(11); T(g == &GO); T(cb == fn_800DA004); cb(g); }
void ftCo_800DC920(Fighter_GObj* g, Fighter_GObj* v) { T(12); T(g == &GO); T(v == &GO2); }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(13); T(g == &GO); T(g == &GO2); }
void lb_8000B1CC(HSD_JObj* j, Vec3* off, Vec3* out)
{ T(14); T((u32) ((unsigned char*) j - JB)); T(off == NULL); *out = POS[npos++ & 1]; }
Fighter_Part ftParts_GetBoneIndex(Fighter* fp, Fighter_Part p) { T(15); T(fp == &F2); T(p); return SRV[10] % 8; }
void ftCo_800DA698(Fighter_GObj* g, bool a) { T(16); T(g == &GO); T(a); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.gr_vel)); HM(F.throw_flags_b3); HM(F.throw_flags == 0); HM(F.x221B_b7); HM(F.accessory1_cb == fn_800DA190);
    HM(F.mv.co.capturedamage.x18 == NULL ? 0xFFu : (u32) ((unsigned char*) F.mv.co.capturedamage.x18 - JB));
    return H;
}

static const int KT[8] = { Ft_Kind_Link, Ft_Kind_CLink, Ft_Kind_Samus, Ft_Kind_Yoshi, 6, 13, 20, 0 };

int main(void)
{
    int op, k, i;
    tu_init();
    for (i = 0; i < 8; i++) { PARTS[i].joint = (HSD_JObj*) (JB + i * 4); PARTS2[i].joint = (HSD_JObj*) (JB + 32 + i * 2); }
    for (op = 0; op < 8; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            npos = 0;
            zero(&X8, sizeof X8); zero(&YA, sizeof YA); zero(&IT, sizeof IT);
            X8.x11 = rnd() % 8;
            FD.x8 = &X8;
            F.ft_data = &FD; F.parts = PARTS; F2.parts = PARTS2;
            F.victim_gobj = &GO2;
            IGO.user_data = &IT;
            F.kind = (rnd() & 1) ? KT[rnd() % 8] : (int) (rnd() % 34);
            F.motion_id = (rnd() & 1) ? 0xD4 : 0xD5;
            F.cur_anim_frame = (float) (rnd() % 50) / 2.0f;
            YA.x124 = (float) (rnd() % 10);
            YA.x128 = YA.x124 + (float) (rnd() % 12);
            for (i = 0; i < 12; i++) YA.x12C[i] = rnd();
            F.dat_attrs = &YA;
            F.u.lk.xC = ((rnd() & 1) || op == 0) ? (Item_GObj*) &IGO : NULL;
            F.u.ss.x223C = ((rnd() & 1) || op == 0) ? (Item_GObj*) &IGO : NULL;
            IT.xDD4_itemVar.linkhookshot.xC = (HSD_JObj*) (JB + 60);
            IT.xDD4_itemVar.linkhookshot.x14 = rnd() & 1;
            F.throw_flags_b3 = rnd() & 1;
            F.x221B_b7 = rnd() & 1;
            F.gr_vel = rg(); F.facing_dir = rg();
            F.x2170 = rw(); CD.x34C = rw(); CD.x350 = rw(); F.co_attrs.walk_max_vel = rw();
            CD.x64 = rg(); F.co_attrs.ground_friction = rg();
            F2.x2226_b2 = (rnd() % 4) == 0;
            for (i = 0; i < 2; i++) { POS[i].x = rw(); POS[i].y = rw(); POS[i].z = rw(); }
            F.mv.co.capturedamage.x18 = (HSD_JObj*) (JB + 40);
            switch (op) {
            case 0: fn_800D9CE8(&GO); break;
            case 1: ftCo_CatchPull_Anim(&GO); break;
            case 2: ftCo_CatchPull_IASA(&GO); break;
            case 3: ftCo_CatchPull_Phys(&GO); break;
            case 4: ftCo_CatchPull_Coll(&GO); break;
            case 5: fn_800DA004(&GO); break;
            case 6: fn_800DA054(&GO); break;
            case 7: fn_800DA190(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
