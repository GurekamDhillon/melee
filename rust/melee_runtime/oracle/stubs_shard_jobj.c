/* Link stubs for the jobj oracle (build_jobj.sh). */
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
void HSD_AObjReqAnim(void) { fprintf(stderr, "unexpected HSD_AObjReqAnim\n"); exit(3); }
void HSD_DObjAddAnimAll(void) { fprintf(stderr, "unexpected HSD_DObjAddAnimAll\n"); exit(3); }
void HSD_DObjAnimAll(void) { fprintf(stderr, "unexpected HSD_DObjAnimAll\n"); exit(3); }
void HSD_DObjRemoveAnimAllByFlags(void) { fprintf(stderr, "unexpected HSD_DObjRemoveAnimAllByFlags\n"); exit(3); }
void HSD_DObjReqAnimAllByFlags(void) { fprintf(stderr, "unexpected HSD_DObjReqAnimAllByFlags\n"); exit(3); }
void HSD_JObjDispSub(void) { fprintf(stderr, "unexpected HSD_JObjDispSub\n"); exit(3); }
void HSD_JObjMakePositionMtx(void) { fprintf(stderr, "unexpected HSD_JObjMakePositionMtx\n"); exit(3); }
void HSD_RObjAddAnimAll(void) { fprintf(stderr, "unexpected HSD_RObjAddAnimAll\n"); exit(3); }
void HSD_RObjAnimAll(void) { fprintf(stderr, "unexpected HSD_RObjAnimAll\n"); exit(3); }
void HSD_RObjRemoveAnimAllByFlags(void) { fprintf(stderr, "unexpected HSD_RObjRemoveAnimAllByFlags\n"); exit(3); }
void HSD_RObjReqAnimAllByFlags(void) { fprintf(stderr, "unexpected HSD_RObjReqAnimAllByFlags\n"); exit(3); }
void splArcLengthPoint(void) { fprintf(stderr, "unexpected splArcLengthPoint\n"); exit(3); }
void ProfBegin(void) {}
int ProfActive(void) { return 0; }
void HSD_AObjInvokeCallBacks(void) { fprintf(stderr, "unexpected HSD_AObjInvokeCallBacks\n"); exit(3); }
void ProfEnd(void) {}
void diag_jobj_mtx(const float* m, int s, int b) { (void) m; (void) s; (void) b; }
int MSL_TrigF_80400770[] = { 0x7FFFFFFF }; /* src/MSL/float.c */
int MSL_TrigF_80400774[] = { 0x7F800000 };
