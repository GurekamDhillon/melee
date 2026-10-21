/* TU: src/melee/ft/kinds/ftCommon/ftCo_Lift.c */
/* C oracle for src/tu/ftCo_Lift.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub (jobj.h's HSD_JObjSetRotationY inline runs for real, on JObjs flagged
 * JOBJ_MTX_INDEP_SRT). Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static HSD_JObj JO[4];
static FighterBone PARTS[4];
static u32 joid(HSD_JObj* j) { int i; for (i = 0; i < 4; i++) if (j == &JO[i]) return i + 1; return j == NULL ? 0 : 99; }
static u32 itid(void* p) { return p == NULL ? 0 : p == (void*) &IGO ? 1 : 99; }
static u32 cbid(HSD_GObjEvent e) { return e == NULL ? 0 : e == ftCo_800974C4 ? 1 : e == ftCo_80096E68 ? 2 : 99; }

#define VSTUB(name, id) void name(Fighter_GObj* g) { T(id); T(g == &GO); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
bool ftCo_80094EA4(Fighter_GObj* g) { T(2); T(g == &GO); return SR(2) & SR(18); }
VSTUB(ft_80084F3C, 3)
VSTUB(ft_80085088, 4)
void Item_8026ABD8(Item_GObj* i, Vec3* p, float f) { T(5); T(itid(i)); T(B(p->x)); T(B(p->y)); T(B(p->z)); T(B(f)); }
VSTUB(ftCo_80090780, 6)
VSTUB(ftCo_Fall_Enter, 7)
void ft_8008403C(Fighter_GObj* g, HSD_GObjEvent cb) { T(8); T(g == &GO); T(cbid(cb)); }
VSTUB(ftAnim_8006EBA4, 9)
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(10); T(g == &GO); return SR(10); }
Fighter_Part ftParts_GetBoneIndex(Fighter* f, Fighter_Part p) { T(11); T(f == &F); T(p); return rnd() % 4; }
float ftPartGetRotZ(Fighter* f, int p) { T(12); T(f == &F); T(p); return rw(); }
void ftPartSetRotY(Fighter* f, int p, float v) { T(13); T(f == &F); T(p); T(B(v)); }
bool itIsHeavy(Item_GObj* i) { T(14); T(itid(i)); return SR(14); }
void HSD_JObjSetMtxDirtySub(HSD_JObj* j) { T(15); T(joid(j)); }

static u32 state_hash(void)
{
    int i;
    H = 0x811C9DC5u;
    HM(F.mv.co.lift.x0); HM(B(F.mv.co.lift.x4)); HM(F.mv.co.lift.x8);
    HM(cbid(F.take_dmg_cb)); HM(B(F.facing_dir)); HM(B(F.gr_vel));
    for (i = 0; i < 4; i++) HM(B(JO[i].rotate.y));
    return H;
}

int main(void)
{
    int op, k, i;
    tu_init();
    for (op = 0; op < 23; op++) {
        for (k = 0; k < 1000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            for (i = 0; i < 4; i++) {
                zero(&JO[i], sizeof JO[i]);
                JO[i].flags = JOBJ_MTX_INDEP_SRT;
                JO[i].rotate.y = rg();
                PARTS[i].x4_jobj2 = &JO[i];
            }
            F.parts = PARTS;
            F.input.lstick[0].x = rg();
            if (rnd() % 3 == 0) F.facing_dir = rg(); else F.facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
            CD.x228 = rg(); CD.x22C = rg();
            CD.x230 = (rnd() & 1) ? (float) (rnd() % 12) : rw();
            F.mv.co.lift.x0 = rnd() & 1; F.mv.co.lift.x4 = (rnd() & 1) ? (float) (rnd() % 12) : rw();
            F.mv.co.lift.x8 = rnd() & 1;
            F.x2222_b6 = rnd() & 1;
            F.gr_vel = rw();
            F.item_gobj = (rnd() & 1) ? (Item_GObj*) &IGO : NULL;
            switch (op) {
            case 0: ftCo_80096D9C(&GO); break;
            case 1: ftCo_LiftWait_Anim(&GO); break;
            case 2: ftCo_LiftWait_IASA(&GO); break;
            case 3: ftCo_LiftWait_Phys(&GO); break;
            case 4: ftCo_80096E68(&GO); break;
            case 5: ftCo_LiftWait_Coll(&GO); break;
            case 6: ret = ftCo_80096EF8(&GO); break;
            case 7: ftCo_80096F48(&GO); break;
            case 8: ftCo_LiftWalk_Anim(&GO); break;
            case 9: ftCo_LiftWalk_IASA(&GO); break;
            case 10: ftCo_LiftWalk_Phys(&GO); break;
            case 11: ftCo_LiftWalk_Coll(&GO); break;
            case 12: ret = ftCo_800970E0(&GO); break;
            case 13: ftCo_80097130(&GO); break;
            case 14: ftCo_LiftTurn_Anim(&GO); break;
            case 15: ftCo_LiftTurn_IASA(&GO); break;
            case 16: ftCo_LiftTurn_Phys(&GO); break;
            case 17: ftCo_LiftTurn_Coll(&GO); break;
            case 18: ftCo_800974C4(&GO); break;
            case 19: ftCo_8009750C(&GO); break;
            case 20: ftCo_LiftTurn_Anim(&GO); ftCo_LiftTurn_Anim(&GO); ftCo_LiftTurn_Anim(&GO); break;
            case 21: ftCo_80097130(&GO); ftCo_LiftTurn_Anim(&GO); break;
            case 22: ftCo_80096F48(&GO); ftCo_80096F48(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
