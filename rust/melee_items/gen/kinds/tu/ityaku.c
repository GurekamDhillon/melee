/* TU: src/melee/it/kinds/ityaku.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/kinds/mk_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
#include "kinds_common.h"
static HSD_GObj GO[3];
static Item IT[3];
static HSD_JObj JO[48];
static DynamicBoneTable DBT[3];
static Article ART[3];
static ItemAttr ATTR[3];
static ItemCommonData ICD;
static unsigned char SPEC[3][128] __attribute__((aligned(16)));
static Vec3 VA[4];

static unsigned char GRD[64];
static Article* ARTP[118];
static void CB6(Item_GObj* g, Ground* gr) { T(7001); T(PI(g)); T(PI(gr)); }
static void CB7(Item_GObj* g, Ground* gr, Vec3* v, HSD_GObj* f, float x) { T(7002); T(PI(g)); T(PI(gr)); TVEC(v); T(PI(f)); T(B(x)); }
static void CB8(Item_GObj* g, Ground* gr, HSD_GObj* t) { T(7003); T(PI(g)); T(PI(gr)); T(PI(t)); }
#include "ityaku_gen.h"

static void spec_fill(int prof)
{
    int i;
    (void) i;
    switch (prof) {
    case 0: {

    } break;
    }
}
static void setup(int prof)
{
    int i, k;
    (void) i; (void) k;
    zero(GO, sizeof GO); zero(IT, sizeof IT); zero(JO, sizeof JO); zero(DBT, sizeof DBT); zero(ART, sizeof ART);
    zero(ATTR, sizeof ATTR); zero(&ICD, sizeof ICD); zero(SPEC, sizeof SPEC); zero(VA, sizeof VA);
    for (k = 0; k < 3; k++) { fill_HSD_GObj(&GO[k]); fill_Item(&IT[k]); fill_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) { fill_HSD_JObj(&JO[i]); JO[i].flags &= ~JOBJ_USE_QUATERNION; }
    fill_ItemCommonData(&ICD);
    for (i = 0; i < 4; i++) { VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (k = 0; k < 3; k++) {
        GO[k].user_data = &IT[k];
        GO[k].hsd_obj = &JO[k];
        JO[k].child = &JO[3 + k];
        IT[k].entity = &GO[k];
        IT[k].xBBC_dynamicBoneTable = &DBT[k];
        IT[k].xC4_article_data = &ART[k];
        IT[k].xCC_item_attr = &ATTR[k];
        ART[k].x0_common_attr = &ATTR[k];
        ART[k].x4_specialAttributes = SPEC[k];
        for (i = 0; i < 14; i++) DBT[k].bones[i] = &JO[6 + 14 * k + i];
    }

    IT[0].owner = (rnd() & 3) ? &GO[1] : 0;
    IT[0].xC64_reflectGObj = (rnd() & 3) ? &GO[1] : 0;
    if (rnd() & 1) IT[0].msid = (int) (rnd() % 14);
    if (rnd() & 1) IT[0].facing_dir = (rnd() & 1) ? 1.0f : -1.0f;
    IT[0].ground_or_air = (rnd() & 1) ? GA_Air : GA_Ground;
    spec_fill(prof);
    zero(ARTP, sizeof ARTP); ARTP[117] = &ART[0]; it_804D6D38 = ARTP;
    for (k = 0; k < 3; k++) {
        IT[k].xDD4_itemVar.yaku.x2 = (s16) (1 + (rnd() & 1));
        if (IT[k].xDD4_itemVar.yaku.x2 == 1) { IT[k].xDD4_itemVar.yaku.x4 = &JO[3 + (rnd() % 3)]; } else { IT[k].xDD4_itemVar.yaku.x4_vec.x = rw(); IT[k].xDD4_itemVar.yaku.x4_vec.y = rw(); IT[k].xDD4_itemVar.yaku.x4_vec.z = rw(); }
        IT[k].xDD4_itemVar.yaku.x10 = (rnd() & 1) ? (Ground*) GRD : 0;
        IT[k].xDD4_itemVar.yaku.x14 = (rnd() & 1) ? CB6 : 0;
        IT[k].xDD4_itemVar.yaku.x18 = (rnd() & 1) ? CB7 : 0;
        IT[k].xDD4_itemVar.yaku.x1C = (rnd() & 1) ? CB8 : 0;
        IT[k].toucher = (rnd() & 1) ? &GO[(k + 1) % 3] : 0;
        IT[k].xCEC_fighterGObj = (rnd() & 1) ? &GO[(k + 2) % 3] : 0;
    }
}

static void hash_all(void)
{
    int i, k;
    (void) i; (void) k;
    HM(PI(ART[0].x0_common_attr));
    for (k = 0; k < 3; k++) {
        HM((u32) IT[k].xDD4_itemVar.yaku.x2);
        if (IT[k].xDD4_itemVar.yaku.x2 == 1) HM(PI(IT[k].xDD4_itemVar.yaku.x4)); else { HM(B(IT[k].xDD4_itemVar.yaku.x4_vec.x)); HM(B(IT[k].xDD4_itemVar.yaku.x4_vec.y)); HM(B(IT[k].xDD4_itemVar.yaku.x4_vec.z)); }
        HM(PI(IT[k].xDD4_itemVar.yaku.x10)); HM(FI((const void*) IT[k].xDD4_itemVar.yaku.x14)); HM(FI((const void*) IT[k].xDD4_itemVar.yaku.x18)); HM(FI((const void*) IT[k].xDD4_itemVar.yaku.x1C));
        IT[k].xDD4_itemVar.yaku.x4 = 0; IT[k].xDD4_itemVar.yaku.x4_vec.x = 0; IT[k].xDD4_itemVar.yaku.x4_vec.y = 0; IT[k].xDD4_itemVar.yaku.x4_vec.z = 0;
        IT[k].xDD4_itemVar.yaku.x10 = 0; IT[k].xDD4_itemVar.yaku.x14 = 0; IT[k].xDD4_itemVar.yaku.x18 = 0; IT[k].xDD4_itemVar.yaku.x1C = 0;
    }
    for (k = 0; k < 3; k++) { hash_HSD_GObj(&GO[k]); hash_Item(&IT[k]); hash_ItemAttr(&ATTR[k]); }
    for (i = 0; i < 48; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); }
}

static const unsigned char PROFTAB[] = { 0, 0, 0, 0, 0, 0, 0 };

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(IT, sizeof IT); preg(JO, sizeof JO); preg(DBT, sizeof DBT);
    preg(ART, sizeof ART); preg(ATTR, sizeof ATTR); preg(&ICD, sizeof ICD); preg(SPEC, sizeof SPEC); preg(VA, sizeof VA);
    preg(GRD, sizeof GRD); preg(&it_803F8C08, sizeof it_803F8C08);
    freg((const void*) CB6); freg((const void*) CB7); freg((const void*) CB8);
    freg_all();
    for (op = 0; op < 7; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            int prof = 0;
            tu_case(op, k);
            prof = PROFTAB[op];
            setup(prof);
            switch (op) {
            case 0: { Ground* a0 = (Ground*) GRD; int a1 = (int) fv(); int a2 = (int) (fv() % 20); HSD_JObj* a3 = (fv() & 1) ? &JO[3 + (fv() % 3)] : 0; Vec3* a4 = (fv() & 1) ? &VA[0] : 0; int a5 = (int) (fv() & 1); void (*a6)(Item_GObj*, Ground*) = (fv() & 1) ? CB6 : 0; void (*a7)(Item_GObj*, Ground*, Vec3*, HSD_GObj*, f32) = (fv() & 1) ? CB7 : 0; void (*a8)(Item_GObj*, Ground*, HSD_GObj*) = (fv() & 1) ? CB8 : 0; ret = PI(it_802E6AEC(a0, a1, a2, a3, a4, a5, a6, a7, a8)); } break; /* it_802E6AEC */
            case 1: {  it_2E6A_UnkMotion19_Phys(&GO[0]); } break; /* it_2E6A_UnkMotion19_Phys */
            case 2: {  ret = (u32) it_2E6A_Logic117_DmgDealt(&GO[0]); } break; /* it_2E6A_Logic117_DmgDealt */
            case 3: {  ret = (u32) it_2E6A_Logic117_DmgReceived(&GO[0]); } break; /* it_2E6A_Logic117_DmgReceived */
            case 4: {  ret = (u32) it_802E7054(&GO[0]); } break; /* it_802E7054 */
            case 5: {  it_2E6A_Logic117_EvtUnk(&GO[0], &GO[1]); } break; /* it_2E6A_Logic117_EvtUnk */
            case 6: { { int j; for (j = 0; j < 20; j++) { HM((u32) it_803F8C8C[j].anim_id); HM(FI((const void*) it_803F8C8C[j].animated)); HM(FI((const void*) it_803F8C8C[j].physics_updated)); HM(FI((const void*) it_803F8C8C[j].collided)); } } } break; /* state tables */
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
