/* LINK: src/sysdolphin/baselib/gobj.c src/sysdolphin/baselib/gobjinit.c src/sysdolphin/baselib/gobjplink.c src/sysdolphin/baselib/gobjgxlink.c src/sysdolphin/baselib/gobjproc.c src/sysdolphin/baselib/gobjobject.c src/sysdolphin/baselib/gobjuserdata.c */
/* PLAIN: rust/melee_runtime/gen/pure/stubs_gobj_native.c */
/* State oracle for melee_runtime::tu::gobj_native: the real gobj*.c (shipping contraction) driven by a
 * random scenario of library calls over a pool of GObjs; after every operation the whole library
 * state (process links, render links, proc queues, the globals, every live GObj / proc) is hashed
 * with pointers replaced by creation-order ids. One line per case: case hash-chain. */
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/gobjplink.h>
#include <sysdolphin/baselib/gobjgxlink.h>
#include <sysdolphin/baselib/gobjproc.h>
#include <sysdolphin/baselib/gobjobject.h>
#include <sysdolphin/baselib/gobjuserdata.h>
#include "common.h"
extern int printf(const char*, ...);
extern int fflush(void*);
extern void* calloc(unsigned, unsigned);
extern char* getenv(const char*);
extern int atoi(const char*);

extern void GObj_RemoveUserData(HSD_GObj*);
extern HSD_GObj* CreateGObj(s32, u16, u8, u8, HSD_GObj*);
extern void HSD_GObjPLink_ChangeGObjPri_Unk(u32, HSD_GObj*, u8, u8, HSD_GObj*);
extern void GObj_SetupGXLinkMaxSorted(HSD_GObj*, GObj_RenderFunc, u32);
extern void HSD_GObjGXLink_8039084C(HSD_GObj*);
extern void HSD_GObjGXLink_80390908(HSD_GObj*, u8, u8);
extern void HSD_GObjGXLink_803909D8(HSD_GObj*, HSD_GObj*);
extern void HSD_GObjProc_UnqueueProc(HSD_GObjProc*);
extern void HSD_GObjProc_UnlinkProcFromGObj(HSD_GObjProc*);
extern void* HSD_GObjObject_80390ADC(HSD_GObj*);
extern HSD_GObj* HSD_GObjObject_80390A3C(u16, u8);

#define MAXG 24
#define MAXP 48
static HSD_GObj* GP[MAXG]; static int GA[MAXG]; static int ng;
static HSD_GObjProc* PP[MAXP]; static int PA[MAXP]; static int np;
static u64_ H;
static int gscript;
static int dumpw;
static void hw(unsigned w) { if (dumpw) printf("w %u\n", w); H = fnv(&w, 4, H); }
static int gid(HSD_GObj* g) { int i; if (!g) return -1; for (i = ng - 1; i >= 0; i--) if (GP[i] == g && GA[i]) return i; return -2; }
static int pid(HSD_GObjProc* p) { int i; if (!p) return -1; for (i = np - 1; i >= 0; i--) if (PP[i] == p && PA[i]) return i; return -2; }
static void reg_g(HSD_GObj* g) { if (ng < MAXG) { GP[ng] = g; GA[ng] = 1; ng++; } }
static void reg_p(HSD_GObjProc* p) { if (np < MAXP) { PP[np] = p; PA[np] = 1; np++; } }
static void kill_g(HSD_GObj* g) { int i = gid(g); if (i >= 0) GA[i] = 0; }
static void kill_p(HSD_GObjProc* p) { int i = pid(p); if (i >= 0) PA[i] = 0; }

/* ---- callbacks: each folds a marker into the hash and may use the library on its own GObj ---- */
static unsigned trace;
static int dumpt;
static void tr(unsigned m) { if (dumpt) printf("t %u\n", m); trace = trace * 16777619u ^ m; }
static void cb_nop(HSD_GObj* g) { tr(1000u + (unsigned) (gid(g) + 2)); }
static void cb_free(HSD_GObj* g) { tr(2000u + (unsigned) (gid(g) + 2)); HSD_GObjFree(g); }
static void cb_rmproc(HSD_GObj* g) { tr(3000u + (unsigned) (gid(g) + 2)); if (g->proc) HSD_GObjProc_RemoveProc(g->proc); }
static void cb_chpri(HSD_GObj* g) { tr(4000u + (unsigned) (gid(g) + 2)); HSD_GObjPLink_ChangeGObjPri_Unk(0, g, (u8) ((g->p_link + 1) % 4), 3, NULL); }
static void cb_addproc(HSD_GObj* g) { tr(5000u + (unsigned) (gid(g) + 2)); HSD_GObj_SetupProc(g, cb_nop, 1); }
static void (*CBS[5])(HSD_GObj*) = { cb_nop, cb_free, cb_rmproc, cb_chpri, cb_addproc };
static void render_a(HSD_GObj* g, int pass) { tr(6000u + (unsigned) (gid(g) + 2) * 8u + (unsigned) pass); }
static void render_b(HSD_GObj* g, int pass) { tr(7000u + (unsigned) (gid(g) + 2) * 8u + (unsigned) pass); }
static void ud_remove(void* d) { tr(8000u + (unsigned) (*(int*) d)); }
static void kind0_free(HSD_Obj* o) { tr(9000u + (unsigned) (*(int*) o)); }
static void kind1_free(HSD_Obj* o) { tr(9100u + (unsigned) (*(int*) o)); }
static GObjFunc kfuncs[2] = { kind0_free, kind1_free };
static GObjFuncs kfs = { 0, 2, kfuncs };
static int objs[4] = { 11, 22, 33, 44 };

