extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/melee/it/item.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_driver.py. Each line: op case ret state-hash trace-hash. */
static unsigned char DOBBUF[4][64] __attribute__((aligned(16)));
#include TU_SRC
#include "mtu_common.h"
#include "world.h"
#include "item_gen.h"

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

static int CUR_OP = -1;
enum { OP_Item_80266F3C = 0, OP_Item_80266F70 = 1, OP_Item_80266FA8 = 2, OP_Item_80266FCC = 3, OP_ItUnkHoldKind = 4, OP_HSD_JObjSetScaleItem = 5, OP_HSD_JObjSetFacingDirItem = 6, OP_Item_80267130 = 7, OP_Item_80267454 = 8, OP_Item_802674AC = 9, OP_Item_802675A8 = 10, OP_Item_802676F4 = 11, OP_Item_8026784C = 12, OP_Item_80267978 = 13, OP_Item_80267AA8 = 14, OP_Item_802680CC = 15, OP_Item_8026814C = 16, OP_Item_802682F0 = 17, OP_Item_8026849C = 18, OP_Item_80268560 = 19, OP_foobar = 20, OP_foobar2 = 21, OP_foobar3 = 22, OP_Item_8026862C = 23, OP_Item_ScriptSpawnAtCap = 24, OP_Item_ScriptSpawnCapTest = 25, OP_Item_80268B18 = 26, OP_Item_80268B5C = 27, OP_Item_80268B9C = 28, OP_Item_80268BE0 = 29, OP_Item_80268D34 = 30, OP_Item_80268DD4 = 31, OP_Item_80268E40 = 32, OP_Item_80268E5C = 33, OP_Item_802693E4 = 34, OP_Item_802694CC = 35, OP_Item_80269528 = 36, OP_Item_802696CC = 37, OP_Item_802697D4 = 38, OP_Item_80269978 = 39, OP_Item_80269A9C = 40, OP_Item_80269B60 = 41, OP_Item_80269BE4 = 42, OP_Item_80269C5C = 43, OP_Item_80269CA0 = 44, OP_Item_80269CC4 = 45, OP_Item_80269DC8 = 46, OP_Item_80269F14 = 47, OP_Item_8026A0A0 = 48, OP_Item_8026A0FC = 49, OP_func_8026A158_helper = 50, OP_Item_8026A158 = 51, OP_func_8026A1E8_inline = 52, OP_Item_8026A1E8 = 53, OP_processCallback = 54, OP_OnTakeDamageThink = 55, OP_OnClankThink = 56, OP_OnGiveDamageThink = 57, OP_EnterHitlagThink = 58, OP_checkHitLag = 59, OP_Item_8026A294 = 60, OP_Item_ScriptCommitDamage = 61, OP_Item_8026A788 = 62, OP_Item_8026A810 = 63, OP_Item_8026A848 = 64, OP_DestroyItemInline = 65, OP_ItemSwitch = 66, OP_RunGObjCallback = 67, OP_func_8026A8EC_inline1 = 68, OP_func_8026A8EC_inline2 = 69, OP_func_8026A8EC_inline3 = 70, OP_Item_8026A8EC = 71, OP_Item_8026AB54 = 72, OP_Item_8026ABD8 = 73, OP_Item_8026AC74 = 74, OP_Item_8026AD20 = 75, OP_Item_8026ADC0 = 76, OP_Item_OnUserDataRemove = 77, OP_Item_8026AE60 = 78, OP_Item_MexSfx = 79, OP_Item_8026AE84 = 80, OP_Item_8026AF0C = 81, OP_Item_8026AFA0 = 82, OP_Item_8026B034 = 83, OP_Item_8026B074 = 84, OP_Item_8026B0B4 = 85, OP_Item_IsGrabbable = 86 };

