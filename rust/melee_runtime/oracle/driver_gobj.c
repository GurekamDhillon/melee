/* Oracle driver for the baselib GObj system: gobj.c, gobjplink.c, gobjgxlink.c, gobjproc.c,
 * gobjobject.c, gobjuserdata.c, gobjinit.c (see build_gobj.sh).
 *
 * A seeded xorshift stream drives random operation sequences (create/free/reprioritise gobjs,
 * queue/remove procs, GX link moves, flag ops, RunProcs with proc callbacks that act on the
 * running gobj, render walks, object/user-data attach/detach). tests/oracle_gobj.rs runs the
 * same generator against melee_runtime::gobj. Every callback writes a log line, and after each
 * operation the whole list state is dumped to a string whose FNV-1a 64 hash is written. Only
 * operations the C itself keeps memory-safe are generated (see the comments at each op).
 * Generated inputs only, no disc data. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
typedef unsigned u32; typedef int s32; typedef unsigned short u16; typedef unsigned char u8;
typedef unsigned long long u64;

typedef struct HSD_GObjProc HSD_GObjProc;
typedef struct HSD_GObj HSD_GObj;
typedef void (*RenderFn)(HSD_GObj*, int);
typedef void (*EventFn)(HSD_GObj*);
struct HSD_GObj {
    u16 classifier; u8 p_link, gx_link, p_priority, render_priority, obj_kind, user_data_kind;
    HSD_GObj *next, *prev, *next_gx, *prev_gx; HSD_GObjProc* proc; RenderFn render_cb;
    u64 gxlink_prios; void* hsd_obj; void* user_data; void (*user_data_remove_func)(void*);
    void* x34_unk;
};
struct HSD_GObjProc {
    HSD_GObjProc *child, *next, *prev; u8 s_link;
    u8 flags_1 : 1; u8 flags_2 : 1; u8 flags_3 : 2; u8 flags_4 : 2;
    HSD_GObj* gobj; EventFn on_invoke;
};
typedef void (*GObjFunc)(void*);
typedef struct GObjFuncs { struct GObjFuncs* next; u8 size; GObjFunc* funcs; } GObjFuncs;
typedef struct { u8 p_link_max, gx_link_max, gproc_pri_max; GObjFuncs* funcs; u64* unk_2; } InitData;
struct Delayed {
    union { u32 flags; struct { u32 in_delayed_proc : 1, delay_remove_gobj : 1, delay_remove_proc : 1, delay_change_gobj_pri : 1; }; };
    u32 type; u8 p_link; u8 p_prio; HSD_GObj* gobj;
};

extern u8 HSD_GObj_CameraKind; extern signed char HSD_GObj_LightKind; extern u8 HSD_GObj_JObjKind;
extern signed char HSD_GObj_FogKind;
extern HSD_GObjProc** HSD_GObj_ProcList; extern HSD_GObjProc** HSD_GObj_GObjProcHead;
extern s32 HSD_GObj_804D783C; extern HSD_GObjProc* HSD_GObj_CurrentInvokedProc;
extern s32 HSD_GObj_CurrentInvokedSLink; extern HSD_GObjProc* HSD_GObj_NextInvokedProc;
extern HSD_GObj** HSD_GObjPLinkHead; extern HSD_GObj** plinklow_gobjs; extern HSD_GObj** HSD_GObjGXLinkHead;
extern HSD_GObj** HSD_GObj_804D7820; extern HSD_GObj* HSD_GObj_CurrentInvokedProcGObj;
extern HSD_GObj* HSD_GObj_804D7818; extern HSD_GObj* HSD_GObj_804D7814;
extern struct Delayed HSD_GObj_DelayedProcInfo; extern InitData HSD_GObjLibInitData;
extern char gobj_alloc_data[];

HSD_GObj* CreateGObj(s32, u16, u8, u8, HSD_GObj*);
void HSD_GObjFree(HSD_GObj*);
void HSD_GObjPLink_ChangeGObjPri_Unk(u32, HSD_GObj*, u8, u8, HSD_GObj*);
void GObj_SetupGXLink(HSD_GObj*, RenderFn, u8, u32);
void GObj_SetupGXLinkMax(HSD_GObj*, RenderFn, u32);
void GObj_SetupGXLinkMaxSorted(HSD_GObj*, RenderFn, u32);
void HSD_GObjGXLink_8039084C(HSD_GObj*);
void HSD_GObjGXLink_80390908(HSD_GObj*, u8, u8);
void HSD_GObjGXLink_803909D8(HSD_GObj*, HSD_GObj*);
HSD_GObjProc* HSD_GObj_SetupProc(HSD_GObj*, EventFn, u8);
void HSD_GObjProc_RemoveProc(HSD_GObjProc*);
void HSD_GObjProc_RemoveAllProcs(HSD_GObj*);
HSD_GObj* HSD_GObjObject_80390A3C(u16, u8);
void HSD_GObjObject_80390A70(HSD_GObj*, u8, void*);
void* HSD_GObjObject_80390ADC(HSD_GObj*);
void HSD_GObjObject_80390B0C(HSD_GObj*);
void GObj_InitUserData(HSD_GObj*, u8, void (*)(void*), void*);
void GObj_RemoveUserData(HSD_GObj*);
void HSD_GObj_80390C5C(HSD_GObj*); void HSD_GObj_80390C84(HSD_GObj*);
void HSD_GObj_80390CAC(HSD_GObj*); void HSD_GObj_80390CD4(HSD_GObj*);
void HSD_GObj_RunProcs(void); u32 HSD_GObj_80390EB8(s32);
void HSD_GObj_80390ED0(HSD_GObj*, u32); void HSD_GObj_80390FC0(void);
void HSD_GObjSetInitDefaults(InitData*); void HSD_GObjInit(InitData*);

/* ---- allocation serials (stubs_gobj.c allocates with a 16-byte header) ---- */
static u32 ser(void* p) { return p ? *(u32*) ((char*) p - 16) : 0; }

