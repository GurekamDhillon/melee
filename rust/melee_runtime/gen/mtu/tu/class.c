extern int atoi(const char*);
extern char* getenv(const char*);
/* TU: src/sysdolphin/baselib/class.c */
/* C oracle for the Rust port of this TU: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by gen/mtu/mk_lb.py. Each line: op case ret state-hash trace-hash. */

#include TU_SRC
#include "mtu_common.h"
static HSD_ClassInfo CI[4];
static HSD_Class CL[3];
static struct { float x, y, z; } VA[4];
static u8 U8A[16];
static int IA[4];
static u32 UA[4];
static float FA[4];
static unsigned char BUF[256];
static void* PPA[4];
static char STRS[4][64] = { "a.dat", "sym", "LbAd.dat", "x" };

static HSD_ClassInfo* pick_HSD_ClassInfo(void) { u32 r = rnd(); return &CI[(r >> 8) % 4]; }
static HSD_ClassInfo* picko_HSD_ClassInfo(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CI[(r >> 8) % 4]; }
static HSD_Class* pick_HSD_Class(void) { u32 r = rnd(); return &CL[(r >> 8) % 3]; }
static HSD_Class* picko_HSD_Class(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &CL[(r >> 8) % 3]; }
struct HSD_GObj; typedef struct HSD_GObj HSD_GObj;
static void cbev1(HSD_GObj* g) { T(7001); T(PI(g)); }
static void cbev2(HSD_GObj* g) { T(7002); T(PI(g)); }
static int cbp1(HSD_GObj* g) { T(7003); T(PI(g)); return (int) (fv() & 1); }
static int cbp2(HSD_GObj* g) { T(7004); T(PI(g)); return (int) (fv() & 1); }
static void cbev3(HSD_GObj* g, HSD_GObj* h) { T(7005); T(PI(g)); T(PI(h)); }
static void cbi1(int a) { T(7006); T((u32) a); }
static void cbi2(int a) { T(7007); T((u32) a); }
static void* pick_fn(int kind)
{
    u32 r = rnd();
    if ((r & 3) == 0) return 0;
    switch (kind) {
    case 1: return (r & 4) ? (void*) cbev1 : (void*) cbev2;
    case 2: return (r & 4) ? (void*) cbp1 : (void*) cbp2;
    case 3: return (void*) cbev3;
    case 4: return (r & 4) ? (void*) cbi1 : (void*) cbi2;
    }
    return 0;
}
static void* pick_cb(int kind) { void* f = pick_fn(kind); return f ? f : (kind == 2 ? (void*) cbp1 : kind == 3 ? (void*) cbev3 : kind == 4 ? (void*) cbi1 : (void*) cbev1); }
static void freg_world(void) { freg((const void*) cbev1); freg((const void*) cbev2); freg((const void*) cbp1); freg((const void*) cbp2); freg((const void*) cbev3); freg((const void*) cbi1); freg((const void*) cbi2);  }

static unsigned char ARENA[65536]; static unsigned ARENA_P; static int FAIL1; static int PK;
#define PNEXT() (PIECES + 256 * (PK++ & 15))
static unsigned char PIECES[8192];
static void* arena_alloc(int n) { void* p; if (n <= 0 || ARENA_P + ((n + 7) & ~7) > sizeof ARENA) return 0; p = ARENA + ARENA_P; ARENA_P += (n + 7) & ~7; return p; }
static void cb_amn(HSD_ClassInfo* c) { T(8201); T(PI(c)); }
static int cb_init(HSD_Class* c) { T(8202); T(PI(c)); return (int) (fv() % 3) - 1; }
static void cb_rel(HSD_Class* c) { T(8203); T(PI(c)); }
static void cb_dest(HSD_Class* c) { T(8204); T(PI(c)); }
static HSD_Class* cb_alloc(HSD_ClassInfo* c) { T(8205); T(PI(c)); return (fv() % 5) ? (HSD_Class*) arena_alloc(c->head.obj_size) : 0; }
static void ii0(void) { T(8210); CI[0].head.flags |= 1; }
static void ii1(void) { T(8211); CI[1].head.flags |= 1; }
static void ii2(void) { T(8212); CI[2].head.flags |= 1; }
static void ii3(void) { T(8213); CI[3].head.flags |= 1; }

#include "class_gen.h"



