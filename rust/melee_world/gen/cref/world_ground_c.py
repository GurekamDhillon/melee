"""C half of the tu::ground oracle world (see spec_ground.py)."""
ROWS = ("grBb_Route grBb grCn grCs grEF1 grEF2 grEF3 grFigureGet grFs grFz grGb grGd grGr grHeal grHr grI1 grI2 grIm grIz grKg "
        "grKr grMc grNBa grNKr grNLa grOk grOp grOt grOy grPs grPu grPushOn grRc grSh_Route grSh grSt grTCLink grTCa grTDk grTDr "
        "grTFc grTFe grTFx grTGn grTGw grTIc grTKb grTKp grTLg grTLk grTMewtwo grTMr grTMs grTNs grTPc grTPe grTPk grTPr grTSk "
        "grTSs grTYs grTZd grTe grVe grYt grZe_Route grZe").split()
ROWNAMES = [r + "_StageData" for r in ROWS]

DECL_C = r"""
static int ANULL, STDNULL, GSN, AMODE, MI, JI, GI, NGRI, LNI, WP, WI, ARO, SD0N, GFAIL, JLN;
static s8 CKV;
static StageParam SP[4]; static GroundParam GP; static UnkArchiveStruct ARCH; static UnkStageDat STD;
static struct UnkStageDat_x8_t X8[8]; static HSD_Joint HJ[8]; static HSD_CameraDescPerspective CAMD[2];
static GrJoint GJA[8], GJB[8]; static s16 SPA[8];
typedef struct { void* joint; s16* pairs; s32 pair_count; } EntT;
static EntT ENT[4]; static s16 PRS[4][8];
static HSD_Spline SPL[4]; static HSD_Spline* SPLP[4];
static LightOverrideEntry LOE[6]; static struct GroundShadowEntry GSE[6];
static HSD_LightDesc LD[6]; static HSD_LightAnim LAN[6]; static HSD_LightAnim* ANL[6][2]; static LightList LL[6]; static LightList* LLP[3][8];
static HSD_LObj LO[4]; static HSD_WObj WO[4]; static HSD_AObj AO[4];
static HSD_Fog FOG; static HSD_FogDesc FD; static HSD_GObj FGO;
static HSD_GObj NG[4]; static Ground NGR[4]; static HSD_JObj JN[8]; static HSD_CObj COB;
static CollJoint CJ[16]; static CollLine CLN[24]; static MapCollData MCD, MCP[4]; static Vec2 MV[8]; static MapLine MLN[8]; static MapJoint MJN[8];
static u8 ARENA[4096]; static u8 BUF[64]; static HSD_GObj* PL[16];
static StageIdPair PAIR; static struct GroundItemData GID[3]; static struct GroundItemData* ITD[4]; static void* AYA[6];
static Article ART; static ItemStateArray ISA; static Article* TB[256]; static int DUM[8]; static u8 DUMD[16]; static u8 ARCHB[16];
typedef struct { void* n; void* a; void* f; } LnT;
static LnT LNP[4];
static StageCallbacks SCB[3][8]; static char STRB[4][8] = { "GrNr", "GrCs", "GrPs", "GrZz" };
static float YAK[48]; static int OUTI[8]; static float OUTF[4]; static int I35[40]; static GXColor GXC[4]; static u32 OUTU;
static StageData* SD0[160]; static StageData* ROWTAB[160];

void OSPanic(const char* file, int line, const char* msg, ...) { (void) file; (void) line; (void) msg; printf("OSPanic");  fflush(0); exit(4); }
static void sd_oi(void) { T(9001); }
static void sd_od(int a) { T(9002); T((u32) a); }
static void sd_ol(void) { T(9003); }
static void sd_os(void) { T(9004); }
static int sd_c4(void) { T(9005); return (int) (fv() & 1); }
static DynamicsDesc* sd_tl(int a) { T(9006); T((u32) a); return ((fv() & 1) != 0) ? 0 : (DynamicsDesc*) DUMD; }
static int sd_cs(Vec* p, int a, HSD_JObj* j) { T(9007); TV(p); T((u32) a); T(PI((const void*) j)); return (int) (fv() & 1); }
static int pr90(Vec* p, int i) { T(9008); TV(p); T((u32) i); return (int) (fv() & 1); }
static int pr94(Vec* p, int i) { T(9009); TV(p); T((u32) i); return (int) (fv() & 1); }
static void lnfn(s32 a) { T(9010); T((u32) a); }
static void cbi(int a) { T(9011); T((u32) a); }
static void ev1(HSD_GObj* g) { T(9012); T(PI((const void*) g)); }
static void ev2(HSD_GObj* g) { T(9013); T(PI((const void*) g)); }
static void ev3(HSD_GObj* g) { T(9014); T(PI((const void*) g)); }

static void fill_GroundParam(GroundParam* p);
static void fill_StageInfo(StageInfo* p);

static u32 rowid(const StageData* p)
{
    int i;
    if (p == 0) return 0;
    for (i = 0; i < 160; i++) if (ROWTAB[i] == p) return (u32) (i + 1);
    return 0xFFFFu;
}
static int pick_stk(void)
{
    int cnt = GP.stage_param_count, wild = 0, i;
    u32 r;
    if (cnt <= 0) return (int) (fv() % 8) - 1;
    for (i = 0; i < cnt; i++) if ((int) SP[i].stkind == -1) wild = 1;
    r = fv() % 6;
    if (r == 0 && (wild || cnt == 1)) return (int) (fv() % 8) - 1;
    return (int) SP[fv() % (u32) cnt].stkind;
}
static int pick_stk_any(void)
{
    if ((fv() % 6) == 0) return (int) (fv() % 8) - 1;
    return pick_stk();
}
static int pick_kind(void)
{
    int k = (int) (fv() % 160);
    if (k == 23 || k == 26) k = 1 + (int) (fv() % 20);
    return k;
}
static int pick_gk(void)
{
    static const int GK[10] = { Gr_Kind_Castle, Gr_Kind_Corneria, Gr_Kind_Unk26, Gr_Kind_Inishie2, Gr_Kind_RCruise, Gr_Kind_Yorster,
                                Gr_Kind_MuteCity, Gr_Kind_Shrine, Gr_Kind_Kongo, Gr_Kind_OldKongo };
    u32 r = fv() % 14;
    if (r < 10) return GK[r];
    return pick_kind();
}
static int pick_idx(void)
{
    u32 r = fv() % 6;
    if (r == 0) return (int) (fv() % 10);
    if (r == 1) return 0x94 + (int) (fv() % 5);
    if (r == 2) return 0x99 + (int) (fv() % 26);
    if (r == 3) return 0xBD + (int) (fv() % 10);
    if (r == 4) return 0x7F;
    return (int) (fv() % 261);
}
static void setpair(int nn)
{
    int k = (int) (fv() % 160);
    if (nn && (k == 23 || k == 26)) k = 1 + (int) (fv() % 20);
    if (!nn && (fv() % 12) == 0) k = ((fv() & 1) != 0) ? 23 : 26;
    PAIR.grkind = (GrKind) k;
    PAIR.stkind = (StKind) pick_stk();
}
static void set_gsn(int n) { GSN = n; STD.unk24 = n; }
static void lights_pre(void)
{
    u32 style = fv() % 3;
    StageData* sd;
    if (style == 0) { set_gsn(0); return; }
    set_gsn(5);
    sd = stage_datas[stage_info.grkind];
    if (sd->callbacks == 0) sd->callbacks = SCB[0];
    sd->callbacks[0].flags_b0 = 1;
    STD.unkC = 1 + (int) (fv() % 8);
}
static u32 pick_bgmflags(void)
{
    u32 f = 0;
    if ((fv() % 2) != 0) f |= 4;
    if ((fv() % 3) == 0) f |= 0x40;
    if ((fv() % 3) == 0) f |= 0x10; else if ((fv() % 3) == 0) f |= 0x20;
    if (!(f & 4)) f |= ((fv() % 2) != 0) ? 1 : 2;
    return f;
}
static u32 pick_snd(void)
{
    if ((fv() % 4) == 0) return 540000;
    if ((fv() % 3) == 0) return 540001;
    return fv();
}
static Ground* pick_gr(void)
{
    if ((fv() % 8) == 0) return 0;
    return &GR[fv() & 3];
}
static void* alloc_(int n)
{
    void* p;
    if (AMODE >= 1) {
        int k = MI++;
        if ((k & 3) == 0) return &MCP[(k >> 2) & 3];
        if (AMODE == 2 && (k & 3) == 2) return 0;
    } else {
        if (n == 64) return BUF;
        if (n == (int) sizeof(Ground)) { if (GFAIL == 2) return 0; return &NGR[NGRI++ & 3]; }
        if (n == (int) sizeof(LnT)) return &LNP[LNI++ & 3];
    }
    p = &ARENA[ARO];
    ARO += (n + 7) & ~7;
    return p;
}
static void setrow(StageData* d)
{
    d->grkind = (GrKind) (fv() % 160);
    d->callbacks = ((fv() % 6) == 0) ? 0 : SCB[fv() % 3];
    d->data1 = STRB[fv() & 3];
    d->on_init = sd_oi; d->on_demo_init = sd_od; d->on_load = sd_ol; d->on_start = sd_os; d->callback4 = sd_c4;
    d->on_touch_line = ((fv() & 3) == 0) ? 0 : sd_tl;
    d->on_check_shadow_render = ((fv() & 3) == 0) ? 0 : sd_cs;
    d->flags2 = fv();
    d->joints = ((fv() % 5) == 0) ? 0 : GJB;
    d->joint_count = (d->joints != 0) ? fv() % 7 : 0;
}
static void setup_world(void)
{
    int i, j, k;
    ANULL = 0; STDNULL = 0; GFAIL = 0; JLN = 0; GSN = 5; AMODE = 0; MI = 0; JI = 0; GI = 0; NGRI = 0; LNI = 0; WP = 1; WI = 1; ARO = 0;
    CKV = (s8) (fv() % 30);
    zero(SP, sizeof SP); zero(&GP, sizeof GP); zero(&ARCH, sizeof ARCH); zero(&STD, sizeof STD); zero(X8, sizeof X8); zero(HJ, sizeof HJ);
    zero(CAMD, sizeof CAMD); zero(GJA, sizeof GJA); zero(GJB, sizeof GJB); zero(SPA, sizeof SPA); zero(ENT, sizeof ENT); zero(PRS, sizeof PRS);
    zero(SPL, sizeof SPL); zero(SPLP, sizeof SPLP); zero(LOE, sizeof LOE); zero(GSE, sizeof GSE); zero(LD, sizeof LD); zero(LAN, sizeof LAN);
    zero(ANL, sizeof ANL); zero(LL, sizeof LL); zero(LLP, sizeof LLP); zero(LO, sizeof LO); zero(WO, sizeof WO); zero(AO, sizeof AO);
    zero(&FOG, sizeof FOG); zero(&FD, sizeof FD); zero(&FGO, sizeof FGO); zero(NG, sizeof NG); zero(NGR, sizeof NGR); zero(JN, sizeof JN);
    zero(&COB, sizeof COB); zero(CJ, sizeof CJ); zero(CLN, sizeof CLN); zero(&MCD, sizeof MCD); zero(MCP, sizeof MCP); zero(MV, sizeof MV);
    zero(MLN, sizeof MLN); zero(MJN, sizeof MJN); zero(ARENA, sizeof ARENA); zero(BUF, sizeof BUF); zero(PL, sizeof PL); zero(&PAIR, sizeof PAIR);
    zero(GID, sizeof GID); zero(ITD, sizeof ITD); zero(AYA, sizeof AYA); zero(&ART, sizeof ART); zero(&ISA, sizeof ISA); zero(TB, sizeof TB);
    zero(DUM, sizeof DUM); zero(DUMD, sizeof DUMD); zero(LNP, sizeof LNP); zero(SCB, sizeof SCB); zero(YAK, sizeof YAK);
    zero(OUTI, sizeof OUTI); zero(OUTF, sizeof OUTF); zero(I35, sizeof I35); zero(GXC, sizeof GXC); OUTU = 0;
    zero(&stage_info, sizeof stage_info); fill_StageInfo(&stage_info);

    /* a proper tree for the base jobjs (the template's links revisit nodes) and an animation here and there */
    for (i = 0; i < 16; i++) {
        JP[i].child = (2 * i + 1 < 16) ? &JP[2 * i + 1] : 0;
        JP[i].next = ((i & 1) != 0 && i + 1 < 16) ? &JP[i + 1] : 0;
        JP[i].parent = (i > 0) ? &JP[(i - 1) / 2] : 0;
        AO[i & 3].curr_frame = rg();
        JP[i].aobj = ((fv() % 4) == 0) ? &AO[fv() & 3] : 0;
    }
    for (i = 0; i < 4; i++) {
        GR[i].map_id = (GrKind) (fv() % 8);
        GR[i].x8_callback = ((fv() % 3) == 0) ? 0 : ev1;
        GR[i].xC_callback = ((fv() % 3) == 0) ? 0 : ev2;
        GR[i].x1C_callback = ((fv() % 3) == 0) ? 0 : ev3;
        GR[i].x18 = ((fv() % 2) != 0) ? &NG[fv() & 3] : 0;
        for (j = 0; j < 8; j++) GR[i].x20[j] = ((fv() % 3) == 0) ? -1 : (int) (fv() % 20);
        GO[i].next = (i < 2) ? &GO[i + 1] : 0;
        GO[i].classifier = ((fv() % 3) == 0) ? 12 : (((fv() % 3) == 0) ? 13 : (u16) (fv() % 20));
        GO[i].obj_kind = ((fv() % 2) != 0) ? 7 : (u8) (fv() % 10);
    }
    HSD_GObj_LightKind = 7;
    HSD_GObjPLinkHead = ((fv() % 6) == 0) ? 0 : PL;
    PL[HSD_GOBJ_PLINK_LIGHT] = ((fv() % 5) == 0) ? 0 : &GO[0];

    /* ground param, stage params */
    fill_GroundParam(&GP);
    GP.stage_params = SP; GP.stage_param_count = 1 + (int) (fv() % 4);
    if ((fv() % 3) == 0) GP.y = 1.0f;
    for (i = 0; i < 4; i++) {
        SP[i].stkind = (StKind) ((i == 0 && (fv() % 4) == 0) ? -1 : (int) (fv() % 6));
        { u32 r6 = fv() % 6; SP[i].x4 = (r6 == 0) ? -2 : (r6 == 1) ? -1 : (s32) (fv() % 40); }
        SP[i].x8 = ((fv() % 3) == 0) ? -1 : (s32) (fv() % 40);
        { u32 r6 = fv() % 6; SP[i].xC = (r6 == 0) ? (u32) -2 : (r6 == 1) ? (u32) -1 : fv() % 40; }
        SP[i].x10 = ((fv() % 3) == 0) ? (u32) -1 : fv() % 40;
        SP[i].x14 = (s16) (fv() % 12); SP[i].x16 = (s16) (fv() % 130); SP[i].x18 = (s16) (fv() % 200);
        for (j = 0; j < 37; j++) SP[i].x1A[j] = (s16) ((int) (fv() % 50) - 10);
    }
    stage_info.param = &GP;

    /* the archive */
    ARCH.unk0 = (HSD_Archive*) ARCHB; ARCH.unk4 = &STD; ARCH.unk8 = fv();
    STD.unk0 = ENT; STD.unk4 = (s32) (fv() % 5); STD.unk8 = X8; STD.unkC = 1 + (s32) (fv() % 8);
    STD.unk10 = SPLP; STD.unk14 = (s32) (fv() % 5); STD.unk18 = LOE; STD.unk1C = (s32) (fv() % 7);
    STD.unk20 = GSE; STD.unk24 = GSN;
    for (i = 0; i < 8; i++) {
        HJ[i].child = (2 * i + 1 < 8) ? &HJ[2 * i + 1] : 0;
        HJ[i].next = ((i & 1) != 0 && i + 1 < 8) ? &HJ[i + 1] : 0;
        HJ[i].flags = fv();
        HJ[i].rotation.x = rg(); HJ[i].rotation.y = rg(); HJ[i].rotation.z = rg();
        HJ[i].scale.x = rg(); HJ[i].scale.y = rg(); HJ[i].scale.z = rg();
        HJ[i].position.x = rg(); HJ[i].position.y = rg(); HJ[i].position.z = rg();
        X8[i].unk0 = &HJ[i];
        X8[i].x10 = ((fv() % 3) == 0) ? 0 : &CAMD[fv() & 1];
        X8[i].x18 = LLP[fv() % 3];
        X8[i].x1C = ((fv() % 4) == 0) ? 0 : &FD;
        X8[i].unk20 = GJA; X8[i].unk24 = (s32) (fv() % 5);
        X8[i].x2C = SPA; X8[i].x30 = (s32) (fv() % 5);
        GJA[i].x = (s16) (fv() % 16); GJA[i].y = (s16) (fv() % 8); GJA[i].z = (s16) (fv() % 50);
        GJB[i].x = (s16) (fv() % 16); GJB[i].y = (s16) (fv() % 8); GJB[i].z = (s16) (fv() % 50);
        SPA[i] = (s16) ((int) (fv() % 24) - 2);
    }
    for (i = 0; i < 4; i++) {
        ENT[i].joint = &HJ[fv() % 8]; ENT[i].pairs = PRS[i]; ENT[i].pair_count = (s32) (fv() % 4);
        for (j = 0; j < 4; j++) {
            PRS[i][2 * j] = (s16) (fv() % 12);
            if ((fv() & 1) != 0) PRS[i][2 * j + 1] = (s16) (220 + fv() % 40); else PRS[i][2 * j + 1] = (s16) (fv() % 261);
        }
        SPLP[i] = &SPL[i];
        SPL[i].numcv = (s16) (fv() % 5);
    }
    for (i = 0; i < 6; i++) {
        LOE[i].desc = &LD[fv() % 6]; LOE[i].a = fv() & 1; LOE[i].b = fv() & 1; LOE[i].c = fv() & 1;
        LD[i].flags = (u16) fv();
        LAN[i].next = 0;
        ANL[i][0] = &LAN[i]; ANL[i][1] = 0;
        LL[i].desc = &LD[i]; LL[i].anims = ANL[i];
    }
    LD[0].flags &= ~3; LD[1].flags &= ~3;
    for (i = 0; i < 5; i++) { GSE[i].unk0 = &LAN[i]; GSE[i].flag = fv() & 1; }
    LLP[0][0] = &LL[0]; LLP[0][1] = &LL[1]; LLP[0][2] = &LL[2]; LLP[0][3] = &LL[3]; LLP[0][4] = &LL[4]; LLP[0][5] = 0;
    LLP[1][0] = &LL[1]; LLP[1][1] = &LL[0]; LLP[1][2] = &LL[5]; LLP[1][3] = &LL[2]; LLP[1][4] = 0;
    LLP[2][0] = &LL[0]; LLP[2][1] = &LL[1]; LLP[2][2] = &LL[3]; LLP[2][3] = &LL[5]; LLP[2][4] = &LL[4]; LLP[2][5] = 0;
    for (i = 0; i < 4; i++) {
        LO[i].next = ((i & 1) == 0) ? &LO[i + 1] : 0;
        LO[i].aobj = ((fv() % 3) == 0) ? &AO[fv() & 3] : 0;
        LO[i].position = ((fv() % 3) == 0) ? 0 : &WO[fv() & 3];
        LO[i].interest = ((fv() % 3) == 0) ? 0 : &WO[fv() & 3];
    }
    FOG.start = rg(); FOG.end = rg(); FOG.color.r = (u8) fv(); FOG.color.g = (u8) fv(); FOG.color.b = (u8) fv(); FOG.color.a = (u8) fv();
    FD.start = rg(); FD.end = rg();
    FGO.hsd_obj = ((fv() % 5) == 0) ? 0 : &FOG;
    stage_info.x12C = ((fv() % 5) == 0) ? 0 : &FGO;
    for (i = 0; i < 4; i++) { GXC[i].r = (u8) fv(); GXC[i].g = (u8) fv(); GXC[i].b = (u8) fv(); GXC[i].a = (u8) fv(); }
    for (i = 0; i < 40; i++) I35[i] = (int) fv();
    for (i = 0; i < 8; i++) OUTI[i] = (int) fv();
    for (i = 0; i < 4; i++) OUTF[i] = rg();
    OUTU = fv();
    for (i = 0; i < 48; i++) YAK[i] = rg();
    for (i = 0; i < 64; i++) BUF[i] = (u8) (fv() & 1);
    for (i = 0; i < 4; i++) JN[i].child = ((fv() & 1) != 0) ? &JP[1 + fv() % 15] : 0;

    /* stage_info pointer fields */
    for (i = 0; i < 64; i++) stage_info.map_gobjs[i] = ((fv() % 3) == 0) ? 0 : &GO[fv() & 3];
    for (i = 0; i < 261; i++) stage_info.x280[i] = ((fv() & 1) != 0) ? &JP[fv() % 16] : 0;
    stage_info.x694[0] = ((fv() & 1) != 0) ? (void*) &JP[fv() % 16] : 0;
    stage_info.x694[1] = ((fv() & 1) != 0) ? (void*) GO[fv() & 3].hsd_obj : 0;
    stage_info.x694[2] = ((fv() & 1) != 0) ? (void*) &COB : 0;
    stage_info.x694[3] = 0;
    for (i = 0; i < 3; i++) { LNP[i].n = (i < 2) ? &LNP[i + 1] : 0; LNP[i].a = (void*) (unsigned long long) (fv() % 50); LNP[i].f = (void*) lnfn; }
    stage_info.x6A4 = ((fv() % 3) == 0) ? 0 : (void*) &LNP[0];
    for (i = 0; i < 3; i++) { GID[i].unk0 = (s32) (fv() % 30); GID[i].unk4 = &ART; ITD[i] = &GID[i]; }
    ITD[3] = 0;
    stage_info.itemdata = ((fv() % 3) == 0) ? 0 : ITD;
    AYA[0] = 0; AYA[1] = &DUM[0]; AYA[2] = &DUM[1]; AYA[3] = ((fv() & 1) != 0) ? (void*) &DUM[2] : 0; AYA[4] = &DUM[3]; AYA[5] = 0;
    stage_info.ald_yaku_all = ((fv() % 3) == 0) ? 0 : (void***) AYA;
    ART.xC_itemStates = &ISA;
    for (i = 0; i < 256; i++) TB[i] = &ART;
    it_804D6D38 = TB;
    stage_info.coll_data = ((fv() % 4) == 0) ? 0 : &MCD;
    stage_info.map_ptcl = ((fv() & 1) != 0) ? (void*) &DUM[4] : 0; stage_info.map_texg = ((fv() & 1) != 0) ? (void*) &DUM[5] : 0;
    stage_info.yakumono_param = ((fv() & 1) != 0) ? (void*) YAK : 0;
    stage_info.map_plit = ((fv() % 3) == 0) ? 0 : LLP[fv() % 3];
    stage_info.x90 = ((fv() % 3) == 0) ? 0 : pr90; stage_info.x94 = ((fv() % 3) == 0) ? 0 : pr94;
    stage_info.on_touch_line = ((fv() % 3) == 0) ? 0 : sd_tl;
    stage_info.on_check_shadow_render = ((fv() % 3) == 0) ? 0 : sd_cs;
    for (i = 0; i < 3; i++) for (j = 0; j < 8; j++) {
        SCB[i][j].flags_b0 = ((fv() % 4) == 0); SCB[i][j].flags_b1 = ((fv() % 4) == 0); SCB[i][j].flags_b2 = ((fv() % 2) == 0);
    }
    { Vec* vv[6]; vv[0] = &stage_info.x130; vv[1] = &stage_info.x13C; vv[2] = &stage_info.x148; vv[3] = &stage_info.x154; vv[4] = &stage_info.x160; vv[5] = &stage_info.x16C;
      for (i = 0; i < 6; i++) { float k2 = ((fv() % 2) == 0) ? 30.0f : 1.0f; vv[i]->x *= k2; vv[i]->y *= k2; vv[i]->z *= k2; } }
    { float kk = ((fv() % 3) == 0) ? 100.0f : 1.0f; stage_info.x70C = rg() * kk; stage_info.x710 = rg() * kk; stage_info.x718 = rg() * kk; stage_info.x71C = rg() * kk; }
    if ((fv() % 3) == 0) stage_info.x714 = -1;
    if ((fv() % 3) == 0) stage_info.x720 = -1;
    stage_info.grkind = (GrKind) pick_kind();
    for (i = 0; i < 8; i++) { MV[i].x = rg(); MV[i].y = rg(); }
    MCD.verts = MV; MCD.vert_count = (int) (fv() % 7); MCD.lines = MLN; MCD.line_count = (int) (fv() % 7);
    MCD.joints = MJN; MCD.joint_count = (int) (fv() % 7); MCD.x2C = (int) fv();
    for (i = 0; i < 5; i++) { MCD.ranges[i].start = (s16) (fv() % 100); MCD.ranges[i].count = (s16) (fv() % 100); }
    for (i = 0; i < 8; i++) {
        MLN[i].v0_idx = (u16) fv(); MLN[i].v1_idx = (u16) fv(); MLN[i].prev_id0 = (s16) fv(); MLN[i].next_id0 = (s16) fv();
        MLN[i].prev_id1 = (s16) fv(); MLN[i].next_id1 = (s16) fv(); MLN[i].hi_flags = (u16) fv(); MLN[i].lo_flags = (u16) fv();
        MJN[i].left_bound = rg(); MJN[i].bottom_bound = rg(); MJN[i].right_bound = rg(); MJN[i].top_bound = rg();
        MJN[i].vtx_start = (s16) fv(); MJN[i].vtx_count = (s16) fv();
        for (k = 0; k < 5; k++) { MJN[i].ranges[k].start = (s16) fv(); MJN[i].ranges[k].count = (s16) fv(); }
    }
    for (i = 0; i < 16; i++) CJ[i].xC = (s16) fv();
    mpColl_804D64AC = (int) (fv() % 100);
    Ground_804D6950 = BUF; Ground_804D6954 = (s16) (fv() % 100);

    /* stage rows */
    if (!SD0N) { for (i = 0; i < 160; i++) SD0[i] = stage_datas[i]; SD0N = 1; }
    for (i = 0; i < 160; i++) stage_datas[i] = SD0[i];
    zero(Ground_MexStageDatas, sizeof Ground_MexStageDatas);
    Ground_MexInitStages_done = 0;
    for (i = 0; i < 160; i++) ROWTAB[i] = 0;
    ROWTAB[0] = &Ground_StageData;
@ROWSETUP@
    for (i = 0; i < 89; i++) ROWTAB[71 + i] = &Ground_MexStageDatas[i];
    for (i = 1; i < 68; i++) setrow(ROWTAB[i]);
    for (i = 71; i < 160; i++) setrow(ROWTAB[i]);
}
"""

ROWSETUP_C = "\n".join(f"    ROWTAB[{i + 1}] = &{n};" for i, n in enumerate(ROWNAMES))
DECL_C = DECL_C.replace("@ROWSETUP@", ROWSETUP_C)
