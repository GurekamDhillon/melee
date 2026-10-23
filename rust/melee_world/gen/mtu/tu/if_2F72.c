/* TU: src/melee/if/if_2F72.c */
/* C oracle for src/tu/if_2F72.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub (memzero is real). Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
void memzero(void* p, int n)
{
    unsigned char* a = p;
    int i;
    for (i = 0; i < n; i++) a[i] = 0;
}
static HSD_GObj G[8], NG[4];
static HSD_JObj JT[8], NJ[4];
static HudIndex HI;
static Vec3 HPOS[6];
static HSD_GObjProc PROC0;
static HSD_Archive* ARCP;
static char ARCH[16];
static DynamicModelDesc DMD;
static DynamicModelDesc* DMDA[1];
static char JOINTB[16];
static int gm_ret(void)
{
    u32 x = fv() % 4;
    return (x == 0) ? -2 : ((x == 1) ? -1 : (int) x);
}
#include "if_2F72_gen.h"

static void cbe1(int idx) { T(8001); T((u32) idx); }
static void cbe2(int idx) { T(8002); T((u32) idx); }
static void (*pickcb(void))(int)
{
    switch (rnd() % 3) {
    case 0: return cbe1;
    case 1: return cbe2;
    default: return 0;
    }
}

static void setup(void)
{
    int i;
    zero(G, sizeof G); zero(NG, sizeof NG); zero(JT, sizeof JT); zero(NJ, sizeof NJ);
    zero(&HI, sizeof HI); zero(&PROC0, sizeof PROC0); zero(&DMD, sizeof DMD);
    zero(ifStatus_803F9628, sizeof ifStatus_803F9628);
    for (i = 0; i < 8; i++) { fill_HSD_GObj(&G[i]); fill_HSD_JObj(&JT[i]); G[i].hsd_obj = &JT[i]; }
    for (i = 0; i < 4; i++) { fill_HSD_GObj(&NG[i]); fill_HSD_JObj(&NJ[i]); }
    fill_HudIndex(&HI);
    for (i = 0; i < 6; i++) { HPOS[i].x = rw(); HPOS[i].y = rw(); HPOS[i].z = rw(); }
    for (i = 0; i < 8; i++) {
        struct Element_803F9628* e = &ifStatus_803F9628[i];
        fill_Element_803F9628(e);
        e->x0 = (rnd() % 3 == 0) ? 0 : &G[rnd() % 8];
        e->x10 = (u8) (rnd() % 30);
        e->x11 = (u8) (rnd() % 30);
        e->x18 = pickcb();
        e->x1C = pickcb();
    }
    zero(&scinfstc_models, sizeof scinfstc_models);
    DMDA[0] = &DMD;
    DMD.joint = (rnd() & 1) ? (HSD_Joint*) JOINTB : 0;
    scinfstc_models.scene_models = (rnd() % 4 == 0) ? 0 : DMDA;
    for (i = 0; i < 6; i++) {
        scinfstc_models.slots[i].gobj = (rnd() % 3 == 0) ? 0 : &G[rnd() % 8];
        scinfstc_models.slots[i].gobj2 = (rnd() % 3 == 0) ? 0 : &G[rnd() % 8];
    }
    ARCP = (rnd() & 1) ? (HSD_Archive*) ARCH : 0;
    HSD_GObj_JObjKind = (u8) (rnd() % 4);
}

static void hash_all(void)
{
    int i;
    for (i = 0; i < 8; i++) { hash_HSD_GObj(&G[i]); hash_HSD_JObj(&JT[i]); hash_Element_803F9628(&ifStatus_803F9628[i]); }
    for (i = 0; i < 4; i++) { hash_HSD_GObj(&NG[i]); hash_HSD_JObj(&NJ[i]); }
    hash_HudIndex(&HI);
    for (i = 0; i < 6; i++) { HM(B(HPOS[i].x)); HM(B(HPOS[i].y)); HM(B(HPOS[i].z)); }
    HM(PI(scinfstc_models.scene_models));
    for (i = 0; i < 6; i++) { HM(PI(scinfstc_models.slots[i].gobj)); HM(PI(scinfstc_models.slots[i].gobj2)); }
    HM(PI(ARCP)); HM(PI(DMD.joint));
}

int main(void)
{
    int op, k, i;
    for (i = 0; i < 8; i++) preg(&G[i], sizeof(HSD_GObj));
    for (i = 0; i < 4; i++) preg(&NG[i], sizeof(HSD_GObj));
    for (i = 0; i < 8; i++) preg(&JT[i], sizeof(HSD_JObj));
    for (i = 0; i < 4; i++) preg(&NJ[i], sizeof(HSD_JObj));
    preg(ifStatus_803F9628, sizeof ifStatus_803F9628); preg(&HI, sizeof HI); preg(HPOS, sizeof HPOS);
    preg(&PROC0, sizeof PROC0); preg(ARCH, sizeof ARCH); preg(&DMD, sizeof DMD); preg(DMDA, sizeof DMDA);
    preg(JOINTB, sizeof JOINTB); preg(&ARCP, sizeof ARCP); preg(&scinfstc_models, sizeof scinfstc_models);
    freg_all();
    freg((const void*) cbe1); freg((const void*) cbe2);
    for (op = 0; op < 14; op++) {
        for (k = 0; k < 1500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: { int g = (int) (rnd() % 8), e = (int) (rnd() % 8); ret = (u32) fn_802F7288(&G[g], &ifStatus_803F9628[e]); break; }
            case 1: if_802F73C4(&G[rnd() % 8]); break;
            case 2: if_802F74D0(&G[rnd() % 8]); break;
            case 3: fn_802F75D4(&G[rnd() % 8]); break;
            case 4: fn_802F7670(&G[rnd() % 8]); break;
            case 5: {
                int si = (int) (rnd() % 6), g = (int) (rnd() % 8);
                int cb = (int) fv();
                scinfstc_models.slots[si].gobj = &G[g];
                fn_802F770C(&G[g], cb);
                break;
            }
            case 6: {
                int g = (int) (rnd() % 8);
                u8 slot = (u8) (rnd() % 6);
                u16 a2 = (u16) fv();
                HSD_GObj* gg = (rnd() & 1) ? &G[g] : 0;
                ret = PI(fn_802F77F8(gg, slot, a2));
                break;
            }
            case 7: fn_802F7994(&G[rnd() % 8]); break;
            case 8: if_802F7AF8((s32) (rnd() % 6)); break;
            case 9: if_802F7BB4((s32) (rnd() % 6)); break;
            case 10: if_802F7C30((s32) (rnd() % 6)); break;
            case 11: if_802F7D08((s32) (rnd() % 6)); break;
            case 12: if_802F7E24(); break;
            case 13: if_802F7E7C(); break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
