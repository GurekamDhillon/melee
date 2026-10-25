/* TU: src/melee/ft/fighter.c */
/* C oracle for src/tu/fighter.rs (the per-frame fighter functions): the whole TU included verbatim; every
 * callee out of it is a recording stub (gen/mtu/stubs.py); the state is a random fighter world filled field by
 * field (gen/mtu/fill.py). Each line: op case ret state-hash trace. */
#include TU_SRC
/* the game links lbtrigf.c's atan2f (not the CRT's); the port calls melee_runtime::lbtrigf */
#include <melee/lb/lbtrigf.c>
#include "mtu_common.h"
/* PC profiler diagnostics: not game logic, not ported, not traced */
int ProfActive(void) { return 0; }
void ProfBegin(int a, int b) { (void) a; (void) b; }
void ProfEnd(void) {}
void ProfCount(int a) { (void) a; }

#define NG 3
#define NJ 8
static HSD_GObj GO[NG];
static HSD_JObj JO[NJ];
static Fighter FT[NG];
static ftData FDATA[NG];
static ftCommonData CD;
static u8 PADB[96][2048];
static FighterBone PARTS[NG][16];
static struct plAllocInfo PAI;
static void* PLCO[23];
static struct Fighter_804D64FC_t SRC64;
static u32 SEEDV;
static HSD_GObj IGO[2];
static Item ITM[2];
static HSD_GObj* PLINKV[16];
static u32 OWNER;
static UnkCostumeStruct CLB[256];

static HSD_GObj* pick_HSD_GObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &GO[(r >> 8) % NG]; }
static HSD_JObj* pick_HSD_JObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &JO[(r >> 8) % NJ]; }
static Fighter* pick_Fighter(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &FT[(r >> 8) % NG]; }
static Item* pick_Item(void) { return 0; }
static HSD_DObj* pick_HSD_DObj(void) { return 0; }
static void cbev1(HSD_GObj* g) { T(7001); T(PI(g)); }
static void cbev2(HSD_GObj* g) { T(7002); T(PI(g)); }
static int cbp1(HSD_GObj* g) { T(7003); T(PI(g)); return (int) (fv() & 1); }
static int cbp2(HSD_GObj* g) { T(7004); T(PI(g)); return (int) (fv() & 1); }
static void cbev3(HSD_GObj* g, HSD_GObj* h) { T(7005); T(PI(g)); T(PI(h)); }
static void* pick_fn(int kind)
{
    u32 r = rnd();
    if ((r & 3) == 0) return 0;
    switch (kind) {
    case 1: return (r & 4) ? (void*) cbev1 : (void*) cbev2;
    case 2: return (r & 4) ? (void*) cbp1 : (void*) cbp2;
    case 3: return (void*) cbev3;
    }
    return 0;
}
static void* pick_cb(int kind) { void* f = pick_fn(kind); return f ? f : (kind == 2 ? (void*) cbp1 : kind == 3 ? (void*) cbev3 : (void*) cbev1); }

#include "fighter_stubs.h"
/* ftParts_SetupParts allocates fp->parts: here the fixed bone table */
void ftParts_SetupParts(Fighter_GObj* g) { T(1901); T(PI(g)); ((Fighter*) g->user_data)->parts = PARTS[0]; }
HSD_Archive* lbArchive_LoadSymbols(const char* filename, void* symbols, ...)
{
    T(1900); T(SH(filename));
    *(void***) symbols = PLCO;
    return 0;
}
#include "fighter_state.h"

