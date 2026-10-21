/* Link stubs for the HSD oracle (build_hsd.sh). sqrtf is math_ppc.h's extern inline, which emits
 * no out-of-line body here (no inlining under -disable-llvm-passes); forward to wrap.c's copy. */
float w_sqrtf(float);
float sqrtf(float x) { return w_sqrtf(x); }
void* HSD_ObjAlloc(void* d) { (void) d; return 0; }
void HSD_ObjFree(void* d, void* p) { (void) d; (void) p; }
void HSD_ObjAllocInit(void* d, unsigned s, unsigned a) { (void) d; (void) s; (void) a; }
/* -fno-builtin leaves MSL's __fabsf -> fabsf as a call; the sign-bit clear it stands for. */
float fabsf(float x) { union { float f; unsigned u; } v; v.f = x; v.u &= 0x7FFFFFFFu; return v.f; }
