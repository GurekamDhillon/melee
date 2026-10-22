/* TU: src/melee/gr/grdisplay.c */
/* C oracle for src/tu/grdisplay.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
static HSD_GObj GOBJ, X18G, CURG, CAMG, FOGG, FIG[3];
static Ground GP;
static HSD_JObj JT[7];
static HSD_CObj CAMCO, OTHCO;
static char FOGO[16];
static LbShadow SHAD[3];
static HSD_GObj* PLINK[16];
static Mtx VM;
#include "grdisplay_gen.h"

static int ocsr(Vec3* p, int a, HSD_JObj* j)
{
    T(5000); T(B(p->x)); T(B(p->y)); T(B(p->z)); T((u32) a); T(PI(j));
    return (int) fv();
}

static void setup(void)
{
    int i, n;
    zero(&GOBJ, sizeof GOBJ); zero(&X18G, sizeof X18G); zero(&CURG, sizeof CURG);
    zero(&CAMG, sizeof CAMG); zero(&FOGG, sizeof FOGG); zero(FIG, sizeof FIG);
    zero(&GP, sizeof GP); zero(JT, sizeof JT); zero(SHAD, sizeof SHAD); zero(PLINK, sizeof PLINK);
    fill_HSD_GObj(&GOBJ); fill_HSD_GObj(&X18G); fill_HSD_GObj(&CURG); fill_HSD_GObj(&CAMG); fill_HSD_GObj(&FOGG);
    for (i = 0; i < 3; i++) fill_HSD_GObj(&FIG[i]);
    fill_Ground(&GP);
    for (i = 0; i < 3; i++) fill_LbShadow(&SHAD[i]);
    GOBJ.user_data = &GP;
    GOBJ.hsd_obj = &JT[0];
    n = (int) (rnd() % 4);
    GP.x18 = (n == 0) ? 0 : ((n & 1) ? &X18G : &CURG);
    CURG.hsd_obj = &JT[2];
    X18G.hsd_obj = (rnd() & 1) ? CURG.hsd_obj : (void*) &JT[1];
    CAMG.hsd_obj = ((rnd() & 3) == 0) ? 0 : (void*) &CAMCO;
    FOGG.hsd_obj = (rnd() & 1) ? (void*) FOGO : 0;
    HSD_GObj_804D7818 = &CURG;
    HSD_GObjPLinkHead = PLINK;
    n = (int) (rnd() % 4);
    for (i = 0; i < n; i++) FIG[i].next = (i + 1 < n) ? &FIG[i + 1] : 0;
    PLINK[HSD_GOBJ_PLINK_FIGHTER] = n ? &FIG[0] : 0;
    stage_info.on_check_shadow_render = ocsr;
    for (i = 0; i < 7; i++) {
        u32 r;
        fill_HSD_JObj(&JT[i]);
        r = rnd();
        if ((r & 3) == 0) JT[i].flags |= JOBJ_INSTANCE;
        if (((r >> 2) & 3) == 0) JT[i].flags |= JOBJ_HIDDEN;
        if ((r >> 4) & 1) JT[i].flags |= JOBJ_MTX_DIRTY;
        if (((r >> 5) & 7) == 0) JT[i].flags |= JOBJ_USER_DEF_MTX;
        if ((r >> 8) & 1) JT[i].flags |= (rnd() & 0xF) << 18;
        if ((r >> 9) & 1) JT[i].flags |= (rnd() & 0xF) << 28;
        JT[i].child = (i < 6 && (rnd() & 3) != 0) ? &JT[i + 1] : 0;
        JT[i].next = (i < 6 && (rnd() % 3) != 0) ? &JT[i + 1 + (int) (rnd() % (u32) (6 - i))] : 0;
        if (JT[i].child == 0) JT[i].flags &= ~JOBJ_INSTANCE;
    }
}

static void hash_all(void)
{
    int i;
    hash_HSD_GObj(&GOBJ); hash_HSD_GObj(&X18G); hash_HSD_GObj(&CURG); hash_HSD_GObj(&CAMG); hash_HSD_GObj(&FOGG);
    for (i = 0; i < 3; i++) hash_HSD_GObj(&FIG[i]);
    hash_Ground(&GP);
    for (i = 0; i < 7; i++) hash_HSD_JObj(&JT[i]);
    for (i = 0; i < 3; i++) hash_LbShadow(&SHAD[i]);
}

int main(void)
{
    int op, k;
    preg(&GOBJ, sizeof GOBJ); preg(&X18G, sizeof X18G); preg(&CURG, sizeof CURG);
    preg(&CAMG, sizeof CAMG); preg(&FOGG, sizeof FOGG); preg(FIG, sizeof FIG);
    preg(&GP, sizeof GP); preg(JT, sizeof JT); preg(&CAMCO, sizeof CAMCO); preg(&OTHCO, sizeof OTHCO);
    preg(FOGO, sizeof FOGO); preg(SHAD, sizeof SHAD); preg(PLINK, sizeof PLINK); preg(VM, sizeof VM);
    freg_all();
    freg((const void*) ocsr);
    for (op = 0; op < 4; op++) {
        for (k = 0; k < 2500; k++) {
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: {
                u32 fl = rnd() & 0xF;
                u32 rm = fv();
                grDisplay_801C5B90(&JT[0], (rnd() & 1) ? VM : 0, fl, rm);
                break;
            }
            case 1: grDisplay_801C5DB0_surface_impl(&GOBJ, (int) fv()); break;
            case 2: grDisplay_801C5DB0(&GOBJ, (int) fv()); break;
            case 3: grDisplay_801C5F60(&CAMG, (int) fv()); break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, 0u, H, TR);
        }
    }
    return 0;
}