static void world(void)
{
    int i;
    zero(GO, sizeof GO); zero(JO, sizeof JO); zero(FT, sizeof FT); zero(FDATA, sizeof FDATA);
    for (i = 0; i < NG; i++) fill_Fighter(&FT[i]);
    for (i = 0; i < NJ; i++) fill_HSD_JObj(&JO[i]);
    for (i = 0; i < NG; i++) fill_HSD_GObj(&GO[i]);
    for (i = 0; i < NG; i++) {
        GO[i].user_data = &FT[i];
        GO[i].hsd_obj = &JO[i];
        GO[i].classifier = HSD_GOBJ_CLASS_FIGHTER;
        FT[i].gobj = &GO[i];
        FT[i].ft_data = &FDATA[i];
        FT[i].grab_cb = cbev1; FT[i].grabbed_cb = cbev3;
        FT[i].kind = (FighterKind) ((u32) FT[i].kind % 27u); FT[i].x200C &= 3; FT[i].x2010 &= 3; FT[i].player_id = FT[i].player_id % 4; FT[i].x618_player_id = FT[i].x618_player_id % 4; FT[i].x61A_controller_index = FT[i].x61A_controller_index % 4;
        { int pk = i * 24; FDATA[i].x0 = (void*) PADB[pk++]; FDATA[i].ext_attr = (void*) PADB[pk++]; FDATA[i].x8 = (void*) PADB[pk++]; FDATA[i].xC = (void*) PADB[pk++]; FDATA[i].x10 = (void*) PADB[pk++]; FDATA[i].x14 = (void*) PADB[pk++]; FDATA[i].x18 = (void*) PADB[pk++]; FDATA[i].x1C = (void*) PADB[pk++]; FDATA[i].x20 = (void*) PADB[pk++]; FDATA[i].x24 = (void*) PADB[pk++]; FDATA[i].x28 = (void*) PADB[pk++]; FDATA[i].x2C = (void*) PADB[pk++]; FDATA[i].x30 = (void*) PADB[pk++]; FDATA[i].x34 = (void*) PADB[pk++]; FDATA[i].x38 = (void*) PADB[pk++]; FDATA[i].x3C = (void*) PADB[pk++]; FDATA[i].x40 = (void*) PADB[pk++]; FDATA[i].x44 = (void*) PADB[pk++]; FDATA[i].x48_items = (void*) PADB[pk++]; FDATA[i].x4C_sfx = (void*) PADB[pk++]; FDATA[i].x50 = (void*) PADB[pk++]; FDATA[i].x54 = (void*) PADB[pk++]; FDATA[i].x58 = (void*) PADB[pk++]; FDATA[i].x5C = (void*) PADB[pk++]; }
    }
    for (i = 0; i < NJ; i++) {
        JO[i].parent = i ? &JO[(i - 1) / 2] : 0;
        JO[i].child = 2 * i + 1 < NJ ? &JO[2 * i + 1] : 0;
        JO[i].next = (i & 1) && i + 1 < NJ ? &JO[i + 1] : 0;
    }
    for (i = 0; i < Ft_Kind_Max; i++) CostumeListsForeachCharacter[i].costume_list = CLB;
    p_ftCommonData = &CD;
    for (i = 0; i < NG; i++) {
        int b;
        for (b = 0; b < 16; b++) { zero(&PARTS[i][b], sizeof PARTS[i][b]); PARTS[i][b].joint = &JO[(u32) (i * 5 + b * 3) % NJ]; }
        FT[i].parts = PARTS[i];
    }
    for (i = 0; i < Ft_Kind_Max; i++) gFtDataList[i] = &FDATA[(u32) i % NG];
    for (i = 0; i < 4; i++) { CD.x6DC_colorsByPlayer[i].r = (u8) fv(); CD.x6DC_colorsByPlayer[i].g = (u8) fv(); CD.x6DC_colorsByPlayer[i].b = (u8) fv(); CD.x6DC_colorsByPlayer[i].a = (u8) fv(); }
    CD.x7E4_scaleZ = rg();
    fill_plAllocInfo(&PAI);
    PAI.internal_id = (FighterKind) ((u32) PAI.internal_id % 27u); PAI.slot %= 4;
    {   /* the PlCo.dat symbol table: 23 pointers to registered blocks; rows 76/77 and SRC64's eight are kind-indexed tables */
        int r, k;
        for (i = 0; i < 23; i++) PLCO[i] = PADB[72 + i];
        for (r = 72; r < 96; r++) for (k = 0; k < 256; k++) ((void**) PADB[r])[k] = PADB[(u32) (k * 7 + r) % 72];
        zero(&SRC64, sizeof SRC64);
        SRC64.x4 = (void*) PADB[72]; SRC64.x8 = (void*) PADB[73]; SRC64.xC = (void*) PADB[74]; SRC64.x10 = (void*) PADB[75];
        SRC64.x14 = (void*) PADB[76]; SRC64.x18 = (void*) PADB[77]; SRC64.x1C = (void*) PADB[78]; SRC64.x20 = (void*) PADB[79];
        PLCO[22] = &SRC64;
    }
    Fighter_804D64FC = 0; ftPartsTable = 0; Fighter_804D6540 = 0; Fighter_804D6550 = 0; Fighter_804D654C = 0; Fighter_804D6548 = 0;
    SEEDV = fv(); HSD_RandSeedPtr = &SEEDV;
    for (i = 0; i < 2; i++) { zero(&IGO[i], sizeof IGO[i]); zero(&ITM[i], sizeof ITM[i]); IGO[i].user_data = &ITM[i]; ITM[i].kind = (enum_t) fv(); ITM[i].msid = (enum_t) fv(); ITM[i].pos.x = rw(); ITM[i].pos.y = rw(); ITM[i].pos.z = rw(); ITM[i].x40_vel.x = rw(); ITM[i].x40_vel.y = rw(); ITM[i].x40_vel.z = rw(); }
    IGO[0].next = &IGO[1]; IGO[1].next = 0;
    for (i = 0; i < 16; i++) PLINKV[i] = 0;
    PLINKV[HSD_GOBJ_PLINK_ITEM] = (fv() & 1) ? &IGO[0] : 0;
    HSD_GObjPLinkHead = PLINKV;
}
static void hglob(void)
{
    int i, j;
    void*** f64[8];
    HM(g_spawnNumCounter);
    HM(PI(p_ftCommonData)); HM(PI(Fighter_804D6550)); HM(PI(Fighter_804D654C)); HM(PI(Fighter_804D6548));
    HM(PI(Fighter_804D653C)); HM(PI(Fighter_804D6538)); HM(PI(Fighter_804D6534)); HM(PI(Fighter_804D6530));
    HM(PI(Fighter_GrabMashShake)); HM(PI(Fighter_SmashChargeShakeTable)); HM(PI(Fighter_804D6524)); HM(PI(Fighter_804D6520));
    HM(PI(Fighter_804D651C)); HM(PI(Fighter_804D6518)); HM(PI(Fighter_804D6514)); HM(PI(Fighter_804D6510));
    HM(PI(Fighter_804D650C)); HM(PI(Fighter_804D6508)); HM(PI(Fighter_804D6504)); HM(PI(gCrowdConfig));
    HM(PI(ftPartsTable)); HM(PI(Fighter_804D6540)); HM(PI(Fighter_804D64FC));
    if (ftPartsTable) for (i = 0; i < Ft_Kind_Max; i++) HM(PI(ftPartsTable[i]));
    if (Fighter_804D6540) for (i = 0; i < Ft_Kind_Max; i++) HM(PI(Fighter_804D6540[i]));
    if (Fighter_804D64FC) {
        f64[0] = (void***) &Fighter_804D64FC->x4; f64[1] = (void***) &Fighter_804D64FC->x8; f64[2] = (void***) &Fighter_804D64FC->xC;
        f64[3] = (void***) &Fighter_804D64FC->x10; f64[4] = (void***) &Fighter_804D64FC->x14; f64[5] = (void***) &Fighter_804D64FC->x18;
        f64[6] = (void***) &Fighter_804D64FC->x1C; f64[7] = (void***) &Fighter_804D64FC->x20;
        for (j = 0; j < 8; j++) { HM(PI(*f64[j])); if (*f64[j]) for (i = 0; i < Ft_Kind_Max; i++) HM(PI((*f64[j])[i])); }
    }
    for (i = 0; i < 16; i++) HM(nh_out[i]);
}
static void hworld(void)
{
    int i;
    hglob();
    if (HDBG) { int w; const u32* q = (const u32*) FT; for (w = 0; w < (int) (sizeof FT / 4); w++) if (q[w]) printf("W %05X %08X%c", w * 4, q[w], 10); }
    for (i = 0; i < NG; i++) hash_Fighter(&FT[i]);
    for (i = 0; i < NJ; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < NG; i++) hash_HSD_GObj(&GO[i]);
}

