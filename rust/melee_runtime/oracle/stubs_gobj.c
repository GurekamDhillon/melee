/* Link stubs for the GObj oracle (build_gobj.sh). HSD_ObjAlloc hands out calloc blocks with a
 * 16-byte header holding an allocation serial (one counter per pool, reset per seed by the
 * driver) and never frees, so no address is reused and serials identify objects. The PC hooks
 * are the no-guest-code defaults; ScriptGame_StageSlotDestroyed tells the driver a gobj died. */
#include <stdlib.h>
#include <stdio.h>
extern char gobj_alloc_data[];
unsigned drv_gobj_serial, drv_proc_serial;
void drv_destroyed(void* g);
void drv_rand_render(int begin);
void* HSD_ObjAlloc(void* d)
{
    char* p = calloc(1, 256 + 16);
    *(unsigned*) p = d == (void*) gobj_alloc_data ? ++drv_gobj_serial : ++drv_proc_serial;
    return p + 16;
}
void HSD_ObjFree(void* d, void* p) { (void) d; (void) p; }
void HSD_ObjAllocInit(void* d, unsigned s, unsigned a) { (void) d; (void) s; (void) a; }
void* HSD_MemAlloc(int n) { return calloc(1, (size_t) n); }
double __frsqrte(double x) { (void) x; abort(); }
int GObjLogEnabled(void) { return 0; }
int ProfActive(void) { return 0; }
int ProfEnabled(void) { return 0; }
void ProfBegin(int a, int b) { (void) a; (void) b; }
void ProfEnd(void) {}
void ProfCounter(int a, int b) { (void) a; (void) b; }
int lbHeap_Free(int h) { (void) h; return 0; }
int ScriptStageHostHidden;
int ScriptGame_ProfEnemy(void* g) { (void) g; return 0; }
int ScriptGame_StageSlotCanCreate(void) { return 1; }
void ScriptGame_StageSlotCreated(void* g) { (void) g; }
void ScriptGame_StageSlotDestroyed(void* g) { drv_destroyed(g); }
int ScriptGame_StageSlotProcBegin(void* g) { (void) g; return 0; }
void ScriptGame_StageSlotProcEnd(void) {}
int Snap_SkipRenderCb(void* cb) { (void) cb; return 0; }
void Snap_CbTime(void* cb, int b) { (void) cb; (void) b; }
void HSD_RandRenderBegin(void) { drv_rand_render(1); }
void HSD_RandRenderEnd(void) { drv_rand_render(0); }
void OSReport(const char* f, ...) { (void) f; }
void __assert(const char* f, int l, const char* c) { fprintf(stderr, "ASSERT %s:%d %s\n", f, l, c); exit(3); }
#define NOPE(n) void n(void) { fprintf(stderr, "unexpected call " #n "\n"); exit(4); }
NOPE(HSD_CObjSetCurrent) NOPE(HSD_CObjGetCurrent) NOPE(HSD_CObjEndCurrent) NOPE(HSD_FogSet)
NOPE(HSD_JObjDispAll) NOPE(HSD_JObjRemoveAll) NOPE(HSD_LObjRemoveAll) NOPE(HSD_LObjSetupInit)
NOPE(HSD_LObj_803668EC)
