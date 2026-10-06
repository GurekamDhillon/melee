/* Native Mario-reference backend, included in ftdata.c after its tables.
 * Author owns independent effective attributes/animation/state rows. Shared
 * retail model/clip/effect/audio descriptors are immutable asset references.
 * No MxDt row or ftFunction archive is constructed or installed for this kind.
 */
#include "geno_define_game.h"
#include "geno_plan.h"
#include "geno.h"
#include <sysdolphin/baselib/memory.h>
#include <melee/ft/kinds/ftMario/forward.h>
#include <melee/ft/kinds/ftMario/types.h>
#include <melee/ft/kinds/ftCommon/forward.h>

MotionState* GenoDefine_CommonRows[Ft_Kind_Max];
MotionState* GenoDefine_SpecificRows[Ft_Kind_Max];
u32 GenoDefine_ContentIdentity[Ft_Kind_Max][2];
u32 GenoDefine_InterpreterAttempts[Ft_Kind_Max];
static UnkCostumeStruct GenoDefine_Costumes[Ft_Kind_Max - Ft_Kind_Mex0][16];
static Fighter_WaitAnimData* GenoDefine_MarioBaseline;
/* slice 4: a base "none" define's own costume names, parts table and part-visibility stand-ins (all static, kind-indexed). */
static Fighter_CostumeStrings GenoDefine_CostumeStrings[Ft_Kind_Max - Ft_Kind_Mex0][16];
static FighterPartsTable GenoDefine_Parts[Ft_Kind_Max - Ft_Kind_Mex0];
static u8 GenoDefine_PartsJ2P[Ft_Kind_Max - Ft_Kind_Mex0][128];
static u8 GenoDefine_PartsP2J[Ft_Kind_Max - Ft_Kind_Mex0][56];
static void* GenoDefine_VisTable[16][4];     /* no visibility groups: every row NULL, model_num 0 */
static u16 GenoDefine_TobjZero;
static u16* GenoDefine_TobjTable[16];        /* no texture animations: n_costume_tobjs 0 */
extern int Geno_DefineIsNone(int kind);
extern int Geno_DefineCostumeCount(int kind);
extern const char* Geno_DefineString(int kind, int which, int costume);
extern int Geno_DefinePlanInt(int kind, int what, int i);
extern const char* Geno_DefinePlanText(int kind, int which, int i);

/* The fighter's parts table (Fighter_Part slot -> joint, joint -> slot) from its plan; NULL for a define that is not
 * base "none" (it takes the Mario table) or whose plan is unusable. Built once per kind, static storage. */
FighterPartsTable* GenoDefine_PartsTable(int kind)
{
    int slot = kind - Ft_Kind_Mex0;
    if (slot < 0 || slot >= Ft_Kind_Max - Ft_Kind_Mex0 || !Geno_DefineIsNone(kind)) return NULL;
    if (!Geno_DefinePlanInt(kind, 0, 0)) return NULL;
    if (GenoDefine_Parts[slot].parts_num == 0) {
        int nj = Geno_DefinePlanInt(kind, 1, 0), np = Geno_DefinePlanInt(kind, 2, 0);
        memset(GenoDefine_PartsJ2P[slot], 255, sizeof GenoDefine_PartsJ2P[slot]);
        memset(GenoDefine_PartsP2J[slot], 255, sizeof GenoDefine_PartsP2J[slot]);
        memcpy(GenoDefine_PartsJ2P[slot], Geno_DefinePlanText(kind, 2, 0), nj);
        memcpy(GenoDefine_PartsP2J[slot], Geno_DefinePlanText(kind, 3, 0), np);
        GenoDefine_Parts[slot].joint_to_part = GenoDefine_PartsJ2P[slot];
        GenoDefine_Parts[slot].part_to_joint = GenoDefine_PartsP2J[slot];
        GenoDefine_Parts[slot].parts_num = (u32)nj;
    }
    return &GenoDefine_Parts[slot];
}

static void* gd_data_copy(const void* source, int bytes)
{
    void* out = HSD_MemAlloc(bytes);
    HSD_ASSERTREPORT(__LINE__, out != NULL, "Geno definition descriptor allocation exhausted\n");
    memcpy(out, source, bytes); return out;
}

void GenoDefine_ResetDescriptors(void)
{
    GenoDefine_MarioBaseline = NULL;
    memset(GenoDefine_CommonRows, 0, sizeof GenoDefine_CommonRows);
    memset(GenoDefine_SpecificRows, 0, sizeof GenoDefine_SpecificRows);
}

