// Gekko integer probe (oracle-fix referee): fctiwz, slw/srw/sraw, divw/divwu edge cases on Dolphin. run.sh intprobe
typedef unsigned int u32; typedef int s32;
__attribute__((noinline, used)) void OSReport(const char *fmt, ...) { __asm__ volatile("" : : "r"(fmt) : "memory"); }
static volatile u32 sink;
static s32 fctiwz(double x) { double r; union { double d; unsigned long long u; } c;
  __asm__ volatile("fctiwz %0,%1" : "=f"(r) : "f"(x)); c.d = r; return (s32)(u32)c.u; }
static u32 slw(u32 a, u32 n) { u32 r; __asm__ volatile("slw %0,%1,%2" : "=r"(r) : "r"(a), "r"(n)); return r; }
static u32 srw(u32 a, u32 n) { u32 r; __asm__ volatile("srw %0,%1,%2" : "=r"(r) : "r"(a), "r"(n)); return r; }
static s32 sraw(s32 a, u32 n) { s32 r; __asm__ volatile("sraw %0,%1,%2" : "=r"(r) : "r"(a), "r"(n)); return r; }
static s32 divw(s32 a, s32 b) { s32 r; __asm__ volatile("divw %0,%1,%2" : "=r"(r) : "r"(a), "r"(b)); return r; }
static u32 divwu(u32 a, u32 b) { u32 r; __asm__ volatile("divwu %0,%1,%2" : "=r"(r) : "r"(a), "r"(b)); return r; }
void main_probe(void) {
  union { double d; unsigned long long u; } nan = { .u = 0x7FF8000000000000ull }, inf = { .u = 0x7FF0000000000000ull };
  OSReport("REF fctiwz nan %08x\n", (u32)fctiwz(nan.d));
  OSReport("REF fctiwz -nan %08x\n", (u32)fctiwz(-nan.d));
  OSReport("REF fctiwz inf %08x -inf %08x\n", (u32)fctiwz(inf.d), (u32)fctiwz(-inf.d));
  OSReport("REF fctiwz 3e9 %08x -3e9 %08x -1e-5 %08x 2.9 %08x\n", (u32)fctiwz(3e9), (u32)fctiwz(-3e9), (u32)fctiwz(-1e-5), (u32)fctiwz(2.9));
  for (u32 n = 30; n < 66; n += 1) if (n < 34 || n > 62)
    OSReport("REF shift %u slw %08x srw %08x sraw %08x\n", n, slw(0x80000001u, n), srw(0x80000001u, n), (u32)sraw((s32)0x80000001, n));
  OSReport("REF divw 5/0 %08x -5/0 %08x min/-1 %08x\n", (u32)divw(5, 0), (u32)divw(-5, 0), (u32)divw((s32)0x80000000, -1));
  OSReport("REF divwu 5/0 %08x\n", divwu(5, 0));
  OSReport("REF done\n");
  for (;;) sink++;
}
__attribute__((naked, section(".text.start"))) void _start(void) {
  __asm__ volatile("lis 1,0x8170\n ori 1,1,0xff00\n"
                   "mfmsr 3\n ori 3,3,0x2000\n mtmsr 3\n isync\n"   // MSR[FP] on
                   "b main_probe\n");
}
