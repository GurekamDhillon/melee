/* Link stubs for the gobj_native oracle: allocator (calloc), and the object / render callees of
 * gobj.c that the driver never reaches. */
#include <stdlib.h>
#include <stdio.h>
/* objalloc.c as the real pool behaves: a freed cell goes on a LIFO free list per pool and the next
 * allocation of that pool takes it back (contents untouched); a fresh cell comes from calloc. */
#define NPOOL 4
#define NFREE 512
static void* pool_key[NPOOL]; static void* pool_free[NPOOL][NFREE]; static int pool_n[NPOOL];
static int pool_of(void* d) { int i; for (i = 0; i < NPOOL; i++) { if (pool_key[i] == d) return i; if (!pool_key[i]) { pool_key[i] = d; return i; } } return NPOOL - 1; }
void* HSD_ObjAlloc(void* d) { int i = pool_of(d); if (pool_n[i] > 0) return pool_free[i][--pool_n[i]]; return calloc(1, 256); }
void HSD_ObjFree(void* d, void* p) { int i = pool_of(d); if (pool_n[i] < NFREE) pool_free[i][pool_n[i]++] = p; }
void HSD_ObjAllocInit(void* d, unsigned s, unsigned a) { int i = pool_of(d); (void) s; (void) a; pool_n[i] = 0; }
void* HSD_MemAlloc(int size) { return calloc(1, (unsigned) size); }
void HSD_Panic(const char* f, int l, const char* m) { fprintf(stderr, "panic %s:%d %s\n", f, l, m); exit(4); }
void __assert(const char* f, int l, const char* m) { fprintf(stderr, "assert %s:%d %s\n", f, l, m); exit(4); }
void OSReport(const char* m, ...) { (void) m; }
void ProfBegin(void) {}
int ProfActive(void) { return 0; }
int ProfEnabled(void) { return 0; }
int GObjLogEnabled(void) { return 0; }
int HSD_CObjSetCurrent(void* c) { (void) c; return 0; }
void HSD_CObjEndCurrent(void) {}
void* HSD_CObjGetCurrent(void) { return 0; }
void HSD_FogSet(void* f) { (void) f; }
void HSD_JObjDispAll(void* a, void* b, unsigned c, int d) { (void) a; (void) b; (void) c; (void) d; }
void HSD_JObjRemoveAll(void* j) { (void) j; }
void HSD_LObjRemoveAll(void* j) { (void) j; }
void HSD_LObjSetupInit(void* c) { (void) c; }
void HSD_LObj_803668EC(void* l) { (void) l; }
void HSD_RandRenderBegin(void) {}
void HSD_RandRenderEnd(void) {}
void ProfCounter(int a, unsigned b) { (void) a; (void) b; }
void ProfEnd(void) {}
int ScriptGame_ProfEnemy(void* g) { (void) g; return 0; }
void Snap_CbTime(void* cb, int b) { (void) cb; (void) b; }
int Snap_SkipRenderCb(void* cb) { (void) cb; return 0; }
unsigned lbHeap_Free(int h) { (void) h; return 0; }
