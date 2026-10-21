/* TU: src/melee/ft/kinds/ftCommon/ftCo_DamageScrew.c */
/* C oracle for src/tu/ftCo_DamageScrew.rs. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

void ftCommon_8007DB58(Fighter_GObj* g) { T(1); T(g == &GO); }
void ftCo_8009750C(Fighter_GObj* g) { T(3); T(g == &GO); }
void ftCo_800DD168(Fighter_GObj* g) { T(4); T(g == &GO); }
void ftCommon_8007D5D4(Fighter* f) { T(2); T(f == &F); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(7); T(g == &GO); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(o == NULL); }
bool ftAnim_IsFramesRemaining(Fighter_GObj* g) { T(9); return SR(3); }
void ftCo_800968C8(Fighter_GObj* g) { T(10); T(g == &GO); }
void ft_80084DB0(Fighter_GObj* g) { T(11); T(g == &GO); }
void ftCo_AirCatchHit_Coll(Fighter_GObj* g) { T(12); T(g == &GO); }
void ft_80083B68(Fighter_GObj* g) { T(13); T(g == &GO); }

static u32 state_hash(void)
{
    H = 0x811C9DC5u;
    HM(F.cmd_vars[0]); HM(B(F.self_vel.x)); HM(B(F.self_vel.y)); HM(B(F.self_vel.z));
    return H;
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 6; op++) {
        for (k = 0; k < 1000; k++) {
            u32 ret = 0;
            tu_case(op, k);
            F.dmg.x1860_element = (rnd() & 1) ? HitElement_Scball : rnd() % 16;
            F.x2070.x2071_b0_3 = rnd() & 15;
            F.cmd_vars[0] = rnd() & 1;
            F.ground_or_air = rnd() & 1;
            F.co_attrs.screw_attack_launch_velocity = rw();
            F.self_vel.x = rg(); F.self_vel.y = rg(); F.self_vel.z = rg();
            switch (op) {
            case 0: ret = ftCo_800D2FA4(&GO); break;
            case 1: ftCo_800D3004(&GO); break;
            case 2: ftCo_DamageScrew_Anim(&GO); break;
            case 3: ftCo_DamageScrew_IASA(&GO); break;
            case 4: ftCo_DamageScrew_Phys(&GO); break;
            case 5: ftCo_DamageScrew_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