int main(void)
{
    int op, k;
    preg(PADB, sizeof PADB); preg(CLB, sizeof CLB); preg(&CD, sizeof CD); preg(PARTS, sizeof PARTS); preg(&SRC64, sizeof SRC64); preg(IGO, sizeof IGO); preg(ITM, sizeof ITM); preg(&PAI, sizeof PAI);
    preg(GO, sizeof GO); preg(JO, sizeof JO); preg(FT, sizeof FT); preg(FDATA, sizeof FDATA);
    freg((const void*) cbev1); freg((const void*) cbev2); freg((const void*) cbp1); freg((const void*) cbp2); freg((const void*) cbev3);
    for (op = 0; op < 36; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            union { float f; u32 u; } rf_;
            tu_case(op, k);
            world();
            ASRT_ARMED = 1;
            if (_setjmp(ASRT_JB, 0) != 0) { printf("%d %d ASSERT%c", op, k, 10); fflush(0); continue; }
            switch (op) {
            case 0: Fighter_8006C5F4(&GO[rnd() % NG]); break;
            case 1: Fighter_CallAcessoryCallbacks_8006C624(&GO[rnd() % NG]); break;
            case 2: Fighter_8006C80C(&GO[rnd() % NG]); break;
            case 3: Fighter_UnkProcessGrab_8006CA5C(&GO[rnd() % NG]); break;
            case 4: Fighter_8006CB94(&GO[rnd() % NG]); break;
            case 5: Fighter_8006D9AC(&GO[rnd() % NG]); break;
            case 6: Fighter_UnkCallCameraCallback_8006D9EC(&GO[rnd() % NG]); break;
            case 7: Fighter_8006DA4C(&GO[rnd() % NG]); break;
            case 8: Fighter_8006A1BC(&GO[rnd() % NG]); break;
            case 9: Fighter_8006A360(&GO[rnd() % NG]); break;
            case 10: Fighter_8006ABA0(&GO[rnd() % NG]); break;
            case 11: Fighter_UnkIncrementCounters_8006ABEC(&GO[rnd() % NG]); break;
            case 12: Fighter_Spaghetti_8006AD10(&GO[rnd() % NG]); break;
            case 13: Fighter_procUpdate(&GO[rnd() % NG]); break;
            case 14: Fighter_UnkApplyTransformation_8006C0F0(&GO[rnd() % NG]); break;
            case 15: Fighter_procMap(&GO[rnd() % NG]); break;
            case 16: Fighter_ProcessHit_8006D1EC(&GO[rnd() % NG]); break;
            case 17: Fighter_UnkProcessDeath_80068354(&GO[rnd() % NG]); break;
            case 18: Fighter_UnkUpdateCostumeJoint_800686E4(&GO[rnd() % NG]); break;
            case 19: Fighter_ResetInputData_80068854(&GO[rnd() % NG]); break;
            case 20: Fighter_UnkInitReset_80067C98(&FT[rnd() % NG]); break;
            case 21: Fighter_UnkUpdateVecFromBones_8006876C(&FT[rnd() % NG]); break;
            case 22: Fighter_UnkInitLoad_80068914(&GO[rnd() % NG], &PAI); break;
            case 23: Fighter_Create_Inline2(&GO[rnd() % NG]); break;
            case 24: ret = PI(Fighter_Create(&PAI)); break;
            case 25: Fighter_FirstInitialize_80067A84(); break;
            case 26: Fighter_800679B0(); break;
            case 27: Fighter_LoadCommonData(); break;
            case 28: { int sl = (int) (rnd() & 1); ret = PI(ftCommonData_ExtendKindTable((void**) PADB[76 + sl], sl)); break; }
            case 29: ret = PI(ftCommonData_ExtendCpuTables((rnd() & 7) ? &SRC64 : 0)); break;
            case 30: ret = ftSlippi_IsOffscreen(&FT[rnd() % NG]) != 0; break;
            case 31: ftReplay_TraceFighter(&FT[rnd() % NG]); break;
            case 32: ret = RB_NeutralHash(); break;
            case 33: ScriptGame_CpuTechnicalClear((int) (fv() % 6)); break;
            case 34: ScriptGame_CpuAssistClear((int) (fv() % 6)); break;
            case 35: ret = (u32) ftUcf_FlickCounter((int) (rnd() % 4)); break;
            }
            hworld();
            ASRT_ARMED = 0;
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
            fflush(0);
        }
    }
    return 0;
}