int ScriptGame_StageSlotCanCreate(void) { return 1; }
void ScriptGame_StageSlotCreated(HSD_GObj* g) { (void) g; }
void ScriptGame_StageSlotDestroyed(HSD_GObj* g) { (void) g; }
int ScriptGame_StageSlotProcBegin(HSD_GObj* g) { (void) g; return 0; }
void ScriptGame_StageSlotProcEnd(void) {}
int ScriptStageHostHidden;

static void state(void)
{
    int l, i;
    HSD_GObj* g; HSD_GObjProc* p;
    hw(HSD_GObj_804D783C); hw((unsigned) HSD_GObj_DelayedProcInfo.flags);
    hw((unsigned) pid(HSD_GObj_CurrentInvokedProc)); hw((unsigned) gid(HSD_GObj_CurrentInvokedProcGObj));
    hw((unsigned) pid(HSD_GObj_NextInvokedProc)); hw((unsigned) HSD_GObj_CurrentInvokedSLink);
    for (l = 0; l <= HSD_GObjLibInitData.p_link_max; l++) {
        hw(0xA0000000u + l);
        for (g = HSD_GObjPLinkHead[l]; g; g = g->next) hw((unsigned) gid(g));
        hw((unsigned) gid(plinklow_gobjs[l]));
        for (i = 0; i <= HSD_GObjLibInitData.gproc_pri_max; i++)
            hw((unsigned) pid(HSD_GObj_ProcList[l + i * (HSD_GObjLibInitData.p_link_max + 1)]));
    }
    for (l = 0; l <= HSD_GObjLibInitData.gx_link_max + 1; l++) {
        hw(0xB0000000u + l);
        for (g = HSD_GObjGXLinkHead[l]; g; g = g->next_gx) hw((unsigned) gid(g));
        hw((unsigned) gid(HSD_GObj_804D7820[l]));
    }
    for (i = 0; i <= HSD_GObjLibInitData.gproc_pri_max; i++) {
        hw(0xC0000000u + i);
        for (p = HSD_GObj_GObjProcHead[i]; p; p = p->next) { hw((unsigned) pid(p)); hw((unsigned) pid(p->prev)); }
    }
    for (i = 0; i < ng; i++) {
        if (!GA[i]) continue;
        g = GP[i];
        hw(g->classifier); hw(g->p_link); hw(g->gx_link); hw(g->p_priority); hw(g->render_priority);
        hw(g->obj_kind); hw(g->user_data_kind);
        hw((unsigned) gid(g->next)); hw((unsigned) gid(g->prev)); hw((unsigned) gid(g->next_gx)); hw((unsigned) gid(g->prev_gx));
        hw((unsigned) pid(g->proc)); hw(g->render_cb == render_a ? 1 : g->render_cb == render_b ? 2 : g->render_cb ? 3 : 0);
        hw((unsigned) g->gxlink_prios); hw((unsigned) (g->gxlink_prios >> 32));
        hw(g->hsd_obj ? (unsigned) *(int*) g->hsd_obj : 0); hw(g->user_data ? (unsigned) *(int*) g->user_data : 0);
        hw(g->user_data_remove_func ? 1 : 0);
    }
    for (i = 0; i < np; i++) {
        if (!PA[i]) continue;
        p = PP[i];
        hw((unsigned) pid(p->child)); hw((unsigned) pid(p->next)); hw((unsigned) pid(p->prev));
        hw(p->s_link); hw(p->flags_1); hw(p->flags_2); hw(p->flags_3); hw((unsigned) gid(p->gobj));
        { int c; for (c = 0; c < 5; c++) if (p->on_invoke == CBS[c]) hw(100u + c); }
    }
}

