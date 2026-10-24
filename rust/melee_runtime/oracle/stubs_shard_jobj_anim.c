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
void HSD_JObjDispSub(void) { fprintf(stderr, "unexpected HSD_JObjDispSub\n"); exit(3); }
void HSD_JObjMakePositionMtx(void) { fprintf(stderr, "unexpected HSD_JObjMakePositionMtx\n"); exit(3); }
void hsdInitClassInfo(void) { fprintf(stderr, "unexpected hsdInitClassInfo\n"); exit(3); }
void ProfBegin(void) {}
int ProfActive(void) { return 0; }
char hsdObj[64];
void HSD_CObjGetCurrent(void) { fprintf(stderr, "unexpected HSD_CObjGetCurrent\n"); exit(3); }
void HSD_DObjLoadDesc(void) { fprintf(stderr, "unexpected HSD_DObjLoadDesc\n"); exit(3); }
void HSD_DObjRemoveAll(void) { fprintf(stderr, "unexpected HSD_DObjRemoveAll\n"); exit(3); }
void HSD_DObjResolveRefsAll(void) { fprintf(stderr, "unexpected HSD_DObjResolveRefsAll\n"); exit(3); }
void HSD_IDGetDataFromTable(void) { fprintf(stderr, "unexpected HSD_IDGetDataFromTable\n"); exit(3); }
void HSD_IDInsertToTable(void) { fprintf(stderr, "unexpected HSD_IDInsertToTable\n"); exit(3); }
void HSD_IDRemoveByIDFromTable(void) { fprintf(stderr, "unexpected HSD_IDRemoveByIDFromTable\n"); exit(3); }
void HSD_JObjDisp(void) { fprintf(stderr, "unexpected HSD_JObjDisp\n"); exit(3); }
void HSD_RObjLoadDesc(void) { fprintf(stderr, "unexpected HSD_RObjLoadDesc\n"); exit(3); }
void HSD_RObjRemoveAll(void) { fprintf(stderr, "unexpected HSD_RObjRemoveAll\n"); exit(3); }
void HSD_RObjResolveRefsAll(void) { fprintf(stderr, "unexpected HSD_RObjResolveRefsAll\n"); exit(3); }
void hsdIsDescendantOf(void) { fprintf(stderr, "unexpected hsdIsDescendantOf\n"); exit(3); }
void hsdNew(void) { fprintf(stderr, "unexpected hsdNew\n"); exit(3); }
void hsdSearchClassInfo(void) { fprintf(stderr, "unexpected hsdSearchClassInfo\n"); exit(3); }
void ProfEnd(void) {}
void diag_jobj_mtx(const float* m, int s, int b) { (void) m; (void) s; (void) b; }
int MSL_TrigF_80400770[] = { 0x7FFFFFFF }; /* src/MSL/float.c */
int MSL_TrigF_80400774[] = { 0x7F800000 };
int ppc_is_guest_code(unsigned a){return 0;}
void* guest_scratch(unsigned s){return 0;}
void* __va_arg(void*a,int t){abort();return 0;}
