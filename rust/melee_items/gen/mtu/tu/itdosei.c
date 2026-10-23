extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/it/kinds/itdosei.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_driver.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
#include "world.h"
#include "itdosei_gen.h"

static void setup_world(void)
{
    int i;
    u32 r;
    (void) r;
    zero(GO, sizeof GO);
    zero(IT, sizeof IT);
    zero(FT, sizeof FT);
    zero(JO, sizeof JO);
    zero(ATTR, sizeof ATTR);
    zero(ART, sizeof ART);
    zero(LT, sizeof LT);
    zero(DBT, sizeof DBT);
    zero(CM, sizeof CM);
    zero(STT, sizeof STT);
    zero(SDS, sizeof SDS);
    zero(ICD, sizeof ICD);
    zero(CDT, sizeof CDT);
    zero(&SP, sizeof SP);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 3; i++) { EC[i].top = rw(); EC[i].bottom = rw(); EC[i].right = rw(); EC[i].left = rw(); }
    for (i = 0; i < 16; i++) PLINK[i] = 0;
    for (i = 0; i < 9; i++) fill_HSD_GObj(&GO[i]);
    for (i = 0; i < 4; i++) fill_Item(&IT[i]);
    for (i = 0; i < 3; i++) fill_Fighter(&FT[i]);
    for (i = 0; i < 8; i++) fill_HSD_JObj(&JO[i]);
    for (i = 0; i < 2; i++) fill_ItemAttr(&ATTR[i]);
    for (i = 0; i < 2; i++) fill_Article(&ART[i]);
    for (i = 0; i < 2; i++) fill_ItemLogicTable(&LT[i]);
    for (i = 0; i < 4; i++) fill_DynamicBoneTable(&DBT[i]);
    for (i = 0; i < 2; i++) fill_CmSubject(&CM[i]);
    for (i = 0; i < 4; i++) fill_ItemStateTable(&STT[i]);
    for (i = 0; i < 3; i++) fill_ItemStateDesc(&SDS[i]);
    for (i = 0; i < 1; i++) fill_ItemCommonData(&ICD[i]);
    for (i = 0; i < 2; i++) fill_CollData(&CDT[i]);
    fill_SpawnItem(&SP);
    fillg_all();
    /* gobjs: 0..3 items, 4..6 fighters, 7..8 others; one P-link chain per kind */
    for (i = 0; i < 9; i++) {
        GO[i].hsd_obj = &JO[i % 8];
        GO[i].user_data = i < 4 ? (void*) &IT[i] : i < 7 ? (void*) &FT[i - 4] : (void*) BOGUS;
        GO[i].classifier = i < 4 ? HSD_GOBJ_CLASS_ITEM : i < 7 ? HSD_GOBJ_CLASS_FIGHTER : 3;
        GO[i].next = 0;
        GO[i].prev = 0;
    }
    for (i = 0; i < 3; i++) GO[i].next = &GO[i + 1];
    GO[1].prev = &GO[0];
    for (i = 4; i < 6; i++) GO[i].next = &GO[i + 1];
    PLINK[HSD_GOBJ_PLINK_ITEM] = &GO[0];
    PLINK[HSD_GOBJ_PLINK_FIGHTER] = &GO[4];
    for (i = 0; i < 4; i++) {
        IT[i].entity = &GO[i];
        IT[i].xCC_item_attr = &ATTR[i & 1];
        IT[i].xC4_article_data = &ART[i & 1];
        IT[i].xB8_itemLogicTable = &LT[i & 1];
        IT[i].xBBC_dynamicBoneTable = &DBT[i];
        IT[i].xBC_itemStateContainer = &STT[0];
        IT[i].xD0_itemStateDesc = &SDS[i % 3];
        IT[i].xBC0 = BOGUS;
        IT[i].x378_itemColl.x0_gobj = &GO[i];
    }
    for (i = 0; i < 3; i++) FT[i].gobj = &GO[4 + i];
    for (i = 0; i < 8; i++) { JO[i].parent = i ? &JO[(i - 1) / 2] : 0; JO[i].child = 2 * i + 1 < 8 ? &JO[2 * i + 1] : 0; JO[i].next = (i & 1) && i + 1 < 8 ? &JO[i + 1] : 0; }
    for (i = 0; i < 4; i++) for (r = 0; r < 4; r++) DBT[i].bones[r] = &JO[(i + r * 3) % 8];
    for (i = 0; i < 2; i++) { ART[i].x10_modelDesc = 0; LT[i].states = &STT[0]; }
}