/* Capture before the first Mario attachment can change script pointers.
 * Animation-bank pointers are resolved later independently for each clone. */
void GenoDefine_CaptureMario(void)
{
    if (Geno_DefineCount() && !GenoDefine_MarioBaseline && gFtDataList[0])
        GenoDefine_MarioBaseline = gd_data_copy(gFtDataList[0]->xC,
            ftData_Table_Unk0[0].count*sizeof(*GenoDefine_MarioBaseline));
}

void GenoDefine_InitKinds(void)
{
    int i, n = Geno_DefineCount();
    extern void Player_MexSetMapping(int ck, int fk);
    for (i = 0; i < n; ++i) {
        int ck = Geno_DefineCKAt(i), fk = ck-1, base = Geno_DefineBaseKind(fk);
        if (base < 0 || fk < Ft_Kind_Mex0 || fk >= Ft_Kind_Max) continue;
        /* Native callbacks are explicit versioned Mario adapters, independent
           of m-ex fighter_function lookup and its PPC bridge. */
#define GD_INHERIT(table) table[fk] = table[base]
        GD_INHERIT(ftData_OnLoad); GD_INHERIT(ftData_OnDeath);
        GD_INHERIT(ftData_OnUserDataRemove); GD_INHERIT(ftData_CharacterStateTables);
        if (GenoDefine_SpecificRows[fk]) ftData_CharacterStateTables[fk] = GenoDefine_SpecificRows[fk];
        GD_INHERIT(ftData_SpecialN); GD_INHERIT(ftData_SpecialAirN);
        GD_INHERIT(ftData_SpecialS); GD_INHERIT(ftData_SpecialAirS);
        GD_INHERIT(ftData_SpecialHi); GD_INHERIT(ftData_SpecialAirHi);
        GD_INHERIT(ftData_SpecialLw); GD_INHERIT(ftData_SpecialAirLw);
        GD_INHERIT(ftData_OnAbsorb); GD_INHERIT(ftData_OnItemPickupExt);
        GD_INHERIT(ftData_OnItemInvisible); GD_INHERIT(ftData_OnItemVisible);
        GD_INHERIT(ftData_OnItemDropExt); GD_INHERIT(ftData_OnItemPickup);
        GD_INHERIT(ftData_OnItemDrop); GD_INHERIT(ftData_UnkMotionStates1);
        GD_INHERIT(ftData_UnkMotionStates2); GD_INHERIT(ftData_OnKnockbackEnter);
        GD_INHERIT(ftData_OnKnockbackExit); GD_INHERIT(ftData_UnkMotionStates3);
        GD_INHERIT(ftData_UnkMotionStates4); GD_INHERIT(ftKindCalcIndiviParamTable);
        GD_INHERIT(ftData_UnkMtxFunc0); GD_INHERIT(ftData_803C24EC);
        GD_INHERIT(ftData_UnkDemoCallbacks0); GD_INHERIT(ftData_UnkMotionStates0);
        GD_INHERIT(ftData_803C1F40); GD_INHERIT(ftData_803C23E4);
        GD_INHERIT(ftData_803C2468); GD_INHERIT(ftData_803C2360);
        GD_INHERIT(ftData_UnkMotionStates5); GD_INHERIT(ftData_UnkBytePerCharacter);
        GD_INHERIT(ftData_UnkCallbackPairs0);
#undef GD_INHERIT
        if (Geno_DefineIsNone(fk)) {
            /* Own files by name: the mod's files/ resolve ahead of the disc. No data (Pl*.dat) file: the fighter's
             * ftData is built natively in GenoDefine_Load. Callbacks still come from the named Mario common tables
             * above (this slice); every Geno-bound special replaces its entry. */
            int c, nc = Geno_DefineCostumeCount(fk), slot = fk - Ft_Kind_Mex0;
            ftData_803C1F40[fk].a = NULL; ftData_803C1F40[fk].b = NULL;
            ftData_803C23E4[fk] = (char*)Geno_DefineString(fk, 0, 0);
            for (c = 0; c < nc && c < 16; ++c) {
                GenoDefine_CostumeStrings[slot][c].dat_filename = (char*)Geno_DefineString(fk, 2, c);
                GenoDefine_CostumeStrings[slot][c].joint_name = (char*)Geno_DefineString(fk, 3, c);
                GenoDefine_CostumeStrings[slot][c].matanim_joint_name = (char*)Geno_DefineString(fk, 4, c);
            }
            ftData_803C2360[fk] = GenoDefine_CostumeStrings[slot];
            { static int said; if (!said++) OSReport("geno: base none kind %d: files %s, costumes %d (first %s)\n", fk, ftData_803C23E4[fk], nc, GenoDefine_CostumeStrings[slot][0].dat_filename); }
        }
        /* Do not invoke Mario's global init twice under a new id. */
        ftData_Table_Unk1[fk] = NULL;
        ftData_Table_Unk0[fk].count = ftData_Table_Unk0[base].count;
        ftData_UnkIntPairs[fk].count = ftData_UnkIntPairs[base].count;
        CostumeListsForeachCharacter[fk].costume_list = GenoDefine_Costumes[fk-Ft_Kind_Mex0];
        CostumeListsForeachCharacter[fk].numCostumes = Geno_DefineIsNone(fk) ? Geno_DefineCostumeCount(fk)
                                                      : CostumeListsForeachCharacter[base].numCostumes;
        Player_MexSetMapping(ck, fk);
        GenoDefine_ContentIdentity[fk][0] = Geno_DefineIdentityWord(fk, 0);
        GenoDefine_ContentIdentity[fk][1] = Geno_DefineIdentityWord(fk, 1);
    }
}


