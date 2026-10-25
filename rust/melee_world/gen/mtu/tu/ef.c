/* TU: src/melee/ef/{efdata,eflib,efasync}.c as one TU */
/* C oracle for the Rust ports of these TUs: the game's files included verbatim (the efSync_Spawn call sites
 * counted, see specs_ef.py); every callee out of them is a recording stub. Written by gen/mtu/mk_ef.py.
 * Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#define __assert mtu_unused_assert
#include "mtu_common.h"
#undef __assert
extern void setbuf(void*, char*);
extern void* __acrt_iob_func(unsigned);
extern int fprintf(void*, const char*, ...);
extern char* getenv(const char*);
extern void RaiseException(unsigned long, unsigned long, unsigned long, const void*);
static int ASLINE[512], LASTAL; static const char* LASTC; static const char* ASCOND[512];
void __assert(const char* file, u32 line, const char* cond) { (void) file; LASTC = cond; LASTAL = (int) line; RaiseException(0xE0A55E87ul, 1, 0, 0); }
void* memset(void* d, int c, unsigned long n) { unsigned char* p = (unsigned char*) d; while (n--) *p++ = (unsigned char) c; return d; }
void memzero(void* d, int n) { unsigned char* p = (unsigned char*) d; while (n-- > 0) *p++ = 0; }
int __HI(double x) { union { double d; u64_t_ u; } c; c.d = x; return (int) (c.u >> 32); }
int __LO(double x) { union { double d; u64_t_ u; } c; c.d = x; return (int) (u32) c.u; }
static EF_Effect EFF[10];
static EF_QueuedEffect QE[8];
static EF_QueuedEffect QH[1];
static HSD_GObj GO[9];
static HSD_JObj JO[16];
static Fighter FT[2];
static Item ITM[1];
static HSD_DObj DO[4];
static HSD_MObj MO[4];
static HSD_TObj TO[8];
static HSD_TObjTev TV[8];
static HSD_Generator GEN[3];
static HSD_psAppSRT APS[3];
static HSD_Particle PT[2];
static HSD_GObjProc PROC[2];
static EF_EffectDesc DESC[64];
static EF_DAT_Entry DE[2];
static EF_DAT_Entry DEINIT[65];
static EfMexBehavior BHV;
static u8 BT1[16], BT2[16];
static char STRS[3][24] = { "MexA.dat", "MexB.dat", "MexC.dat" };
static HSD_Archive ARC[1];
static HSD_GObj* PLH[64];
static Vec3 V3[6];
static float FL[6];
static struct _HSD_PSCmdList CMD[5];
static struct _HSD_PSCmdList* PREF[0x220];
static u8 AJ[4];
static int EFN, QN;
static s32 OUTI[4];
static void* OUTP[2];
static int mtu_dbgp;
static int ASSN[512];
static void fprintf_stub(int op) { fprintf(__acrt_iob_func(2), "ASSERTS %d %d line %d %s%c", op, ASSN[op], ASLINE[op], ASCOND[op] ? ASCOND[op] : "", 10); }
static float RNN(void) { static const float k[4] = { 1.0f, 13.0f, 100.0f, 1000.0f }; float a = (float) ((int) (rnd() % 41) - 20) / 20.0f; return a * k[rnd() % 4]; }
#include "ef_gen.h"
/* the Fighter fields the ef callbacks read (the full record has bitfield overlays the walker cannot map) */
static void fill_FighterEf(Fighter* p)
{
    p->facing_dir = rg(); p->motion_id = (s32) fv(); p->cmd_vars[0] = fv(); p->cmd_vars[1] = fv(); p->cmd_vars[2] = fv(); p->cmd_vars[3] = fv();
    p->mv.co.common.x4.x = rg(); p->mv.co.common.x4.y = rg(); p->mv.co.common.x4.z = rg();
    p->mv.co.common.x10 = (rnd() & 1) ? (void*) &AJ[0] : 0;
    p->coll_data.floor.normal.x = rg(); p->coll_data.floor.normal.y = rg();
    p->u.kb.hat.kind = (int) fv(); p->kind = (int) fv();
}
static void hash_FighterEf(const Fighter* p)
{
    HM(B(p->facing_dir)); HM((u32) p->motion_id); HM(p->cmd_vars[0]); HM(p->cmd_vars[1]); HM(p->cmd_vars[2]); HM(p->cmd_vars[3]);
    HM(B(p->mv.co.common.x4.x)); HM(B(p->mv.co.common.x4.y)); HM(B(p->mv.co.common.x4.z)); HM(PI(p->mv.co.common.x10));
    HM(B(p->coll_data.floor.normal.x)); HM(B(p->coll_data.floor.normal.y)); HM((u32) p->u.kb.hat.kind); HM((u32) p->kind);
}