/* ---- output ---- */
static FILE* o;
static char dump[1 << 16]; static int dn;
static void dp(const char* f, ...);
#include <stdarg.h>
static void dp(const char* f, ...) { va_list a; va_start(a, f); dn += vsnprintf(dump + dn, sizeof dump - dn, f, a); va_end(a); }
static u64 fnv(const char* s, int n) { u64 h = 0xcbf29ce484222325ull; int i; for (i = 0; i < n; i++) { h ^= (u8) s[i]; h *= 0x100000001b3ull; } return h; }

static uint64_t st;
static uint64_t nx(void) { st ^= st << 13; st ^= st >> 7; st ^= st << 17; return st; }
static u32 rn(u32 n) { return (u32) (nx() >> 33) % n; }

static int PM, GM, SM;
#define MAXG 4096
static HSD_GObj* G[MAXG]; static int ng;
static u64 pause_mask;
extern unsigned drv_gobj_serial, drv_proc_serial;
void drv_rand_render(int b) { fprintf(o, b ? "rb\n" : "re\n"); }

/* the stubs call this when the C frees a gobj (ScriptGame_StageSlotDestroyed) */
void drv_destroyed(HSD_GObj* g)
{
    int i;
    for (i = 0; i < ng; i++) if (G[i] == g) { memmove(G + i, G + i + 1, (ng - i - 1) * sizeof *G); ng--; return; }
}

