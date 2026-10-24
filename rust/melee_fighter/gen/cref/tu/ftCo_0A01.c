/* TU: src/melee/ft/kinds/ftCommon/ftCo_0A01.c */
/* C oracle for src/tu/ftCo_0A01.rs (mirrored by tests/tu_ftCo_0A01.rs). Ops: 0..3 ftCo_GetCpu
 * L/C stick X/Y, 4 L trigger, 5 R trigger, 6 buttons, 7 ftCo_IsCpuControlled, 8 ftCo_800A0DA4.
 * Each line: op case state-hash trace. */
#include TU_SRC
#include "tu_common.h"

int fflush(void*);

Gm_PKind Player_8003248C(int s, int sub) { T(1); T(s); T(sub); return SR(0) ? Gm_PKind_Cpu : Gm_PKind_Human; }
void lbColl_800083C4(HurtCapsule* h)
{
    T(2);
    h->a_pos.x = rw(); h->a_pos.y = rw();
    h->b_pos.x = rw(); h->b_pos.y = rw();
}

static void hf(float f) { HM(B(f)); }

int main(void)
{
    int op, k, i;
    tu_init();
    for (op = 0; op < 9; op++) {
        for (k = 0; k < 2000; k++) {
            float r = 0;
            u32 ri = 0;
            tu_case(op, k);
            F.cpu.lstick.x = (s8) rnd(); F.cpu.lstick.y = (s8) rnd();
            F.cpu.cstick.x = (s8) rnd(); F.cpu.cstick.y = (s8) rnd();
            F.cpu.ltrigger = (u8) rnd(); F.cpu.rtrigger = (u8) rnd();
            F.cpu.buttons = rnd();
            F.cpu.kind = rnd() % 12;
            F.player_id = rnd() % 6;
            F.is_sub_fighter = rnd() & 1;
            F.hurt_capsules_len = rnd() % 16;
            F.cur_pos.x = rw(); F.cur_pos.y = rw();
            F.x34_scale.y = rw();
            F.facing_dir = rg();
            for (i = 0; i < 15; i++) F.hurt_capsules[i].capsule.scale = rw();
            F.cpu.x55C = rw(); F.cpu.x560 = rw(); F.cpu.x564 = rw(); F.cpu.x568 = rw();
            H = 0x811C9DC5u;
            switch (op) {
            case 0: r = ftCo_GetCpuLStickX(&F); break;
            case 1: r = ftCo_GetCpuLStickY(&F); break;
            case 2: r = ftCo_GetCpuCStickX(&F); break;
            case 3: r = ftCo_GetCpuCStickY(&F); break;
            case 4: r = ftCo_GetCpuLTrigger(&F); break;
            case 5: r = ftCo_GetCpuRTrigger(&F); break;
            case 6: ri = ftCo_GetCpuButtons(&F); break;
            case 7: ri = ftCo_IsCpuControlled(&F); break;
            case 8: ftCo_800A0DA4(&F); break;
            }
            hf(r); HM(ri);
            hf(F.cpu.x55C); hf(F.cpu.x560); hf(F.cpu.x564); hf(F.cpu.x568);
            for (i = 0; i < 15; i++) { hf(F.hurt_capsules[i].capsule.a_pos.x); hf(F.hurt_capsules[i].capsule.b_pos.y); }
            printf("%d %d %08X %08X\n", op, k, H, TR); fflush(0);
        }
    }
    return 0;
}
