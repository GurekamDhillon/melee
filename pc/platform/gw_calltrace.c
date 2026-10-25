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

static void ct_dump(void)
{
    char tmp[MAX_PATH + 8];
    FILE *f;
    uint32_t i;
    if (ct_state != 1) return;
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
        if (p == NULL || p[0] == 0 || strlen(p) >= sizeof ct_path) {
            ct_state = 2;
            return 0;
        }
        strcpy(ct_path, p);
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