struct it_MObjInfo { char pad[64]; };
struct it_MObjInfo it_mobj;
static HSD_AnimJoint AJ[4]; static HSD_MatAnimJoint MJ[4]; static HSD_ShapeAnimJoint SJ[4];
static ItemModelDesc MD[2]; static ItemDynamics DY[2]; static BoneDynamicsDesc BDD[4];
static ItemStateArray STA; static HSD_Joint JNT[2];
static Article* A24[43]; static Article* A38[118]; static Article* A30[47];
static Fighter_804D653C_t CA04[4];
static s32 pick_kind(int spawn)
{
    u32 r = rnd();
    switch (r & 7) {
    case 0: return 0x1000 + (r >> 8) % 0x200;
    case 1: return 0x1200 + (r >> 8) % 0x50;
    case 2: return spawn ? 5000 + (r >> 8) % 40 : (s32) ((r >> 8) % 300);
    case 3: return 0x20000 + (r >> 8) % 20;
    default: return (r >> 8) % 300;
    }
}


static void setup_stem(void)
{
    int i;
    (void) i;
    it_804D6D28 = &ICD[0];

{
    int k;
    zero(AJ, sizeof AJ); zero(MJ, sizeof MJ); zero(SJ, sizeof SJ); zero(MD, sizeof MD); zero(DY, sizeof DY); zero(BDD, sizeof BDD);
    zero(&STA, sizeof STA); zero(JNT, sizeof JNT); zero(DOBBUF, sizeof DOBBUF); zero(CA04, sizeof CA04);
    for (i = 0; i < 4; i++) {
        AJ[i].child = (i < 3 && (rnd() & 1)) ? &AJ[i + 1 + rnd() % (3 - i)] : 0;
        AJ[i].next = (i < 3 && (rnd() & 1)) ? &AJ[i + 1 + rnd() % (3 - i)] : 0;
        MJ[i].child = (i < 3 && (rnd() & 1)) ? &MJ[i + 1 + rnd() % (3 - i)] : 0;
        MJ[i].next = (i < 3 && (rnd() & 1)) ? &MJ[i + 1 + rnd() % (3 - i)] : 0;
        SJ[i].child = (i < 3 && (rnd() & 1)) ? &SJ[i + 1 + rnd() % (3 - i)] : 0;
        SJ[i].next = (i < 3 && (rnd() & 1)) ? &SJ[i + 1 + rnd() % (3 - i)] : 0;
        ((HSD_DObj*) DOBBUF[i])->next = i < 3 ? (HSD_DObj*) DOBBUF[i + 1] : 0;
        ((HSD_DObj*) DOBBUF[i])->mobj = (rnd() & 1) ? (HSD_MObj*) &U8A[0] : 0;
        BDD[i].bone_id = rnd() % 4;
        CA04[i].unk5 = (u8) (rnd() & 1);
        STT[i].anim_id = (int) (rnd() % 9) - 1;
    }
    for (k = 0; k < 8; k++) {
        STA.x0_itemStateDesc[k].x0_anim_joint = (rnd() & 3) ? &AJ[rnd() % 4] : 0;
        STA.x0_itemStateDesc[k].x4_matanim_joint = (rnd() & 3) ? &MJ[rnd() % 4] : 0;
        STA.x0_itemStateDesc[k].x8_parameters = (rnd() & 3) ? &SJ[rnd() % 4] : 0;
    }
    for (i = 0; i < 3; i++) {
        SDS[i].x0_anim_joint = (rnd() & 3) ? &AJ[rnd() % 4] : 0;
        SDS[i].x4_matanim_joint = (rnd() & 3) ? &MJ[rnd() % 4] : 0;
        SDS[i].x8_parameters = (rnd() & 3) ? &SJ[rnd() % 4] : 0;
    }
    for (i = 0; i < 2; i++) {
        int bc = (rnd() % 3 == 0) ? 0 : (int) (rnd() % 4) + 1;
        ART[i].x0_common_attr = &ATTR[i];
        ART[i].xC_itemStates = &STA;
        ART[i].x10_modelDesc = &MD[i];
        MD[i].x0_joint = (rnd() & 1) ? (HSD_Joint*) &JNT[rnd() & 1] : 0;
        MD[i].x4_bone_count = bc;
        DY[i].count = rnd() % 4;
        DY[i].dyn_descs = BDD;
        ART[i].x14_dynamics = (bc && (rnd() & 1)) ? &DY[i] : 0;
    }
    for (k = 0; k < 43; k++) A24[k] = (CUR_OP == OP_Item_8026862C && k % 17 == 5) ? 0 : &ART[rnd() % 2];
    for (k = 0; k < 118; k++) A38[k] = (CUR_OP == OP_Item_8026862C && k % 17 == 5) ? 0 : &ART[rnd() % 2];
    for (k = 0; k < 47; k++) A30[k] = (CUR_OP == OP_Item_8026862C && k % 17 == 5) ? 0 : &ART[rnd() % 2];
    for (k = 0; k < 30; k++) it_804A0F60[k] = &ART[rnd() % 2];
    it_804D6D24 = A24; it_804D6D38 = A38; it_804D6D30 = A30; it_804D6D04 = CA04;
    for (i = 0; i < 4; i++) {
        IT[i].kind = pick_kind(0);
        IT[i].x374_dynamicBonesNum = rnd() % 5;
        IT[i].atk_victim = pick_igo();
        IT[i].grab_victim = pick_igo();
        IT[i].grab_dealt = (__typeof__(IT[0].grab_dealt)) pick_cb(1);
        IT[i].grabbed_for_victim = (__typeof__(IT[0].grabbed_for_victim)) pick_cb(3);
        IT[i].xC8_joint = (rnd() & 1) ? &JNT[rnd() & 1] : 0;
        IT[i].x548_colorOverlay.x28_colanim.i = rnd() % 4;
        IT[i].xC6C = (float) (rnd() % 5) * 0.5f;
        /* xD90.x2070_int <- xC74 is a whole-word write over a bitfield overlay (x86 LSB-first vs GC MSB-first views): only bytes 0 and 3 vary */
        IT[i].xC74 = (s32) ((rnd() & 0xFF) | ((rnd() & 0xFF) << 24));
    }
    for (i = 0; i < 8; i++) JO[i].flags &= ~JOBJ_USE_QUATERNION;
    SP.kind = pick_kind(1);
    for (i = 0; i < 26; i++) ((s32*) &Item_804A0C64)[i] = (s32) (fv() & 7);
    Item_804A0CCC.count = rnd(); Item_804A0CCC.x154.b0 = rnd() & 1; Item_804A0CCC.x154.b1 = rnd() & 1; Item_804A0CCC.x154.b2 = rnd() & 1; Item_804A0CCC.x154.b3 = rnd() & 1; Item_804A0CCC.x154.b4 = rnd() & 1; Item_804A0CCC.x154.b5 = rnd() & 1; Item_804A0CCC.x154.b6 = rnd() & 1; Item_804A0CCC.x154.b7 = rnd() & 1; for (i = 0; i < 0xB0; i++) ((u8*) &Item_804A0CCC)[i] = (u8) rnd();
    Item_804A0E24.last_kind = (s32) rnd(); Item_804A0E24.previous_kind = (s32) rnd(); Item_804A0E24.rare_spawned = rnd() & 1;
}

}

