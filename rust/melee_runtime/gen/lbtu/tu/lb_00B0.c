/* TU: src/melee/lb/lb_00B0.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/lbtu/mk_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
#include "lbtu_common.h"
static HSD_JObj JO[4];
static HSD_Joint JT[2];
static HSD_AObj AO[4];
static HSD_DObj DO[3];
static HSD_PObj PO[3];
static HSD_ShapeSet SS[2];
static HSD_MObj MO[2];
static HSD_TObj TO[3];
static HSD_RObj RO[3];
static HSD_AnimJoint AJ[4];
static HSD_AObjDesc AD[2];
static HSD_TevDesc TD[3];
static HSD_TExpTevDesc TED[2];
static HSD_TExp TE[3];
static HSD_LObj LO[3];
static DynamicModelDesc DMD[1];
static Vec V[3];
static unsigned char MZ[64];
static float FL[4];
static float* FPP[2];
static HSD_AnimJoint* AJP[4];
static HSD_MatAnimJoint* MAJP[4];
static HSD_ShapeAnimJoint* SAJP[4];
static unsigned char MAJB[4][16], SAJB[4][16];
#include "lb_00B0_gen.h"

static void setup(void)
{
    int i, k;
    (void) i; (void) k;
    zero(JO, sizeof JO);
    zero(JT, sizeof JT);
    zero(AO, sizeof AO);
    zero(DO, sizeof DO);
    zero(PO, sizeof PO);
    zero(SS, sizeof SS);
    zero(MO, sizeof MO);
    zero(TO, sizeof TO);
    zero(RO, sizeof RO);
    zero(AJ, sizeof AJ);
    zero(AD, sizeof AD);
    zero(TD, sizeof TD);
    zero(TED, sizeof TED);
    zero(TE, sizeof TE);
    zero(LO, sizeof LO);
    zero(DMD, sizeof DMD);
    zero(V, sizeof V);
    zero(MZ, sizeof MZ);
    for (i = 0; i < 4; i++) fill_HSD_JObj(&JO[i]);
    for (i = 0; i < 2; i++) fill_HSD_Joint(&JT[i]);
    for (i = 0; i < 4; i++) fill_HSD_AObj(&AO[i]);
    for (i = 0; i < 3; i++) fill_HSD_DObj(&DO[i]);
    for (i = 0; i < 3; i++) fill_HSD_PObj(&PO[i]);
    for (i = 0; i < 2; i++) fill_HSD_ShapeSet(&SS[i]);
    for (i = 0; i < 2; i++) fill_HSD_MObj(&MO[i]);
    for (i = 0; i < 3; i++) fill_HSD_TObj(&TO[i]);
    for (i = 0; i < 3; i++) fill_HSD_RObj(&RO[i]);
    for (i = 0; i < 4; i++) fill_HSD_AnimJoint(&AJ[i]);
    for (i = 0; i < 2; i++) fill_HSD_AObjDesc(&AD[i]);
    for (i = 0; i < 3; i++) fill_HSD_TevDesc(&TD[i]);
    for (i = 0; i < 2; i++) fill_HSD_TExpTevDesc(&TED[i]);
    for (i = 0; i < 3; i++) fill_HSD_TExp(&TE[i]);
    for (i = 0; i < 3; i++) fill_HSD_LObj(&LO[i]);
    for (i = 0; i < 1; i++) fill_DynamicModelDesc(&DMD[i]);
    for (i = 0; i < 3; i++) fill_Vec(&V[i]);
    for (k = 0; k < 4; k++) {
        JO[k].next = 0; JO[k].parent = 0; JO[k].child = 0;
        JO[k].aobj = (rnd() & 1) ? &AO[rnd() % 4] : 0;
        JO[k].u.dobj = (rnd() & 1) ? &DO[rnd() % 3] : 0;
    }
    for (k = 1; k < 4; k++) {
        int p = (int) (rnd() % k);
        HSD_JObj* c = JO[p].child;
        if (!c) JO[p].child = &JO[k]; else { while (c->next) c = c->next; c->next = &JO[k]; }
        JO[k].parent = &JO[p];
    }
    for (k = 0; k < 2; k++) { JT[k].next = (rnd() & 1) ? &JT[1 - k] : 0; JT[k].child = 0; }
    JT[1].next = 0;
    JT[0].child = (rnd() & 1) ? &JT[1] : 0;
    for (k = 0; k < 3; k++) {
        DO[k].next = (k < 2 && (rnd() & 1)) ? &DO[k + 1] : 0;
        DO[k].pobj = (rnd() & 1) ? &PO[rnd() % 3] : 0;
        DO[k].mobj = (rnd() & 1) ? &MO[rnd() % 2] : 0;
        PO[k].next = (k < 2 && (rnd() & 1)) ? &PO[k + 1] : 0;
        PO[k].flags = (rnd() & 1) ? 0x1000 : (u16) fv();
        PO[k].u.shape_set = &SS[rnd() % 2];
        TO[k].next = (k < 2 && (rnd() & 1)) ? &TO[k + 1] : 0;
        TO[k].aobj = (rnd() & 1) ? &AO[rnd() % 4] : 0;
        RO[k].next = (k < 2 && (rnd() & 1)) ? &RO[k + 1] : 0;
        LO[k].next = k < 2 ? &LO[k + 1] : 0;
        TD[k].next = (k < 2 && (rnd() & 1)) ? &TD[k + 1] : 0;
        TD[k].u.tevconf.clr_out_reg = fv() % 4; TD[k].u.tevconf.alpha_out_reg = fv() % 4;
        TE[k].cnst.next = (k < 2 && (rnd() & 1)) ? &TE[k + 1] : 0;
        TE[k].cnst.reg = (u8) (fv() % 10);
    }
    LO[2].flags &= ~3;
    for (k = 0; k < 2; k++) {
        SS[k].aobj = (rnd() & 1) ? &AO[rnd() % 4] : 0;
        MO[k].aobj = (rnd() & 1) ? &AO[rnd() % 4] : 0;
        MO[k].tobj = (rnd() & 1) ? &TO[rnd() % 3] : 0;
        MO[k].tevdesc = &TED[k];
        MO[k].texp = (rnd() & 1) ? &TE[rnd() % 3] : 0;
        TED[k].desc.next = (rnd() & 1) ? &TD[rnd() % 3] : 0;
        TED[k].desc.u.tevconf.clr_out_reg = fv() % 4; TED[k].desc.u.tevconf.alpha_out_reg = fv() % 4;
    }
    for (k = 0; k < 4; k++) {
        AJ[k].child = (k < 3 && (rnd() & 1)) ? &AJ[k + 1] : 0;
        AJ[k].next = (k < 2 && (rnd() & 1)) ? &AJ[k + 2] : 0;
        AJ[k].aobjdesc = (rnd() & 1) ? &AD[rnd() % 2] : 0;
        AJP[k] = &AJ[k]; MAJP[k] = (HSD_MatAnimJoint*) MAJB[k]; SAJP[k] = (HSD_ShapeAnimJoint*) SAJB[k];
    }
    DMD[0].anims = (rnd() & 1) ? AJP : 0; DMD[0].matanims = (rnd() & 1) ? MAJP : 0; DMD[0].shapeanims = (rnd() & 1) ? SAJP : 0;
    for (k = 0; k < 4; k++) FL[k] = rf();
    FPP[0] = &FL[0]; FPP[1] = &FL[1];
    for (k = 0; k < 64; k++) MZ[k] = (unsigned char) (rnd() | 1);
}
static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    for (k = 0; k < 4; k++) { HM(PIB(JO[k].u.dobj)); JO[k].u.dobj = 0; }
    for (k = 0; k < 3; k++) { HM(PIB(PO[k].u.shape_set)); PO[k].u.shape_set = 0; HM(PIB(TE[k].cnst.next)); TE[k].cnst.next = 0; }
    for (i = 0; i < 4; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 2; i++) hash_HSD_Joint(&JT[i]);
    for (i = 0; i < 4; i++) hash_HSD_AObj(&AO[i]);
    for (i = 0; i < 3; i++) hash_HSD_DObj(&DO[i]);
    for (i = 0; i < 3; i++) hash_HSD_PObj(&PO[i]);
    for (i = 0; i < 2; i++) hash_HSD_ShapeSet(&SS[i]);
    for (i = 0; i < 2; i++) hash_HSD_MObj(&MO[i]);
    for (i = 0; i < 3; i++) hash_HSD_TObj(&TO[i]);
    for (i = 0; i < 3; i++) hash_HSD_RObj(&RO[i]);
    for (i = 0; i < 4; i++) hash_HSD_AnimJoint(&AJ[i]);
    for (i = 0; i < 2; i++) hash_HSD_AObjDesc(&AD[i]);
    for (i = 0; i < 3; i++) hash_HSD_TevDesc(&TD[i]);
    for (i = 0; i < 2; i++) hash_HSD_TExpTevDesc(&TED[i]);
    for (i = 0; i < 3; i++) hash_HSD_TExp(&TE[i]);
    for (i = 0; i < 3; i++) hash_HSD_LObj(&LO[i]);
    for (i = 0; i < 1; i++) hash_DynamicModelDesc(&DMD[i]);
    for (i = 0; i < 3; i++) hash_Vec(&V[i]);
    for (k = 0; k < 4; k++) HM(B(FL[k]));
    for (k = 0; k < 64; k++) HM(MZ[k]);
}
int main(void)
{
    int op, k;
    preg(JO, sizeof JO);
    preg(JT, sizeof JT);
    preg(AO, sizeof AO);
    preg(DO, sizeof DO);
    preg(PO, sizeof PO);
    preg(SS, sizeof SS);
    preg(MO, sizeof MO);
    preg(TO, sizeof TO);
    preg(RO, sizeof RO);
    preg(AJ, sizeof AJ);
    preg(AD, sizeof AD);
    preg(TD, sizeof TD);
    preg(TED, sizeof TED);
    preg(TE, sizeof TE);
    preg(LO, sizeof LO);
    preg(DMD, sizeof DMD);
    preg(V, sizeof V);
    preg(MZ, sizeof MZ);
    preg(FL, sizeof FL);
    preg(MAJB, sizeof MAJB);
    preg(SAJB, sizeof SAJB);
    preg(FPP, sizeof FPP);
    preg(AJP, sizeof AJP);
    preg(MAJP, sizeof MAJP);
    preg(SAJP, sizeof SAJP);
    freg_all();

    for (op = 0; op < 42; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup();
            switch (op) {
            case 0: {   ret = (u32) lb_8000B074(&JO[0]); } break; /* lb_8000B074 */
            case 1: {   ret = (u32) lb_8000B09C(&JO[0]); } break; /* lb_8000B09C */
            case 2: {   ret = (u32) lb_8000B134(&JO[0]); } break; /* lb_8000B134 */
            case 3: {   lb_8000B1CC(&JO[0], &V[0], &V[1]); } break; /* lb_8000B1CC */
            case 4: {   lb_8000B4FC(&JO[0], &JT[0]); } break; /* lb_8000B4FC */
            case 5: {   lb_8000B5DC(&JO[0], &JT[0]); } break; /* lb_8000B5DC */
            case 6: {   lb_8000B6A4(&JO[0], &JT[0]); } break; /* lb_8000B6A4 */
            case 7: {   lb_8000B760(&JO[0], &JT[0]); } break; /* lb_8000B760 */
            case 8: {   lb_8000B804(&JO[0], &JT[0]); } break; /* lb_8000B804 */
            case 9: {  s32 v2 = (s32) ((int) (fv() & 1)); lb_8000B9D8(&JO[0], &FPP[0], v2); } break; /* lb_8000B9D8 */
            case 10: {  float v1 = (float) (rf()); lb_8000BA0C(&JO[0], v1); } break; /* lb_8000BA0C */
            case 11: {  float v1 = (float) (rf()); lbDObjSetRateAll(&DO[0], v1); } break; /* lbDObjSetRateAll */
            case 12: {  float v1 = (float) (rf()); lbDObjReqAnimAll(&DO[0], v1); } break; /* lbDObjReqAnimAll */
            case 13: {   ret = PI(lbFindJObjWithAObj(&JO[0])); } break; /* lbFindJObjWithAObj */
            case 14: {   ret = B(lbGetJObjFramerate(&JO[0])); } break; /* lbGetJObjFramerate */
            case 15: {   ret = B(lbGetJObjCurrFrame(&JO[0])); } break; /* lbGetJObjCurrFrame */
            case 16: {   ret = B(lbGetJObjEndFrame(&JO[0])); } break; /* lbGetJObjEndFrame */
            case 17: {   ret = PI(lb_8000BECC(&AJ[0])); } break; /* lb_8000BECC */
            case 18: {   ret = B(lb_8000BFF0(&AJ[0])); } break; /* lb_8000BFF0 */
            case 19: {  s32 v1 = (s32) ((int) (fv() % 4)); HSD_AnimJoint ** v2 = (HSD_AnimJoint **) (((fv() & 1) ? AJP : 0)); HSD_MatAnimJoint ** v3 = (HSD_MatAnimJoint **) (((fv() & 1) ? MAJP : 0)); HSD_ShapeAnimJoint ** v4 = (HSD_ShapeAnimJoint **) (((fv() & 1) ? SAJP : 0)); lb_8000C07C(&JO[0], v1, v2, v3, v4); } break; /* lb_8000C07C */
            case 20: {  s32 v1 = (s32) ((int) (fv() % 4)); lb_8000C0E8(&JO[0], v1, &DMD[0]); } break; /* lb_8000C0E8 */
            case 21: {  ssize_t v1 = (ssize_t) ((long long) (fv() % 65)); memzero(MZ, v1); } break; /* memzero */
            case 22: {   lb_8000C1C0(&JO[0], &JO[1]); } break; /* lb_8000C1C0 */
            case 23: {   lb_8000C228(&JO[0], &JO[1]); } break; /* lb_8000C228 */
            case 24: {   lb_8000C290(&JO[0], &JO[1]); } break; /* lb_8000C290 */
            case 25: {   lb_8000C2F8(&JO[0], &JO[1]); } break; /* lb_8000C2F8 */
            case 26: {   lb_8000C390(&JO[0]); } break; /* lb_8000C390 */
            case 27: {  u32 v1 = (u32) (fv()); float v2 = (float) (rf()); lb_8000C420(&JO[0], v1, v2); } break; /* lb_8000C420 */
            case 28: {  float v3 = (float) (rf()); float v4 = (float) (rf()); lb_8000C490(&JO[0], &JO[1], &JO[2], v3, v4); } break; /* lb_8000C490 */
            case 29: {   lbCopyJObjSRT(&JO[0], &JO[1]); } break; /* lbCopyJObjSRT */
            case 30: {  float v3 = (float) (rf()); float v4 = (float) (rf()); lb_8000C868(&JT[0], &JO[0], &JO[1], v3, v4); } break; /* lb_8000C868 */
            case 31: {  s32 v0 = (s32) ((int) (fv() % 9)); HSD_TExp * v2 = (HSD_TExp *) (((fv() & 1) ? &TE[0] : 0)); HSD_TExp * v3 = (HSD_TExp *) (((fv() & 1) ? &TE[1] : 0)); ret = (u32) lbGetFreeColorRegImpl(v0, &TD[0], v2, v3); } break; /* lbGetFreeColorRegImpl */
            case 32: {  s32 v0 = (s32) ((int) (fv() % 9)); ret = (u32) lbGetFreeColorRegister(v0, &MO[0], &TE[0]); } break; /* lbGetFreeColorRegister */
            case 33: {  s32 v0 = (s32) ((int) (fv() % 8)); ret = (u32) lb_8000CC8C(v0); } break; /* lb_8000CC8C */
            case 34: {  s32 v0 = (s32) ((int) (fv() % 4)); ret = (u32) lb_8000CCA4(v0); } break; /* lb_8000CCA4 */
            case 35: {  s32 v0 = (s32) ((int) (fv() % 9)); HSD_TExp * v2 = (HSD_TExp *) (0); HSD_TExp * v3 = (HSD_TExp *) (0); ret = (u32) lbGetFreeAlphaRegImpl(v0, &TD[0], v2, v3); } break; /* lbGetFreeAlphaRegImpl */
            case 36: {  s32 v0 = (s32) ((int) (fv() % 9)); ret = (u32) lbGetFreeAlphaRegister(v0, &MO[0], &TE[0]); } break; /* lbGetFreeAlphaRegister */
            case 37: {  s32 v0 = (s32) ((int) (fv() % 8)); ret = (u32) lb_8000CD90(v0); } break; /* lb_8000CD90 */
            case 38: {  s32 v0 = (s32) ((int) (fv() % 4)); ret = (u32) lb_8000CDA8(v0); } break; /* lb_8000CDA8 */
            case 39: {   ret = PI(lb_8000CDC0(&LO[0])); } break; /* lb_8000CDC0 */
            case 40: {   lb_8000CE30(&DO[0], &DO[1]); } break; /* lb_8000CE30 */
            case 41: {   lb_8000CE40(&JO[0], &DO[0]); } break; /* lb_8000CE40 */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
