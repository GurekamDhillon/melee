#include <Runtime/platform.h>

#include <melee/lb/forward.h>
#include <sysdolphin/baselib/forward.h>

#include "ground.h"
#include "grzakogenerator.h"
#include "inlines.h"
#include "types.h"
#include <dolphin/mtx.h>
#include <sysdolphin/baselib/gobjproc.h>

/* 223864 */ static void grTSeak_OnDemoInit(bool);
/* 223868 */ static void grTSeak_OnInit(void);
/* 2238D8 */ static void grTseak_OnLoad(void);
/* 2238DC */ static void grTseak_OnStart(void);
/* 223900 */ static bool grTSeak_80223900(void);
#if defined(TARGET_PC)
/* 223908 */
/**
 * EXTERNAL ON PURPOSE - the m-ex custom-stage path does not work without it.
 *
 * This is the generic map-gobj creator that every m-ex custom stage's interpreted @c onInit
 * calls; m-ex standardised on it, which is why the only creator it patches is this one
 * (@c "SSS Expansion/grFunction References/Create_map_gobj/GrTSk.asm"). All 25 of Akaneia's
 * added stages call guest @c 0x80223908 and no other creator.
 *
 * The guest->native bridge invokes every target through @c gw_ppc_native_fn, which is cdecl:
 * arguments on the stack. That is correct for the gwtool-retargeted game functions, which are
 * external and get the platform C ABI. It is NOT correct for a function that is @c static in its
 * TU - the backend may give an internal function a private convention, and on i686 LLVM passes
 * the first argument in ECX. This one did; its prologue was @c "mov edi, ecx". So the bridge
 * pushed @c map_id on the stack, the callee read ECX, and all three calls arrived as 0:
 * Meta Crystal built map model 0 three times and never built 1 or 2, so two of its three stage
 * gobjs never reached @c GObj_SetupGXLink and the screen stayed black.
 *
 * External linkage forces the platform C ABI, which is what the bridge already assumes. Kept
 * under #TARGET_PC so the matching build is untouched.
 *
 * It is not the only one: @c tools/mex_port/audit_bridge_abi.py finds 31 such targets. The other
 * 30 are latent until something bridges to them.
 */
HSD_GObj* grTSeak_80223908(int);
#else
/* 223908 */ static HSD_GObj* grTSeak_80223908(int);
#endif
/* 2239F0 */ static void stageGObj0_OnInit(Ground_GObj*);
/* 223A1C */ static bool grTSeak_80223A1C(Ground_GObj*);
/* 223A24 */ static void grTSeak_80223A24(Ground_GObj*);
/* 223A28 */ static void grTSeak_80223A28(Ground_GObj*);
/* 223A2C */ static void stageGObj2_OnInit(Ground_GObj*);
/* 223A7C */ static bool grTSeak_80223A7C(Ground_GObj*);
/* 223A84 */ static void stageGObj2_GObjProc(Ground_GObj*);
/* 223AB8 */ static void grTSeak_80223AB8(Ground_GObj*);
/* 223ABC */ static void stageGObj1_OnInit(Ground_GObj*);
/* 223B0C */ static bool grTSeak_80223B0C(Ground_GObj*);
/* 223B14 */ static void stageGObj1_GObjProc(Ground_GObj*);
/* 223B34 */ static void grTSeak_80223B34(Ground_GObj*);
/* 223B38 */ static DynamicsDesc* grTSeak_OnTouchLine(enum_t);
/* 223B40 */ static bool grTSeak_OnCheckShadowRender(Vec3*, int, HSD_JObj*);

static StageCallbacks grTSk_StageCallbacks[] = {
    {
        stageGObj0_OnInit,
        grTSeak_80223A1C,
        grTSeak_80223A24,
        grTSeak_80223A28,
        0,
    },
    {
        stageGObj1_OnInit,
        grTSeak_80223B0C,
        stageGObj1_GObjProc,
        grTSeak_80223B34,
        0,
    },
    {
        stageGObj2_OnInit,
        grTSeak_80223A7C,
        stageGObj2_GObjProc,
        grTSeak_80223AB8,
        (1 << 31) | (1 << 30),
    },
    { 0 },
};

