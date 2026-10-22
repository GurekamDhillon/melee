/* Shared helpers of the melee_modes TU state oracles (tests/tu_<stem>.rs mirror every function
 * here in tests/common/mod.rs, token for token).
 *
 * A driver #includes the game TU verbatim, then this header, then the generated stubs
 * (<stem>_stubs.inc), then runs seeded cases on a synthetic game state. Every stub and every
 * dump prints one text line; the Rust test replays the same cases on the Rust port with the
 * mirrored helpers and compares the whole output. Pointers are never printed raw: they print
 * as an arena offset (a<off>), a registered static (name+off), a known function (f:name) or
 * "x" (anything else).
 */
#ifndef ORACLE_H
#define ORACLE_H
#include <stdio.h>
#include <string.h>
extern char* getenv(const char*);
extern void exit(int);
extern int _flushall(void);
static void dbgflush(void)
{
    if (getenv("ORACLE_UNBUF")) _flushall();
}

typedef unsigned long long ou64;
typedef long long oi64;

static unsigned int ORS = 1;
static unsigned int rnd(void)
{
    ORS ^= ORS << 13;
    ORS ^= ORS >> 17;
    ORS ^= ORS << 5;
    return ORS;
}
static void rseed(unsigned int s) { ORS = s ? s : 1; }
/* sane floats: zeros, tiny, and a wide range; never NaN/inf (their payload propagation is not
 * what the port is about) */
static float rf(void)
{
    unsigned int k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    if (k == 2) return (float) (rnd() % 7) * 1e-4f;
    return ((float) ((int) (rnd() % 200001) - 100000)) / 37.0f;
}
static unsigned int fbits(float f) { union { float f; unsigned int u; } c; c.f = f; return c.u; }
static ou64 dbits(double f) { union { double f; ou64 u; } c; c.f = f; return c.u; }

#define ARENA_SZ (1u << 22)
extern void* VirtualAlloc(void*, unsigned long long, unsigned long, unsigned long);
static unsigned char* ARENA_P;
/* the arena lives below 4 GB at a fixed address: the game casts pointers through int/u32 */
static void arena_init(void)
{
    if (!ARENA_P) ARENA_P = (unsigned char*) VirtualAlloc((void*) 0x20000000ull, ARENA_SZ, 0x3000, 4);
    if (!ARENA_P) { printf("arena_init failed"); exit(9); }
}
static ou64 ARENA_USED = 64;
static void arena_reset(void)
{
    memset(ARENA_P, 0, (unsigned long) ARENA_USED);
    ARENA_USED = 64;
}
static void* AZ(ou64 n)
{
    void* p = ARENA_P + ARENA_USED;
    ARENA_USED += (n + 15) & ~15ull;
    return p;
}

static void* AZR(ou64 n);
#define DUMPARENA() dump("arena", ARENA_P + 64, ARENA_USED - 64)
typedef struct { const char* name; const unsigned char* p; ou64 n; } OReg;
static OReg OREGS[64];
static int NOREG;
static void reg(const char* name, const void* p, ou64 n)
{
    OREGS[NOREG].name = name;
    OREGS[NOREG].p = (const unsigned char*) p;
    OREGS[NOREG].n = n;
    NOREG++;
}
typedef struct { const char* name; ou64 f; } OFn;
static OFn OFNS[128];
static int NOFN;
static void fnreg(const char* name, const void* f)
{
    OFNS[NOFN].name = name;
    OFNS[NOFN].f = (ou64) (unsigned long long) f;
    NOFN++;
}