static const int DID[] = { 1000, 1001, 1002, 1003, 1004, 1005, 1006, 1007, 1008, 1009, 1010, 1011, 1012, 1013, 1014, 1015, 1016, 1017, 1018, 1019, 1020, 1021, 1022, 1023, 1024, 1025, 1026, 1027, 1028, 1029, 1030, 1031, 1032, 1033, 1034, 1035, 1036, 1037, 1038, 1039, 1040, 1041, 1042, 1043, 1044, 1045, 1046, 1047, 1048, 1049, 1050, 1051, 1052, 1053, 1054, 1055, 1056, 1057, 1058, 1059, 1060, 1061, 1062, 1063, 1064, 1065, 1066, 1067, 1068, 1069, 1070, 1071, 1072, 1073, 1074, 1075, 1076, 1077, 1078, 1079, 1080, 1081, 1082, 1083, 1084, 1085, 1086, 1087, 1088, 1089, 1090, 1091, 1092, 1093, 1094, 1095, 1096, 1097, 1098, 1099, 1100, 1101, 1102, 1111, 1103, 1112, 1104, 1113, 1105, 1114, 1107, 1108, 1109, 1110, 1106, 1115, 1116, 1117, 1118, 1119, 1120, 1121, 1122, 1123, 1124, 1125, 1126, 1127, 1128, 1129, 1130, 1131, 1132, 1133, 1134, 1135, 1136, 1137, 1138, 1139, 1143, 1140, 1141, 1142 };
static const char* DKIND[] = { "VVFVVV", "VVVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "VFVVVV", "VVVVVV", "VFFVVV", "VFFVVV", "VFFVVV", "VFFVVV", "VVVVVV", "VVVVVV", "VFVVVV", "VVVVVV", "VFFVVV", "VFFVVV", "VFFVVV", "VVVVVV", "VVVVVV", "VVVVVV", "VFFVVV", "VFVVVV", "VFFVVV", "VFFVVV", "VFFVVV", "JVVVVV", "JVVVVV", "VFVVVV", "VVVVVV", "VFVVVV", "VVVVVV", "VFVVVV", "JVVVVV", "VFVVVV", "VFVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "JVVVVV", "JVVVVV", "JVVVVV", "JFVVVV", "VVVVVV", "JUVVVV", "JUVVVV", "JUVVVV", "JUVVVV", "JVVVVV", "VVVVVV", "VVVVVV", "JFVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "JVVVVV", "JFVVVV", "JFVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "VFFUUV", "VFVVVV", "VVVVVV", "VFVVVV", "VFVVVV", "VFVVVV", "VFVVVV", "VFVVVV", "JVVVVV", "VVVVVV", "VFVVVV", "VFVVVV", "VFVVVV", "JFVVVV", "JFVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "JVVVVV", "JFVVVV", "VFVVVV", "VFVVVV", "VVVVVV", "VVVVVV", "VFVVVV", "VVVVVV", "VVVVVV", "JVVVVV", "VFVVVV", "PPVVVV", "JVVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "JFVVVV", "JFFVVV", "JFFVVV", "JFFVVV", "JFFVVV", "JFFVVV", "JFFVVV", "JFFVVV", "JFFVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JFVVVV", "JVVVVV", "VVVVVV", "VVVVVV", "VVVVVV", "VVVVVV" };
static int is_d(int g) { unsigned i; for (i = 0; i < sizeof DID / sizeof DID[0]; i++) if (DID[i] == g) return 1; return 0; }
static const int SPLIST[] = { 19000, 19001, 19002, 47000, 47001, 47002, 45, 46, 49, 295, 2, 6, 163, 167, 170, 242, 298, 306, 307, 365, 366, 367, 368, 2018, 212, 579, 227, 252, 255, 247, 31000, 289, 85, 485 };
static void (*const UPD[])(EF_Effect*) = { efLib_Cb_SetOffsetY_FromParamY, efLib_Cb_SetScale_FromParamX, efLib_Cb_SetRotYAndTransition, efLib_Cb_SetJObjOffsetZ, efLib_Cb_SetRotY_FromFighterDir, efLib_Cb_SetRotYZ_FromFighter, efLib_Cb_Fall_FromParamY, efLib_Cb_SetOffset_FromParams, efLib_Cb_LifetimeEndSpawn, efLib_Cb_SetScaleRotY_FromFighter, efLib_Cb_SetRotYZ_FromParamZ_FighterDir, efLib_Cb_ftMr_SpecialLw, efLib_Cb_ftLg_SpecialLw, efLib_Cb_ftKp_SpecialHi, efLib_Cb_ftCo_Bury, efLib_Cb_ApplyStoredAlpha, efLib_Cb_AccumOffset_FromParams };
#define NUPD (int) (sizeof UPD / sizeof UPD[0])

