/* TU: src/melee/ft/ftparts.c */
/* C oracle for src/tu/ftparts.rs (SetupParts, 8007462C, 800749CC, 800750C8, 80075240, the visibility
 * tables, 80074E58, 80074194, 8007487C, JObjSetRotation): the whole TU included verbatim; every callee
 * out of it is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"

extern int fflush(void*);
#define KIND 7
FighterPartsTable** ftPartsTable;
Fighter_804D6540_t** Fighter_804D6540;
static FighterPartsTable* PTBL[Ft_Kind_Max];
static Fighter_804D6540_t* T6540L[Ft_Kind_Max];
ftData_UnkModelStruct ftData_UnkIntBoolFunc0;
HSD_ObjAllocData fighter_parts_alloc_data, fighter_dobj_list_alloc_data;
HSD_GObj* HSD_GObj_804D7814;

static HSD_JObj JT[16], JA[16];
static HSD_DObj DP[64];
static HSD_MObj MO[64];
static HSD_TObj TO[128];
static HSD_DObj* DL[128];
static HSD_DObj* DL2[128];
static FighterBone PARTS[40];
static FighterPartsTable PT;
static u8 J2P[64], P2J[64];
static Fighter_804D6540_t T6540;
static Fighter_804D6540_x0_t X6540[4];
static FtPartsVisLookup LKA[5][12];
static TempS TSA[5][12][3];
static u8 UB[5][12][3][4];
static void* VT[3][4];
static FtPartsDesc FDESC;
static struct ftData_x8 FX8;
static ftData FDATA;
static int NJ, NND;

static u32 EP(const void* p)
{
    unsigned long long a = (unsigned long long) p;
    if (p == 0) return 0;
    if (a >= (unsigned long long) JT && a < (unsigned long long) (JT + 16)) return 0x100u + (u32) ((HSD_JObj*) p - JT);
    if (a >= (unsigned long long) JA && a < (unsigned long long) (JA + 16)) return 0x200u + (u32) ((HSD_JObj*) p - JA);
    if (a >= (unsigned long long) DP && a < (unsigned long long) (DP + 64)) return 0x300u + (u32) ((HSD_DObj*) p - DP);
    if (a >= (unsigned long long) TO && a < (unsigned long long) (TO + 128)) return 0x400u + (u32) ((HSD_TObj*) p - TO);
    if (p == DL) return 0x501;
    if (p == DL2) return 0x502;
    if (p == PARTS) return 0x503;
    if (p == &F) return 0x504;
    return 0xFFFF;
}
static u32 LKP(const void* p)
{
    int r;
    if (p == 0) return 0;
    for (r = 0; r < 5; r++) if (p == LKA[r]) return (u32) r + 1;
    return 0xFF;
}

HSD_DObj* HSD_JObjGetDObj(HSD_JObj* j) { if (j == NULL || (j->flags & (JOBJ_PTCL | JOBJ_SPLINE))) return NULL; return j->u.dobj; }
u32 HSD_JObjGetFlags(HSD_JObj* j) { return j ? j->flags : 0; }
HSD_TObj* HSD_MObjGetTObj(HSD_MObj* m) { return m ? m->tobj : NULL; }
HSD_TObj* HSD_TObjGetNext(HSD_TObj* t) { return t ? t->next : NULL; }
void* HSD_ObjAlloc(HSD_ObjAllocData* d)
{
    T(1); T(d == &fighter_parts_alloc_data);
    return d == &fighter_parts_alloc_data ? (void*) PARTS : (void*) DL;
}
void HSD_DObjSetFlags(HSD_DObj* d, u32 f) { T(2); T(EP(d)); T(f); if (d) d->flags |= f; }
void HSD_DObjClearFlags(HSD_DObj* d, u32 f) { T(3); T(EP(d)); T(f); if (d) d->flags &= ~f; }
void HSD_JObjClearFlags(HSD_JObj* j, u32 f) { T(4); T(EP(j)); T(f); }
void HSD_JObjSetMtxDirtySub(HSD_JObj* j) { T(9); T(EP(j)); }
static void model_cb(Fighter* fp, int m, bool b) { T(5); T(fp == &F); T((u32) m); T((u32) b); }
int Mex_CostumeVisIdx(int fk, int costume) { T(6); T((u32) fk); T((u32) costume); return (int) (rnd() % 3); }

static void gen_tree(void)
{
    int i, par[16], nd = 0;
    NJ = 1 + (int) (rnd() % 10);
    zero(JT, sizeof JT); zero(JA, sizeof JA); zero(DP, sizeof DP); zero(MO, sizeof MO); zero(TO, sizeof TO);
    for (i = 0; i < NJ; i++) {
        u32 r = rnd();
        par[i] = i ? (int) ((r >> 8) % (u32) i) : -1;
        JT[i].flags = (r % 6 == 0) ? JOBJ_INSTANCE : 0;
        JT[i].flags |= rnd() & (JOBJ_LIGHTING | JOBJ_TEXGEN | JOBJ_SPECULAR | JOBJ_UNK_B18 | JOBJ_UNK_B19 | JOBJ_UNK_B20);
    }
    for (i = NJ - 1; i >= 1; i--) {
        JT[i].parent = &JT[par[i]];
        JT[i].next = JT[par[i]].child;
        JT[par[i]].child = &JT[i];
    }
    for (i = 0; i < NJ; i++) {
        JA[i].flags = JT[i].flags;
        JA[i].parent = JT[i].parent ? &JA[JT[i].parent - JT] : NULL;
        JA[i].child = JT[i].child ? &JA[JT[i].child - JT] : NULL;
        JA[i].next = JT[i].next ? &JA[JT[i].next - JT] : NULL;
    }
    for (i = 0; i < NJ; i++) {
        int c = (int) (rnd() % 4), q;
        HSD_DObj* first = NULL;
        for (q = c - 1; q >= 0; q--) {
            DP[nd + q].next = first;
            first = &DP[nd + q];
        }
        JT[i].u.dobj = first;
        nd += c;
    }
    NND = nd;
}
static int dry(HSD_JObj* root)
{
    HSD_JObj* jobj = root;
    int part = 0;
    while (jobj != NULL) {
        if (ftParts_8007506C(KIND, part) != 0) { part++; continue; }
        part++;
        if (!(jobj->flags & JOBJ_INSTANCE) && jobj->child != NULL) { jobj = jobj->child; continue; }
        if (jobj->next != NULL) { jobj = jobj->next; continue; }
        while (1) {
            if (jobj->parent == NULL) { jobj = NULL; break; }
            if (jobj->parent->next != NULL) { jobj = jobj->parent->next; break; }
            jobj = jobj->parent;
        }
    }
    return part;
}
static void tables(void)
{
    int i;
    zero(&T6540, sizeof T6540); zero(X6540, sizeof X6540);
    T6540.x4 = (s32) (rnd() % 4);
    T6540.x0 = X6540;
    for (i = 0; i < 4; i++) X6540[i].x0 = (u8) (rnd() % (u32) (NJ + 3));
    ftPartsTable = PTBL; Fighter_804D6540 = T6540L;
    Fighter_804D6540[KIND] = (rnd() % 8 == 0) ? NULL : &T6540;
    ftPartsTable[KIND] = &PT;
    PT.joint_to_part = J2P;
    PT.part_to_joint = P2J;
}
static void hbone(FighterBone* b)
{
    HM(EP(b->joint)); HM(EP(b->x4_jobj2));
    HM(b->flags_b0); HM(b->flags_b1); HM(b->flags_b2); HM(b->flags_b3); HM(b->flags_b4); HM(b->flags_b5); HM(b->flags_b6); HM(b->flags_b7);
    HM(b->flags2_b0); HM(b->flags2_b1); HM(b->flags2_b2); HM(b->flags2_b3); HM(b->flags2_b4); HM(b->flags2_b5); HM(b->flags2_b6); HM(b->flags2_b7);
    HM(b->xC); HM(b->xD);
}
static void vis_world(void)
{
    int r, m, s, q, i;
    zero(LKA, sizeof LKA); zero(TSA, sizeof TSA); zero(UB, sizeof UB); zero(VT, sizeof VT);
    NND = 1 + (int) (rnd() % 8);
    for (i = 0; i < 128; i++) { DL[i] = NULL; DL2[i] = NULL; }
    for (i = 0; i < 64; i++) DP[i].flags = rnd();
    for (i = 0; i < NND; i++) { DL[i] = (rnd() % 5 == 0) ? NULL : &DP[i]; DL2[i] = (rnd() % 5 == 0) ? NULL : &DP[8 + i]; }
    F.dobj_list.count = (u32) NND; F.dobj_list.data = DL;
    F.x203C.count = (u32) NND; F.x203C.data = DL2;
    for (r = 0; r < 5; r++) {
        for (m = 0; m < 12; m++) {
            LKA[r][m].x0 = (s32) (rnd() % 3);
            LKA[r][m].x4 = TSA[r][m];
            for (s = 0; s < 3; s++) {
                TSA[r][m][s].x0 = (s32) (rnd() % 3);
                TSA[r][m][s].x4 = UB[r][m][s];
                for (q = 0; q < 4; q++) UB[r][m][s][q] = (u8) (rnd() % (u32) (NND + 2));
            }
        }
    }
    for (r = 0; r < 5; r++) {
        F.x5AC.xC[r] = (rnd() % 4 == 0) ? NULL : LKA[r];
        F.x5AC.cleared[r] = (u8) (rnd() & 1);
    }
    F.x5AC.model_num = rnd() % 5;
    for (i = 0; i < 12; i++) { F.x5F4_arr[i].idx = (s8) ((int) (rnd() % 5) - 1); F.x5F4_arr[i].prev = (s8) ((int) (rnd() % 5) - 1); }
    for (r = 0; r < 3; r++) for (s = 0; s < 4; s++) VT[r][s] = (rnd() % 3 == 0) ? NULL : (void*) LKA[rnd() % 5];
    FDESC.model_num = rnd() % 12;
    FDESC.vis_table = VT;
    FX8.x0 = FDESC;
    FDATA.x8 = &FX8;
    F.ft_data = &FDATA;
    F.x619_costume_id = (u8) (rnd() % 3);
    F.kind = KIND;
}
static void hvis(void)
{
    int i;
    HM(F.x5AC.model_num);
    for (i = 0; i < 5; i++) { HM(LKP(F.x5AC.xC[i])); HM(F.x5AC.cleared[i]); }
    for (i = 0; i < 12; i++) { HM((u32) (int) F.x5F4_arr[i].idx); HM((u32) (int) F.x5F4_arr[i].prev); }
    for (i = 0; i < 64; i++) HM(DP[i].flags);
    HM(F.x221D_b2);
}

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < 12; op++) {
        for (k = 0; k < 1500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            H = 0x811C9DC5u;
            zero(PARTS, sizeof PARTS); zero(P2J, sizeof P2J); zero(J2P, sizeof J2P);
            GO.user_data = &F; F.gobj = &GO;
            F.player_id = (u8) (rnd() % 4);
            F.kind = KIND;
            gen_tree();
            tables();
            ftData_UnkIntBoolFunc0.model_events[KIND] = (rnd() & 1) ? model_cb : NULL;
            switch (op) {
            case 0:
                PT.parts_num = (u32) dry(&JT[0]);
                GO.hsd_obj = &JT[0];
                F.parts = PARTS; F.dobj_list.data = DL; F.dobj_list.count = 99;
                ftParts_SetupParts(&GO);
                { int i; for (i = 0; i < 20; i++) hbone(&PARTS[i]); for (i = 0; i < NND; i++) HM(EP(DL[i])); HM(F.dobj_list.count); }
                break;
            case 1:
                PT.parts_num = (u32) dry(&JA[0]);
                F.x8AC_animSkeleton = &JA[0];
                F.parts = PARTS;
                ftParts_8007462C(&GO);
                { int i; for (i = 0; i < 20; i++) hbone(&PARTS[i]); }
                break;
            case 2:
                vis_world();
                ftParts_800749CC(&GO);
                hvis();
                break;
            case 3:
                vis_world();
                { u32 a = rnd() % 5; ftParts_800750C8(&F, a, (bool) (rnd() & 1)); }
                hvis();
                break;
            case 4:
                {
                    int i, j, n, total = 0;
                    vis_world();
                    for (i = 0; i < NND; i++) {
                        DP[i].mobj = NULL;
                        if (rnd() % 3 != 0) {
                            HSD_TObj* prev = NULL;
                            int cnt = (int) (rnd() % 3);
                            DP[i].mobj = &MO[i];
                            MO[i].tobj = NULL;
                            for (j = cnt - 1; j >= 0; j--) {
                                TO[i * 3 + j].next = prev;
                                prev = &TO[i * 3 + j];
                            }
                            MO[i].tobj = prev;
                            total += cnt;
                        }
                    }
                    for (i = 0; i < NND; i++) { DL[i] = (rnd() % 5 == 0) ? NULL : &DP[i]; }
                    /* only lists whose TObjs all belong to a non-NULL DObj count */
                    total = 0;
                    for (i = 0; i < NND; i++) if (DL[i] && DL[i]->mobj) { HSD_TObj* t = DL[i]->mobj->tobj; while (t) { total++; t = t->next; } }
                    if (total == 0) break;
                    n = (int) (rnd() % (u32) total);
                    ret = EP(ftParts_80075240(&F.dobj_list, n));
                }
                break;
            case 5:
                vis_world();
                { int idx = (int) (rnd() % 5); ftParts_80074B6C(&F, &F.x5AC, idx, (rnd() & 1) ? &F.dobj_list : &F.x203C); }
                hvis();
                break;
            case 6:
                vis_world();
                { int idx = (int) (rnd() % 5); ftParts_80074CA0(&F.x5AC, idx, (rnd() & 1) ? &F.dobj_list : &F.x203C); }
                hvis();
                break;
            case 7:
                vis_world();
                { int idx = (int) (rnd() % 5); ftParts_80074D7C(&F.x5AC, idx, (rnd() & 1) ? &F.dobj_list : &F.x203C); }
                hvis();
                break;
            case 8:
                {
                    int i;
                    PT.parts_num = 1 + rnd() % 13;
                    for (i = 0; i < 64; i++) P2J[i] = (u8) (rnd() % PT.parts_num);
                    F.parts = NULL; F.dobj_list.data = NULL;
                    ftParts_80074E58(&F);
                    for (i = 0; i < (int) PT.parts_num; i++) hbone(&PARTS[i]);
                    HM(EP(F.parts)); HM(EP(F.dobj_list.data));
                }
                break;
            case 9:
                {
                    Vec4 q;
                    q.x = rw(); q.y = rw(); q.z = rw(); q.w = rw();
                    JT[0].flags = (rnd() & 1) ? (JOBJ_MTX_INDEP_SRT | JOBJ_USE_QUATERNION) : (JOBJ_MTX_DIRTY | JOBJ_USE_QUATERNION);
                    JT[0].rotate.x = rw(); JT[0].rotate.y = rw();
                    ftParts_JObjSetRotation(&JT[0], &q);
                    HM(JT[0].flags); HM(B(JT[0].rotate.x)); HM(B(JT[0].rotate.y)); HM(B(JT[0].rotate.z)); HM(B(JT[0].rotate.w));
                }
                break;
            case 10:
                {
                    int di = (int) (rnd() % 6);
                    u32 depth = rnd() % 5;
                    int jj = (int) (rnd() % (u32) NJ);
                    F.parts = PARTS; F.dobj_list.data = DL;
                    ftParts_80074194(&F, &PARTS[3], &JT[jj], &di, depth);
                    hbone(&PARTS[3]);
                    { int i; for (i = 0; i < di && i < 128; i++) HM(EP(DL[i])); HM((u32) di); }
                }
                break;
            case 11:
                vis_world();
                ftParts_8007487C(&FDESC, &F.x5AC, (u32) (rnd() % 3), &F.dobj_list, &F.x203C);
                hvis();
                break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
            fflush(0);
        }
    }
    return 0;
}