static void hash_world(int ft)
{
    int i;
    (void) ft;
    for (i = 0; i < 9; i++) hash_HSD_GObj(&GO[i]);
    for (i = 0; i < 4; i++) hash_Item(&IT[i]);
    if (ft) for (i = 0; i < 3; i++) hash_Fighter(&FT[i]);
    for (i = 0; i < 8; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 2; i++) hash_ItemAttr(&ATTR[i]);
    for (i = 0; i < 2; i++) hash_Article(&ART[i]);
    for (i = 0; i < 2; i++) hash_ItemLogicTable(&LT[i]);
    for (i = 0; i < 4; i++) hash_DynamicBoneTable(&DBT[i]);
    for (i = 0; i < 2; i++) hash_CmSubject(&CM[i]);
    for (i = 0; i < 4; i++) hash_ItemStateTable(&STT[i]);
    for (i = 0; i < 3; i++) hash_ItemStateDesc(&SDS[i]);
    for (i = 0; i < 1; i++) hash_ItemCommonData(&ICD[i]);
    for (i = 0; i < 2; i++) hash_CollData(&CDT[i]);
    hash_SpawnItem(&SP);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
    for (i = 0; i < 3; i++) { HM(B(EC[i].top)); HM(B(EC[i].bottom)); HM(B(EC[i].right)); HM(B(EC[i].left)); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 4; i++) { HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(PI(PLINK[i]));
}

static u32 SATTR[2][6];

static void setup_stem(void)
{
    int i;
    (void) i;
    it_804D6D28 = &ICD[0];
{ int q, w; for (q = 0; q < 2; q++) { ART[q].x4_specialAttributes = SATTR[q]; for (w = 0; w < 6; w++) { if (w == 1 || w == 3 || w == 4 || w == 5) ((int*) SATTR[q])[w] = (int) (fv() % 6); else ((float*) SATTR[q])[w] = rw(); } } } { int q; for (q = 0; q < 8; q++) JO[q].flags &= ~JOBJ_USE_QUATERNION; } { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; v->xDD4 = (int) (fv() % 5); v->xDD8 = (int) (fv() % 4); v->xDDC = rw(); v->xDE0 = rw(); v->xDE4.x = rw(); v->xDE4.y = rw(); v->xDE4.z = rw(); v->xDF0 = (int) (fv() % 4); v->xDF4 = rw(); v->xDF8.x = rw(); v->xDF8.y = rw(); v->xDF8.z = rw(); } }
}

static void hash_stem(void)
{
    int i;
    (void) i;
    hashg_all();
    HM(PI(it_804D6D28));
{ int q, w; for (q = 0; q < 2; q++) for (w = 0; w < 6; w++) HM(SATTR[q][w]); }
}

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    freg_all();
    freg_world();