StageData grTSk_StageData = {
    Gr_Kind_TSeak,
    grTSk_StageCallbacks,
    "/GrTSk.dat",
    grTSeak_OnInit,
    grTSeak_OnDemoInit,
    grTseak_OnLoad,
    grTseak_OnStart,
    grTSeak_80223900,
    grTSeak_OnTouchLine,
    grTSeak_OnCheckShadowRender,
    1,
};

void grTSeak_OnDemoInit(bool unk0) {}

void grTSeak_OnInit(void)
{
    Ground_InitTargetStage(grTSeak_80223908);
}

void grTseak_OnLoad(void) {}

void grTseak_OnStart(void)
{
    grZakoGenerator_801CAE04(NULL);
}

bool grTSeak_80223900(void)
{
    return false;
}

HSD_GObj* grTSeak_80223908(int arg0)
{
    HSD_GObj* gobj;
    StageCallbacks* callbacks = &grTSk_StageCallbacks[arg0];
#if defined(TARGET_PC)
    /* Every m-ex custom stage's onInit calls THIS creator (it is the one m-ex repoints at
     * `Get grFunction`), so the table it indexes must be the running stage's, not grTSk's.
     * Mex_GrCallbacks returns NULL for a vanilla stage, which keeps the line above. */
    {
        extern void* Mex_GrCallbacks(void);
        extern int Mex_GrTrace(void);
        StageCallbacks* mex_cbs = (StageCallbacks*) Mex_GrCallbacks();
        if (mex_cbs != NULL) {
            callbacks = &mex_cbs[arg0];
        }
        if (Mex_GrTrace()) {
            OSReport("grtrace: create_map_gobj(%d): mex_cbs=%p callbacks=%p "
                     "on_init=%p proc=%p cb3=%p flags=0x%08X\n",
                     arg0, mex_cbs, callbacks, callbacks->on_init,
                     callbacks->gobj_proc, callbacks->callback3, callbacks->flags);
        }
    }
#endif

    gobj = Ground_GetStageGObj(arg0);
#if defined(TARGET_PC)
    {
        extern int Mex_GrTrace(void);
        if (Mex_GrTrace()) {
            OSReport("grtrace: create_map_gobj(%d): gobj=%p\n", arg0, gobj);
        }
    }
#endif

    if (gobj != NULL) {
        Ground_SetupStageCallbacks(gobj, callbacks);
    } else {
        OSReport("%s:%d: couldn t get gobj(id=%d)\n", __FILE__, 0xC3, arg0);
    }

    return gobj;
}

static void stageGObj0_OnInit(Ground_GObj* gobj)
{
    Ground_StartMapAnim(gobj);
}

bool grTSeak_80223A1C(Ground_GObj* gobj)
{
    return false;
}

void grTSeak_80223A24(Ground_GObj* gobj) {}

void grTSeak_80223A28(Ground_GObj* gobj) {}

static void stageGObj2_OnInit(Ground_GObj* gobj)
{
    Ground_InitMapCollAndAnim(gobj);
}

bool grTSeak_80223A7C(Ground_GObj* gobj)
{
    return false;
}

static void stageGObj2_GObjProc(Ground_GObj* gobj)
{
    Ground_UpdateWindAndMapColl(gobj);
}

void grTSeak_80223AB8(Ground_GObj* gobj) {}

static void stageGObj1_OnInit(Ground_GObj* gobj)
{
    Ground_InitMapCollAndAnim(gobj);
}

bool grTSeak_80223B0C(Ground_GObj* gobj)
{
    return false;
}

static void stageGObj1_GObjProc(Ground_GObj* gobj)
{
    Ground_UpdateMapColl(gobj);
}

void grTSeak_80223B34(Ground_GObj* gobj) {}

DynamicsDesc* grTSeak_OnTouchLine(enum_t arg0)
{
    return NULL;
}

bool grTSeak_OnCheckShadowRender(Vec3* arg0, int arg1, HSD_JObj* arg2)
{
    return true;
}
