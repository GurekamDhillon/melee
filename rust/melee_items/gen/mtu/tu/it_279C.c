extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/it/it_279C.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_driver.py. Each line: op case ret state-hash trace-hash. */
#include <Runtime/platform.h>
#include <melee/ef/efsync.h>
#include <melee/ef/efasync.h>
#define NARGN_(_1,_2,_3,_4,_5,_6,_7,_8,_9,N,...) N
#define NARGN(...) NARGN_(__VA_ARGS__,9,8,7,6,5,4,3,2,1,0)
#define EFA(x) _Generic((x), Vec3*: 3, float*: 2, int: 0, default: 1), (unsigned long long) (x)
#define CAT_(a,b) a##b
#define CAT(a,b) CAT_(a,b)
#define EFS_2(id, g) oracle_efsync_n(0, id, g)
#define EFS_3(id, g, a) oracle_efsync_n(1, id, g, EFA(a))
#define EFS_4(id, g, a, b) oracle_efsync_n(2, id, g, EFA(a), EFA(b))
#define EFS_5(id, g, a, b, c) oracle_efsync_n(3, id, g, EFA(a), EFA(b), EFA(c))
#define EFS_6(id, g, a, b, c, d) oracle_efsync_n(4, id, g, EFA(a), EFA(b), EFA(c), EFA(d))
#define EFAS_5(g, q, k, id, j) oracle_efasync_n(0, g, q, k, id, j)
#define EFAS_6(g, q, k, id, j, a) oracle_efasync_n(1, g, q, k, id, j, EFA(a))
#define EFAS_7(g, q, k, id, j, a, b) oracle_efasync_n(2, g, q, k, id, j, EFA(a), EFA(b))
#define EFAS_8(g, q, k, id, j, a, b, c) oracle_efasync_n(3, g, q, k, id, j, EFA(a), EFA(b), EFA(c))
#define EFAS_9(g, q, k, id, j, a, b, c, d) oracle_efasync_n(4, g, q, k, id, j, EFA(a), EFA(b), EFA(c), EFA(d))
#define efSync_Spawn(...) CAT(EFS_, NARGN(__VA_ARGS__))(__VA_ARGS__)
#define efAsync_Spawn(...) CAT(EFAS_, NARGN(__VA_ARGS__))(__VA_ARGS__)
void* oracle_efsync_n(int, int, HSD_GObj*, ...);
void oracle_efasync_n(int, HSD_GObj*, void*, u32, u32, HSD_JObj*, ...);
static void* ARCP;

#include TU_SRC
#include "mtu_common.h"
#include "world.h"
#include "it_279C_gen.h"

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



static void setup_stem(void)
{
    int i;
    (void) i;

{ int q, k, n; setup_xpools(); zero(S70, sizeof S70); zero(S74, sizeof S74); for (q = 0; q < 2; q++) { S70[q].x2070_int = fv(); S74[q].x207C.x = fv(); S74[q].x207C.y = fv(); } n = rnd() % 28; for (k = 0; k < 32; k++) KL[k] = (k >= n) ? It_PKind_Terminate : It_PKind_Start + rnd() % (It_PKind_Terminate - It_PKind_Start); for (q = 0; q < 3; q++) { zero(&FT[q].x2070, sizeof FT[q].x2070); FT[q].x2070.x2070_int = fv(); zero(&FT[q].x2074, sizeof FT[q].x2074); FT[q].x2074.x2074_vec.x = rw(); FT[q].x2074.x2074_vec.y = rw(); FT[q].x2074.x207C.x = fv(); FT[q].x2074.x207C.y = fv(); FT[q].x2074.x2084 = fv(); FT[q].x2074.x2088 = fv(); } for (q = 0; q < 2; q++) { PDA[q].x38 = rnd() % 7; ART[q].x4_specialAttributes = (void*) &PDA[q]; } for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); ps->x0 = fv(); ps->x4.x = rw(); ps->x4.y = rw(); ps->x4.z = rw(); ps->x10.x = rw(); ps->x10.y = rw(); ps->x10.z = rw(); ps->x1C = rw(); ps->x20 = rw(); ps->x24 = rw(); ps->x28 = rw(); for (k = 0; k < 6; k++) ps->x2C[k] = rw(); ps->x44 = rw(); ps->x48 = rw(); ps->x4C = rw(); ps->x50 = rw(); ps->x58 = rnd() % 6; ps->x54 = rnd() % 8; ps->x5C = rnd() % 3; zero(&IT[q].xD90, sizeof IT[q].xD90); IT[q].xD90.x2070_int = fv(); } for (q = 0; q < 8; q++) JO[q].flags &= ~JOBJ_USE_QUATERNION; }
}

