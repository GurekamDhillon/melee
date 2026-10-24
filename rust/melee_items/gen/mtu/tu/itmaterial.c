extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/it/itmaterial.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_driver.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
#include "world.h"
#include "itmaterial_gen.h"

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

static unsigned char TBOG[16]; static GXColor GCOL[2]; static HSD_TExp TX[3];
static void cb_stev(HSD_MObj* m, HSD_TObj* t, u32 r) { T(7010); T(PI(m)); T(PI(t)); T(r); }
static void* IT0P;

static void setup_stem(void)
{
    int i;
    (void) i;

{ static int init; static struct it_MObjInfo p0; int q; if (!init) { p0 = it_mobj; init = 1; } it_mobj = p0; setup_xpools(); zero(TX, sizeof TX); for (q = 0; q < 3; q++) fill_HSD_TECnst(&TX[q].cnst); for (q = 0; q < 16; q++) TBOG[q] = (u8) rnd(); for (q = 0; q < 2; q++) { GCOL[q].r = rnd(); GCOL[q].g = rnd(); GCOL[q].b = rnd(); GCOL[q].a = rnd(); } for (q = 0; q < 2; q++) MO[q].parent.class_info = (HSD_ClassInfo*) &MI[0]; MI[0].setup_tev = cb_stev; hsdMObj.parent.release = (void*) cbev1; hsdMObj.parent.amnesia = (void*) cbev2; hsdMObj.load = (void*) cbp1; hsdMObj.make_texp = (void*) cbev3; for (q = 0; q < 5; q++) TO[q].next = (q < 2 && (rnd() & 1)) ? &TO[q + 1] : 0; TO[4].imagedesc = (rnd() & 1) ? (void*) TBOG : 0; for (q = 0; q < 2; q++) MO[q].tobj = (rnd() & 3) ? &TO[rnd() % 3] : 0; tobj_shadows = (rnd() & 1) ? &TO[3] : 0; tobj_toon = (rnd() & 1) ? &TO[4] : 0; for (q = 0; q < 2; q++) MO[q].mat = (rnd() % 5) ? &MA[rnd() & 1] : &MA[0]; for (q = 0; q < 2; q++) MO[q].pe = (rnd() & 1) ? &PE[rnd() & 1] : 0; for (q = 0; q < 3; q++) DO[q].next = (q < 2 && (rnd() & 1)) ? &DO[q + 1] : 0; for (q = 0; q < 3; q++) DO[q].mobj = (rnd() % 4) ? &MO[rnd() & 1] : 0; HSD_GObj_804D7814 = &GO[rnd() % 4]; if (HDBG) printf("S %08X %08X %08X %08X%c", PI(MO[0].tobj), PI(MO[1].tobj), PI(tobj_shadows), PI(tobj_toon), 10); }
}

static void hash_stem(void)
{
    int i;
    (void) i;
    hashg_all();

hash_xpools(); hash_it_MObjInfo(&it_mobj); hash_GXColor(&GCOL[0]); hash_GXColor(&GCOL[1]); for (int q = 0; q < 3; q++) hash_HSD_TECnst(&TX[q].cnst);
}

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    freg_all();
    freg_world();
preg_xpools(); preg(TBOG, sizeof TBOG); preg(GCOL, sizeof GCOL); preg(TX, sizeof TX); freg((const void*) cb_stev);
    for (op = 0; op < 5; op++) {
        int n = 0, ft = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; ft = 0; break;  /* it_80277D08 */
        case 1: n = 300; ft = 0; break;  /* fn_80277D8C */
        case 2: n = 300; ft = 0; break;  /* it_80277F90 */
        case 3: n = 300; ft = 0; break;  /* it_80278108 */
        case 4: n = 300; ft = 0; break;  /* it_80278574 */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
            setup_stem();
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: {  it_80277D08();  } break;
            case 1: { HSD_MObj * a0 = &MO[rnd() & 1]; u32 a1 = fv(); u32 a2 = fv(); fn_80277D8C(a0, a1, a2);  } break;
            case 2: { Item * a0 = &IT[rnd() % 4]; HSD_MObj * a1 = &MO[rnd() & 1]; HSD_TExp * a2 = &TX[rnd() % 3]; ret = PI(it_80277F90(a0, a1, a2));  } break;
            case 3: { Item * a0 = &IT[rnd() % 4]; HSD_MObj * a1 = &MO[rnd() & 1]; HSD_TExp * a2 = (rnd() & 1) ? &TX[rnd() % 3] : 0; it_80278108(a0, a1, a2);  } break;
            case 4: { HSD_GObj * a0 = &GO[rnd() % 4]; GXColor * a1 = &GCOL[rnd() & 1]; it_80278574(a0, a1);  } break;
            }
            hash_world(ft);
            hash_stem();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
