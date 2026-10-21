/* TU: src/melee/ft/kinds/ftCommon/ftCo_CatchWait.c */
/* C oracle for src/tu/ftCo_CatchWait.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static FighterBone PARTS[8];
static unsigned char JB[64];
static struct ftData FD;
static struct ftData_x8 X8;

void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
void efAsync_Spawn(HSD_GObj* g, void* q, u32 kind, u32 gfx, HSD_JObj* j, ...)
{ T(2); T(g == &GO); T(q == (void*) &F.x60C); T(kind); T(gfx); T((u32) ((unsigned char*) j - JB)); }
void ftCommon_8007EBAC(Fighter* fp, u32 a, u32 b) { T(3); T(fp == &F); T(a); T(b); }
void ftCommon_8007E2F4(Fighter* fp, s16 v) { T(4); T(fp == &F); T(v); }
void fn_800DB6C8(Fighter_GObj* g) { T(5); T(g == &GO2); }
void fn_800DA054(Fighter_GObj* g) { T(6); T(g == &GO); }
void fn_800DA4FC(Fighter_GObj* g) { T(7); T(g == &GO); }
bool ftCo_800DD1E4(Fighter_GObj* g) { T(8); T(g == &GO); return SR(1); }
void it_802A7AAC(HSD_GObj* g) { T(9); T(g == &GO); }
void it_802BAC3C(Fighter_GObj* g) { T(10); T(g == &GO); }
void ftCommon_CalcGroundAccel_Deaccel(Fighter* fp, float f) { T(11); T(fp == &F); T(B(f)); }
void ftCommon_SetSelfMovementFromGroundedMovement(Fighter_GObj* g) { T(12); T(g == &GO); }
void ft_800841B8(Fighter_GObj* g, HSD_GObjEvent cb) { T(13); T(g == &GO); T(cb == fn_800DA440); cb(g); }
void ftCo_800DC920(Fighter_GObj* g, Fighter_GObj* v) { T(14); T(g == &GO); T(v == &GO2); }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(15); T(g == &GO); T(g == &GO2); }
static void dummy(HSD_GObj* g) { (void) g; }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(B(F.gr_vel)); HM(F.accessory1_cb == fn_800DA4A0); HM(F.take_dmg_cb == fn_800DA490); HM(F.take_dmg_cb == NULL);
    HM(F.x221B_b7);
    HM(F.mv.co.capturedamage.x18 == NULL ? 0xFFu : (u32) ((unsigned char*) F.mv.co.capturedamage.x18 - JB));
    return H;
}

int main(void)
{
    int op, k, i;
    tu_init();
    for (i = 0; i < 8; i++) PARTS[i].joint = (HSD_JObj*) (JB + i * 4);
    for (op = 0; op < 10; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&X8, sizeof X8);
            X8.x11 = rnd() % 8;
            FD.x8 = &X8;
            F.ft_data = &FD; F.parts = PARTS;
            F.victim_gobj = &GO2; F.gr_vel = rg();
            F.kind = rnd() % 34;
            F.input.pressed_buttons = rnd() & 0x1FF;
            CD.x64 = rg(); F.co_attrs.ground_friction = rg();
            F.take_dmg_cb = (rnd() & 1) ? dummy : NULL;
            F.x221B_b7 = rnd() & 1;
            switch (op) {
            case 0: fn_800DA1D8(&GO); break;
            case 1: fn_800DA2B0(&GO); break;
            case 2: ftCo_CatchWait_Anim(&GO); break;
            case 3: ftCo_CatchWait_IASA(&GO); break;
            case 4: ftCo_CatchWait_Phys(&GO); break;
            case 5: ftCo_CatchWait_Coll(&GO); break;
            case 6: fn_800DA440(&GO); break;
            case 7: fn_800DA490(&GO); break;
            case 8: fn_800DA4A0(&GO); break;
            case 9: ret = fn_800DA4C0(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