preg(SATTR, sizeof SATTR);
    for (op = 0; op < 63; op++) {
        int n = 0, ft = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; ft = 0; break;  /* itDosei_SetSpeed */
        case 1: n = 300; ft = 0; break;  /* HSD_JObjSetRotationZero */
        case 2: n = 300; ft = 0; break;  /* HSD_JObjSetRotationZeroWithMtxDirty */
        case 3: n = 300; ft = 0; break;  /* itDosei_FacingAngle */
        case 4: n = 300; ft = 0; break;  /* itDosei_Logic7_Spawned */
        case 5: n = 300; ft = 0; break;  /* itDosei_80281390 */
        case 6: n = 300; ft = 0; break;  /* itDosei_UnkMotion0_Anim */
        case 7: n = 300; ft = 0; break;  /* itDosei_UnkMotion0_Phys */
        case 8: n = 300; ft = 0; break;  /* itDosei_UnkMotion0_Coll */
        case 9: n = 300; ft = 0; break;  /* itDosei_80281734 */
        case 10: n = 300; ft = 0; break;  /* itDosei_802817A0 */
        case 11: n = 300; ft = 0; break;  /* itDosei_UnkMotion1_Anim */
        case 12: n = 300; ft = 0; break;  /* itDosei_UnkMotion1_Phys */
        case 13: n = 300; ft = 0; break;  /* itDosei_UnkMotion1_Coll */
        case 14: n = 300; ft = 0; break;  /* itDosei_80281C6C */
        case 15: n = 300; ft = 0; break;  /* itDosei_UnkMotion2_Anim */
        case 16: n = 300; ft = 0; break;  /* itDosei_UnkMotion2_Phys */
        case 17: n = 300; ft = 0; break;  /* itDosei_SetupWalk_FC */
        case 18: n = 300; ft = 0; break;  /* itDosei_UnkMotion2_Coll */
        case 19: n = 300; ft = 0; break;  /* itDosei_80282074 */
        case 20: n = 300; ft = 0; break;  /* itDosei_UnkMotion3_Anim */
        case 21: n = 300; ft = 0; break;  /* itDosei_UnkMotion3_Phys */
        case 22: n = 300; ft = 0; break;  /* itDosei_UnkMotion5_Coll */
        case 23: n = 300; ft = 0; break;  /* itDosei_Logic7_PickedUp */
        case 24: n = 300; ft = 0; break;  /* itDosei_UnkMotion4_Anim_inline */
        case 25: n = 300; ft = 0; break;  /* itDosei_UnkMotion4_GetAnimSpeed */
        case 26: n = 300; ft = 0; break;  /* itDosei_UnkMotion4_GetAttrs */
        case 27: n = 300; ft = 0; break;  /* itDosei_UnkMotion4_Anim */
        case 28: n = 300; ft = 0; break;  /* itDosei_UnkMotion4_Phys */
        case 29: n = 300; ft = 0; break;  /* itDosei_GetJObj */
        case 30: n = 300; ft = 0; break;  /* itDosei_Logic7_Dropped */
        case 31: n = 300; ft = 0; break;  /* itDosei_Logic7_Thrown */
        case 32: n = 300; ft = 0; break;  /* itDosei_UnkMotion5_Anim */
        case 33: n = 300; ft = 0; break;  /* itDosei_UnkMotion5_Phys */
        case 34: n = 300; ft = 0; break;  /* itDosei_Logic7_EnteredAir */
        case 35: n = 300; ft = 0; break;  /* itDosei_UnkMotion6_Anim */
        case 36: n = 300; ft = 0; break;  /* itDosei_UnkMotion6_Phys */
        case 37: n = 300; ft = 0; break;  /* itDosei_UnkMotion6_Coll */
        case 38: n = 300; ft = 0; break;  /* itDosei_80282BFC */
        case 39: n = 300; ft = 0; break;  /* itDosei_UnkMotion8_Anim */
        case 40: n = 300; ft = 0; break;  /* itDosei_UnkMotion8_Phys */
        case 41: n = 300; ft = 0; break;  /* itDosei_UnkMotion8_Coll */
        case 42: n = 300; ft = 0; break;  /* itDosei_80282CD4 */
        case 43: n = 300; ft = 0; break;  /* itDosei_UnkMotion7_Anim */
        case 44: n = 300; ft = 0; break;  /* itDosei_UnkMotion7_Phys */
        case 45: n = 300; ft = 0; break;  /* itDosei_UnkMotion7_Coll */
        case 46: n = 300; ft = 0; break;  /* itDosei_80282DE4 */
        case 47: n = 300; ft = 0; break;  /* itDosei_UnkMotion9_Anim */
        case 48: n = 300; ft = 0; break;  /* itDosei_UnkMotion9_Phys */
        case 49: n = 300; ft = 0; break;  /* itDosei_UnkMotion9_Coll */
        case 50: n = 300; ft = 0; break;  /* itDosei_UnkMotion10_Anim */
        case 51: n = 300; ft = 0; break;  /* itDosei_UnkMotion10_Phys */
        case 52: n = 300; ft = 0; break;  /* itDosei_UnkMotion10_Coll */
        case 53: n = 300; ft = 0; break;  /* itDosei_Logic7_DmgReceived */
        case 54: n = 300; ft = 0; break;  /* itDosei_UnkMotion11_Anim */
        case 55: n = 300; ft = 0; break;  /* itDosei_UnkMotion11_Phys */
        case 56: n = 300; ft = 0; break;  /* itDosei_UnkMotion11_Coll */
        case 57: n = 300; ft = 0; break;  /* itDosei_Logic7_DmgDealt */
        case 58: n = 300; ft = 0; break;  /* itDosei_Logic7_Reflected */
        case 59: n = 300; ft = 0; break;  /* itDosei_Logic7_Clanked */
        case 60: n = 300; ft = 0; break;  /* itDosei_Logic7_HitShield */
        case 61: n = 300; ft = 0; break;  /* itDosei_Logic7_ShieldBounced */
        case 62: n = 300; ft = 0; break;  /* itDosei_Logic7_EvtUnk */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
            setup_stem();
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: { Item_GObj * a0 = &GO[rnd() % 4]; Item * a1 = &IT[rnd() % 4]; f32 a2 = rw(); itDosei_SetSpeed(a0, a1, a2); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 1: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_JObjSetRotationZero(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 2: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_JObjSetRotationZeroWithMtxDirty(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 3: { Item_GObj * a0 = &GO[rnd() % 4]; f32 a1 = rw(); ret = B(itDosei_FacingAngle(a0, a1)); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 4: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_Logic7_Spawned(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 5: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_80281390(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 6: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion0_Anim(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 7: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_UnkMotion0_Phys(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 8: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion0_Coll(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 9: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_80281734(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 10: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_802817A0(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 11: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion1_Anim(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 12: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_UnkMotion1_Phys(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 13: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion1_Coll(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 14: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_80281C6C(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 15: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion2_Anim(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 16: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_UnkMotion2_Phys(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 17: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_SetupWalk_FC(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 18: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion2_Coll(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 19: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_80282074(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 20: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion3_Anim(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 21: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_UnkMotion3_Phys(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 22: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion5_Coll(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 23: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_Logic7_PickedUp(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 24: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_JObj * a1 = &JO[rnd() % 8]; itDosei_UnkMotion4_Anim_inline(a0, a1); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 25: { itDoseiAttributes * a0 = (itDoseiAttributes*) SATTR[rnd() & 1]; ret = B(itDosei_UnkMotion4_GetAnimSpeed(a0)); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 26: { Item * a0 = &IT[rnd() % 4]; ret = PI(itDosei_UnkMotion4_GetAttrs(a0)); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 27: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion4_Anim(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 28: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_UnkMotion4_Phys(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 29: { Item_GObj * a0 = &GO[rnd() % 4]; ret = PI(itDosei_GetJObj(a0)); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 30: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_Logic7_Dropped(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 31: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_Logic7_Thrown(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 32: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion5_Anim(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 33: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_UnkMotion5_Phys(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 34: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_Logic7_EnteredAir(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 35: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion6_Anim(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 36: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_UnkMotion6_Phys(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 37: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion6_Coll(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 38: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_80282BFC(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 39: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion8_Anim(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 40: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_UnkMotion8_Phys(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 41: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion8_Coll(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 42: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_80282CD4(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 43: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion7_Anim(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 44: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_UnkMotion7_Phys(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 45: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion7_Coll(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 46: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_80282DE4(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 47: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion9_Anim(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 48: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_UnkMotion9_Phys(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 49: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion9_Coll(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 50: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion10_Anim(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 51: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_UnkMotion10_Phys(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 52: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion10_Coll(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 53: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_Logic7_DmgReceived(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 54: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion11_Anim(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 55: { Item_GObj * a0 = &GO[rnd() % 4]; itDosei_UnkMotion11_Phys(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 56: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_UnkMotion11_Coll(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 57: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_Logic7_DmgDealt(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 58: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_Logic7_Reflected(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 59: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_Logic7_Clanked(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 60: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_Logic7_HitShield(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 61: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) itDosei_Logic7_ShieldBounced(a0); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            case 62: { Item_GObj * a0 = &GO[rnd() % 4]; Item_GObj * a1 = &GO[rnd() % 4]; itDosei_Logic7_EvtUnk(a0, a1); { int q; for (q = 0; q < 4; q++) { itDosei_ItemVars* v = &IT[q].xDD4_itemVar.dosei; HM(v->xDD4); HM(v->xDD8); HM(B(v->xDDC)); HM(B(v->xDE0)); HM(B(v->xDE4.x)); HM(B(v->xDE4.y)); HM(B(v->xDE4.z)); HM(v->xDF0); HM(B(v->xDF4)); HM(B(v->xDF8.x)); HM(B(v->xDF8.y)); HM(B(v->xDF8.z)); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); } } } break;
            }
            hash_world(ft);
            hash_stem();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
