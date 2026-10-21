/* Link stubs for the fog oracle (build_tobj.sh). HSD_ObjAlloc is calloc-backed; the JObj calls
 * the link pulls in are logged no-ops (the driver never gives an AObjDesc a non-zero obj_id). */
#include <stdlib.h>
#include <stdio.h>
float fabsf(float x) { union { float f; unsigned u; } v; v.f = x; v.u &= 0x7FFFFFFFu; return v.f; }
void* HSD_ObjAlloc(void* d) { (void) d; return calloc(1, 256); }
void HSD_ObjFree(void* d, void* p) { (void) d; free(p); }
void HSD_ObjAllocInit(void* d, unsigned s, unsigned a) { (void) d; (void) s; (void) a; }
int ppc_is_guest_code(unsigned a) { (void) a; return 0; }
void* guest_scratch(unsigned s) { (void) s; return 0; }
void HSD_JObjUnref(void* j) { (void) j; fprintf(stderr, "unexpected JObjUnref\n"); exit(3); }
void* HSD_JObjLoadJoint(void* j) { (void) j; fprintf(stderr, "unexpected JObjLoadJoint\n"); exit(3); }
void HSD_Panic(const char* f, int l, const char* m) { fprintf(stderr, "panic %s:%d %s\n", f, l, m); exit(4); }
void __assert(const char* f, int l, const char* m) { fprintf(stderr, "assert %s:%d %s\n", f, l, m); exit(4); }
void OSReport(const char* m, ...) { (void) m; }
void* __va_arg(void* ap, int t) { (void) ap; (void) t; fprintf(stderr, "unexpected va_arg\n"); exit(3); return 0; }
void* hsdNew(void* c) { (void) c; return calloc(1, 256); }
void hsdInitClassInfo(void) { fprintf(stderr, "unexpected hsdInitClassInfo\n"); exit(3); }
char hsdObj[64];