static void preg_world(void)
{
    preg(CI, sizeof CI);
    preg(CL, sizeof CL);
    preg(VA, sizeof VA); preg(U8A, sizeof U8A); preg(IA, sizeof IA); preg(UA, sizeof UA); preg(FA, sizeof FA); preg(BUF, sizeof BUF); preg(STRS, sizeof STRS); preg(PPA, sizeof PPA);
}

static void setup_world(void)
{
    int i;
    (void) i;
    zero(CI, sizeof CI);
    zero(CL, sizeof CL);
    for (i = 0; i < 16; i++) { U8A[i] = (u8) rnd(); }
    for (i = 0; i < 4; i++) { IA[i] = (int) fv(); UA[i] = fv(); FA[i] = rw(); VA[i].x = rw(); VA[i].y = rw(); VA[i].z = rw(); }
    for (i = 0; i < 256; i++) BUF[i] = (unsigned char) rnd();
    BUF[255] = 0;
    for (i = 0; i < 4; i++) PPA[i] = 0;
    for (i = 0; i < 4; i++) fill_HSD_ClassInfo(&CI[i]);
    for (i = 0; i < 3; i++) fill_HSD_Class(&CL[i]);
    fillg_all();
    fillo_all();
}

static void hash_world(void)
{
    int i;
    (void) i;
    for (i = 0; i < 4; i++) hash_HSD_ClassInfo(&CI[i]);
    for (i = 0; i < 3; i++) hash_HSD_Class(&CL[i]);
    for (i = 0; i < 4; i++) { HM(B(VA[i].x)); HM(B(VA[i].y)); HM(B(VA[i].z)); HM(B(FA[i])); HM((u32) IA[i]); HM(UA[i]); }
    for (i = 0; i < 16; i++) HM(U8A[i]);
    for (i = 0; i < 256; i++) HM(BUF[i]);
    for (i = 0; i < 4; i++) HM(PI(PPA[i]));
    hashg_all();
    hasho_all();
}

enum { OP_ClassInfoInit = 0, OP_hsdInitClassInfo = 1, OP_OSReport_PrintSpaces = 2, OP_GetMemoryEntry = 3, OP_hsdAllocMemPiece = 4, OP_hsdFreeMemPiece = 5, OP__hsdClassAlloc = 6, OP__hsdClassInit = 7, OP__hsdClassRelease = 8, OP__hsdClassDestroy = 9, OP__hsdClassAmnesia = 10, OP__hsdClassInfoInit = 11, OP_hsdNew = 12, OP_hsdChangeClass = 13, OP_hsdIsDescendantOf = 14, OP_hsdObjIsDescendantOf = 15, OP_class_set_flags = 16, OP_ForgetClassLibraryReal = 17, OP_ForgetClassLibraryChild = 18, OP_hsdForgetClassLibrary = 19, OP_hsdSearchClassInfo = 20, OP_DumpClassStat = 21, OP_hsdDumpClassStat = 22 };