static int GFX(void)
{
    u32 r = rnd();
    int bank = (int) (rnd() % 48);
    int id = (int) (rnd() % 64);
    if (r % 24 == 0) return (int) (fv() % 70) * 1000 + id;
    if (r % 24 == 1) return 65000 + id;
    return bank * 1000 + id;
}
static int GG(void)
{
    u32 r = rnd();
    static const int sp[] = { 0x121, 0xFF, 0x7918, 0xFC, 0xF7 };
    if (r % 3 == 0) return sp[rnd() % 5];
    return GFX();
}
static int SPG(void)
{
    u32 r = rnd();
    if (r % 6 == 0) return GFX();
    return SPLIST[rnd() % (sizeof SPLIST / sizeof SPLIST[0])];
}
static HSD_GObj* MXG(void)
{
    u32 r = rnd() % 6;
    if (r == 0) return 0;
    if (r == 1) return &GO[8];
    return &GO[6 + (rnd() & 1)];
}
static int MXID(void)
{
    u32 r = rnd() % 8;
    if (r == 0) return (int) (rnd() % 5000);
    if (r == 1) return 5000 + 1000 * (int) (rnd() % 5);
    return 5000 + (int) (rnd() % 4100);
}
static void* ARGP(int kind)
{
    switch (kind) {
    case 0: return &V3[rnd() % 6];
    case 1: return &FL[rnd() % 6];
    default: return &JO[rnd() % 16];
    }
}
static void* ARGD(int d, int i)
{
    char k = DKIND[d][i];
    switch (k) {
    case 'V': return &V3[rnd() % 6];
    case 'F': return &FL[rnd() % 6];
    case 'J': return &JO[rnd() % 16];
    case 'U': return (void*) (unsigned long long) (fv() * 0x10101u);
    default: return &V3[rnd() % 6];
    }
}
static void* disp(s32 g, HSD_GObj* o, ...) { __builtin_va_list ap; void* r; __builtin_va_start(ap, o); r = efAsync_Dispatch(g, o, ap); __builtin_va_end(ap); return r; }
static HSD_Generator* w_attach_scale(s32 g, HSD_GObj* o, ...) { __builtin_va_list ap; HSD_Generator* r; __builtin_va_start(ap, o); r = efLib_CreateGenerator_Attach_Scale(g, ap, o); __builtin_va_end(ap); return r; }
static HSD_Generator* w_setscale(s32 g, ...) { __builtin_va_list ap; HSD_Generator* r; __builtin_va_start(ap, g); r = efLib_CreateGenerator_AppSRT_SetScale(g, ap); __builtin_va_end(ap); return r; }
static HSD_Generator* w_facing(s32 g, ...) { __builtin_va_list ap; HSD_Generator* r; __builtin_va_start(ap, g); r = efLib_CreateGenerator_AppSRT_SetFacingDir(g, ap); __builtin_va_end(ap); return r; }
static HSD_Generator* w_facingscale(s32 g, ...) { __builtin_va_list ap; HSD_Generator* r; __builtin_va_start(ap, g); r = efLib_CreateGenerator_AppSRT_SetFacingDirScale(g, ap); __builtin_va_end(ap); return r; }