/* " a<off>" / " name+off" / " f:name" / " null" / " x" */
static void trp(const void* p)
{
    dbgflush();
    ou64 v = (ou64) (unsigned long long) p;
    int i;
    if (v == 0) { printf(" null"); return; }
    if (v >= (ou64) (unsigned long long) ARENA_P && v < (ou64) (unsigned long long) ARENA_P + ARENA_SZ) {
        printf(" a%llu", v - (ou64) (unsigned long long) ARENA_P);
        return;
    }
    for (i = 0; i < NOREG; i++) {
        ou64 b = (ou64) (unsigned long long) OREGS[i].p;
        if (v >= b && v < b + OREGS[i].n) { printf(" %s+%llu", OREGS[i].name, v - b); return; }
    }
    for (i = 0; i < NOFN; i++) {
        if (OFNS[i].f == v) { printf(" f:%s", OFNS[i].name); return; }
    }
    printf(" x");
}
#define TRI(x) printf(" %lld", (oi64) (x))
#define TRU(x) printf(" %llu", (ou64) (x))
#define TRF(x) printf(" %08x", fbits(x))
#define TRD(x) printf(" %016llx", dbits(x))
#define TRP(x) trp((const void*) (x))
/* a record passed by value: its bytes */
static void trrec(const void* p, ou64 n)
{
    const unsigned char* b = (const unsigned char*) p;
    ou64 i;
    printf(" {");
    for (i = 0; i < n; i++) printf("%02x", (unsigned) b[i]);
    printf("}");
}
#define TRREC(p, n) trrec((const void*) (p), (ou64) (n))
/* a C string literal (rodata): printed by content, an arena/static pointer by position */
static void trs(const char* s)
{
    if (s == 0) { printf(" null"); return; }
    {
        ou64 v = (ou64) (unsigned long long) s;
        if (v >= (ou64) (unsigned long long) ARENA_P && v < (ou64) (unsigned long long) ARENA_P + ARENA_SZ) { trp(s); return; }
    }
    {
        const unsigned char* q = (const unsigned char*) s;
        printf(" \"");
        for (; *q; q++) {
            if (*q < 0x20 || *q >= 0x7f) printf("\\x%02x", (unsigned) *q);
            else printf("%c", (int) *q);
        }
        printf("\"");
    }
}
#define TRS(x) trs((const char*) (x))

static ou64 HACC = 0xcbf29ce484222325ull;
/* FNV-1a over the bytes with every pointer-sized word that points into the arena, a registered
 * static or a known function replaced by its symbolic form */
static void dump(const char* tag, const void* p, ou64 n)
{
    const unsigned char* b = (const unsigned char*) p;
    ou64 h = 0xcbf29ce484222325ull, i, j;
    for (i = 0; i < n; i++) {
        unsigned char c = b[i];
        if ((i & 7) == 0 && i + 8 <= n) {
            ou64 w = 0;
            int k;
            for (j = 0; j < 8; j++) w |= (ou64) b[i + j] << (8 * j);
            if (w >= (ou64) (unsigned long long) ARENA_P && w < (ou64) (unsigned long long) ARENA_P + ARENA_SZ) {
                w = 0xA0A0000000000000ull | (w - (ou64) (unsigned long long) ARENA_P);
            } else {
                for (k = 0; k < NOREG; k++) {
                    ou64 base = (ou64) (unsigned long long) OREGS[k].p;
                    if (w >= base && w < base + OREGS[k].n) { w = 0xB0B0000000000000ull | ((ou64) k << 32) | (w - base); break; }
                }
                if (k == NOREG) {
                    for (k = 0; k < NOFN; k++) if (OFNS[k].f == w) { w = 0xC0C0000000000000ull | (ou64) k; break; }
                }
            }
            for (j = 0; j < 8; j++) {
                unsigned char x = (unsigned char) (w >> (8 * j));
                h ^= x;
                h *= 0x100000001b3ull;
            }
            i += 7;
            continue;
        }
        h ^= c;
        h *= 0x100000001b3ull;
    }
    if (getenv("ORACLE_VERBOSE")) {
        printf("dump %s %llu %016llx\n", tag, n, h);
        if (getenv("ORACLE_HEX")) {
            ou64 q;
            printf("hex");
            for (q = 0; q < n; q++) printf(" %02x", b[q]);
            printf("\n");
        }
    } else {
        HACC = (HACC ^ h) * 0x100000001b3ull;
        HACC ^= n;
    }
}
/* end of a case's state: the folded hash of every dump since the last one */
static void state_end(void)
{
    if (!getenv("ORACLE_VERBOSE")) printf("state %016llx\n", HACC);
    HACC = 0xcbf29ce484222325ull;
}
#define DUMP(tag, obj) dump(tag, &(obj), sizeof(obj))
#define SIZE(T) printf("sizeof " #T " %llu\n", (ou64) sizeof(T))
#define CASE(i) (printf("case %d\n", (int) (i)), dbgflush())

/* fill n bytes with the shared byte stream */
static void rfill(void* p, ou64 n);
static void* AZR(ou64 n)
{
    void* p = AZ(n);
    rfill(p, n);
    return p;
}
static void rfill(void* p, ou64 n)
{
    unsigned char* b = (unsigned char*) p;
    ou64 i;
    for (i = 0; i < n; i++) b[i] = (unsigned char) (rnd() >> 11);
}
#endif
