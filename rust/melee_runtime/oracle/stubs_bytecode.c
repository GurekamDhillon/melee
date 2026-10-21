/* Link stubs for the bytecode oracle (build_bytecode.sh). HSD_ObjAlloc is calloc-backed. */
#include <stdlib.h>
#include <stdio.h>
float fabsf(float x) { union { float f; unsigned u; } v; v.f = x; v.u &= 0x7FFFFFFFu; return v.f; }
void* HSD_ObjAlloc(void* d) { (void) d; return calloc(1, 64); }
void HSD_ObjFree(void* d, void* p) { (void) d; free(p); }
void HSD_ObjAllocInit(void* d, unsigned s, unsigned a) { (void) d; (void) s; (void) a; }
void HSD_Panic(const char* f, int l, const char* m) { fprintf(stderr, "panic %s:%d %s\n", f, l, m); exit(4); }
void __assert(const char* f, int l, const char* m) { fprintf(stderr, "assert %s:%d %s\n", f, l, m); exit(4); }
void OSReport(const char* m, ...) { fprintf(stderr, "OSReport %s", m); }
void Replay_RandTrace(unsigned c, unsigned s, int st) { (void) c; (void) s; (void) st; }
int MSL_TrigF_80400770[] = { 0x7FFFFFFF }; /* src/MSL/float.c */
int MSL_TrigF_80400774[] = { 0x7F800000 };