static void setup(void)
{
    int i, j, ord[6], split;
    zero(EFF, sizeof EFF); zero(QE, sizeof QE); zero(QH, sizeof QH); zero(GO, sizeof GO); zero(JO, sizeof JO); zero(FT, sizeof FT);
    zero(ITM, sizeof ITM); zero(DO, sizeof DO); zero(MO, sizeof MO); zero(TO, sizeof TO); zero(TV, sizeof TV); zero(GEN, sizeof GEN);
    zero(APS, sizeof APS); zero(PT, sizeof PT); zero(PROC, sizeof PROC); zero(DESC, sizeof DESC); zero(DE, sizeof DE); zero(PLH, sizeof PLH);
    zero(V3, sizeof V3); zero(FL, sizeof FL); zero(OUTI, sizeof OUTI); zero(OUTP, sizeof OUTP); zero(ARC, sizeof ARC); zero(CMD, sizeof CMD);
    EFN = 6; QN = 4;
    for (i = 0; i < 9; i++) { fill_HSD_GObj(&GO[i]); GO[i].hsd_obj = &JO[i]; GO[i].gx_link = (rnd() & 1) ? 7 : 8; GO[i].obj_kind = (rnd() % 3 == 0) ? (u8) rnd() : 2; }
    for (i = 0; i < 16; i++) { fill_HSD_JObj(&JO[i]); JO[i].flags &= ~(1u << 17); if (rnd() & 1) JO[i].flags |= (1 << 6); if (rnd() % 4 == 0) JO[i].flags |= (1 << 25); }
    fill_FighterEf(&FT[0]); fill_FighterEf(&FT[1]);
    for (i = 0; i < 10; i++) fill_EF_Effect(&EFF[i]);
    for (i = 0; i < 8; i++) fill_EF_QueuedEffect(&QE[i]);
    fill_EF_QueuedEffect(&QH[0]);
    for (i = 0; i < 4; i++) { fill_HSD_DObj(&DO[i]); fill_HSD_MObj(&MO[i]); }
    for (i = 0; i < 8; i++) { fill_HSD_TObj(&TO[i]); fill_HSD_TObjTev(&TV[i]); }
    for (i = 0; i < 3; i++) { fill_HSD_Generator(&GEN[i]); fill_HSD_psAppSRT(&APS[i]); }
    for (i = 0; i < 2; i++) { fill_HSD_Particle(&PT[i]); fill_HSD_GObjProc(&PROC[i]); fill_EF_DAT_Entry(&DE[i]); }
    for (i = 0; i < 64; i++) fill_EF_EffectDesc(&DESC[i]);
    for (i = 0; i < 6; i++) { V3[i].x = RNN(); V3[i].y = RNN(); V3[i].z = RNN(); FL[i] = RNN(); }
    for (i = 0; i < 16; i++) { BT1[i] = (u8) (rnd() % 9); BT2[i] = (u8) (rnd() % 9); }
    BHV.mdl_num = (s32) (rnd() % 12); BHV.mdl_type = BT1; BHV.ptcl_num = (s32) (rnd() % 12); BHV.ptcl_type = BT2;
    /* effects and their gobjs */
    for (i = 0; i < 6; i++) {
        GO[i].user_data = &EFF[i]; EFF[i].gobj = &GO[i]; EFF[i].parent_gobj = &GO[6 + (rnd() & 1)];
        EFF[i].attach_jobj = (rnd() % 4) ? &JO[8 + rnd() % 8] : 0;
        EFF[i].update = (rnd() % 4) ? UPD[rnd() % NUPD] : 0;
        EFF[i].user_data = (rnd() % 3) ? (void*) &JO[rnd() % 16] : 0;
        EFF[i].next = &EFF[rnd() % 6];
        if (rnd() % 4 == 0) EFF[i].lifetime = 1;
        EFF[i].state_flags = (rnd() % 3 == 0) ? (u8) (rnd() % 3) : 0;
        EFF[i].scale_flags = (u8) (rnd() % 3);
        EFF[i].is_async = (u8) (rnd() & 1);
    }
    GO[6].user_data = &FT[0]; GO[7].user_data = &FT[1]; GO[6].classifier = 4; GO[7].classifier = 4;
    GO[8].classifier = 6; GO[8].user_data = &ITM[0]; ITM[0].mex_original_owner = (rnd() & 3) ? &GO[6 + (rnd() & 1)] : 0;
    for (i = 0; i < 2; i++) {
        u32 m = rnd() % 5;
        if (m == 0) FT[i].motion_id = 349; else if (m == 1) FT[i].motion_id = 357; else if (m == 2) FT[i].motion_id = 359;
        FT[i].kind = (rnd() % 3 == 0) ? 4 : (int) (rnd() % 127);
    }
    /* jobj forest: parents point to lower indices; effect roots own a sibling chain 8..15 */
    for (i = 1; i < 16; i++) JO[i].parent = (rnd() & 1) ? &JO[rnd() % i] : 0;
    for (i = 0; i < 6; i++) JO[i].child = &JO[8 + (rnd() & 1)];
    for (i = 8; i < 15; i++) JO[i].next = &JO[i + 1];
    JO[6].child = (rnd() & 1) ? &JO[10] : 0; JO[7].child = (rnd() & 1) ? &JO[11] : 0; JO[8].child = (rnd() & 1) ? &JO[12] : 0;
    for (i = 0; i < 4; i++) { DO[i].mobj = (rnd() % 4) ? &MO[i] : 0; }
    for (i = 0; i < 8; i++) { TO[i].next = &TO[(i + 1) % 8]; TO[i].tev = &TV[i]; }
    for (i = 0; i < 3; i++) {
        GEN[i].appsrt = (rnd() & 1) ? &APS[rnd() % 3] : 0;
        GEN[i].cmdList = (rnd() % 3) ? (u8*) CMD[rnd() % 5].cmdList : 0;
    }
    for (i = 0; i < 2; i++) { PT[i].appsrt = (rnd() & 1) ? &APS[rnd() % 3] : 0; PROC[i].s_link = (u8) (rnd() % 12); }
    for (i = 0; i < 0x220; i++) PREF[i] = &CMD[4];
    PREF[0x96] = &CMD[0]; PREF[0x97] = &CMD[1]; PREF[0x98] = &CMD[2]; PREF[0x21B] = &CMD[3];
    for (i = 0; i < 65; i++) ptclref_804D0E5C[i] = 0;
    ptclref_804D0E5C[0] = PREF;
    for (i = 0; i < 64; i++) {
        DESC[i].model_desc.joint = (rnd() % 4) ? (HSD_Joint*) &AJ[0] : 0;
        DESC[i].model_desc.animjoint = (rnd() & 1) ? (HSD_AnimJoint*) &AJ[1] : 0;
        DESC[i].model_desc.matanim_joint = (rnd() % 4 == 0) ? (HSD_MatAnimJoint*) &AJ[2] : 0;
        DESC[i].model_desc.shapeanim_joint = (rnd() % 4 == 0) ? (HSD_ShapeAnimJoint*) &AJ[3] : 0;
        if (rnd() & 1) DESC[i].lifetime = (float) (rnd() % 700) / 10.0f;
    }
    for (i = 0; i < 2; i++) { DE[i].ef_DAT_file = (rnd() & 1) ? STRS[0] : 0; DE[i].effDataTable_name = (rnd() & 1) ? STRS[1] : 0; }
    for (i = 0; i < 65; i++) {
        efAsync_DatEntries[i] = DEINIT[i];
        efAsync_DatEntries[i].data = (rnd() % 8) ? (void*) DESC : 0;
        efAsync_MexBhv[i] = (rnd() & 1) ? &BHV : 0;
    }
    efAsync_MexReady = (int) (rnd() & 1);
    for (i = 0; i < 128; i++) ftData_UnkBytePerCharacter[i] = (rnd() % 6 == 0) ? 0xFF : (u8) (rnd() % 70);
    /* efLib */
    efLib_EffectCount = rnd() % 64; efLib_LoadKind = rnd() & 1; efLib_AnimCount = (s32) (rnd() % 8);
    zero(efLib_AnimQueue, sizeof efLib_AnimQueue);
    for (i = 0; i < efLib_AnimCount; i++) ((HSD_JObj**) efLib_AnimQueue)[i] = &JO[rnd() % 16];
    for (i = 0; i < 8; i++) {
        fill_EF_ParamEntry(&efLib_ParamTable[i]);
        efLib_ParamTable[i].gobj = (rnd() & 3) ? &GO[rnd() % 8] : 0;
        j = (int) (rnd() % 4);
        if (j == 0) efLib_ParamTable[i].gfx_id = 0x417; else if (j == 1) efLib_ParamTable[i].gfx_id = 0x419;
    }
    hsd_804D7900 = 0;
    HSD_GObj_JObjKind = 2;
    HSD_GObj_CurrentInvokedProc = (rnd() & 1) ? &PROC[0] : 0;
    HSD_GObjPLinkHead = PLH;
    for (i = 0; i < 6; i++) ord[i] = i;
    for (i = 5; i > 0; i--) { j = (int) (rnd() % (u32) (i + 1)); { int t = ord[i]; ord[i] = ord[j]; ord[j] = t; } }
    split = (int) (rnd() % 7);
    PLH[11] = split ? &GO[ord[0]] : 0;
    for (i = 0; i + 1 < split; i++) GO[ord[i]].next = &GO[ord[i + 1]];
    PLH[12] = split < 6 ? &GO[ord[split]] : 0;
    for (i = split; i + 1 < 6; i++) GO[ord[i]].next = &GO[ord[i + 1]];
    /* the queue: QE[0..3] chained below the head */
    for (i = 0; i < 4; i++) {
        QE[i].spawn_kind = (u8) (rnd() % 10); QE[i].gfx_id = GFX(); QE[i].jobj = &JO[8 + rnd() % 8];
        if (QE[i].spawn_kind == 9) QE[i].gfx_id = 5000 + (s32) (rnd() % 4100);
        QE[i].next = (i < 3) ? &QE[i + 1] : 0;
    }
    QH[0].next = (rnd() % 4) ? &QE[0] : 0;
}