/* A base "none" define's own model data in its ftData: the part-visibility and texture-animation stand-ins (the Courier
 * has no model-part groups) and the animation rows. Row i of Mario's 303-row table is the subaction index every
 * overlay and every common MotionState uses; its clip is the one the package's motion_rows give the motion states
 * that point at it (own clip beats alias beats placeholder). The row keeps Mario's script words (sounds, effects,
 * hit timing for every move a define does not author) and takes the package's symbol, offset and size, so the
 * figatree played is the define's own. */

/* The ftData fields that name joints (indices into fp->parts, which is joint order) hold the donor's numbers in the copy
 * of Mario's struct; a base "none" define takes the package's, resolved by role in the plan (plan.json "ftdata"). The
 * floats beside them (IK lengths, ECB offsets, ledge snap, centre scale) stay Mario's until step (d) fits them. */
static void GenoDefine_JointFields(int kind, ftData* out, struct ftData_x8* x8)
{
    union { int i; float f; } u;
    int i, n;
    u8* b = &x8->x10;
    ftData_x30* x30;
    ftHurtboxInit* inits;
    ftData_x34* x34 = gd_data_copy(out->x34, sizeof *x34);
    ftData_x38* x38 = gd_data_copy(out->x38, 2*sizeof *x38);
    ftData_x44_t* x44 = gd_data_copy(out->x44, sizeof *x44);
    int* x54 = gd_data_copy(out->x54, 5*sizeof(int));
    struct ftData_x58_t* x58 = gd_data_copy(out->x58, sizeof *x58);
    for (i = 0; i < 5; ++i) b[i] = (u8)Geno_DefinePlanInt(kind, 20, i);
    x34->x0 = (Fighter_Part)Geno_DefinePlanInt(kind, 21, 0);
    for (i = 0; i < 2; ++i) x38[i].x0 = Geno_DefinePlanInt(kind, 22, i);
    x44->unk0 = (s16)Geno_DefinePlanInt(kind, 23, 0); x44->unk2 = (s16)Geno_DefinePlanInt(kind, 23, 1);
    x44->unk4 = (s16)Geno_DefinePlanInt(kind, 23, 2); x44->unk6 = (s16)Geno_DefinePlanInt(kind, 23, 3);
    x44->unk8 = (s16)Geno_DefinePlanInt(kind, 23, 4); x44->unkA = (s16)Geno_DefinePlanInt(kind, 23, 5);
    for (i = 0; i < 5; ++i) x54[i] = Geno_DefinePlanInt(kind, 24, i);
    /* IK pairs: [0],[1] thigh R,L; [2],[3] knee; [4],[5] foot; (arm pairs follow and are unused by ft_0899) */
    x58->x0 = (u8)Geno_DefinePlanInt(kind, 25, 0); x58->x1 = (u8)Geno_DefinePlanInt(kind, 25, 1);
    x58->x8 = (u8)Geno_DefinePlanInt(kind, 25, 2); x58->x9 = (u8)Geno_DefinePlanInt(kind, 25, 3);
    x58->x10 = (u8)Geno_DefinePlanInt(kind, 25, 4); x58->x11 = (u8)Geno_DefinePlanInt(kind, 25, 5);
    out->x34 = x34; out->x38 = x38; out->x44 = x44; out->x54 = x54; out->x58 = x58;
    n = Geno_DefinePlanInt(kind, 5, 0);
    x30 = gd_data_copy(out->x30, sizeof *x30);
    inits = gd_data_copy(out->x30->inits, n*sizeof *inits);
    for (i = 0; i < n; ++i) {
        inits[i].bone_idx = (Fighter_Part)Geno_DefinePlanInt(kind, 30, i);
        inits[i].height = (HurtHeight)Geno_DefinePlanInt(kind, 31, i);
        inits[i].is_grabbable = (u32)Geno_DefinePlanInt(kind, 32, i);
        u.i = Geno_DefinePlanInt(kind, 40, i*8+0); inits[i].a_offset.x = u.f;
        u.i = Geno_DefinePlanInt(kind, 40, i*8+1); inits[i].a_offset.y = u.f;
        u.i = Geno_DefinePlanInt(kind, 40, i*8+2); inits[i].a_offset.z = u.f;
        u.i = Geno_DefinePlanInt(kind, 40, i*8+3); inits[i].b_offset.x = u.f;
        u.i = Geno_DefinePlanInt(kind, 40, i*8+4); inits[i].b_offset.y = u.f;
        u.i = Geno_DefinePlanInt(kind, 40, i*8+5); inits[i].b_offset.z = u.f;
        u.i = Geno_DefinePlanInt(kind, 40, i*8+6); inits[i].scale = u.f;
    }
    x30->count = n; x30->inits = inits;
    out->x30 = x30;
    OSReport("geno: base none kind %d: ftData joint fields from the plan: ECB joints %d,%d,%d,%d,%d,%d; %d hurtboxes\n", kind,
             x44->unk0, x44->unk2, x44->unk4, x44->unk6, x44->unk8, x44->unkA, n);
}

