/* Link stubs for the generator oracle (build_generator.sh). */
#include <stdlib.h>
#include <stdio.h>
float fabsf(float x) { union { float f; unsigned u; } v; v.f = x; v.u &= 0x7FFFFFFFu; return v.f; }
void HSD_Panic(const char* f, int l, const char* m) { fprintf(stderr, "panic %s:%d %s\n", f, l, m); exit(4); }
void __assert(const char* f, int l, const char* m) { fprintf(stderr, "assert %s:%d %s\n", f, l, m); exit(4); }
void OSReport(const char* m, ...) { (void) m; }
void ProfBegin(void) {}
int ProfActive(void) { return 0; }
void ProfEnd(void) {}
int MSL_TrigF_80400770[] = { 0x7FFFFFFF }; /* src/MSL/float.c */
int MSL_TrigF_80400774[] = { 0x7F800000 };
void Replay_RandTrace(unsigned c, unsigned s, int st) { (void) c; (void) s; (void) st; }