static void hash_all(void)
{
    int i;
    for (i = 0; i < 10; i++) hash_EF_Effect(&EFF[i]);
    for (i = 0; i < 8; i++) hash_EF_QueuedEffect(&QE[i]);
    hash_EF_QueuedEffect(&QH[0]);
    for (i = 0; i < 9; i++) hash_HSD_GObj(&GO[i]);
    for (i = 0; i < 16; i++) hash_HSD_JObj(&JO[i]);
    hash_FighterEf(&FT[0]); hash_FighterEf(&FT[1]);
    for (i = 0; i < 4; i++) { hash_HSD_DObj(&DO[i]); hash_HSD_MObj(&MO[i]); }
    for (i = 0; i < 8; i++) { hash_HSD_TObj(&TO[i]); hash_HSD_TObjTev(&TV[i]); }
    for (i = 0; i < 3; i++) { hash_HSD_Generator(&GEN[i]); hash_HSD_psAppSRT(&APS[i]); }
    for (i = 0; i < 2; i++) { hash_HSD_Particle(&PT[i]); hash_HSD_GObjProc(&PROC[i]); hash_EF_DAT_Entry(&DE[i]); }
    for (i = 0; i < 6; i++) { HM(B(V3[i].x)); HM(B(V3[i].y)); HM(B(V3[i].z)); HM(B(FL[i])); }
    HM((u32) EFN); HM((u32) QN);
    HM(efLib_EffectCount); HM(efLib_LoadKind); HM((u32) efLib_AnimCount);
    for (i = 0; i < 16; i++) HM(PI(((void**) efLib_AnimQueue)[i]));
    for (i = 0; i < 8; i++) hash_EF_ParamEntry(&efLib_ParamTable[i]);
    for (i = 0; i < 3; i++) HM(FI((const void*) lbl_803BF810[i]));
    HM(FI((const void*) hsd_804D7900));
    for (i = 0; i < 65; i++) { HM(SH(efAsync_DatEntries[i].ef_DAT_file)); HM(SH(efAsync_DatEntries[i].effDataTable_name)); HM(PI(efAsync_DatEntries[i].data)); HM(PI(efAsync_MexBhv[i])); }
    HM((u32) efAsync_MexReady);
    HM((u32) OUTI[2]);
}

