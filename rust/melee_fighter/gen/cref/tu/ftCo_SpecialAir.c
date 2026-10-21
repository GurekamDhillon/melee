/* TU: src/melee/ft/kinds/ftCommon/ftCo_SpecialAir.c */
#include TU_SRC
#include "tu_common.h"
#include "ftco_s209_fill.h"

HSD_GObjEvent ftData_SpecialAirHi[Ft_Kind_Max], ftData_SpecialAirLw[Ft_Kind_Max], ftData_SpecialAirS[Ft_Kind_Max],
    ftData_SpecialAirN[Ft_Kind_Max];
static void vanilla_fn(HSD_GObj* g) { (void) g; }
static u32 VAN(void* v) { return v == NULL ? 0 : v == (void*) vanilla_fn ? 1 : 9; }

int ScriptGame_CapsActionAllowed(Fighter* f, int flag) { T(1); T(PI(f)); T((u32) flag); return SR(1) | SR(17); }
int Geno_SpecialEnter(Fighter_GObj* g, int which) { T(2); T(PI(g)); T((u32) which); return SR(2); }
void Mex_SpecialHiAirDispatch(int kind, void* g, void* v) { T(3); T((u32) kind); T(PI(g)); T(VAN(v)); }
void Mex_SpecialLwAirDispatch(int kind, void* g, void* v) { T(4); T((u32) kind); T(PI(g)); T(VAN(v)); }
void Mex_SpecialSAirDispatch(int kind, void* g, void* v) { T(5); T((u32) kind); T(PI(g)); T(VAN(v)); }
void Mex_SpecialNAirDispatch(int kind, void* g, void* v) { T(6); T((u32) kind); T(PI(g)); T(VAN(v)); }
void ftCommon_UpdateFacing(Fighter* f) { T(8); T(PI(f)); }

#define NOPS 1
#define NCASE 8000

int main(void)
{
    int op, k, i;
    tu_init();
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0;
            tu_case(op, k);
            s209_fill();
            for (i = 0; i < Ft_Kind_Max; i++) {
                ftData_SpecialAirHi[i] = ftData_SpecialAirLw[i] = ftData_SpecialAirS[i] = ftData_SpecialAirN[i] = NULL;
            }
            ftData_SpecialAirHi[F.kind] = (SRV[1] & 1) ? vanilla_fn : NULL;
            ftData_SpecialAirLw[F.kind] = (SRV[1] & 2) ? vanilla_fn : NULL;
            ftData_SpecialAirS[F.kind] = (SRV[1] & 4) ? vanilla_fn : NULL;
            ftData_SpecialAirN[F.kind] = (SRV[1] & 8) ? vanilla_fn : NULL;
            /* some cases with a neutral stick so the N branch is reached */
            if (SRV[0] & 1) { F.input.lstick[0].x = 0; F.input.lstick[0].y = 0; }
            ret = ftCo_SpecialAir_CheckInput(&GO);
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
