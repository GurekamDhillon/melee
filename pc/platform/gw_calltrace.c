/* gw_calltrace.c - the call tracer behind tools/rust-port/trace (docs/rust-port/trace/).
 *
 * Only a tracing build uses this. Game TUs compiled through tools/rust-port/trace/gwtool_trace.sh
 * carry an entry hook, `__cyg_profile_func_enter(fn, callsite)`, in every function (inlined ones
 * included, because the hook is inserted before inlining). This file is the hook. It is compiled
 * with every shim but is NOT in the curated link list, so the normal executable never contains it
 * (gw_script.c defaults gw_calltrace_set_frame to a no-op with /alternatename).
 *
 * MELEE_CALLTRACE=<file>   write the table there (text, rewritten whenever it changed and at exit):
 *     #calltrace,v1,base=<hex image base>,k0=<ticks before match frame 0>,f0=<first match frame>,ticks=<n>
 *     <rva hex>,<first tick>,<call count>
 * A "tick" is one gw_Script tick (gw_calltrace_set_frame is called once at its end), so the logic
 * of match frame F runs in tick k0 + F - f0; a first tick below k0 is "before the match" (boot,
 * menus, loading). tools/rust-port/trace/calltrace.py resolves the rvas through melee-pc.map.
 * MELEE_CALLTRACE_SEQ=<file>   also write the ORDERED call list per match frame (consolidation step 1; the Rust
 *     side is `melee golden --calltrace`, compare with tools/rust-port/trace/seqdiff.py):
 *     #calltrace-seq,v1,base=<hex image base>
 *     <match frame>,<rva hex>,<position in the frame>
 *   MELEE_CALLTRACE_FRAMES=a-b limits the frames kept (default all; a 1800-frame run is large). Calls of a tick are
 *   buffered and written at the tick's end, labelled with the match frame given to gw_calltrace_set_frame (negative
 *   frames, before the match, are not written). Every instrumented function is logged, inlined ones included.
 *   MELEE_CALLTRACE may be unset when only the sequence is wanted.
 * Thread-safe (open addressing, CAS on the key); the hook costs a hash probe and one atomic add. */
#include <windows.h>
#include <intrin.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define CT_BITS 17
#define CT_SLOTS (1u << CT_BITS)

typedef struct {
    volatile LONG key;        /* absolute function address, 0 = empty */
    volatile LONG first_tick;
    volatile LONG64 count;
} CtSlot;

static CtSlot ct_tab[CT_SLOTS];
static volatile LONG ct_state;      /* 0 = untouched, 3 = initialising, 1 = on, 2 = off */
static volatile LONG ct_tick;       /* set_frame calls so far */
static volatile LONG ct_dirty;
static volatile LONG ct_k0 = -1, ct_f0;
static LONG ct_last_dump;
static char ct_path[MAX_PATH];
static char ct_seq_path[MAX_PATH];
static FILE *ct_seq_file;
#define CT_SEQ_MAX (1u << 22)
static uint32_t ct_seq_buf[CT_SEQ_MAX];  /* current tick's calls (absolute addresses) */
static volatile LONG ct_seq_n;
static long ct_seq_lo = -1000000, ct_seq_hi = 1000000000;

static void ct_dump(void)
{
    char tmp[MAX_PATH + 8];
    FILE *f;
    uint32_t i;
    if (ct_state != 1 || ct_path[0] == 0) return;
    snprintf(tmp, sizeof tmp, "%s.tmp", ct_path);
    f = fopen(tmp, "w");
    if (f == NULL) return;
    fprintf(f, "#calltrace,v1,base=%lx,k0=%ld,f0=%ld,ticks=%ld\n",
            (unsigned long) (uintptr_t) GetModuleHandleA(NULL), (long) ct_k0, (long) ct_f0, (long) ct_tick);
    for (i = 0; i < CT_SLOTS; i++) {
        LONG key = ct_tab[i].key;
        if (key != 0)
            fprintf(f, "%lx,%ld,%lld\n", (unsigned long) (key - (LONG) (uintptr_t) GetModuleHandleA(NULL)),
                    (long) ct_tab[i].first_tick, (long long) ct_tab[i].count);
    }
    fclose(f);
    MoveFileExA(tmp, ct_path, MOVEFILE_REPLACE_EXISTING);
}