int main(void)
{
    int op, k;
    setbuf(__acrt_iob_func(1), 0);
    for (op = 0; op < 65; op++) DEINIT[op] = efAsync_DatEntries[op];
    preg(EFF, sizeof EFF); preg(QE, sizeof QE); preg(QH, sizeof QH); preg(GO, sizeof GO); preg(JO, sizeof JO); preg(FT, sizeof FT);
    preg(ITM, sizeof ITM); preg(DO, sizeof DO); preg(MO, sizeof MO); preg(TO, sizeof TO); preg(TV, sizeof TV); preg(GEN, sizeof GEN);
    preg(APS, sizeof APS); preg(PT, sizeof PT); preg(PROC, sizeof PROC); preg(DESC, sizeof DESC); preg(DE, sizeof DE);
    preg(&BHV, sizeof BHV); preg(ARC, sizeof ARC); preg(PLH, sizeof PLH); preg(V3, sizeof V3); preg(FL, sizeof FL);
    preg(CMD, sizeof CMD); preg(PREF, sizeof PREF); preg(AJ, sizeof AJ); preg(STRS, sizeof STRS); preg(BT1, sizeof BT1); preg(BT2, sizeof BT2);
    freg_all();
    for (op = 0; op < 78; op++) {
        for (k = 0; k < 400; k++) {
            u32 ret = 0;
            tu_case(op, k);
            mtu_dbgp = (op == DBG_OP && k == DBG_K);
            setup();
            __try {
            switch (op) {
            case 0: { efLib_Init(); } break;
            case 1: { { s32 f = (s32) fv(); efLib_SetFlags(&GO[6 + (rnd() & 1)], f); } } break;
            case 2: { efLib_Destroy(&GO[rnd() % 8]); } break;
            case 3: { efLib_DestroyAll(&GO[6 + (rnd() & 1)]); } break;
            case 4: { efLib_PauseAll(&GO[6 + (rnd() & 1)]); } break;
            case 5: { efLib_ResumeAll(&GO[6 + (rnd() & 1)]); } break;
            case 6: { efLib_remove_user_data(&EFF[rnd() % 6]); } break;
            case 7: { efLib_EffectCount = rnd() % 64; efLib_RemoveLast(); } break;
            case 8: { efLib_Update(&GO[rnd() % 6]); } break;
            case 9: { { int g = GFX(); HSD_GObj* p = &GO[6 + (rnd() & 1)]; ret = PI(efLib_Create(g, p)); } } break;
            case 10: { { u32 g = (u32) GFX(); HSD_GObj* p = &GO[6 + (rnd() & 1)]; HSD_JObj* j = &JO[8 + rnd() % 8]; ret = PI(efLib_Create_Attach(g, p, j)); } } break;
            case 11: { { u32 g = (u32) GFX(); HSD_GObj* p = &GO[6 + (rnd() & 1)]; HSD_JObj* j = &JO[8 + rnd() % 8]; ret = PI(efLib_Create_AttachChild(g, p, j)); } } break;
            case 12: { { u32 g = (u32) GFX(); HSD_GObj* p = &GO[6 + (rnd() & 1)]; HSD_JObj* j = &JO[8 + rnd() % 8]; ret = PI(efLib_Create_Attach_Scale(g, p, j)); } } break;
            case 13: { { u32 g = (u32) GFX(); HSD_GObj* p = &GO[6 + (rnd() & 1)]; HSD_JObj* j = &JO[8 + rnd() % 8]; ret = PI(efLib_Create_AttachChild_Scale(g, p, j)); } } break;
            case 14: { { u32 g = (u32) GFX(); HSD_GObj* p = &GO[6 + (rnd() & 1)]; HSD_JObj* j = &JO[8 + rnd() % 8]; ret = PI(efLib_Create_Attach_Scale_FacingDir(g, p, j)); } } break;
            case 15: { { u32 g = (u32) GFX(); HSD_GObj* p = &GO[6 + (rnd() & 1)]; ret = PI(efLib_Create_Attach_Pos(g, p, &V3[0])); } } break;
            case 16: { efLib_render_callback(&GO[rnd() % 8], (int) (rnd() % 5) - 1); } break;
            case 17: { efLib_particles_proc_main(&GO[0]); } break;
            case 18: { efLib_particles_proc_aux(&GO[0]); } break;
            case 19: { { int g = GG(); ret = PI(efLib_CreateGenerator(g, &V3[0])); } } break;
            case 20: { { int g = GFX(); ret = PI(efLib_CreateGenerator_AddAppSRT(g)); } } break;
            case 21: { { int g = GFX(); float d = RNN(); ret = PI(efLib_CreateGenerator_Translate_FacingDir(g, &V3[1], d)); } } break;
            case 22: { { int g = GFX(); ret = PI(efLib_CreateGenerator_Attach(g, &JO[8 + rnd() % 8])); } } break;
            case 23: { { int g = GFX(); ret = PI(efLib_CreateGenerator_Attach_AddAppSRT(g, &JO[8 + rnd() % 8])); } } break;
            case 24: { { int g = GFX(); HSD_JObj* j = &JO[8 + rnd() % 8]; ret = PI(w_attach_scale(g, &GO[6 + (rnd() & 1)], j)); } } break;
            case 25: { { int g = GFX(); HSD_JObj* j = &JO[8 + rnd() % 8]; ret = PI(w_setscale(g, j, &FL[rnd() % 6])); } } break;
            case 26: { { int g = GFX(); HSD_JObj* j = &JO[8 + rnd() % 8]; ret = PI(w_facing(g, j, &FL[rnd() % 6])); } } break;
            case 27: { { int g = GFX(); HSD_JObj* j = &JO[8 + rnd() % 8]; float* f1 = &FL[rnd() % 6]; float* f2 = &FL[rnd() % 6]; ret = PI(w_facingscale(g, j, f1, f2)); } } break;
            case 28: { { int g = GFX(); HSD_GObj* go = &GO[6 + (rnd() & 1)]; HSD_JObj* j = &JO[8 + rnd() % 8]; ret = PI(efLib_CreateGenerator_AppSRT_SetPos(g, go, j, &V3[2])); } } break;
            case 29: { { int g = SPG(); int fl = (int) (rnd() & 1); efLib_SpawnParticleEffect(g / 1000, g, &JO[8 + rnd() % 8], fl); } } break;
            case 30: { { int b = (rnd() & 3) == 0 ? 0x1E : (int) (rnd() % 50); int g = SPG(); efLib_Cb_SPtcl((int) (rnd() % 3), b, g, &JO[8 + rnd() % 8]); } } break;
            case 31: { { int b = (rnd() & 3) == 0 ? 0x1E : (int) (rnd() % 50); int g = SPG(); efLib_Cb_DPtcl((int) (rnd() % 3), b, g, &JO[8 + rnd() % 8]); } } break;
            case 32: { efLib_Cb_ParticleRender(&PT[rnd() % 2]); } break;
            case 33: { efLib_Cb_PtclAppSRTHook(&GEN[rnd() % 3]); } break;
            case 34: { { efLib_Cb_SetOffsetY_FromParamY(&EFF[rnd() % 6]); } } break;
            case 35: { { efLib_Cb_SetScale_FromParamX(&EFF[rnd() % 6]); } } break;
            case 36: { { efLib_Cb_SetRotYAndTransition(&EFF[rnd() % 6]); } } break;
            case 37: { { efLib_Cb_SetJObjOffsetZ(&EFF[rnd() % 6]); } } break;
            case 38: { { efLib_Cb_SetRotY_FromFighterDir(&EFF[rnd() % 6]); } } break;
            case 39: { { efLib_Cb_SetRotYZ_FromFighter(&EFF[rnd() % 6]); } } break;
            case 40: { { efLib_Cb_Fall_FromParamY(&EFF[rnd() % 6]); } } break;
            case 41: { { efLib_Cb_SetOffset_FromParams(&EFF[rnd() % 6]); } } break;
            case 42: { { efLib_Cb_LifetimeEndSpawn(&EFF[rnd() % 6]); } } break;
            case 43: { { efLib_Cb_SetScaleRotY_FromFighter(&EFF[rnd() % 6]); } } break;
            case 44: { { efLib_Cb_SetRotYZ_FromParamZ_FighterDir(&EFF[rnd() % 6]); } } break;
            case 45: { { efLib_Cb_ftMr_SpecialLw(&EFF[rnd() % 6]); } } break;
            case 46: { { efLib_Cb_ftLg_SpecialLw(&EFF[rnd() % 6]); } } break;
            case 47: { { EFF[0].next = &EFF[1]; efLib_Cb_ftKp_SpecialHi(&EFF[0]); } } break;
            case 48: { { efLib_Cb_ftCo_Bury(&EFF[rnd() % 6]); } } break;
            case 49: { { efLib_Cb_ApplyStoredAlpha(&EFF[rnd() % 6]); } } break;
            case 50: { { efLib_Cb_AccumOffset_FromParams(&EFF[rnd() % 6]); } } break;
            case 51: { { s32 c = (s32) (rnd() % 4); u32 k = fv() * 0x10101u; u32 t = fv() * 0x01010101u; efLib_SetTevKonstColor(&JO[rnd() % 16], c, k, t); } } break;
            case 52: { { u8 a = (u8) fv(); efLib_SetParamAlpha(&GO[rnd() % 8], a); } } break;
            case 53: { { s32 g = GFX(); efLib_SetParamGfxId(&GO[rnd() % 8], g); } } break;
            case 54: { { float z = rw(); HSD_JObjSetTranslateZ(&JO[rnd() % 16], z); } } break;
            case 55: { efAsync_LoadAsync((int) (rnd() % 72) - 3); } break;
            case 56: { { u32 len = fv(); int idx = (int) (rnd() % 65); efAsync_OnLoad(&ARC[0], (u8*) &V3[0], len, idx); } } break;
            case 57: { efAsync_LoadSync((int) (rnd() % 72) - 3); } break;
            case 58: { efAsync_QueueFlush(&GO[6 + (rnd() & 1)], &QH[0]); } break;
            case 59: { efAsync_QueueClear(&QH[0]); } break;
            case 60: { efAsync_QueueInit(); } break;
            case 61: { efAsync_MexInit(); } break;
            case 62: { efAsync_QueueProcessDeferred(&GO[6 + (rnd() & 1)], &QE[rnd() % 4]); } break;
            case 63: { { int b = (int) (rnd() % 70) - 2; efAsync_MexNoteArchive(b, (rnd() & 1) ? &ARC[0] : 0); } } break;
            case 64: { { HSD_GObj* g = MXG(); int id = MXID(); ret = (u32) efAsync_MexResolve(g, id, &OUTI[0], &OUTI[1], (HSD_GObj**) &OUTP[0]); HM((u32) OUTI[0]); HM((u32) OUTI[1]); HM(PI(OUTP[0])); } } break;
            case 65: { { HSD_GObj* g = MXG(); int id = MXID(); float f = rw(); float o = rw(); efAsync_MexSpawn(g, &QH[0], id, &JO[8 + rnd() % 8], &V3[3], f, o); } } break;
            case 66: { efAsync_MexProcessDeferred(&GO[6 + (rnd() & 1)], &QE[rnd() % 4]); } break;
            case 67: { { EF_Effect* e = &EFF[rnd() % 6]; efAsync_SetEffectScaleXYZ(e, rw()); } } break;
            case 68: { { EF_Effect* e = &EFF[rnd() % 6]; efAsync_SetEffectRotationZFromPtr(e, &FL[rnd() % 6]); } } break;
            case 69: { { EF_Effect* e = &EFF[rnd() % 6]; efAsync_SetEffectFacingDir(e, rw()); } } break;
            case 70: { efAsync_SetEffectRandomRotationZ(&EFF[rnd() % 6]); } break;
            case 71: { efAsync_SetEffectScale(&EFF[rnd() % 6], &V3[4]); } break;
            case 72: { { u32 kind = rnd() % 9; u32 g = (u32) GFX(); HSD_GObj* go = &GO[6 + (rnd() & 1)]; void* a1 = ARGP((int) (rnd() % 3)); void* a2 = ARGP((int) (rnd() % 3)); void* a3 = ARGP(1); efAsync_Spawn(go, &QH[0], kind, g, &JO[8 + rnd() % 8], a1, a2, a3); } } break;
            case 73: { { int d = (int) ((k + 0) % 144); ret = PI(disp(DID[d], &GO[6 + (rnd() & 1)], ARGD(d, 0), ARGD(d, 1), ARGD(d, 2), ARGD(d, 3), ARGD(d, 4), ARGD(d, 5))); } } break;
            case 74: { { int d = (int) ((k + 48) % 144); ret = PI(disp(DID[d], &GO[6 + (rnd() & 1)], ARGD(d, 0), ARGD(d, 1), ARGD(d, 2), ARGD(d, 3), ARGD(d, 4), ARGD(d, 5))); } } break;
            case 75: { { int d = (int) ((k + 96) % 144); ret = PI(disp(DID[d], &GO[6 + (rnd() & 1)], ARGD(d, 0), ARGD(d, 1), ARGD(d, 2), ARGD(d, 3), ARGD(d, 4), ARGD(d, 5))); } } break;
            case 76: { { HSD_GObj* go = &GO[6 + (rnd() & 1)]; FL[0] = (float) (rnd() % 100000) / 7.0f; ret = PI(disp(0x3E8, go, &V3[0], &FL[0])); } } break;
            case 77: { { int g = (int) (rnd() % 1500); if (is_d(g)) g += 5000; HSD_GObj* go = &GO[6 + (rnd() & 1)]; void* a = ARGP(0); ret = PI(disp(g, go, a, a, a, a)); } } break;
            }
            } __except (_exception_code() == 0xE0A55E87ul) { HM(0xA55E87u); ASSN[op]++; ASLINE[op] = LASTAL; ASCOND[op] = LASTC; }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    if (getenv("MTU_STAT")) { for (op = 0; op < 78; op++) fprintf_stub(op); }
    return 0;
}