static void GenoDefine_BuildOwn(int kind, ftData* out, int n)
{
    extern MotionState ftMr_Init_MotionStateTable[];
    int nmotions = Geno_DefinePlanInt(kind, 4, 0), njoints = Geno_DefinePlanInt(kind, 1, 0);
    struct ftData_x8* x8;
    short pick[512];
    u8 rank[512];
    int m, i, conflicts = 0, mapped = 0;
    HSD_ASSERTREPORT(__LINE__, Geno_DefinePlanInt(kind, 0, 0), "Geno base none: the fighter plan is unusable (see geno: plan lines in the log)\n");
    HSD_ASSERTREPORT(__LINE__, n <= 512, "Geno base none: animation row table too large\n");
    x8 = gd_data_copy(out->x8, sizeof *x8);          /* Mario's x10..x14 are Fighter_Part numbers: roles, resolved by the own parts table */
    x8->x0.model_num = 0;
    x8->x0.vis_table = GenoDefine_VisTable;
    x8->x8.x8 = 0;
    for (i = 0; i < 16; ++i) GenoDefine_TobjTable[i] = &GenoDefine_TobjZero;
    x8->x8.xC = GenoDefine_TobjTable;
    out->x8 = x8;
    GenoDefine_JointFields(kind, out, x8);
    for (i = 0; i < n; ++i) { pick[i] = -1; rank[i] = 0; }
    for (m = 0; m < nmotions && m < ftCo_MS_Count + ftMr_MS_SelfCount; ++m) {
        MotionState* st = m < ftCo_MS_Count ? &ftData_MotionStateList[m] : &ftMr_Init_MotionStateTable[m - ftCo_MS_Count];
        int a = st->anim_id, c = Geno_DefinePlanInt(kind, 13, m);
        if (a < 0 || a >= n || c < 0) continue;
        /* the row's rank: a state with its own name beats one that aliases (the clip name equals Mario's row it serves) */
        if (rank[a] < 1) { pick[a] = (short)c; rank[a] = 1; }
        else if (pick[a] != c) {
            ++conflicts;
            if (conflicts <= 32) OSReport("geno: base none kind %d: motion %d shares animation row %d with another state but names clip %s, not %s; the first stays\n",
                                         kind, m, a, Geno_DefinePlanText(kind, 0, c), Geno_DefinePlanText(kind, 0, pick[a]));
        }
    }
    for (i = 0; i < n; ++i) {
        Fighter_WaitAnimData* row = &out->xC[i];
        if (pick[i] < 0) { row->x8 = 0; row->x4 = 0; continue; }   /* no state of the package points at it: nothing to load */
        row->x0 = (char*)Geno_DefinePlanText(kind, 1, pick[i]);
        row->x4 = (s32)Geno_DefinePlanInt(kind, 10, pick[i]);
        row->x8 = (s32)Geno_DefinePlanInt(kind, 11, pick[i]);
        ++mapped;
    }
    OSReport("geno: base none kind %d: %d of %d animation rows take the package's clips (%d shared-row conflicts); own parts table of %d joints\n",
             kind, mapped, n, conflicts, njoints);
}