/* ---- callbacks ---- */
static void act(HSD_GObj* g, int depth);
static void pcb(int t, HSD_GObj* g) { fprintf(o, "i %d %u %u\n", t, ser(g), ser(HSD_GObj_CurrentInvokedProc)); act(g, 0); }
static void p0(HSD_GObj* g) { pcb(0, g); } static void p1(HSD_GObj* g) { pcb(1, g); }
static void p2(HSD_GObj* g) { pcb(2, g); } static void p3(HSD_GObj* g) { pcb(3, g); }
static EventFn PF[4] = { p0, p1, p2, p3 };
static void rcb(int t, HSD_GObj* g, int pass) { fprintf(o, "r %d %u %d %u %u\n", t, ser(g), pass, ser(HSD_GObj_804D7814), ser(HSD_GObj_804D7818)); }
static void r0(HSD_GObj* g, int p) { rcb(0, g, p); } static void r1(HSD_GObj* g, int p) { rcb(1, g, p); }
static RenderFn RF[2] = { r0, r1 };
static void u0(void* d) { fprintf(o, "u 0 %u\n", (u32) (uintptr_t) d); }
static void u1(void* d) { fprintf(o, "u 1 %u\n", (u32) (uintptr_t) d); }
static void (*UF[2])(void*) = { u0, u1 };
static void o0(void* d) { fprintf(o, "o 0 %u\n", (u32) (uintptr_t) d); }
static void o1(void* d) { fprintf(o, "o 1 %u\n", (u32) (uintptr_t) d); }
static GObjFunc OF[2] = { o0, o1 };
static GObjFuncs uf;
static int tok_p(EventFn f) { int i; for (i = 0; i < 4; i++) if (PF[i] == f) return i; return 9; }
static int tok_r(RenderFn f) { int i; if (!f) return 8; for (i = 0; i < 2; i++) if (RF[i] == f) return i; return 9; }
static int tok_u(void (*f)(void*)) { int i; if (!f) return 8; for (i = 0; i < 2; i++) if (UF[i] == f) return i; return 9; }

/* ---- state dump ---- */
static void do_dump(void)
{
    int l, s, i;
    HSD_GObj* g; HSD_GObjProc* p;
    dn = 0;
    for (l = 0; l <= PM; l++) {
        if (!HSD_GObjPLinkHead[l] && !plinklow_gobjs[l]) continue;
        dp("p%d:", l);
        for (g = HSD_GObjPLinkHead[l]; g; g = g->next) dp(" %u", ser(g));
        dp(" |");
        for (g = plinklow_gobjs[l]; g; g = g->prev) dp(" %u", ser(g));
        dp("\n");
    }
    for (l = 0; l <= GM + 1; l++) {
        if (!HSD_GObjGXLinkHead[l] && !HSD_GObj_804D7820[l]) continue;
        dp("x%d:", l);
        for (g = HSD_GObjGXLinkHead[l]; g; g = g->next_gx) dp(" %u", ser(g));
        dp(" |");
        for (g = HSD_GObj_804D7820[l]; g; g = g->prev_gx) dp(" %u", ser(g));
        dp("\n");
    }
    for (s = 0; s <= SM; s++) {
        dp("q%d:", s);
        for (p = HSD_GObj_GObjProcHead[s]; p; p = p->next)
            dp(" %u<%u:%u%u%u%u/%u/%d", ser(p), ser(p->prev), p->flags_1, p->flags_2, p->flags_3, p->flags_4,
               ser(p->gobj), tok_p(p->on_invoke));
        dp("\n");
    }
    dp("L:");
    for (i = 0; i < (SM + 1) * (PM + 1); i++) dp(" %u", ser(HSD_GObj_ProcList[i]));
    dp("\n");
    for (i = 0; i < ng; i++) {
        g = G[i];
        dp("g %u %u %u %u %u %u %u %u %llx %u %u %d %d", ser(g), g->classifier, g->p_link, g->gx_link,
           g->p_priority, g->render_priority, g->obj_kind, g->user_data_kind, g->gxlink_prios,
           (u32) (uintptr_t) g->hsd_obj, (u32) (uintptr_t) g->user_data, tok_r(g->render_cb),
           tok_u(g->user_data_remove_func));
        for (p = g->proc; p; p = p->child) dp(" %u", ser(p));
        dp("\n");
    }
    /* DelayedProcInfo.gobj is not dumped: once applied it is a stale pointer */
    dp("d %d %x %u %u %u %u %u %u %d\n", HSD_GObj_804D783C, HSD_GObj_DelayedProcInfo.flags,
       HSD_GObj_DelayedProcInfo.type, HSD_GObj_DelayedProcInfo.p_link, HSD_GObj_DelayedProcInfo.p_prio,
       ser(HSD_GObj_CurrentInvokedProcGObj),
       ser(HSD_GObj_CurrentInvokedProc), ser(HSD_GObj_NextInvokedProc), HSD_GObj_CurrentInvokedSLink);
    fprintf(o, "h %016llx\n", fnv(dump, dn));
    if (getenv("GOBJ_DUMP")) fputs(dump, stderr);
}

