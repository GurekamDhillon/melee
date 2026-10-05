/* Native Mario-reference backend, included in ftdata.c after its tables.
 * Author owns independent effective attributes/animation/state rows. Shared
 * retail model/clip/effect/audio descriptors are immutable asset references.
 * No MxDt row or ftFunction archive is constructed or installed for this kind.
 */
#include "geno_define_game.h"
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
        /* Do not invoke Mario's global init twice under a new id. */
        ftData_Table_Unk1[fk] = NULL;
        ftData_Table_Unk0[fk].count = ftData_Table_Unk0[base].count;
        ftData_UnkIntPairs[fk].count = ftData_UnkIntPairs[base].count;
        CostumeListsForeachCharacter[fk].costume_list = GenoDefine_Costumes[fk-Ft_Kind_Mex0];
        CostumeListsForeachCharacter[fk].numCostumes = CostumeListsForeachCharacter[base].numCostumes;
        Player_MexSetMapping(ck, fk);
        GenoDefine_ContentIdentity[fk][0] = Geno_DefineIdentityWord(fk, 0);
        GenoDefine_ContentIdentity[fk][1] = Geno_DefineIdentityWord(fk, 1);
    }
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
    GenoDefine_CommonRows[kind] = gd_data_copy(ftData_MotionStateList, sizeof(MotionState)*ftCo_MS_Count);
    GenoDefine_SpecificRows[kind] = gd_data_copy(ftData_CharacterStateTables[base], sizeof(MotionState)*ftMr_MS_SelfCount);
    GenoGame_DefineApplyRows(kind, GenoDefine_CommonRows[kind], GenoDefine_SpecificRows[kind], n);
    ftData_CharacterStateTables[kind] = GenoDefine_SpecificRows[kind];
    gFtDataList[kind] = out;
    OSReport("geno: native define kind %d loaded: retail:mario; independent common/special rows; interpreter attempts %u\n",
             kind, GenoDefine_InterpreterAttempts[kind]);
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