static void hash_stem(void)
{
    int i;
    (void) i;
    hashg_all();

hash_xpools(); HM(RH(KL, sizeof KL)); HM(RH(S70, sizeof S70)); HM(RH(S74, sizeof S74));
}

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    freg_all();
    freg_world();
preg_xpools(); preg(KL, sizeof KL); preg(S70, sizeof S70); preg(S74, sizeof S74);
    for (op = 0; op < 30; op++) {
        int n = 0, ft = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; ft = 0; break;  /* it_80279C48 */
        case 1: n = 300; ft = 0; break;  /* it_80279CDC */
        case 2: n = 300; ft = 0; break;  /* it_80279D38 */
        case 3: n = 300; ft = 0; break;  /* it_80279D5C */
        case 4: n = 300; ft = 0; break;  /* it_80279E24 */
        case 5: n = 300; ft = 0; break;  /* it_80279FF8 */
        case 6: n = 300; ft = 0; break;  /* it_8027A09C */
        case 7: n = 300; ft = 0; break;  /* it_8027A118 */
        case 8: n = 300; ft = 0; break;  /* it_8027A13C */
        case 9: n = 300; ft = 0; break;  /* it_8027A160 */
        case 10: n = 300; ft = 0; break;  /* it_8027A344 */
        case 11: n = 300; ft = 0; break;  /* it_8027A364 */
        case 12: n = 300; ft = 0; break;  /* it_8027A4D4 */
        case 13: n = 300; ft = 0; break;  /* it_8027A780 */
        case 14: n = 300; ft = 0; break;  /* it_8027A9B8 */
        case 15: n = 300; ft = 0; break;  /* it_8027AAA0 */
        case 16: n = 300; ft = 0; break;  /* it_8027AB64 */
        case 17: n = 300; ft = 0; break;  /* it_8027ADEC */
        case 18: n = 300; ft = 0; break;  /* it_8027AE34 */
        case 19: n = 300; ft = 0; break;  /* it_8027AF50 */
        case 20: n = 300; ft = 0; break;  /* it_8027B070 */
        case 21: n = 300; ft = 0; break;  /* it_8027B0C4 */
        case 22: n = 300; ft = 0; break;  /* it_8027B1F4 */
        case 23: n = 300; ft = 0; break;  /* it_8027B288 */
        case 24: n = 300; ft = 0; break;  /* it_8027B330 */
        case 25: n = 300; ft = 0; break;  /* it_8027B378 */
        case 26: n = 300; ft = 0; break;  /* it_8027B408 */
        case 27: n = 300; ft = 0; break;  /* it_8027B4A4 */
        case 28: n = 300; ft = 0; break;  /* it_8027B508 */
        case 29: n = 300; ft = 0; break;  /* it_8027B564 */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
            setup_stem();
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: { Item_GObj * a0 = &GO[rnd() % 4]; it_80279C48(a0); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 1: { Item_GObj * a0 = &GO[rnd() % 4]; f32 a1 = rw(); it_80279CDC(a0, a1); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 2: { Item_GObj * a0 = &GO[rnd() % 4]; it_80279D38(a0); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 3: { Item_GObj * a0 = &GO[rnd() % 4]; f32 a1 = rw(); it_80279D5C(a0, a1); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 4: { Item_GObj * a0 = &GO[rnd() % 4]; it_80279E24(a0); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 5: { Item_GObj * a0 = &GO[rnd() % 4]; it_80279FF8(a0); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 6: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8027A09C(a0); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 7: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObjEvent a1 = (HSD_GObjEvent) pick_cb(1); ret = (u32) it_8027A118(a0, a1); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 8: { Item_GObj * a0 = &GO[rnd() % 4]; it_8027A13C(a0); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 9: { HSD_JObj * a0 = &JO[rnd() % 8]; Item * a1 = &IT[rnd() % 4]; it_8027A160(a0, a1); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 10: { Item_GObj * a0 = &GO[rnd() % 4]; it_8027A344(a0); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 11: { Item * a0 = &IT[rnd() % 4]; ret = (u32) it_8027A364(a0); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 12: { Item * a0 = &IT[rnd() % 4]; ret = (u32) it_8027A4D4(a0); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 13: { Item * a0 = &IT[rnd() % 4]; void * a1 = (void*) KL; ret = (u32) it_8027A780(a0, a1); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 14: { Item * a0 = &IT[rnd() % 4]; ret = (u32) it_8027A9B8(a0); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 15: { Item_GObj * a0 = &GO[rnd() % 4]; Item * a1 = &IT[rnd() % 4]; s32 a2 = (s32) ((rnd() % 3 == 0) ? It_PKind_Marumine : fv()); it_8027AAA0(a0, a1, a2); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 16: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8027AB64(a0); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 17: { int a0 = fv(); Item_GObj * a1 = &GO[rnd() % 4]; HSD_JObj * a2 = &JO[rnd() % 8]; f32 a3 = rw(); it_8027ADEC(a0, a1, a2, a3); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 18: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8027AE34(a0); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 19: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) it_8027AF50(a0); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 20: { Item_GObj * a0 = &GO[rnd() % 4]; Fighter_GObj * a1 = &GO[4 + rnd() % 3]; it_8027B070(a0, a1); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 21: { Item_GObj * a0 = &GO[rnd() % 4]; SpawnItem * a1 = (SP.x0_parent_gobj = &GO[rnd() % 7], SP.x4_parent_gobj2 = &GO[rnd() % 7], &SP); it_8027B0C4(a0, a1); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 22: { Item_GObj * a0 = &GO[rnd() % 4]; it_8027B1F4(a0); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 23: { Item_GObj * a0 = &GO[rnd() % 4]; volatile u32 a1 = fv(); it_8027B288(a0, a1); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 24: { Item_GObj * a0 = &GO[rnd() % 4]; u32 a1 = fv(); it_8027B330(a0, a1); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 25: { Fighter_GObj * a0 = &GO[rnd() % 4]; Item_GObj * a1 = &GO[rnd() % 4]; f32 a2 = rw(); it_8027B378(a0, a1, a2); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 26: { Item_GObj * a0 = &GO[rnd() % 4]; Item_GObj * a1 = &GO[rnd() % 4]; f32 a2 = rw(); it_8027B408(a0, a1, a2); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 27: { Fighter_GObj * a0 = &GO[rnd() % 4]; Item_GObj * a1 = &GO[rnd() % 4]; it_8027B4A4(a0, a1); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 28: { Item_GObj * a0 = &GO[rnd() % 4]; Item_GObj * a1 = &GO[rnd() % 4]; it_8027B508(a0, a1); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            case 29: { Item_GObj * a0 = &GO[rnd() % 4]; it_8027B564(a0); { int q; for (q = 0; q < 4; q++) { itPokemonSpawn_ItemVars* ps = &IT[q].xDD4_itemVar.pokemon_spawn; int k; HM((u32) ps->x0); HM(B(ps->x4.x)); HM(B(ps->x4.y)); HM(B(ps->x4.z)); HM(B(ps->x10.x)); HM(B(ps->x10.y)); HM(B(ps->x10.z)); HM(B(ps->x1C)); HM(B(ps->x20)); HM(B(ps->x24)); HM(B(ps->x28)); for (k = 0; k < 6; k++) HM(B(ps->x2C[k])); HM(B(ps->x44)); HM(B(ps->x48)); HM(B(ps->x4C)); HM(B(ps->x50)); HM((u32) ps->x54); HM((u32) ps->x58); HM((u32) ps->x5C); zero(&IT[q].xDD4_itemVar, sizeof IT[q].xDD4_itemVar); HM((u32) IT[q].xD90.x2070_int); zero(&IT[q].xD90, sizeof IT[q].xD90); } } } break;
            }
            hash_world(ft);
            hash_stem();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
