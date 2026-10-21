/* TU: src/melee/ft/kinds/ftCommon/ftCo_JumpAerialF1.c */
/* C oracle for src/tu/ftCo_JumpAerialF1.rs: the whole TU included verbatim; every callee out of
 * it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

static struct Fighter_x2D0_t X2D0;

int FighterControls_TapJumpOff(int port) { T(1); T(port); return SR(0); }
float ScriptGame_ModJump(int s, int f, float b) { T(2); T(s); T(f); T(B(b)); return SR(1) ? b * 2.0f : b; }
int ScriptGame_CapsMaxJumps(Fighter* f) { T(3); T(f == &F); return (int) (SRV[3] % 7); }
void Geno_MultiJump(Fighter* fp, int first, int* msid, float* vy)
{ T(4); T(fp == &F); T(first); T(*msid); T(B(*vy)); if (SR(2)) { *msid += 1; *vy = *vy * 3.0f; } }
bool ft_did_jump(Fighter* f, bool a) { T(5); T(f == &F); T(a); return SR(4); }
bool ft_800D2D0C(Fighter_GObj* g) { T(6); T(g == &GO); return SR(5); }
void ft_800D2E7C(Fighter_GObj* g, Vec3* v) { T(7); T(B(v->x)); T(B(v->y)); T(B(v->z)); }
void ftCommon_8007D5D4(Fighter* f) { T(8); f->x1968_jumpsUsed = 1; }
bool ftCo_800D72A0(Fighter* f) { T(9); return SR(6); }
s32 ftCo_800D7268(Fighter* f) { T(10); return SR(7); }
void ftCo_800CBAC4(Fighter_GObj* g, FtMotionId m, Vec3* v, bool a)
{ T(11); T(g == &GO); T(m); T(B(v->x)); T(B(v->y)); T(B(v->z)); T(a); }
void ft_800CB6EC(Fighter* f, s32 a) { T(12); T(f == &F); T(a); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(13); return SR(8); }
void ftCo_FallAerial_Enter(Fighter_GObj* g) { T(14); }
void ftCo_Fall_Enter(Fighter_GObj* g) { T(15); }
void ftCo_JumpAerial_IASA(Fighter_GObj* g) { T(16); T(g == &GO); }
void ft_80084E1C(Fighter_GObj* g, float a, float b, float c) { T(17); T(B(a)); T(B(b)); T(B(c)); }
void ft_80082F28(Fighter_GObj* g) { T(18); T(g == &GO); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.cmd_vars[0]); HM(F.x1968_jumpsUsed); HM(*(s32*) &F.mv.ca.specials.grav);
    return H;
}

int main(void)
{
    int op, k, i;
    tu_init();
    for (op = 0; op < 6; op++) {
        for (k = 0; k < 4000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&X2D0, sizeof X2D0);
            X2D0.x0 = rnd() % 20; X2D0.x4 = rg(); X2D0.x8 = rw(); X2D0.xC = rw(); X2D0.x10 = rw();
            for (i = 0; i < 5; i++) X2D0.x14[i] = rw();
            X2D0.x28 = (int) (rnd() % 8) - 1; X2D0.x2C = rnd() % 400; X2D0.x30 = rnd() % 400;
            F.x2D0 = &X2D0;
            F.player_id = rnd() % 6;
            F.x1968_jumpsUsed = rnd() % 7;
            F.motion_id = (rnd() & 3) == 0 ? 0x9B : rnd() % 400;
            F.input.lstick[0].x = rg(); F.input.lstick[0].y = rg();
            F.input.held_buttons[0] = rnd() & 0xF00;
            F.cmd_vars[0] = rnd() & 1;
            F.co_attrs.max_jumps = rnd() % 7;
            F.co_attrs.air_drift_stick_mul = rw(); F.co_attrs.air_drift_max = rw();
            F.facing_dir = rg();
            *(s32*) &F.mv.ca.specials.grav = rnd();
            CD.tap_jump_threshold = rg(); CD.x258 = rw();
            switch (op) {
            case 0: { int a = rnd() & 1; ret = ftCo_800D730C(&GO, a); break; }
            case 1: ftCo_800D74A4(&GO); break;
            case 2: ftCo_JumpAerialF1_Anim(&GO); break;
            case 3: ftCo_JumpAerialF1_IASA(&GO); break;
            case 4: ftCo_JumpAerialF1_Phys(&GO); break;
            case 5: ftCo_JumpAerialF1_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