static void hash_stem(void)
{
    int i;
    (void) i;
    hashg_all();
    HM(PI(it_804D6D28));
HM(RH(&Item_804A0C64, sizeof Item_804A0C64)); HM((u32) Item_804A0CCC.count); HM((u32) Item_804A0CCC.x154.b0); HM((u32) Item_804A0CCC.x154.b1); HM((u32) Item_804A0CCC.x154.b2); HM((u32) Item_804A0CCC.x154.b3); HM((u32) Item_804A0CCC.x154.b4); HM((u32) Item_804A0CCC.x154.b5); HM((u32) Item_804A0CCC.x154.b6); HM((u32) Item_804A0CCC.x154.b7); HM(RH(Item_804A0CCC.ecb_offset_arr, sizeof Item_804A0CCC.ecb_offset_arr)); HM((u32) Item_804A0E24.last_kind); HM((u32) Item_804A0E24.previous_kind); HM((u32) Item_804A0E24.rare_spawned);
}

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    freg_all();
    freg_world();
preg(AJ, sizeof AJ); preg(MJ, sizeof MJ); preg(SJ, sizeof SJ); preg(MD, sizeof MD); preg(DY, sizeof DY); preg(BDD, sizeof BDD); preg(&STA, sizeof STA); preg(JNT, sizeof JNT); preg(DOBBUF, sizeof DOBBUF); preg(A24, sizeof A24); preg(A38, sizeof A38); preg(A30, sizeof A30); preg(CA04, sizeof CA04); preg(&item_alloc_data, sizeof item_alloc_data); preg(&item_dynamic_bones_alloc_data, sizeof item_dynamic_bones_alloc_data); preg(&item_link_alloc_data, sizeof item_link_alloc_data); preg(&Item_804A0C64, sizeof Item_804A0C64);
    for (op = 0; op < 87; op++) {
        int n = 0, ft = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 300; ft = 0; break;  /* Item_80266F3C */
        case 1: n = 300; ft = 0; break;  /* Item_80266F70 */
        case 2: n = 300; ft = 0; break;  /* Item_80266FA8 */
        case 3: n = 300; ft = 0; break;  /* Item_80266FCC */
        case 4: n = 300; ft = 0; break;  /* ItUnkHoldKind */
        case 5: n = 300; ft = 0; break;  /* HSD_JObjSetScaleItem */
        case 6: n = 300; ft = 0; break;  /* HSD_JObjSetFacingDirItem */
        case 7: n = 300; ft = 0; break;  /* Item_80267130 */
        case 8: n = 300; ft = 0; break;  /* Item_80267454 */
        case 9: n = 300; ft = 0; break;  /* Item_802674AC */
        case 10: n = 300; ft = 0; break;  /* Item_802675A8 */
        case 11: n = 300; ft = 0; break;  /* Item_802676F4 */
        case 12: n = 300; ft = 0; break;  /* Item_8026784C */
        case 13: n = 300; ft = 0; break;  /* Item_80267978 */
        case 14: n = 300; ft = 0; break;  /* Item_80267AA8 */
        case 15: n = 300; ft = 0; break;  /* Item_802680CC */
        case 16: n = 300; ft = 0; break;  /* Item_8026814C */
        case 17: n = 300; ft = 0; break;  /* Item_802682F0 */
        case 18: n = 300; ft = 0; break;  /* Item_8026849C */
        case 19: n = 300; ft = 0; break;  /* Item_80268560 */
        case 20: n = 300; ft = 0; break;  /* foobar */
        case 21: n = 300; ft = 0; break;  /* foobar2 */
        case 22: n = 300; ft = 0; break;  /* foobar3 */
        case 23: n = 300; ft = 0; break;  /* Item_8026862C */
        case 24: n = 300; ft = 0; break;  /* Item_ScriptSpawnAtCap */
        case 25: n = 300; ft = 0; break;  /* Item_ScriptSpawnCapTest */
        case 26: n = 300; ft = 0; break;  /* Item_80268B18 */
        case 27: n = 300; ft = 0; break;  /* Item_80268B5C */
        case 28: n = 300; ft = 0; break;  /* Item_80268B9C */
        case 29: n = 300; ft = 0; break;  /* Item_80268BE0 */
        case 30: n = 300; ft = 0; break;  /* Item_80268D34 */
        case 31: n = 300; ft = 0; break;  /* Item_80268DD4 */
        case 32: n = 300; ft = 0; break;  /* Item_80268E40 */
        case 33: n = 300; ft = 0; break;  /* Item_80268E5C */
        case 34: n = 300; ft = 0; break;  /* Item_802693E4 */
        case 35: n = 300; ft = 0; break;  /* Item_802694CC */
        case 36: n = 300; ft = 0; break;  /* Item_80269528 */
        case 37: n = 300; ft = 0; break;  /* Item_802696CC */
        case 38: n = 300; ft = 0; break;  /* Item_802697D4 */
        case 39: n = 300; ft = 0; break;  /* Item_80269978 */
        case 40: n = 300; ft = 0; break;  /* Item_80269A9C */
        case 41: n = 300; ft = 0; break;  /* Item_80269B60 */
        case 42: n = 300; ft = 0; break;  /* Item_80269BE4 */
        case 43: n = 300; ft = 0; break;  /* Item_80269C5C */
        case 44: n = 300; ft = 0; break;  /* Item_80269CA0 */
        case 45: n = 300; ft = 0; break;  /* Item_80269CC4 */
        case 46: n = 300; ft = 0; break;  /* Item_80269DC8 */
        case 47: n = 300; ft = 0; break;  /* Item_80269F14 */
        case 48: n = 300; ft = 0; break;  /* Item_8026A0A0 */
        case 49: n = 300; ft = 0; break;  /* Item_8026A0FC */
        case 50: n = 300; ft = 0; break;  /* func_8026A158_helper */
        case 51: n = 300; ft = 0; break;  /* Item_8026A158 */
        case 52: n = 300; ft = 0; break;  /* func_8026A1E8_inline */
        case 53: n = 300; ft = 0; break;  /* Item_8026A1E8 */
        case 54: n = 300; ft = 0; break;  /* processCallback */
        case 55: n = 300; ft = 0; break;  /* OnTakeDamageThink */
        case 56: n = 300; ft = 0; break;  /* OnClankThink */
        case 57: n = 300; ft = 0; break;  /* OnGiveDamageThink */
        case 58: n = 300; ft = 0; break;  /* EnterHitlagThink */
        case 59: n = 300; ft = 0; break;  /* checkHitLag */
        case 60: n = 300; ft = 0; break;  /* Item_8026A294 */
        case 61: n = 300; ft = 0; break;  /* Item_ScriptCommitDamage */
        case 62: n = 300; ft = 0; break;  /* Item_8026A788 */
        case 63: n = 300; ft = 0; break;  /* Item_8026A810 */
        case 64: n = 300; ft = 0; break;  /* Item_8026A848 */
        case 65: n = 300; ft = 0; break;  /* DestroyItemInline */
        case 66: n = 300; ft = 0; break;  /* ItemSwitch */
        case 67: n = 300; ft = 0; break;  /* RunGObjCallback */
        case 68: n = 300; ft = 0; break;  /* func_8026A8EC_inline1 */
        case 69: n = 300; ft = 0; break;  /* func_8026A8EC_inline2 */
        case 70: n = 300; ft = 0; break;  /* func_8026A8EC_inline3 */
        case 71: n = 300; ft = 0; break;  /* Item_8026A8EC */
        case 72: n = 300; ft = 0; break;  /* Item_8026AB54 */
        case 73: n = 300; ft = 0; break;  /* Item_8026ABD8 */
        case 74: n = 300; ft = 0; break;  /* Item_8026AC74 */
        case 75: n = 300; ft = 0; break;  /* Item_8026AD20 */
        case 76: n = 300; ft = 0; break;  /* Item_8026ADC0 */
        case 77: n = 300; ft = 0; break;  /* Item_OnUserDataRemove */
        case 78: n = 300; ft = 0; break;  /* Item_8026AE60 */
        case 79: n = 300; ft = 0; break;  /* Item_MexSfx */
        case 80: n = 300; ft = 0; break;  /* Item_8026AE84 */
        case 81: n = 300; ft = 0; break;  /* Item_8026AF0C */
        case 82: n = 300; ft = 0; break;  /* Item_8026AFA0 */
        case 83: n = 300; ft = 0; break;  /* Item_8026B034 */
        case 84: n = 300; ft = 0; break;  /* Item_8026B074 */
        case 85: n = 300; ft = 0; break;  /* Item_8026B0B4 */
        case 86: n = 300; ft = 0; break;  /* Item_IsGrabbable */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
            CUR_OP = op;
            setup_stem();
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            switch (op) {
            case 0: {  ret = (u32) Item_80266F3C();  } break;
            case 1: {  Item_80266F70();  } break;
            case 2: {  Item_80266FA8();  } break;
            case 3: {  Item_80266FCC();  } break;
            case 4: { HSD_GObj * a0 = &GO[rnd() % 4]; ItUnkHoldKind(a0);  } break;
            case 5: { Item * a0 = &IT[rnd() % 4]; HSD_JObj * a1 = &JO[rnd() % 8]; Vec3 * a2 = &VA[rnd() % 4]; HSD_JObjSetScaleItem(a0, a1, a2);  } break;
            case 6: { HSD_JObj * a0 = &JO[rnd() % 8]; Item * a1 = &IT[rnd() % 4]; HSD_JObjSetFacingDirItem(a0, a1);  } break;
            case 7: { HSD_GObj * a0 = &GO[rnd() % 4]; SpawnItem * a1 = &SP; Item_80267130(a0, a1);  } break;
            case 8: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_80267454(a0);  } break;
            case 9: { SpawnItem * a0 = &SP; Item_802674AC(a0);  } break;
            case 10: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_802675A8(a0);  } break;
            case 11: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_802676F4(a0);  } break;
            case 12: { enum_t a0 = fv(); int a1 = fv(); ret = (u32) Item_8026784C(a0, a1);  } break;
            case 13: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_80267978(a0);  } break;
            case 14: { HSD_GObj * a0 = &GO[rnd() % 4]; SpawnItem * a1 = &SP; Item_80267AA8(a0, a1);  } break;
            case 15: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_802680CC(a0);  } break;
            case 16: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_8026814C(a0);  } break;
            case 17: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) Item_802682F0(a0);  } break;
            case 18: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_8026849C(a0);  } break;
            case 19: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_80268560(a0);  } break;
            case 20: { HSD_GObj * a0 = &GO[rnd() % 4]; foobar(a0);  } break;
            case 21: { HSD_GObj * a0 = &GO[rnd() % 4]; foobar2(a0);  } break;
            case 22: { HSD_GObj * a0 = &GO[rnd() % 4]; foobar3(a0);  } break;
            case 23: { SpawnItem * a0 = &SP; ret = PI(Item_8026862C(a0));  } break;
            case 24: { SpawnItem * a0 = &SP; ret = (u32) Item_ScriptSpawnAtCap(a0);  } break;
            case 25: {  ret = (u32) Item_ScriptSpawnCapTest();  } break;
            case 26: { SpawnItem * a0 = &SP; ret = PI(Item_80268B18(a0));  } break;
            case 27: { SpawnItem * a0 = &SP; ret = PI(Item_80268B5C(a0));  } break;
            case 28: { SpawnItem * a0 = &SP; Item_80268B9C(a0);  } break;
            case 29: { HSD_JObj * a0 = &JO[rnd() % 8]; HSD_AnimJoint * a1 = ((rnd() & 3) ? &AJ[rnd() % 4] : 0); HSD_MatAnimJoint * a2 = ((rnd() & 3) ? &MJ[rnd() % 4] : 0); HSD_ShapeAnimJoint * a3 = ((rnd() & 3) ? &SJ[rnd() % 4] : 0); Item * a4 = &IT[rnd() % 4]; Item_80268BE0(a0, a1, a2, a3, a4);  } break;
            case 30: { HSD_GObj * a0 = &GO[rnd() % 4]; struct ItemStateDesc * a1 = &SDS[rnd() % 3]; Item_80268D34(a0, a1);  } break;
            case 31: { HSD_GObj * a0 = &GO[rnd() % 4]; f32 a1 = rw(); Item_80268DD4(a0, a1);  } break;
            case 32: { Item * a0 = &IT[rnd() % 4]; struct ItemStateDesc * a1 = &SDS[rnd() % 3]; Item_80268E40(a0, a1);  } break;
            case 33: { HSD_GObj * a0 = &GO[rnd() % 4]; enum_t a1 = fv() & 3; Item_StateChangeFlags a2 = fv(); Item_80268E5C(a0, a1, a2);  } break;
            case 34: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_802693E4(a0);  } break;
            case 35: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_802694CC(a0);  } break;
            case 36: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_80269528(a0);  } break;
            case 37: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) Item_802696CC(a0);  } break;
            case 38: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_802697D4(a0);  } break;
            case 39: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_80269978(a0);  } break;
            case 40: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_80269A9C(a0);  } break;
            case 41: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_80269B60(a0);  } break;
            case 42: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_80269BE4(a0);  } break;
            case 43: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_80269C5C(a0);  } break;
            case 44: { Item * a0 = &IT[rnd() % 4]; s32 a1 = fv(); Item_80269CA0(a0, a1);  } break;
            case 45: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_80269CC4(a0);  } break;
            case 46: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) Item_80269DC8(a0);  } break;
            case 47: { HSD_GObj * a0 = &GO[rnd() % 4]; ret = (u32) Item_80269F14(a0);  } break;
            case 48: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_8026A0A0(a0);  } break;
            case 49: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_8026A0FC(a0);  } break;
            case 50: { HSD_GObj * a0 = &GO[rnd() % 4]; func_8026A158_helper(a0);  } break;
            case 51: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_8026A158(a0);  } break;
            case 52: { HSD_GObj * a0 = &GO[rnd() % 4]; func_8026A1E8_inline(a0);  } break;
            case 53: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_8026A1E8(a0);  } break;
            case 54: { HSD_GObjPredicate a0 = (HSD_GObjPredicate) pick_cb(2); HSD_GObj * a1 = &GO[rnd() % 4]; Item * a2 = &IT[rnd() % 4]; ret = (u32) processCallback(a0, a1, a2);  } break;
            case 55: { HSD_GObj * a0 = &GO[rnd() % 4]; Item * a1 = &IT[rnd() % 4]; ret = (u32) OnTakeDamageThink(a0, a1);  } break;
            case 56: { HSD_GObj * a0 = &GO[rnd() % 4]; Item * a1 = &IT[rnd() % 4]; ret = (u32) OnClankThink(a0, a1);  } break;
            case 57: { HSD_GObj * a0 = &GO[rnd() % 4]; Item * a1 = &IT[rnd() % 4]; ret = (u32) OnGiveDamageThink(a0, a1);  } break;
            case 58: { HSD_GObj * a0 = &GO[rnd() % 4]; Item * a1 = &IT[rnd() % 4]; EnterHitlagThink(a0, a1);  } break;
            case 59: { f32 a0 = rw(); Item * a1 = &IT[rnd() % 4]; checkHitLag(a0, a1);  } break;
            case 60: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_8026A294(a0);  } break;
            case 61: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) Item_ScriptCommitDamage(a0);  } break;
            case 62: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_8026A788(a0);  } break;
            case 63: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_8026A810(a0);  } break;
            case 64: { HSD_GObj * a0 = &GO[rnd() % 4]; HSD_GObj * a1 = &GO[rnd() % 4]; Item_8026A848(a0, a1);  } break;
            case 65: { HSD_GObj * a0 = &GO[rnd() % 4]; Item * a1 = &IT[rnd() % 4]; DestroyItemInline(a0, a1);  } break;
            case 66: { HSD_GObj * a0 = &GO[rnd() % 4]; ItemSwitch(a0);  } break;
            case 67: { HSD_GObj * a0 = &GO[rnd() % 4]; HSD_GObjEvent a1 = (HSD_GObjEvent) pick_cb(1); RunGObjCallback(a0, a1);  } break;
            case 68: { HSD_GObj * a0 = &GO[rnd() % 4]; func_8026A8EC_inline1(a0);  } break;
            case 69: { HSD_GObj * a0 = &GO[rnd() % 4]; func_8026A8EC_inline2(a0);  } break;
            case 70: { HSD_GObj * a0 = &GO[rnd() % 4]; func_8026A8EC_inline3(a0);  } break;
            case 71: { Item_GObj * a0 = &GO[rnd() % 4]; Item_8026A8EC(a0);  } break;
            case 72: { Item_GObj * a0 = &GO[rnd() % 4]; HSD_GObj * a1 = &GO[rnd() % 4]; Fighter_Part a2 = fv(); Item_8026AB54(a0, a1, a2);  } break;
            case 73: { Item_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; f32 a2 = rw(); Item_8026ABD8(a0, a1, a2);  } break;
            case 74: { HSD_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; Vec3 * a2 = &VA[rnd() % 4]; f32 a3 = rw(); Item_8026AC74(a0, a1, a2, a3);  } break;
            case 75: { HSD_GObj * a0 = &GO[rnd() % 4]; Vec3 * a1 = &VA[rnd() % 4]; Vec3 * a2 = &VA[rnd() % 4]; f32 a3 = rw(); bool a4 = fv(); Item_8026AD20(a0, a1, a2, a3, a4);  } break;
            case 76: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_8026ADC0(a0);  } break;
            case 77: { void * a0 = (void*) &IT[rnd() % 4]; Item_OnUserDataRemove(a0);  } break;
            case 78: {  ret = (u32) Item_8026AE60();  } break;
            case 79: { Item * a0 = &IT[rnd() % 4]; enum_t a1 = fv(); ret = (u32) Item_MexSfx(a0, a1);  } break;
            case 80: { Item * a0 = &IT[rnd() % 4]; enum_t a1 = ((rnd() % 4) == 0 ? 540000 + (int) (rnd() & 1) : (rnd() % 3) == 0 ? 5000 + (int) (rnd() % 5000) : (int) fv()); u8 a2 = fv(); u8 a3 = fv(); Item_8026AE84(a0, a1, a2, a3);  } break;
            case 81: { Item * a0 = &IT[rnd() % 4]; enum_t a1 = ((rnd() % 4) == 0 ? 540000 + (int) (rnd() & 1) : (rnd() % 3) == 0 ? 5000 + (int) (rnd() % 5000) : (int) fv()); u8 a2 = fv(); u8 a3 = fv(); Item_8026AF0C(a0, a1, a2, a3);  } break;
            case 82: { Item * a0 = &IT[rnd() % 4]; enum_t a1 = ((rnd() % 4) == 0 ? 540000 + (int) (rnd() & 1) : (rnd() % 3) == 0 ? 5000 + (int) (rnd() % 5000) : (int) fv()); u8 a2 = fv(); u8 a3 = fv(); Item_8026AFA0(a0, a1, a2, a3);  } break;
            case 83: { Item * a0 = &IT[rnd() % 4]; Item_8026B034(a0);  } break;
            case 84: { Item * a0 = &IT[rnd() % 4]; Item_8026B074(a0);  } break;
            case 85: { HSD_GObj * a0 = &GO[rnd() % 4]; Item_8026B0B4(a0);  } break;
            case 86: { Item_GObj * a0 = &GO[rnd() % 4]; ret = (u32) Item_IsGrabbable(a0);  } break;
            }
            hash_world(ft);
            hash_stem();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
