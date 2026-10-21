/* Link stubs for the anim oracle (build_anim.sh): a calloc-backed HSD_ObjAlloc,
 * and the PC guest-code probes fobj.c calls (no guest code here). */
#include <stdlib.h>
#include <stdio.h>
float fabsf(float x) { union { float f; unsigned u; } v; v.f = x; v.u &= 0x7FFFFFFFu; return v.f; }
void* HSD_ObjAlloc(void* d) { (void) d; return calloc(1, 256); }
void HSD_ObjFree(void* d, void* p) { (void) d; free(p); }
void HSD_ObjAllocInit(void* d, unsigned s, unsigned a) { (void) d; (void) s; (void) a; }
int ppc_is_guest_code(unsigned a) { (void) a; return 0; }
void* guest_scratch(unsigned s) { (void) s; return 0; }