static void ct_exit(void) { ct_dump(); }

static int ct_on(void)
{
    LONG s = ct_state;
    if (s == 1) return 1;
    if (s != 0) return 0;
    if (_InterlockedCompareExchange(&ct_state, 3, 0) != 0) return 0;
    {
        const char *p = getenv("MELEE_CALLTRACE");
        const char *q = getenv("MELEE_CALLTRACE_SEQ");
        const char *r = getenv("MELEE_CALLTRACE_FRAMES");
        if (p != NULL && p[0] != 0 && strlen(p) < sizeof ct_path) strcpy(ct_path, p);
        if (q != NULL && q[0] != 0 && strlen(q) < sizeof ct_seq_path) {
            strcpy(ct_seq_path, q);
            ct_seq_file = fopen(ct_seq_path, "w");
            if (ct_seq_file != NULL)
                fprintf(ct_seq_file, "#calltrace-seq,v1,base=%lx\n", (unsigned long) (uintptr_t) GetModuleHandleA(NULL));
            if (r != NULL) sscanf(r, "%ld-%ld", &ct_seq_lo, &ct_seq_hi);
        }
        if (ct_path[0] == 0 && ct_seq_file == NULL) {
            ct_state = 2;
            return 0;
        }
        atexit(ct_exit);
        ct_state = 1;
    }
    return 1;
}

void __cyg_profile_func_enter(void *fn, void *callsite)
{
    LONG key = (LONG) (uintptr_t) fn;
    uint32_t i;
    (void) callsite;
    if (ct_state != 1 && !ct_on()) return;
    if (ct_seq_file != NULL) {
        LONG n = _InterlockedIncrement(&ct_seq_n) - 1;
        if ((uint32_t) n < CT_SEQ_MAX) ct_seq_buf[n] = (uint32_t) key;
    }
    i = (((uint32_t) key >> 2) * 2654435761u) >> (32 - CT_BITS);
    for (;;) {
        LONG cur = ct_tab[i].key;
        if (cur == 0) {
            if (_InterlockedCompareExchange(&ct_tab[i].key, key, 0) == 0) {
                ct_tab[i].first_tick = ct_tick;
                ct_dirty = 1;
                break;
            }
            cur = ct_tab[i].key;
        }
        if (cur == key) break;
        i = (i + 1) & (CT_SLOTS - 1);
    }
    _InterlockedIncrement64(&ct_tab[i].count);
}

void gw_calltrace_set_frame(int match_frame)
{
    LONG t;
    if (ct_state != 1 && !ct_on()) return;
    t = ct_tick; /* the tick that just ended */
    if (ct_seq_file != NULL) {
        LONG n = ct_seq_n;
        LONG base = (LONG) (uintptr_t) GetModuleHandleA(NULL);
        if (match_frame >= 0 && match_frame >= ct_seq_lo && match_frame <= ct_seq_hi) {
            LONG i;
            if (n > (LONG) CT_SEQ_MAX) {
                fprintf(ct_seq_file, "#overflow,%d,%ld\n", match_frame, (long) n);
                n = CT_SEQ_MAX;
            }
            for (i = 0; i < n; i++)
                fprintf(ct_seq_file, "%d,%lx,%ld\n", match_frame, (unsigned long) (ct_seq_buf[i] - (uint32_t) base), (long) i);
            fflush(ct_seq_file);
        }
        ct_seq_n = 0;
    }
    if (match_frame >= 0 && ct_k0 < 0) {
        ct_k0 = t;
        ct_f0 = match_frame;
    }
    ct_tick = t + 1;
    if ((ct_dirty && t - ct_last_dump >= 30) || t - ct_last_dump >= 600) {
        ct_dirty = 0;
        ct_last_dump = t;
        ct_dump();
    }
}