/* ---- generator ---- */
static HSD_GObj* pick(void) { return ng ? G[rn(ng)] : NULL; }
/* gobjs on p_link `pl` other than `ex`, in G order */
static int gather(HSD_GObj** c, int pl, HSD_GObj* ex)
{
    int i, n = 0;
    for (i = 0; i < ng; i++) if (G[i]->p_link == pl && G[i] != ex) c[n++] = G[i];
    return n;
}
/* position for where/type 2 (after pos, NULL = head) and 3 (before pos; needs one) */
static HSD_GObj* pos_for(int* where, int pl, HSD_GObj* ex)
{
    static HSD_GObj* c[MAXG];
    int n, k;
    if (*where < 2) return NULL;
    n = gather(c, pl, ex);
    if (*where == 2) { k = rn(n + 1); return k < n ? c[k] : NULL; }
    if (n == 0) { *where = 0; return NULL; }
    return c[rn(n)];
}
static HSD_GObj* create(int incb)
{
    int where = incb ? rn(2) : rn(4);
    u16 cls = rn(8); u8 pl = rn(PM + 1), pr = rn(6);
    HSD_GObj* pos = pos_for(&where, pl, NULL);
    HSD_GObj* g = CreateGObj(where, cls, pl, pr, pos);
    fprintf(o, "c %u\n", ser(g));
    if (g) G[ng++] = g;
    return g;
}
static void setup_proc(HSD_GObj* g)
{
    int t = rn(4); u8 pri = rn(SM + 1);
    HSD_GObjProc* p = HSD_GObj_SetupProc(g, PF[t], pri);
    fprintf(o, "s %u\n", ser(p));
}
static void change_pri(HSD_GObj* g)
{
    int t = rn(4); u8 pl = rn(PM + 1), pr = rn(6);
    HSD_GObj* pos = pos_for(&t, pl, g);
    HSD_GObjPLink_ChangeGObjPri_Unk(t, g, pl, pr, pos);
}
static void gx_op(HSD_GObj* g)
{
    int v = rn(3);
    if (g->gx_link == 0xFF) {
        RenderFn r = RF[rn(2)];
        if (v == 0) { u8 l = rn(GM + 1); GObj_SetupGXLink(g, r, l, rn(6)); }
        else if (v == 1) GObj_SetupGXLinkMax(g, r, rn(6));
        else GObj_SetupGXLinkMaxSorted(g, r, rn(6));
    } else if (v == 0) {
        HSD_GObjGXLink_8039084C(g);
    } else if (v == 1) {
        u8 l = rn(GM + 1); HSD_GObjGXLink_80390908(g, l, rn(6));
    } else {
        static HSD_GObj* c[MAXG]; int i, n = 0;
        for (i = 0; i < ng; i++) if (G[i] != g && G[i]->gx_link != 0xFF) c[n++] = G[i];
        if (n) HSD_GObjGXLink_803909D8(g, c[rn(n)]); else HSD_GObjGXLink_8039084C(g);
    }
}
static void flag_op(HSD_GObj* g)
{
    switch (rn(4)) {
    case 0: HSD_GObj_80390C5C(g); break; case 1: HSD_GObj_80390C84(g); break;
    case 2: HSD_GObj_80390CAC(g); break; default: HSD_GObj_80390CD4(g); break;
    }
}
/* inside a proc callback on g: only operations on g itself, creating gobjs, and gx/flag ops
 * (freeing or unlinking another gobj's procs mid-walk can leave the C's NextInvokedProc dangling) */
