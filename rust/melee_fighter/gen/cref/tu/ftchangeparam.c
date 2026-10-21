/* TU: src/melee/ft/ftchangeparam.c */
/* C oracle for src/tu/ftchangeparam.rs (mirrored by tests/tu_ftchangeparam.rs). Op 0:
 * ftCo_CalcYScaledKnockback on generated triples; op 1: ftCo_800CF6E8 over a random attribute
 * block; op 2: ftCo_800D0EC8; op 3: ftCo_800D0CBC. Each line: op case ret hash. */
#include TU_SRC
#include "tu_common.h"

struct Fighter_804D6524_t* Fighter_804D6524;
struct Fighter_804D6520_t* Fighter_804D6520;
struct Fighter_804D651C_t* Fighter_804D651C;
struct Fighter_804D6518_t* Fighter_804D6518;
void Geno_ApplyAttrs(Fighter* fp) { (void) fp; }
void Mex_OnReapplyAttrDispatch(int kind, void* gobj, void* vanilla) { (void) kind; (void) gobj; (void) vanilla; }
HSD_GObjEvent ftKindCalcIndiviParamTable[1];

static struct Fighter_804D6524_t K24;
static struct Fighter_804D6520_t K20;
static struct Fighter_804D651C_t K1C;
static ftCo_DatAttrs AT;
static struct Fighter_x2D0_t X2D0;

static float rs_nz(void) { float s = rw(); return s == 0.0f ? 0.5f : s; }

int main(void)
{
    int op, k, i;
    tu_init();
    Fighter_804D6524 = &K24; Fighter_804D6520 = &K20; Fighter_804D651C = &K1C;
    for (op = 0; op < 4; op++) {
        for (k = 0; k < 2000; k++) {
            u32 ret = 0;
            float a, s, e;
            tu_case(op, k);
            for (i = 0; i < (int) (sizeof K24 / 4); i++) ((float*) &K24)[i] = rw();
            for (i = 0; i < (int) (sizeof K20 / 4); i++) ((float*) &K20)[i] = rw();
            for (i = 0; i < (int) (sizeof K1C / 4); i++) ((float*) &K1C)[i] = rw();
            for (i = 0; i < (int) (sizeof AT / 4); i++) ((float*) &AT)[i] = rw();
            for (i = 0; i < 5; i++) X2D0.x14[i] = rw();
            X2D0.x28 = rnd() % 6;
            a = rw(); s = rs_nz(); e = rw();
            F.x34_scale.y = (rnd() & 1) ? 1.0f : rs_nz();
            F.x197C = (rnd() & 1) ? &GO2 : NULL;
            F.is_metal = rnd() & 1;
            CD.x310 = rw();
            F.x2D0 = &X2D0;
            H = 0x811C9DC5u;
            switch (op) {
            case 0: ret = B(ftCo_CalcYScaledKnockback(a, s, e)); break;
            case 1: ftCo_800CF6E8(&AT, (rnd() & 3) ? s : 1.0f); break;
            case 2: ret = B(ftCo_800D0EC8(&F)); break;
            case 3: ftCo_800D0CBC(&GO); break;
            }
            for (i = 0; i < (int) (sizeof AT / 4); i++) HM(((u32*) &AT)[i]);
            for (i = 0; i < 5; i++) HM(B(X2D0.x14[i]));
            printf("%d %d %08X %08X\n", op, k, ret, H);
        }
    }
    return 0;
}