int main(int argc, char** argv)
{
    int op, k, only = -1;
    if (argc > 1) only = atoi(argv[1]);
    preg_world();
    preg_own();
    freg_all();
    freg_world();
preg(ARENA, sizeof ARENA); preg(PIECES, sizeof PIECES); freg((const void*) cb_amn); freg((const void*) cb_init); freg((const void*) cb_rel); freg((const void*) cb_dest); freg((const void*) cb_alloc); freg((const void*) ii0); freg((const void*) ii1); freg((const void*) ii2); freg((const void*) ii3);
    for (op = 0; op < 23; op++) {
        int n = 0;
        if (only >= 0 && op != only) continue;
        switch (op) {
        case 0: n = 400; break;  /* ClassInfoInit */
        case 1: n = 400; break;  /* hsdInitClassInfo */
        case 2: n = 400; break;  /* OSReport_PrintSpaces */
        case 3: n = 400; break;  /* GetMemoryEntry */
        case 4: n = 400; break;  /* hsdAllocMemPiece */
        case 5: n = 400; break;  /* hsdFreeMemPiece */
        case 6: n = 400; break;  /* _hsdClassAlloc */
        case 7: n = 400; break;  /* _hsdClassInit */
        case 8: n = 400; break;  /* _hsdClassRelease */
        case 9: n = 400; break;  /* _hsdClassDestroy */
        case 10: n = 400; break;  /* _hsdClassAmnesia */
        case 11: n = 400; break;  /* _hsdClassInfoInit */
        case 12: n = 400; break;  /* hsdNew */
        case 13: n = 400; break;  /* hsdChangeClass */
        case 14: n = 400; break;  /* hsdIsDescendantOf */
        case 15: n = 400; break;  /* hsdObjIsDescendantOf */
        case 16: n = 400; break;  /* class_set_flags */
        case 17: n = 400; break;  /* ForgetClassLibraryReal */
        case 18: n = 400; break;  /* ForgetClassLibraryChild */
        case 19: n = 400; break;  /* hsdForgetClassLibrary */
        case 20: n = 400; break;  /* hsdSearchClassInfo */
        case 21: n = 400; break;  /* DumpClassStat */
        case 22: n = 400; break;  /* hsdDumpClassStat */
        }
        for (k = 0; k < n; k++) {
            u32 ret = 0;
            tu_case(op, k);
            setup_world();
{ int q; FAIL1 = 0; PK = 0; memset(ARENA, 0, sizeof ARENA); ARENA_P = 0; memset(PIECES, 0, sizeof PIECES);
 for (q = 0; q < 4; q++) { HSD_ClassInfo* c = &CI[q];
  c->head.info_init = q == 0 ? ii0 : q == 1 ? ii1 : q == 2 ? ii2 : ii3; c->head.flags = fv() & 1;
  c->head.library_name = STRS[fv() % 4]; c->head.class_name = STRS[fv() % 4];
  c->head.obj_size = (s16) (8 + 8 * (fv() % 8)); c->head.info_size = (s16) (sizeof(HSD_ClassInfoHead) + 8 * (fv() % 6));
  c->head.parent = (q > 0 && (fv() & 1)) ? &CI[fv() % q] : 0;
  c->head.child = (q < 3 && (fv() & 1)) ? &CI[q + 1 + fv() % (3 - q)] : 0;
  c->head.next = (q < 3 && (fv() & 1)) ? &CI[q + 1 + fv() % (3 - q)] : 0;
  c->head.nb_exist = fv() % 50; c->head.nb_peak = c->head.nb_exist + fv() % 20;
  c->alloc = (q & 1) ? _hsdClassAlloc : cb_alloc; c->init = (fv() & 1) ? cb_init : _hsdClassInit; c->release = cb_rel; c->destroy = (fv() & 1) ? cb_dest : _hsdClassDestroy;
  c->amnesia = (fv() & 1) ? cb_amn : _hsdClassAmnesia; }
 for (q = 0; q < 3; q++) CL[q].class_info = &CI[fv() % 4];
 hsdClass.head.info_init = _hsdClassInfoInit; hsdClass.head.flags = fv() & 1; hsdClass.head.library_name = STRS[fv() % 4]; hsdClass.head.class_name = STRS[fv() % 4];
 hsdClass.head.obj_size = (s16) (8 + 8 * (fv() % 8)); hsdClass.head.info_size = (s16) sizeof(HSD_ClassInfoHead); hsdClass.head.parent = 0;
 hsdClass.head.child = (fv() & 1) ? &CI[fv() % 4] : 0; hsdClass.head.next = 0; hsdClass.head.nb_exist = fv() % 50; hsdClass.head.nb_peak = fv() % 70;
 hsdClass.alloc = _hsdClassAlloc; hsdClass.init = _hsdClassInit; hsdClass.release = _hsdClassRelease; hsdClass.destroy = _hsdClassDestroy; hsdClass.amnesia = _hsdClassAmnesia;
 memory_list = 0; nb_memory_list = 0; current_hash = (fv() & 1) ? (HSD_Hash*) BUF : 0;
 if (fv() & 1) { int w = fv() % 5; while (w--) { GetMemoryEntry(fv() % 100); } w = fv() % 3; while (w--) { hsdFreeMemPiece(PNEXT(), 1 + fv() % 200); } }
 FAIL1 = (op == OP_GetMemoryEntry || op == OP_hsdAllocMemPiece || op == OP__hsdClassAlloc) && ((fv() & 3) == 0);
}
            if (getenv("MTU_PROG")) { printf("P %d %d%c", op, k, 10); fflush(0); }
            ARMED = 1;
            if (MTU_SETJMP()) { ARMED = 0; printf("%d %d ABORT\n", op, k); continue; }
            switch (op) {
            case 0: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); ClassInfoInit(a0);  } break;
            case 1: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); HSD_ClassInfo * a1 = picko_HSD_ClassInfo(); char * a2 = STRS[rnd() % 4]; char * a3 = STRS[rnd() % 4]; s32 a4 = (s32) (sizeof(HSD_ClassInfoHead) + 8 * (fv() % 8)); s32 a5 = (s32) (8 + 8 * (fv() % 10)); hsdInitClassInfo(a0, a1, a2, a3, a4, a5);  } break;
            case 2: {  s32 a0 = (s32) (fv() % 5) - 1; OSReport_PrintSpaces(a0);  } break;
            case 3: {  s32 a0 = (s32) (((fv() % 32) == 0) ? -1 : (int) (fv() % 140)); ret = PI(GetMemoryEntry(a0));  } break;
            case 4: {  s32 a0 = (s32) (1 + fv() % 1500); ret = PI(hsdAllocMemPiece(a0));  } break;
            case 5: {  void * a0 = (void*) ((fv() & 7) ? (void*) PNEXT() : 0); s32 a1 = (s32) (1 + fv() % 200); hsdFreeMemPiece(a0, a1);  } break;
            case 6: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); ret = PI(_hsdClassAlloc(a0));  } break;
            case 7: {  HSD_Class * a0 = pick_HSD_Class(); ret = (u32) _hsdClassInit(a0);  } break;
            case 8: {  HSD_Class * a0 = pick_HSD_Class(); _hsdClassRelease(a0);  } break;
            case 9: {  HSD_Class * a0 = pick_HSD_Class(); _hsdClassDestroy(a0);  } break;
            case 10: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); _hsdClassAmnesia(a0);  } break;
            case 11: {   _hsdClassInfoInit();  } break;
            case 12: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); ret = PI(hsdNew(a0));  } break;
            case 13: {  void * a0 = (void*) pick_HSD_Class(); void * a1 = (void*) pick_HSD_ClassInfo(); ret = (u32) hsdChangeClass(a0, a1);  } break;
            case 14: {  void * a0 = (void*) picko_HSD_ClassInfo(); void * a1 = (void*) picko_HSD_ClassInfo(); ret = (u32) hsdIsDescendantOf(a0, a1);  } break;
            case 15: {  HSD_Obj * a0 = (HSD_Obj*) picko_HSD_Class(); HSD_ClassInfo * a1 = pick_HSD_ClassInfo(); ret = (u32) hsdObjIsDescendantOf(a0, a1);  } break;
            case 16: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); s32 a1 = (s32) (fv() & 3); s32 a2 = (s32) (fv() & 3); class_set_flags(a0, a1, a2);  } break;
            case 17: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); ForgetClassLibraryReal(a0);  } break;
            case 18: {  const char * a0 = STRS[rnd() % 4]; HSD_ClassInfo * a1 = pick_HSD_ClassInfo(); ForgetClassLibraryChild(a0, a1);  } break;
            case 19: {  const char * a0 = STRS[rnd() % 4]; hsdForgetClassLibrary(a0);  } break;
            case 20: {  const char * a0 = STRS[rnd() % 4]; ret = PI(hsdSearchClassInfo(a0));  } break;
            case 21: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); s32 a1 = (s32) (fv() % 4); DumpClassStat(a0, a1);  } break;
            case 22: {  HSD_ClassInfo * a0 = pick_HSD_ClassInfo(); bool a1 = (fv() & 1); s32 a2 = (s32) (fv() % 4); hsdDumpClassStat(a0, a1, a2);  } break;
            }
            ARMED = 0;
            hash_world();
{ int i, n; hash_HSD_ClassInfo(&hsdClass); HM(PI(memory_list)); HM((u32) nb_memory_list); HM(PI(current_hash));
 for (i = 0; i < nb_memory_list; i++) { HSD_MemoryEntry* e = memory_list[i]; HM(PI(e)); if (e) { HSD_FreeList* f; HM(e->size); HM(e->nb_alloc); HM(e->nb_free); HM(PI(e->next)); HM(PI(e->free_list));
   for (f = e->free_list, n = 0; f && n < 64; f = f->next, n++) HM(PI(f)); } } }
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
