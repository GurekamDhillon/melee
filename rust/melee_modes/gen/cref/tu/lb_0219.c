/* TU: src/melee/lb/lb_0219.c */
/* C oracle for src/tu/lb_0219.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
static HSD_GObj GNEW, G;
static BgFlashUserData UD;
static u8 LBF[64];
static u8 CB[64];
#include "mtu_common.h"
#include "lb_0219_gen.h"

int main(void)
{
    int op, k;
    preg(&GNEW, sizeof GNEW);
    preg(&G, sizeof G);
    preg(&UD, sizeof UD);
    preg(LBF, sizeof LBF);
    preg(CB, sizeof CB);
    preg(&bgflash_alloc_data, sizeof bgflash_alloc_data);
    preg(&lbl_804D63DC, sizeof lbl_804D63DC);
    freg_all();
    for (op = 0; op < 8; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&GNEW, sizeof GNEW);
            zero(&G, sizeof G);
            zero(&UD, sizeof UD);
            fill_BgFlashUserData(&UD);
            fill_HSD_ObjAllocData(&bgflash_alloc_data);
            fill_HSD_GObj(&G);
            G.user_data = &UD;
            flash_gobj = &G;
            lbl_804D63DC = (rnd() & 1) ? (struct Fighter_804D653C_t*) LBF : 0;
            flash_scale = rg();
            switch (op) {
            case 0: lbBgFlash_Free((rnd() & 1) ? (void*) &UD : 0); break;
            case 1: lbBgFlash_SetFlashScale(rg()); break;
            case 2: lbBgFlash_Init((int) fv()); break;
            case 3: lbBgFlash_Proc(&G); break;
            case 4: fn_80021C18(&G, (CommandInfo*) CB, (int) fv()); break;
            case 5: fn_80021C1C(); break;
            case 6: { u32 a = fv(); u32 b = fv(); lbBgFlash_80021C48(a, b); } break;
            case 7: fn_80021C80(&G); break;
            }
            hash_BgFlashUserData(&UD);
            hash_HSD_ObjAllocData(&bgflash_alloc_data);
            hash_HSD_GObj(&G);
            hash_HSD_GObj(&GNEW);
            HM(PI(flash_gobj));
            HM(PI(lbl_804D63DC));
            HM(B(flash_scale));
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
