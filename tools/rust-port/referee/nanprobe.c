// Gekko NaN probe (oracle-fix referee): which operand's NaN an op returns, with which sign, the default NaN of an
// invalid op, and frsp of a double NaN. Single precision ops on f32 bit patterns; results printed as bits.
typedef unsigned int u32; typedef unsigned long long u64;
__attribute__((noinline, used)) void OSReport(const char *fmt, ...) { __asm__ volatile("" : : "r"(fmt) : "memory"); }
static float F(u32 b) { union { u32 u; float f; } c = { b }; return c.f; }
static u32 B(float f) { union { float f; u32 u; } c = { f }; return c.u; }
#define OP2(name, ins) static u32 name(u32 a, u32 b) { float r; __asm__ volatile(ins " %0,%1,%2" : "=f"(r) : "f"(F(a)), "f"(F(b))); return B(r); }
OP2(fadds_, "fadds") OP2(fsubs_, "fsubs") OP2(fmuls_, "fmuls") OP2(fdivs_, "fdivs")
static u32 fmadds_(u32 a, u32 c, u32 b) { float r; __asm__ volatile("fmadds %0,%1,%2,%3" : "=f"(r) : "f"(F(a)), "f"(F(c)), "f"(F(b))); return B(r); }
static u32 fnmsubs_(u32 a, u32 c, u32 b) { float r; __asm__ volatile("fnmsubs %0,%1,%2,%3" : "=f"(r) : "f"(F(a)), "f"(F(c)), "f"(F(b))); return B(r); }
static u32 fneg_(u32 a) { float r; __asm__ volatile("fneg %0,%1" : "=f"(r) : "f"(F(a))); return B(r); }
static u32 fabs_(u32 a) { float r; __asm__ volatile("fabs %0,%1" : "=f"(r) : "f"(F(a))); return B(r); }
static u32 frsp_(u64 d) { union { u64 u; double f; } c = { d }; float r; __asm__ volatile("frsp %0,%1" : "=f"(r) : "f"(c.f)); return B(r); }
static volatile u32 R[40];
void main_probe(void) {
  const u32 QP = 0x7FC00000, QN = 0xFFC00000, QP1 = 0x7FC00001, SP = 0x7F800001, ONE = 0x3F800000, NONE = 0xBF800000, INF = 0x7F800000, Z = 0;
  int n = 0;
  R[n++] = fneg_(QP); R[n++] = fneg_(QN); R[n++] = fneg_(SP); R[n++] = fabs_(QN);
  R[n++] = fadds_(QP, ONE); R[n++] = fadds_(ONE, QN); R[n++] = fadds_(QP, QN); R[n++] = fadds_(QN, QP); R[n++] = fadds_(SP, ONE);
  R[n++] = fsubs_(ONE, QN); R[n++] = fsubs_(QN, ONE);
  R[n++] = fmuls_(QP, NONE); R[n++] = fmuls_(NONE, QN); R[n++] = fmuls_(QN, QP1); R[n++] = fmuls_(QP1, QN);
  R[n++] = fmadds_(QN, ONE, ONE); R[n++] = fmadds_(ONE, QN, ONE); R[n++] = fmadds_(ONE, ONE, QN);
  R[n++] = fmadds_(QP1, ONE, QN); R[n++] = fmadds_(ONE, QP1, QN); R[n++] = fmadds_(QP1, QN, ONE);
  R[n++] = fnmsubs_(QP, ONE, ONE); R[n++] = fnmsubs_(ONE, ONE, QP); R[n++] = fnmsubs_(ONE, QP, ONE);
  R[n++] = fsubs_(INF, INF); R[n++] = fmuls_(Z, INF); R[n++] = fdivs_(Z, Z); R[n++] = fmadds_(Z, INF, ONE);
  R[n++] = frsp_(0xFFF8000000000000ull); R[n++] = frsp_(0x7FF8000000000000ull); R[n++] = frsp_(0x7FF8000123456789ull);
  for (int i = 0; i < n; i++) OSReport("REF %d %08x\n", i, R[i]);
  OSReport("REF done\n");
  for (;;) { }
}
__attribute__((naked, section(".text.start"))) void _start(void) {
  __asm__ volatile("lis 1,0x8170\n ori 1,1,0xff00\n mfmsr 3\n ori 3,3,0x2000\n mtmsr 3\n isync\n b main_probe\n");
}