static void act(HSD_GObj* g, int depth)
{
    switch (rn(12)) {
    case 0: HSD_GObjFree(g); break;
    case 1: HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc); break;
    case 2: change_pri(g); break;
    case 3: setup_proc(g); break;
    case 4: { HSD_GObj* n = create(1); if (n && rn(2)) setup_proc(n); break; }
    case 5: gx_op(g); break;
    case 6: flag_op(g); break;
    default: break;
    }
    if (depth == 0 && rn(4) == 0) act(g, 1);
}
static void obj_op(HSD_GObj* g)
{
    switch (rn(6)) {
    case 0: if (g->obj_kind == 0xFF) { u8 k = rn(2); HSD_GObjObject_80390A70(g, k, (void*) (uintptr_t) (1 + rn(1000))); } break;
    case 1: fprintf(o, "t %u\n", (u32) (uintptr_t) HSD_GObjObject_80390ADC(g)); break;
    case 2: HSD_GObjObject_80390B0C(g); break;
    case 3: if (g->user_data_kind == 0xFF) { u8 k = rn(3); int f = rn(2); GObj_InitUserData(g, k, UF[f], (void*) (uintptr_t) (1 + rn(1000))); } break;
    case 4: GObj_RemoveUserData(g); break;
    default: { u16 c = rn(8); u8 l = rn(PM + 1); fprintf(o, "f %u\n", ser(HSD_GObjObject_80390A3C(c, l))); break; }
    }
}

int main(int argc, char** argv)
{
    int seed, op;
    o = fopen(argv[1], "w");
    for (seed = 0; seed < 40; seed++) {
        InitData d;
        st = 0x9E3779B97F4A7C15ull ^ (u64) (seed + 1) * 0x2545F4914F6CDD1Dull;
        nx();
        if (seed % 8 == 0) HSD_GObjSetInitDefaults(&d);
        else { d.p_link_max = 1 + rn(5); d.gx_link_max = rn(4); d.gproc_pri_max = rn(3); d.funcs = NULL; d.unk_2 = NULL; }
        uf.next = NULL; uf.size = 2; uf.funcs = OF;
        d.funcs = &uf;
        PM = d.p_link_max; GM = d.gx_link_max; SM = d.gproc_pri_max;
        ng = 0; drv_gobj_serial = drv_proc_serial = 0;
        HSD_GObjInit(&d);
        fprintf(o, "S %d %d %d %d k %u %d %u %d e %u %u %u %u\n", seed, PM, GM, SM, HSD_GObj_CameraKind,
                HSD_GObj_LightKind, HSD_GObj_JObjKind, HSD_GObj_FogKind, HSD_GObj_80390EB8(0),
                HSD_GObj_80390EB8(1), HSD_GObj_80390EB8(2), HSD_GObj_80390EB8(3));
        for (op = 0; op < 400; op++) {
            u32 r = rn(20);
            HSD_GObj* g;
            fprintf(o, "O %u\n", r);
            if (r <= 4) create(0);
            else if (r <= 6) { if ((g = pick())) HSD_GObjFree(g); }
            else if (r == 7) { if ((g = pick())) change_pri(g); }
            else if (r <= 9) { if ((g = pick())) setup_proc(g); }
            else if (r == 10) {
                if ((g = pick())) {
                    HSD_GObjProc* c[256]; HSD_GObjProc* p; int n = 0;
                    for (p = g->proc; p && n < 256; p = p->child) c[n++] = p;
                    if (n) HSD_GObjProc_RemoveProc(c[rn(n)]);
                }
            }
            else if (r == 11) { if ((g = pick())) HSD_GObjProc_RemoveAllProcs(g); }
            else if (r <= 13) { if ((g = pick())) gx_op(g); }
            else if (r == 14) { if ((g = pick())) flag_op(g); }
            else if (r <= 16) {
                u32 m = rn(4);
                if (m == 0) HSD_GObjLibInitData.unk_2 = NULL;
                else { pause_mask = m == 1 ? (nx() & ((2ull << PM) - 1)) : 0; HSD_GObjLibInitData.unk_2 = &pause_mask; }
                HSD_GObj_RunProcs();
            }
            else if (r == 17) {
                if ((g = pick())) {
                    u32 mask = rn(8);
                    g->gxlink_prios = nx() & ((2ull << (GM + 1)) - 1);
                    HSD_GObj_80390ED0(g, mask);
                }
            }
            else if (r == 18) { if ((g = pick())) obj_op(g); }
            else HSD_GObj_80390FC0();
            do_dump();
        }
    }
    fclose(o);
    return 0;
}