void GenoDefine_Load(int kind, int base)
{
    extern int Netplay_Enabled(void);
    ftData* source = gFtDataList[base];
    ftData* out;
    int i, n = ftData_Table_Unk0[base].count, demo = ftData_UnkIntPairs[base].count;
    HSD_ASSERTREPORT(__LINE__, !Netplay_Enabled(), "Geno define is offline only: online admission/identity contract unavailable\n");
    if (gFtDataList[kind] != NULL) return;
    out = gd_data_copy(source, sizeof(*out));
    out->x0 = gd_data_copy(source->x0, sizeof(ftCo_DatAttrs));
    out->ext_attr = gd_data_copy(source->ext_attr, sizeof(ftMario_DatAttrs));
    HSD_ASSERTREPORT(__LINE__, GenoDefine_MarioBaseline != NULL, "Geno native Mario preset baseline unavailable\n");
    out->xC = gd_data_copy(GenoDefine_MarioBaseline, n*sizeof(*out->xC));
    out->x14 = gd_data_copy(source->x14, demo*sizeof(*out->x14));
    for (i = 0; i < n+demo; ++i) {
        Fighter_WaitAnimData* row = i < n ? &out->xC[i] : &out->x14[i-n];
        u32 flags = row->x10_animCurrFlags;
        /* Paired thrown clips keep the generic victim skeleton author id. */
        if ((flags & 0x3f) != 0x21)
            row->x10_animCurrFlags = (flags & ~0x3fu) | (kind < 64 ? kind : FT_ANIM_KIND_SELF);
    }
    if (Geno_DefineIsNone(kind)) GenoDefine_BuildOwn(kind, out, n);
    GenoDefine_CommonRows[kind] = gd_data_copy(ftData_MotionStateList, sizeof(MotionState)*ftCo_MS_Count);
    GenoDefine_SpecificRows[kind] = gd_data_copy(ftData_CharacterStateTables[base], sizeof(MotionState)*ftMr_MS_SelfCount);
    GenoGame_DefineApplyRows(kind, GenoDefine_CommonRows[kind], GenoDefine_SpecificRows[kind], n);
    ftData_CharacterStateTables[kind] = GenoDefine_SpecificRows[kind];
    gFtDataList[kind] = out;
    OSReport("geno: native define kind %d loaded: %s; independent common/special rows; interpreter attempts %u\n",
             kind, Geno_DefineIsNone(kind) ? "base none (own model, bank, parts, joint fields; Mario's script words, callbacks and attribute defaults are still the template)" : "retail:mario",
             GenoDefine_InterpreterAttempts[kind]);
    {
        extern int Geno_ProfilePoolWords(int p);
        extern int Geno_PoolWords(void);
        extern int Geno_ProfileForKind(int kind);
        extern int Geno_OverlayCount(int p);
        extern int Geno_StateCount(int p);
        extern unsigned Geno_ProfileStorageBytes(void);
        int p = Geno_ProfileForKind(kind);
        OSReport("geno: define budgets: kind %d overlays %d/%d, pool %d words (all profiles %d/%d), states %d/%d, arena %u/%u\n",
                 kind, Geno_OverlayCount(p), GENO_MAX_OVERLAYS, Geno_ProfilePoolWords(p), Geno_PoolWords(), GENO_POOL_WORDS,
                 Geno_StateCount(p), GENO_MAX_STATES, Geno_ProfileStorageBytes(), 2u * 1024u * 1024u /* GENO_PROFILE_STORAGE_BYTES */);
    }
}

void GenoDefine_BindFighter(Fighter* fp)
{
    if (Geno_DefineBaseKind(fp->kind) < 0) return;
    fp->x1C_actionStateList = GenoDefine_CommonRows[fp->kind];
    fp->x20_actionStateList = GenoDefine_SpecificRows[fp->kind];
    OSReport("geno: native define kind %d spawn: interpreter attempts %u\n", fp->kind, GenoDefine_InterpreterAttempts[fp->kind]);
}

/* Called only from the interpreter entry probe, so a nonzero value means
   actual attempted guest execution, not normal native m-ex fallback dispatch. */
void GenoGame_DefinePPCViolation(int kind)
{
    if (kind >= 0 && kind < Ft_Kind_Max) ++GenoDefine_InterpreterAttempts[kind];
}