static int pick_g(void) { int i, k = 0, ids[MAXG]; for (i = 0; i < ng; i++) if (GA[i]) ids[k++] = i; return k ? ids[rnd() % (unsigned) k] : -1; }
static int pick_gx(void) { int i, k = 0, ids[MAXG]; for (i = 0; i < ng; i++) if (GA[i] && GP[i]->gx_link != 0xFF) ids[k++] = i; return k ? ids[rnd() % (unsigned) k] : -1; }
static int pick_gnx(void) { int i, k = 0, ids[MAXG]; for (i = 0; i < ng; i++) if (GA[i] && GP[i]->gx_link == 0xFF) ids[k++] = i; return k ? ids[rnd() % (unsigned) k] : -1; }
static int pick_same_link(unsigned link) { int i, k = 0, ids[MAXG]; for (i = 0; i < ng; i++) if (GA[i] && GP[i]->p_link == link) ids[k++] = i; return k ? ids[rnd() % (unsigned) k] : -1; }
static int pick_p(void) { int i, k = 0, ids[MAXP]; for (i = 0; i < np; i++) if (PA[i] && gid(PP[i]->gobj) >= 0) ids[k++] = i; return k ? ids[rnd() % (unsigned) k] : -1; }

int main(void)
{
    int k, step;
    int dbg = getenv("GOBJ_DBG") != 0;
    for (k = 0; k < 300; k++) {
        HSD_GObjLibInitDataType init;
        u64_ chain = FNV0;
        ng = np = 0; trace = 2166136261u;
        HSD_GObjSetInitDefaults(&init);
        init.p_link_max = 3; init.gx_link_max = 3; init.gproc_pri_max = 2;
        init.funcs = &kfs; kfs.next = 0;
        if (dbg) printf("init %d\n", k);
        HSD_GObjInit(&init);
        if (dbg) printf("inited\n");
        for (step = 0; step < 60; step++) {
            int op = (int) (rnd() % 26), a, b;
            u32 r = (u32) rnd();
            H = FNV0;
            dumpt = (getenv("GOBJ_DUMP") && atoi(getenv("GOBJ_DUMP")) == k * 100 + step);
            if (dbg) { printf("step %d op %d\n", step, op); fflush(0); }
            switch (op) {
            case 0: case 1: case 2: {
                HSD_GObj* g;
                if (ng >= MAXG) break;
                g = GObj_Create((u16) (r % 5), (u8) ((r >> 4) % 4), (u8) ((r >> 8) % 6));
                reg_g(g); break;
            }
            case 3: {
                HSD_GObj* g; int where = (int) (r % 4), pos = -1;
                u8 pl = (u8) ((r >> 4) % 4);
                if (ng >= MAXG) break;
                if (where >= 2) { pos = pick_same_link(pl); if (pos < 0) break; }
                g = CreateGObj(where, (u16) (r >> 8) % 5, pl, (u8) ((r >> 12) % 6), pos < 0 ? NULL : GP[pos]);
                reg_g(g); break;
            }
            case 4: a = pick_gnx(); if (a < 0) break;
                GObj_SetupGXLink(GP[a], (r & 1) ? render_a : render_b, (u8) ((r >> 4) % 4), (r >> 8) % 6); break;
            case 5: a = pick_gnx(); if (a < 0) break;
                GObj_SetupGXLinkMax(GP[a], (r & 1) ? render_a : render_b, (r >> 8) % 6); break;
            case 6: a = pick_gnx(); if (a < 0) break;
                GObj_SetupGXLinkMaxSorted(GP[a], (r & 1) ? render_a : render_b, (r >> 8) % 6); break;
            case 7: a = pick_gx(); if (a < 0) break; HSD_GObjGXLink_8039084C(GP[a]); break;
            case 8: a = pick_gx(); if (a < 0) break; HSD_GObjGXLink_80390908(GP[a], (u8) (r % 4), (u8) ((r >> 4) % 6)); break;
            case 9: a = pick_gx(); b = pick_gx(); if (a < 0 || b < 0 || a == b) break; HSD_GObjGXLink_803909D8(GP[a], GP[b]); break;
            case 10: case 11: case 12: {
                HSD_GObjProc* p;
                a = pick_g(); if (a < 0 || np >= MAXP) break;
                p = HSD_GObj_SetupProc(GP[a], CBS[r % 5], (u8) ((r >> 4) % 3));
                reg_p(p); break;
            }
            case 13: b = pick_p(); if (b < 0) break;
                { HSD_GObjProc* p = PP[b]; HSD_GObjProc_RemoveProc(p); if (!(HSD_GObj_DelayedProcInfo.flags & 4)) kill_p(p); } break;
            case 14: a = pick_g(); if (a < 0) break;
                { HSD_GObjProc* p = GP[a]->proc; int ids[MAXP], c = 0; while (p) { ids[c++] = pid(p); p = p->child; if (c >= MAXP) break; }
                  HSD_GObjProc_RemoveAllProcs(GP[a]); { int i; for (i = 0; i < c; i++) if (ids[i] >= 0 && !(HSD_GObj_DelayedProcInfo.flags & 4)) PA[ids[i]] = 0; } } break;
            case 15: {
                HSD_GObj* pos = NULL;
                a = pick_g(); if (a < 0) break;
                if ((r % 4) >= 2) {
                    b = pick_same_link((r >> 4) % 4);
                    if (b < 0 || b == a) break;
                    pos = GP[b];
                }
                HSD_GObjPLink_ChangeGObjPri_Unk(r % 4, GP[a], (u8) ((r >> 4) % 4), (u8) ((r >> 8) % 6), pos);
                break;
            }
            case 16: a = pick_g(); if (a < 0 || GP[a]->user_data_kind != 0xFF) break;
                GObj_InitUserData(GP[a], (u8) (r % 3), ud_remove, &objs[r % 4]); break;
            case 17: a = pick_g(); if (a < 0) break; GObj_RemoveUserData(GP[a]); break;
            case 18: a = pick_g(); if (a < 0 || GP[a]->obj_kind != 0xFF) break;
                HSD_GObjObject_80390A70(GP[a], (u8) (r % 2), &objs[(r >> 4) % 4]); break;
            case 19: a = pick_g(); if (a < 0) break;
                { void* o = HSD_GObjObject_80390ADC(GP[a]); hw(o ? (unsigned) *(int*) o : 0xFFFF); } break;
            case 20: a = pick_g(); if (a < 0) break;
                if (HSD_GObj_DelayedProcInfo.flags == 0) {
                    HSD_GObjProc* p = GP[a]->proc; int ids[MAXP], c = 0, i;
                    while (p && c < MAXP) { ids[c++] = pid(p); p = p->child; }
                    HSD_GObjFree(GP[a]); kill_g(GP[a]);
                    for (i = 0; i < c; i++) if (ids[i] >= 0) PA[ids[i]] = 0;
                } break;
            case 21: a = pick_g(); if (a < 0) break;
                { HSD_GObj* f = HSD_GObjObject_80390A3C((u16) (r % 5), (u8) ((r >> 4) % 4)); hw((unsigned) gid(f)); } break;
            case 22: a = pick_g(); if (a < 0) break;
                switch (r % 4) { case 0: HSD_GObj_80390C5C(GP[a]); break; case 1: HSD_GObj_80390C84(GP[a]); break;
                                 case 2: HSD_GObj_80390CAC(GP[a]); break; default: HSD_GObj_80390CD4(GP[a]); }
                break;
            case 24: a = pick_g(); if (a < 0) break;
                GP[a]->gxlink_prios = rnd() & 0x1F;
                HSD_GObj_80390ED0(GP[a], (u32) (r & 7));
                hw((unsigned) gid(HSD_GObj_804D7814)); break;
            case 25: HSD_GObj_80390FC0(); hw((unsigned) gid(HSD_GObj_804D7818)); break;
            case 23: {
                int i;
                HSD_GObj_RunProcs();
                for (i = 0; i < ng; i++) if (GA[i] && !0) { /* freed by a callback: drop from the registry */
                    HSD_GObj* g = GP[i]; int l = 0, found = 0; HSD_GObj* q;
                    for (l = 0; l <= HSD_GObjLibInitData.p_link_max && !found; l++) for (q = HSD_GObjPLinkHead[l]; q; q = q->next) if (q == g) { found = 1; break; }
                    if (!found) GA[i] = 0;
                }
                for (i = 0; i < np; i++) if (PA[i]) {
                    int alive = 0, q; HSD_GObjProc* t;
                    for (q = 0; q <= HSD_GObjLibInitData.gproc_pri_max && !alive; q++) for (t = HSD_GObj_GObjProcHead[q]; t; t = t->next) if (t == PP[i]) { alive = 1; break; }
                    if (!alive) PA[i] = 0;
                }
                /* procs registered by callbacks (cb_addproc) are not in the registry: add them */
                {
                    int q; HSD_GObjProc* t;
                    for (q = 0; q <= HSD_GObjLibInitData.gproc_pri_max; q++) for (t = HSD_GObj_GObjProcHead[q]; t; t = t->next) if (pid(t) == -2 && np < MAXP) reg_p(t);
                }
                break;
            }
            }
            dumpw = (getenv("GOBJ_DUMP") && atoi(getenv("GOBJ_DUMP")) == k * 100 + step);
            hw(trace);
            state();
            dumpw = 0;
            chain = fnv(&H, 8, chain);
            if (dbg) printf("dbg %d %d op%d %016llx\n", k, step, op, H);
        }
        printf("gobj %d %016llx\n", k, chain);
    }
    return 0;
}
