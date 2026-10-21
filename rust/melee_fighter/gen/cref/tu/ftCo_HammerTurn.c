/* TU: src/melee/ft/kinds/ftCommon/ftCo_HammerTurn.c */
/* C oracle for src/tu/ftCo_HammerTurn.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static float FR;
static u32 FL, PART;
static FighterBone PARTS[4];
static HSD_JObj JO[4];
u32 ftCo_800C54C4(Fighter* f) { T(2); T(f == &F); return FL; }
float ftCo_800C548C(Fighter* f) { T(3); T(f == &F); return FR; }
void ftCo_800C4E94(Fighter* f) { T(4); T(f == &F); }
void ftCo_800C4F64(Fighter_GObj* g) { T(5); T(g == &GO); }
bool ftCo_800C97A8(Fighter_GObj* g) { T(6); T(g == &GO); return SR(1); }
void ftCo_Turn_Enter(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c)
{ T(7); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); }
void ftCo_800C4ED8(Fighter_GObj* g) { T(8); T(g == &GO); }
Fighter_Part ftParts_GetBoneIndex(Fighter* f, Fighter_Part p) { T(9); T(f == &F); T(p); return PART; }
void ftPartSetRotY(Fighter* f, int p, f32 r) { T(10); T(f == &F); T(p); T(B(r)); }
void HSD_JObjSetMtxDirtySub(HSD_JObj* j) { T(11); T(j - JO); }
bool ftCo_800C5A50(Fighter_GObj* g) { T(12); T(g == &GO); return SR(2); }
void ft_80084F3C(Fighter_GObj* g) { T(13); T(g == &GO); }
void ftCo_HammerWait_Coll(Fighter_GObj* g) { T(14); T(g == &GO); }

static u32 state_hash(void)
{
    int i;
    H = 0x811C9DC5u;
    HM(B(F.mv.co.walk.fast_anim_frame)); HM(B(F.facing_dir));
    for (i = 0; i < 4; i++) { HM(B(JO[i].rotate.y)); }
    return H;
}

int main(void)
{
    int op, k, i;
    tu_init();
    for (op = 0; op < 6; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            FR = rw(); FL = rnd(); PART = rnd() % 4;
            zero(JO, sizeof JO);
            for (i = 0; i < 4; i++) {
                PARTS[i].x4_jobj2 = &JO[i];
                JO[i].flags = rnd() & (JOBJ_MTX_INDEP_SRT | JOBJ_USER_DEF_MTX | JOBJ_MTX_DIRTY);
                JO[i].rotate.y = rg();
            }
            F.parts = PARTS;
            F.mv.co.walk.fast_anim_frame = (rnd() & 1) ? (float) (int) (rnd() % 12) : rg() * 4;
            F.facing_dir = rg();
            F.x2222_b6 = rnd() & 1;
            CD.x6B0 = (int) (rnd() % 20) + 1;
            switch (op) {
            case 0: ret = ftCo_800C576C(&GO); break;
            case 1: ftCo_800C57B4(&GO); break;
            case 2: ftCo_HammerTurn_Anim(&GO); break;
            case 3: ftCo_HammerTurn_IASA(&GO); break;
            case 4: ftCo_HammerTurn_Phys(&GO); break;
            case 5: ftCo_HammerTurn_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
