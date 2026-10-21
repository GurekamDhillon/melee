/* Link stubs for the robj oracle (build_robj.sh). */
#include <stdlib.h>
#include <stdio.h>
void* orc_alloc(unsigned n) { return calloc(1, n); }
float fabsf(float x) { union { float f; unsigned u; } v; v.f = x; v.u &= 0x7FFFFFFFu; return v.f; }
void* HSD_ObjAlloc(void* d) { (void) d; return calloc(1, 256); }
void HSD_ObjFree(void* d, void* p) { (void) d; (void) p; }
void HSD_ObjAllocInit(void* d, unsigned s, unsigned a) { (void) d; (void) s; (void) a; }
void HSD_Panic(const char* f, int l, const char* m) { fprintf(stderr, "panic %s:%d %s\n", f, l, m); exit(4); }
void __assert(const char* f, int l, const char* m) { fprintf(stderr, "assert %s:%d %s\n", f, l, m); exit(4); }
void OSReport(const char* m, ...) { (void) m; }
/* Unreached by the driver: abort if called. */
void HSD_AObjInitEndCallBack(void) { fprintf(stderr, "unexpected HSD_AObjInitEndCallBack\n"); exit(3); }
void HSD_AObjInterpretAnim(void) { fprintf(stderr, "unexpected HSD_AObjInterpretAnim\n"); exit(3); }
void HSD_AObjLoadDesc(void) { fprintf(stderr, "unexpected HSD_AObjLoadDesc\n"); exit(3); }
void HSD_AObjRemove(void) { fprintf(stderr, "unexpected HSD_AObjRemove\n"); exit(3); }
void HSD_AObjReqAnim(void) { fprintf(stderr, "unexpected HSD_AObjReqAnim\n"); exit(3); }
void HSD_DObjAddAnimAll(void) { fprintf(stderr, "unexpected HSD_DObjAddAnimAll\n"); exit(3); }
void HSD_DObjAnimAll(void) { fprintf(stderr, "unexpected HSD_DObjAnimAll\n"); exit(3); }
void HSD_DObjRemoveAnimAllByFlags(void) { fprintf(stderr, "unexpected HSD_DObjRemoveAnimAllByFlags\n"); exit(3); }
void HSD_DObjReqAnimAllByFlags(void) { fprintf(stderr, "unexpected HSD_DObjReqAnimAllByFlags\n"); exit(3); }
void HSD_JObjDispSub(void) { fprintf(stderr, "unexpected HSD_JObjDispSub\n"); exit(3); }
void HSD_JObjMakePositionMtx(void) { fprintf(stderr, "unexpected HSD_JObjMakePositionMtx\n"); exit(3); }
void hsdInitClassInfo(void) { fprintf(stderr, "unexpected hsdInitClassInfo\n"); exit(3); }
void splArcLengthPoint(void) { fprintf(stderr, "unexpected splArcLengthPoint\n"); exit(3); }
void ProfBegin(void) {}
int ProfActive(void) { return 0; }
char hsdObj[64];
void HSD_AObjInvokeCallBacks(void) { fprintf(stderr, "unexpected HSD_AObjInvokeCallBacks\n"); exit(3); }
void HSD_CObjGetCurrent(void) { fprintf(stderr, "unexpected HSD_CObjGetCurrent\n"); exit(3); }
void HSD_DObjLoadDesc(void) { fprintf(stderr, "unexpected HSD_DObjLoadDesc\n"); exit(3); }
void HSD_DObjRemoveAll(void) { fprintf(stderr, "unexpected HSD_DObjRemoveAll\n"); exit(3); }
void HSD_DObjResolveRefsAll(void) { fprintf(stderr, "unexpected HSD_DObjResolveRefsAll\n"); exit(3); }
void HSD_IDInsertToTable(void) { fprintf(stderr, "unexpected HSD_IDInsertToTable\n"); exit(3); }
void HSD_IDRemoveByIDFromTable(void) { fprintf(stderr, "unexpected HSD_IDRemoveByIDFromTable\n"); exit(3); }
void HSD_JObjDisp(void) { fprintf(stderr, "unexpected HSD_JObjDisp\n"); exit(3); }
void hsdNew(void) { fprintf(stderr, "unexpected hsdNew\n"); exit(3); }
void hsdSearchClassInfo(void) { fprintf(stderr, "unexpected hsdSearchClassInfo\n"); exit(3); }
void ProfEnd(void) {}
void diag_jobj_mtx(const float* m, int s, int b) { (void) m; (void) s; (void) b; }
int MSL_TrigF_80400770[] = { 0x7FFFFFFF }; /* src/MSL/float.c */
int MSL_TrigF_80400774[] = { 0x7F800000 };
int hsdObjIsDescendantOf(void* o, void* p) { (void) o; (void) p; return 1; }
void hsdDelete(void* o) { (void) o; fprintf(stderr, "unexpected hsdDelete\n"); exit(3); }
void* HSD_MemAlloc(unsigned n) { return calloc(1, n); }
unsigned HSD_GetNbBits(unsigned c) { int r = 0, i; for (i = 0; i < 32; i++) if (c & (1u << i)) r++; return r; }
void hsdIsDescendantOf(void) { fprintf(stderr, "unexpected hsdIsDescendantOf\n"); exit(3); }
