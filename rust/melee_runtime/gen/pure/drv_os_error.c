/* C oracle for melee_runtime::tu::os_error::OSReport: OSReport copied from libs/dolphin/src/dolphin/os/OSError.c
 * (that file does not compile natively: PPC section attributes) and called with random arguments; each call's output is followed by a marker line "@@ <case>". */
#include "common.h"
extern int printf(const char*, ...);
extern int snprintf(char*, unsigned, const char*, ...);
/* OSReport (libs/dolphin/src/dolphin/os/OSError.c) is vprintf(msg, args): the same text through snprintf
 * (that file does not compile natively - PPC section attributes - and the toolchain's vprintf is not linkable). */
#define OSReport(...) do { char b_[1024]; snprintf(b_, sizeof b_, __VA_ARGS__); printf("%s", b_); } while (0)
extern int fflush(void*);

static const char* STRS[4] = { "alpha", "Beta gamma", "", "a-much-longer-string" };

int main(void)
{
    int k;
    for (k = 0; k < 200; k++) {
        int a = (int) rnd(), b = (int) (rnd() % 100000) - 50000;
        unsigned u = (unsigned) rnd();
        double d = (double) rndf(), e = (double) rndf() * 1000.0;
        const char* s = STRS[rnd() % 4];
        char c = (char) ('a' + rnd() % 26);
        OSReport("%s:%d: couldn t get gobj(id=%d)\n", s, b, a);
        printf("@@ %d.1\n", k);
        OSReport("talk no = %d\n", a);
        printf("@@ %d.2\n", k);
        OSReport("*** Beam Damage! %f\n", d);
        printf("@@ %d.3\n", k);
        OSReport("pos = %f : tgrpos = %f\n", d, e);
        printf("@@ %d.4\n", k);
        OSReport("oioi... %08x\n", u);
        printf("@@ %d.5\n", k);
        OSReport("%3d|%-5d|%+d|%05d\n", b, b, b, b);
        printf("@@ %d.6\n", k);
        OSReport("%x %X %o %u %c\n", u, u, u, u, c);
        printf("@@ %d.7\n", k);
        OSReport("%.2f %8.3f %5.1f\n", d, d, e);
        printf("@@ %d.8\n", k);
        OSReport("%s|%10s|%-10s|%.3s\n", s, s, s, s);
        printf("@@ %d.9\n", k);
        OSReport("no conversions here\n");
        printf("@@ %d.10\n", k);
    }
    return 0;
}
