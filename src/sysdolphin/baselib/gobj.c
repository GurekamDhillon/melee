#include <gameworld/profiler_game.h>
#include "gobj.h"

#include "fog.h"
#include "gobjplink.h"
#include "gobjproc.h"
#include "lobj.h"

#if defined(TARGET_PC)
#include <dolphin/os.h>
#include "random.h"
extern int GObjLogEnabled(void);
extern int ScriptStageHostHidden;

static void gobj_trace_render(const char* walk, HSD_GObj* gobj, GObj_RenderFunc cb)
{
    if (GObjLogEnabled()) {
        OSReport("gobj: %s cb=%p gobj=%p class=%u owner=%p plink=%u gx=%u obj=%u\n",
                 walk, cb, gobj, gobj->classifier, gobj->user_data, gobj->p_link,
                 gobj->gx_link, gobj->obj_kind);
    }
}
#endif


HSD_GObjProc** HSD_GObj_ProcList;
HSD_GObjProc** HSD_GObj_GObjProcHead;
s32 HSD_GObj_804D783C;
HSD_GObjProc* HSD_GObj_CurrentInvokedProc;
s32 HSD_GObj_CurrentInvokedSLink;
HSD_GObjProc* HSD_GObj_NextInvokedProc;
HSD_GObj** HSD_GObjPLinkHead;
HSD_GObj** plinklow_gobjs;
HSD_GObj** HSD_GObjGXLinkHead;
HSD_GObj** HSD_GObj_804D7820;
HSD_GObj* HSD_GObj_CurrentInvokedProcGObj;
HSD_GObj* HSD_GObj_804D7818;
HSD_GObj* HSD_GObj_804D7814;
GObjFunc* HSD_GObj_804D7810;

int HSD_GObj_804085F0[] = { 1, 4, 2, 0 };

static inline void GObj_SetFlag1_inline(HSD_GObjProc* proc, u8 value)
{
    while (proc != NULL) {
        proc->flags_1 = value;
        proc = proc->child;
    }
}

static inline void GObj_SetFlag2_inline(HSD_GObjProc* proc, u8 value)
{
    while (proc != NULL) {
        proc->flags_2 = value;
        proc = proc->child;
    }
}

void HSD_GObj_80390C5C(HSD_GObj* gobj)
{
    GObj_SetFlag1_inline(gobj->proc, 1);
}

void HSD_GObj_80390C84(HSD_GObj* gobj)
{
    GObj_SetFlag1_inline(gobj->proc, 0);
}

void HSD_GObj_80390CAC(HSD_GObj* gobj)
{
    GObj_SetFlag2_inline(gobj->proc, 0);
}

void HSD_GObj_80390CD4(HSD_GObj* gobj)
{
    HSD_GObjProc* p = gobj->proc;

    while (p != NULL) {
        p->flags_3 = HSD_GObj_804D783C;
        p = p->child;
    }
}

/// GObj_RunProcs
void HSD_GObj_RunProcs(void)
{
    PC_PROF_BEGIN(GW_PROF_LOGIC, 0);
#if defined(TARGET_PC)
    if (ProfEnabled()) {
        extern unsigned lbHeap_Free(int heap);
        /* Per logic frame; admission heaps from the six-slot source audit. */
        ProfCounter(GW_PROF_HEAP0_FREE, lbHeap_Free(0));
        ProfCounter(GW_PROF_HEAP3_FREE, lbHeap_Free(3));
        ProfCounter(GW_PROF_HEAP4_FREE, lbHeap_Free(4));
        ProfCounter(GW_PROF_HEAP5_FREE, lbHeap_Free(5));
    }
#endif
    s32 i;
    HSD_GObjProc* proc;
    HSD_GObj* gobj;

    u64 var_r31 =
        HSD_GObjLibInitData.unk_2 != NULL ? *HSD_GObjLibInitData.unk_2 : 0;
    HSD_GObj_804D783C += 1;
    if (HSD_GObj_804D783C > 2) {
        HSD_GObj_804D783C = 0;
    }

    for (i = 0; i <= HSD_GObjLibInitData.gproc_pri_max; i++) {
        HSD_GObj_CurrentInvokedSLink = i;
        proc = HSD_GObj_GObjProcHead[i];
        while (proc != NULL) {
            HSD_GObj_NextInvokedProc = proc->next;
            if (proc->flags_3 != HSD_GObj_804D783C) {
                proc->flags_3 = HSD_GObj_804D783C;
                gobj = proc->gobj;
                if (!(var_r31 & (1LL << gobj->p_link)) && !(proc->flags_1) &&
                    !(proc->flags_2))
                {
                    HSD_GObj_CurrentInvokedProcGObj = gobj;
                    HSD_GObj_CurrentInvokedProc = proc;
#if defined(TARGET_PC)
                    {
                        int zone = GW_PROF_OBJECT_CALLBACK;
                        int detail = (gobj->classifier << 16) | gobj->p_link;
                        switch (gobj->classifier) {
                        case 4: zone = GW_PROF_FIGHTER; break;
                        case 6: case 7:
                            zone = GW_PROF_ITEMS;
                            if (ProfEnabled()) {
                                extern int ScriptGame_ProfEnemy(HSD_GObj*);
                                if (ScriptGame_ProfEnemy(gobj)) zone = GW_PROF_ENEMIES;
                            }
                            break;
                        case 3: zone = GW_PROF_STAGE; break;
                        case 8: zone = GW_PROF_PARTICLES; break;
                        case 14: zone = GW_PROF_HUD; break;
                        case 19: zone = GW_PROF_CAMERA; break;
                        }
                        PC_PROF_BEGIN(zone, detail);
                        {
                            extern int ScriptGame_StageSlotProcBegin(HSD_GObj*);
                            extern void ScriptGame_StageSlotProcEnd(void);
                            int owned = ScriptGame_StageSlotProcBegin(gobj);
                            if (!ScriptStageHostHidden || gobj->classifier != 3 || owned)
                                proc->on_invoke(proc->gobj);
                            if (owned) ScriptGame_StageSlotProcEnd();
                        }
                        PC_PROF_END();
                    }
#else
                    proc->on_invoke(proc->gobj);
#endif
                    HSD_GObj_NextInvokedProc = proc->next;
                    if (HSD_GObj_DelayedProcInfo.x0.flags != 0) {
                        HSD_GObj_DelayedProcInfo.x0.x0.in_delayed_proc = 1;
                        if (HSD_GObj_DelayedProcInfo.x0.x0.delay_remove_gobj) {
                            HSD_GObjFree(proc->gobj);
                        } else {
                            if (HSD_GObj_DelayedProcInfo.x0.x0
                                    .delay_change_gobj_pri)
                            {
                                HSD_GObjPLink_ChangeGObjPri_Unk(
                                    HSD_GObj_DelayedProcInfo.type, proc->gobj,
                                    HSD_GObj_DelayedProcInfo.p_link,
                                    HSD_GObj_DelayedProcInfo.p_prio,
                                    HSD_GObj_DelayedProcInfo.gobj);
                            }
                            if (HSD_GObj_DelayedProcInfo.x0.x0
                                    .delay_remove_proc)
                            {
                                HSD_GObjProc_RemoveProc(proc);
                            }
                        }
                        HSD_GObj_DelayedProcInfo.x0.flags = 0;
                    }
                    HSD_GObj_CurrentInvokedProcGObj = NULL;
                    HSD_GObj_CurrentInvokedProc = NULL;
                }
            }
            proc = HSD_GObj_NextInvokedProc;
        }
    }

    PC_PROF_END();
}

/// GObj_GetFlagFromArray
u32 HSD_GObj_80390EB8(s32 i)
{
    return HSD_GObj_804085F0[i];
}

static inline void render_gobj(HSD_GObj* cur, int i)
{
    HSD_GObj* saved = HSD_GObj_804D7814;
    HSD_GObj_804D7814 = cur;
    cur->render_cb(cur, i);
    HSD_GObj_804D7814 = saved;
}

/// GObj_SetTextureCamera
void HSD_GObj_80390ED0(HSD_GObj* gobj, u32 mask)
{
    PC_PROF_BEGIN(GW_PROF_DRAW, 0);
    s32 i = 0;
#if defined(TARGET_PC)
    HSD_RandRenderBegin();
#endif
    while (mask) {
        if (mask & 1) {
            u64 prios = gobj->gxlink_prios;
            s32 j = 0;
            while (prios) {
                if (prios & 1) {
                    HSD_GObj* cur;
                    for (cur = HSD_GObjGXLinkHead[j]; cur != NULL;
                         cur = cur->next_gx)
                    {
#if defined(TARGET_PC)
                        gobj_trace_render("camera", cur, cur->render_cb);
#endif
                        if (cur->render_cb != NULL) {
                            render_gobj(cur, i);
                        }
                    }
                }
                j++;
                prios >>= 1;
            }
        }
        i++;
        mask >>= 1;
    }

#if defined(TARGET_PC)
    HSD_RandRenderEnd();
#endif
    PC_PROF_END();
}

/// GObj_RunGXLinkMaxCallbacks
void HSD_GObj_80390FC0(void)
{
    PC_PROF_BEGIN(GW_PROF_DRAW, 0);
    HSD_GObj* saved;
    HSD_GObj* cur = HSD_GObjGXLinkHead[HSD_GObjLibInitData.gx_link_max + 1];
#if defined(TARGET_PC)
    HSD_RandRenderBegin();
#endif
    while (cur != NULL) {
#if defined(TARGET_PC)
        /* The Classic step-7 splash installs max-link render objects. Use the
         * callback tested at the start of this iteration: a second load at the
         * call site called address 0 in the archived IntroEasy frame trace. */
        GObj_RenderFunc render_cb = cur->render_cb;
        gobj_trace_render("max", cur, render_cb);
        if (render_cb != NULL) {
            saved = HSD_GObj_804D7818;
            HSD_GObj_804D7818 = cur;
            {
                extern void Snap_CbTime(void* cb, int begin);
                extern int Snap_SkipRenderCb(void* cb);
                if (!Snap_SkipRenderCb((void*) render_cb)) {
                    Snap_CbTime((void*) render_cb, 1);
                    render_cb(cur, 0);
                    Snap_CbTime((void*) render_cb, 0);
                }
            }
            HSD_GObj_804D7818 = saved;
        }
#else
        if (cur->render_cb != NULL) {
            saved = HSD_GObj_804D7818;
            HSD_GObj_804D7818 = cur;
            cur->render_cb(cur, 0);
            HSD_GObj_804D7818 = saved;
        }
#endif
        cur = cur->next_gx;
    }

#if defined(TARGET_PC)
    HSD_RandRenderEnd();
#endif
    PC_PROF_END();
}

struct _unk_gobj_struct HSD_GObj_DelayedProcInfo;
HSD_ObjAllocData gobjproc_alloc_data;
HSD_ObjAllocData gobj_alloc_data;
HSD_GObjLibInitDataType HSD_GObjLibInitData;
