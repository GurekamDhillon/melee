/* gw_snap.c - in-process savestates and SyncTest: the foundation of rollback.
 *
 *   MELEE_SYNCTEST=<k>   during MELEE_SLP playback, roll back k frames EVERY frame and resimulate
 *                        them on the replay's inputs, comparing each resimulated frame's state with
 *                        the first pass. Any difference is state rollback would get wrong.
 *
 * WHAT A SNAPSHOT IS (_research/rollback-netcode.md section 2):
 *   - MEM1, all 24 MB at 0x80000000 (heaps, HSD objects, loaded files) - strategy "copy it all"
 *     first; exclusions are carved out as SyncTest proves they are needed.
 *   - The game's own native globals: every section-3 (.data/.bss) symbol of a game object
 *     (src_melee_*, src_sysdolphin_*, libs_dolphin_*, and the game's commons `_gw_*`), found by
 *     walking melee-pc.map beside the exe, minus an exclusion list (audio, pad queue, video, perf,
 *     card, movies, devcom, rumble). Renderer, CRT, shim and m-ex runtime state is never touched.
 *   - The replay module's cursor (gw_replay.c), which is native and decides which inputs a frame
 *     gets.
 *
 * SYNCTEST (GGPO's synctest, in the scene loop, gmscene.c gm_801A4D34): with the replay armed, a
 * render tick runs k+1 logic iterations instead of 1. The first saves S[F] (the state at the start
 * of the new frame F), loads S[F-k] and runs F-k; the next k-1 run F-k+1..F-1; the last runs F. At
 * the start of every one after the load, the live state is compared byte for byte with the
 * first-pass snapshot of that frame - so a mismatch names the first frame and the exact bytes, and
 * gw_snap_dump attributes them to map symbols or MEM1 addresses. While resimulating, sound effects
 * are suppressed (gw_Snap_Resimulating, read by lbAudioAx).
 */
#include "gw.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>

#define GW_SNAP_MAX_RANGES 4096
#define GW_SNAP_MAX_SYMS 8192
#define GW_SNAP_MAX_SLOTS 16

typedef struct {
    uint32_t va;
    uint32_t len;
} GwSnapRange;

typedef struct {
    uint32_t va;
    uint32_t len;
    char name[64];
    char obj[48];
} GwSnapSym;

typedef struct {
    int frame; /* the frame this is the START of; INT_MIN = empty */
    uint8_t *mem1;
    uint8_t *globals;
    int replay_cursor[4];
    /* DIRTY-PAGE MODE: a superset of the pages where the live MEM1 may differ from this slot's
       copy (1 bit per 4 KB page). Live writes (GetWriteWatch) are OR'd into every slot's set; a
       save copies only the pages in the chosen slot's set; a load copies only the pages in the
       target's set back and folds that set into every other slot's (their difference from the new
       live state is at most (their set) | (the target's)). */
    uint64_t *dirty;
    uint64_t hash; /* gw_snap_hash() at save time, 0 in full mode */
} GwSnapSlot;

/* rumble state: reset to idle on every load (see sn_scatter) */
static GwSnapRange sn_rumble[64];
static int sn_nrumble;
static uint32_t sn_devcom_roots[16]; /* devcom list heads, resolved from the map */
static int sn_ndevcom_roots;

static struct {
    int tried, enabled, k;
    int ready;
    int session; /* a rollback session (gw_rollback.c) owns the snapshots: no SyncTest measuring */
    GwSnapRange ranges[GW_SNAP_MAX_RANGES];
    int nranges;
    uint32_t globals_len;
    GwSnapSym syms[GW_SNAP_MAX_SYMS]; /* section 3, for attributing differences */
    int nsyms;
    GwSnapSlot slot[GW_SNAP_MAX_SLOTS];
    int nslots;
    uint8_t *cmp_globals; /* scratch for gathering the live globals */
    volatile int *deferred_count;
    uint32_t particle_list; /* psdisp.c's particle_list[17] */
    GwSnapRange cmp_skip[64]; /* saved and restored, but render-owned: not compared */
    int ncmp_skip;
    GwSnapRange mask[8192];   /* per compare: live render-owned heap objects */
    int nmask;
    volatile int *ppc_depth;
    /* RENDER-OWNED bytes, measured: every byte the render pass changes is masked out of
       comparisons from then on (it is still saved and restored) */
    uint8_t *pre_mem1, *pre_globals;
    uint8_t *rmask_mem1, *rmask_globals; /* 1 bit per byte */
    int pre_valid;
    uint32_t rmask_bytes;
    /* the tick plan */
    int plan_rollback; /* the next iteration loads S[F-k] */
    int target;        /* F: the new frame of this tick */
    int resim;         /* inside a resimulation */
    int cur_is_resim;  /* the iteration now running re-runs a frame (F-k..F-1) */
    int mismatches;
    int checked;
    int passes;
    double ms_save, ms_load, ms_cmp;
    int n_save, n_load, n_cmp;
    /* dirty-page tracking (MELEE_SNAP_MODE=dirty, the default; =full copies everything) */
    int dirty_mode, verify, hash_on;
    uint32_t npages;
    uint64_t *hash_dirty;     /* pages written since gw_snap_hash last looked */
    uint64_t *page_h;         /* the live pages' hashes as of that look */
    uint64_t hash_mem;        /* combined page hashes (MEM1 part) */
    uint32_t hash_node_pg[64]; /* pages forced dirty last call (async nodes zeroed in the hash) */
    int n_hash_node_pg;
    uint64_t hash_ready;
    double ms_poll, ms_hash;
    long n_poll, n_dirty_pages, n_copy_pages, n_hash_pages, n_hash_calls;
    long n_verify_fail;
} sn = { 0 };

uint64_t gw_snap_hash(void);
void gw_Snap_Time(int what, int begin);
extern int gw_Replay_Frame(void);
extern void gw_Replay_GetCursor(int out[4]);
extern void gw_Replay_SetCursor(const int in[4]);

static double sn_ms(void) {
    LARGE_INTEGER f, c;
    QueryPerformanceFrequency(&f);
    QueryPerformanceCounter(&c);
    return (double) c.QuadPart * 1000.0 / (double) f.QuadPart;
}

/* ---- the state range table, from the map ---------------------------------------------------- */

static int sn_game_object(const char *obj, const char *name) {
    static const char *const excl_obj[] = {
        "src_melee_lb_lbaudio_ax.c.obj",          /* sound: the synth is never rolled back */
        "src_sysdolphin_baselib_synth.c.obj",
        "src_sysdolphin_baselib_axdriver.c.obj",
        "src_sysdolphin_baselib_audio.c.obj",
        "src_sysdolphin_baselib_video.c.obj",     /* VI / XFB */
        "src_sysdolphin_baselib_perf.c.obj",
        "src_sysdolphin_baselib_state.c.obj",    /* shadow of the HOST GX state: restoring it desyncs from the GPU */
        "src_sysdolphin_baselib_memory.c.obj",   /* heap-usage diagnostics only (hsd_allocs, caller_hits) */
        "src_sysdolphin_baselib_devcom.c.obj",
        "src_melee_lb_lbcardgame.c.obj",          /* memory card */
        "src_melee_lb_lbcardnew.c.obj",
        "src_melee_lb_lbmthp.c.obj",              /* movies */
        "src_melee_lb_lbsnap.c.obj",
        "libs_dolphin_src_dolphin_thp_THPDec.c.obj",
    };
    static const char *const excl_sym[] = {
        "_gw_HSD_PadLibData", /* the raw pad queue: filled by the pad alarm, not by logic */
        "_gmMain_8046B108",   /* its storage */
        "_gw_HSD_VIData",     /* VI retrace/XFB bookkeeping: the renderer's, not the game's */
        "_gw_start_time",     /* wall-clock play time (lbtime) - differs between any two passes */
    };
    size_t i;
    int game = strncmp(obj, "src_melee_", 10) == 0 || strncmp(obj, "src_sysdolphin_", 15) == 0 ||
               strncmp(obj, "libs_dolphin_", 13) == 0 ||
               (strcmp(obj, "<common>") == 0 && strncmp(name, "_gw_", 4) == 0);
    if (!game) {
        return 0;
    }
    for (i = 0; i < sizeof excl_obj / sizeof excl_obj[0]; ++i) {
        if (strcmp(obj, excl_obj[i]) == 0) {
            return 0;
        }
    }
    for (i = 0; i < sizeof excl_sym / sizeof excl_sym[0]; ++i) {
        if (strcmp(name, excl_sym[i]) == 0) {
            return 0;
        }
    }
    return 1;
}

static int sn_sym_cmp(const void *a, const void *b) {
    const GwSnapSym *x = (const GwSnapSym *) a, *y = (const GwSnapSym *) b;
    return x->va < y->va ? -1 : x->va > y->va;
}

static int sn_load_map(void) {
    char path[MAX_PATH];
    char line[1024];
    FILE *f;
    uint32_t sec3_end = 0, sec3_start = 0, sec3_base = 0;
    int i;
    GetModuleFileNameA(NULL, path, sizeof path);
    {
        char *dot = strrchr(path, '.');
        if (dot == NULL) {
            return -1;
        }
        strcpy(dot, ".map");
    }
    f = fopen(path, "r");
    if (f == NULL) {
        gw_log("snap: no map beside the exe (%s)", path);
        return -1;
    }
    sn.nsyms = 0;
    while (fgets(line, sizeof line, f) != NULL) {
        unsigned off, len, va;
        char a[256], b[256], c[64];
        /* section table: " 0003:00000000 0006ebe8H .data  DATA" */
        if (sscanf(line, " 0003:%x %xH %255s", &off, &len, a) == 3 && line[14] == ' ') {
            if (strcmp(a, ".data") == 0 || strcmp(a, ".bss") == 0 || strncmp(a, ".data$", 6) == 0) {
                if (sec3_end < off + len) sec3_end = off + len;
            }
            continue;
        }
        /* symbols: " 0003:0000a1b0       _name        1000xxxx f?  obj" */
        if (sscanf(line, " 0003:%x %255s %x %255s %63s", &off, a, &va, b, c) >= 4) {
            const char *obj = strcmp(b, "f") == 0 || strcmp(b, "i") == 0 ? c : b;
            if (sn.nsyms < GW_SNAP_MAX_SYMS) {
                GwSnapSym *s = &sn.syms[sn.nsyms++];
                s->va = va;
                s->len = 0;
                snprintf(s->name, sizeof s->name, "%s", a);
                snprintf(s->obj, sizeof s->obj, "%s", obj);
            }
            if (sec3_base == 0) sec3_base = va - off;
            if (strcmp(a, "_gw_deferred_count") == 0) sn.deferred_count = (volatile int *) (uintptr_t) va;
            if (strcmp(a, "_particle_list") == 0) sn.particle_list = va;
            /* devcom's request-list heads (devcom.static.h): every HSD_DevCom reachable from
               these is asynchronous state that a load must leave alone. */
            if (strcmp(a, "_devComStatus") == 0 || strcmp(a, "_HSD_DevCom_804C6330") == 0 ||
                strcmp(a, "_HSD_DevCom_804D77F0") == 0 || strcmp(a, "_HSD_DevCom_804D77FC") == 0 ||
                strcmp(a, "_dvdDC") == 0 || strcmp(a, "_aramDC") == 0) {
                int slots = strcmp(a, "_devComStatus") == 0 || strcmp(a, "_HSD_DevCom_804C6330") == 0
                                ? 4
                                : (strcmp(a, "_HSD_DevCom_804D77FC") == 0 ? 2 : 1);
                int k;
                for (k = 0; k < slots && sn_ndevcom_roots < 16; ++k) {
                    sn_devcom_roots[sn_ndevcom_roots++] = va + 4u * (uint32_t) k;
                }
            }
            if (strcmp(a, "_gw_ppc_depth") == 0) sn.ppc_depth = (volatile int *) (uintptr_t) va;
        }
    }
    fclose(f);
    qsort(sn.syms, (size_t) sn.nsyms, sizeof sn.syms[0], sn_sym_cmp);
    sec3_start = sec3_base;
    for (i = 0; i < sn.nsyms; ++i) {
        uint32_t end = i + 1 < sn.nsyms ? sn.syms[i + 1].va : sec3_start + sec3_end;
        sn.syms[i].len = end > sn.syms[i].va ? end - sn.syms[i].va : 0;
    }
    /* merge the game symbols into ranges */
    sn.nranges = 0;
    sn.globals_len = 0;
    for (i = 0; i < sn.nsyms; ++i) {
        GwSnapSym *s = &sn.syms[i];
        if (s->len == 0 || s->len > 0x400000 || !sn_game_object(s->obj, s->name)) {
            continue;
        }
        if (sn.nranges > 0 &&
            sn.ranges[sn.nranges - 1].va + sn.ranges[sn.nranges - 1].len == s->va) {
            sn.ranges[sn.nranges - 1].len += s->len;
        } else if (sn.nranges < GW_SNAP_MAX_RANGES) {
            sn.ranges[sn.nranges].va = s->va;
            sn.ranges[sn.nranges].len = s->len;
            ++sn.nranges;
        }
        sn.globals_len += s->len;
    }
    /* RENDER-OWNED state: advanced by the render pass, which resimulated frames never get.
       Saved and restored with everything else (so it stays consistent with MEM1), but not
       compared - a difference there is not a logic difference. Evidence: SyncTest k=1's first
       mismatch (frame -121) was entirely psdisp.c (the particle DISPLAY: sort lists, per-render
       frame stamps, view matrices) plus gm_80479D58.unk_4, the scene loop's render counter. */
    sn.ncmp_skip = 0;
    for (i = 0; i < sn.nsyms && sn.ncmp_skip < 64; ++i) {
        GwSnapSym *s = &sn.syms[i];
        if (s->len == 0) continue;
        if (strcmp(s->obj, "src_sysdolphin_baselib_psdisp.c.obj") == 0) {
            sn.cmp_skip[sn.ncmp_skip].va = s->va;
            sn.cmp_skip[sn.ncmp_skip].len = s->len;
            ++sn.ncmp_skip;
        } else if (strcmp(s->name, "_gw_HSD_PadMasterStatus") == 0 ||
                   strcmp(s->name, "_gw_HSD_PadGameStatus") == 0 ||
                   strcmp(s->name, "_gw_HSD_PadCopyStatus") == 0 ||
                   strcmp(s->name, "_controller_map") == 0) {
            /* _controller_map (gm_1A36.c) is the menu button/trigger/repeat map that
               gm_EvaluateAllControllerInputs rebuilds every frame from the live pad - the same
               family as the pad statuses. SyncTest k=7 found it (frame 2148: one byte, 0x00 vs
               0x20, from the live pad's state at the moment of the resimulated frame). */
            /* INPUTS, renewed from the live pad queue each logic frame: during playback fighters
               take the replay's inputs instead, so these are not state to compare */
            sn.cmp_skip[sn.ncmp_skip].va = s->va;
            sn.cmp_skip[sn.ncmp_skip].len = s->len;
            ++sn.ncmp_skip;
        } else if (strcmp(s->obj, "src_sysdolphin_baselib_rumble.c.obj") == 0 ||
                   strcmp(s->name, "_gw_HSD_Rumble_804C22E0") == 0 ||
                   strcmp(s->name, "_gmMain_8046B1F8") == 0) {
            /* rumble: advanced by the pad side (per host poll, not per logic frame). It is SAVED and
               restored - its scripts point into the heap, and leaving it stale crashed the run in
               HSD_PadRumbleInterpret1 - but a difference in it is not a logic difference. */
            sn.cmp_skip[sn.ncmp_skip].va = s->va;
            sn.cmp_skip[sn.ncmp_skip].len = s->len;
            ++sn.ncmp_skip;
            if (sn_nrumble < 64) {
                sn_rumble[sn_nrumble].va = s->va;
                sn_rumble[sn_nrumble].len = s->len;
                ++sn_nrumble;
            }
        } else if (strcmp(s->name, "_gm_80479D58") == 0) {
            sn.cmp_skip[sn.ncmp_skip].va = s->va + 4; /* unk_4, the render counter */
            sn.cmp_skip[sn.ncmp_skip].len = 4;
            ++sn.ncmp_skip;
        }
    }
    gw_log("snap: %d section-3 symbols, %d game ranges, %u bytes of game globals; MEM1 %u bytes",
           sn.nsyms, sn.nranges, sn.globals_len, gw_mem1_size);
    return 0;
}

/* ---- write-watch: which MEM1 pages changed ------------------------------------------------- */

#define SN_PAGE 4096u
#define SN_BM_WORDS(n) (((n) + 63u) / 64u)

extern int gw_mem1_watched;

static void sn_bm_set(uint64_t *bm, uint32_t pg) { bm[pg >> 6] |= 1ull << (pg & 63); }
static int sn_bm_test(const uint64_t *bm, uint32_t pg) { return (int) ((bm[pg >> 6] >> (pg & 63)) & 1); }
static void sn_bm_fill(uint64_t *bm, uint32_t npages) {
    uint32_t i;
    memset(bm, 0, SN_BM_WORDS(npages) * 8);
    for (i = 0; i < npages; ++i) {
        sn_bm_set(bm, i);
    }
}

/* Ask Windows which pages changed since the last poll (and reset the watch), and record them in
 * every slot's dirty set and the hash's. Everything that reads or writes the sets calls this first. */
static void sn_poll(void) {
    static PVOID addrs[6144 + 64];
    ULONG_PTR count = sn.npages;
    DWORD gran = 0;
    double t0 = sn_ms();
    UINT r;
    int i;
    ULONG_PTR c;
    if (!sn.dirty_mode) {
        return;
    }
    r = GetWriteWatch(WRITE_WATCH_FLAG_RESET, (PVOID) (uintptr_t) 0x80000000u, gw_mem1_size, addrs,
                      &count, &gran);
    if (r != 0) {
        /* cannot tell: everything is suspect */
        for (i = 0; i < sn.nslots; ++i) {
            sn_bm_fill(sn.slot[i].dirty, sn.npages);
        }
        if (sn.hash_dirty != NULL) {
            sn_bm_fill(sn.hash_dirty, sn.npages);
        }
        return;
    }
    for (c = 0; c < count; ++c) {
        uint32_t pg = (uint32_t) (((uintptr_t) addrs[c] - 0x80000000u) / SN_PAGE);
        if (pg >= sn.npages) {
            continue;
        }
        for (i = 0; i < sn.nslots; ++i) {
            sn_bm_set(sn.slot[i].dirty, pg);
        }
        if (sn.hash_dirty != NULL) {
            sn_bm_set(sn.hash_dirty, pg);
        }
    }
    sn.n_poll++;
    sn.n_dirty_pages += (long) count;
    sn.ms_poll += sn_ms() - t0;
}

static void sn_watch_reset(void) {
    if (sn.dirty_mode) {
        ResetWriteWatch((PVOID) (uintptr_t) 0x80000000u, gw_mem1_size);
    }
}

/* Copy every page of `bm` between `live` and `slot` (dir 0: live -> slot, 1: slot -> live), merging
 * neighbours into one memcpy. Returns the pages copied. */
static uint32_t sn_copy_pages(const uint64_t *bm, uint8_t *slot, int dir) {
    uint8_t *live = (uint8_t *) (uintptr_t) 0x80000000u;
    uint32_t pg = 0, copied = 0;
    while (pg < sn.npages) {
        uint32_t start, n;
        if (!sn_bm_test(bm, pg)) {
            /* skip whole clear words fast */
            if ((pg & 63) == 0 && bm[pg >> 6] == 0) {
                pg += 64;
            } else {
                ++pg;
            }
            continue;
        }
        start = pg;
        while (pg < sn.npages && sn_bm_test(bm, pg)) {
            ++pg;
        }
        n = pg - start;
        if (dir == 0) {
            memcpy(slot + (size_t) start * SN_PAGE, live + (size_t) start * SN_PAGE, (size_t) n * SN_PAGE);
        } else {
            memcpy(live + (size_t) start * SN_PAGE, slot + (size_t) start * SN_PAGE, (size_t) n * SN_PAGE);
        }
        copied += n;
    }
    return copied;
}

/* ---- a fast hash, and the incremental state hash ------------------------------------------------
 *
 * 64-bit, four independent lanes of multiply/xorshift over 32-byte blocks (~6-8 GB/s single
 * thread; a 4 KB page is ~0.6 us). Self-contained: no dependency. Not cryptographic - it detects
 * desyncs and differences, nothing more. */
static uint64_t sn_h64(const uint8_t *p, size_t n, uint64_t seed) {
    const uint64_t K1 = 0x9E3779B185EBCA87ull, K2 = 0xC2B2AE3D27D4EB4Full;
    uint64_t a = seed + K1, b = seed ^ K2, c = seed * K1 + 1, d = ~seed;
    size_t i;
    for (i = 0; i + 32 <= n; i += 32) {
        uint64_t w0, w1, w2, w3;
        memcpy(&w0, p + i, 8);
        memcpy(&w1, p + i + 8, 8);
        memcpy(&w2, p + i + 16, 8);
        memcpy(&w3, p + i + 24, 8);
        a = (a ^ w0) * K1; a ^= a >> 31;
        b = (b ^ w1) * K2; b ^= b >> 29;
        c = (c ^ w2) * K1; c ^= c >> 32;
        d = (d ^ w3) * K2; d ^= d >> 27;
    }
    for (; i < n; ++i) {
        a = (a ^ p[i]) * K1;
    }
    a ^= (b << 21 | b >> 43) + (c << 42 | c >> 22) * K2 + d;
    a ^= a >> 33; a *= K2; a ^= a >> 29; a *= K1; a ^= a >> 32;
    return a;
}

/* ---- save / load / compare -------------------------------------------------------------------- */

static void sn_gather(uint8_t *out) {
    int i;
    for (i = 0; i < sn.nranges; ++i) {
        memcpy(out, (const void *) (uintptr_t) sn.ranges[i].va, sn.ranges[i].len);
        out += sn.ranges[i].len;
    }
}

/* NOT used for the ordinary restore - see gw_snap_load. Kept for the places that want the
 * not-written-by-logic mask honoured. */
static void sn_restore(uint8_t *dst, const uint8_t *src, uint32_t len, const uint8_t *mask,
                       uint32_t bit0) {
    uint32_t off;
    for (off = 0; off < len; off += 4096) {
        uint32_t n = len - off < 4096 ? len - off : 4096, j;
        /* the mask BYTES covering this block, so the common clean block costs one memcmp-sized
           scan of 512 bytes rather than 4096 bit tests */
        uint32_t mb = (bit0 + off) >> 3, me = (bit0 + off + n - 1) >> 3;
        int dirty = 0;
        for (j = mb; j <= me; ++j) {
            if (mask[j] != 0) {
                dirty = 1;
                break;
            }
        }
        if (!dirty) {
            memcpy(dst + off, src + off, n);
            continue;
        }
        for (j = 0; j < n; ++j) {
            uint32_t b = bit0 + off + j;
            if (!(mask[b >> 3] & (1u << (b & 7)))) {
                dst[off + j] = src[off + j];
            }
        }
    }
}

static void sn_scatter(const uint8_t *in) {
    int i;
    for (i = 0; i < sn.nranges; ++i) {
        memcpy((void *) (uintptr_t) sn.ranges[i].va, in, sn.ranges[i].len);
        in += sn.ranges[i].len;
    }
    /* Rumble is a cosmetic, pad-side script interpreter: restoring it mid-script rolled its
       cursor back into a loop (k=7 hung inside HSD_PadRumbleInterpret1) and leaving it stale
       crashed on freed heap. Idle (all zero) is the only state that is always safe. */
    for (i = 0; i < sn_nrumble; ++i) {
        memset((void *) (uintptr_t) sn_rumble[i].va, 0, sn_rumble[i].len);
    }
}

/* THE ASYNCHRONOUS WORLD IS NOT ROLLED BACK. A load rewinds the simulation; the disc head does
 * not rewind with it. devcom's request nodes (HSD_DevCom, 0x24 bytes) live on the game heap and
 * are linked from devcom's own statics - which are NOT saved, being an excluded object. Restoring
 * the nodes under statics that were not restored left the queue pointing at requests that had
 * already completed, and the run died inside HSD_DevComDVDMemCallback. So the nodes are lifted
 * out of the live memory before a load and put straight back afterwards. */
#define SN_DEVCOM_NODE 0x24
#define SN_NODE_LEN SN_DEVCOM_NODE
#define SN_DEVCOM_MAX 32
static struct {
    uint32_t va;
    uint8_t bytes[SN_DEVCOM_NODE];
} sn_async[SN_DEVCOM_MAX];
static int sn_nasync;

/* Game memory is big-endian wherever it lives - the game's own statics included. */
static uint32_t sn_be32_at(uintptr_t addr) {
    const uint8_t *p = (const uint8_t *) addr;
    return ((uint32_t) p[0] << 24) | ((uint32_t) p[1] << 16) | ((uint32_t) p[2] << 8) | p[3];
}

static uint32_t sn_be32(uint32_t va) {
    if (va < 0x80000000u || va + 4 > 0x80000000u + gw_mem1_size) {
        return 0;
    }
    return sn_be32_at((uintptr_t) va);
}

/* Collect every node reachable from devcom's list heads. */
static void sn_async_collect(void) {
    int i, guard;
    sn_nasync = 0;
    for (i = 0; i < sn_ndevcom_roots; ++i) {
        uint32_t node = sn_be32_at((uintptr_t) sn_devcom_roots[i]);
        for (guard = 0; node >= 0x80000000u && guard < SN_DEVCOM_MAX; ++guard) {
            int j;
            for (j = 0; j < sn_nasync; ++j) {
                if (sn_async[j].va == node) {
                    break;
                }
            }
            if (j == sn_nasync && sn_nasync < SN_DEVCOM_MAX) {
                sn_async[sn_nasync].va = node;
                memcpy(sn_async[sn_nasync].bytes, (const void *) (uintptr_t) node,
                       SN_DEVCOM_NODE);
                ++sn_nasync;
            }
            node = sn_be32(node); /* HSD_DevCom.next is the first field */
        }
    }
}

/* FIXED ASYNC RANGES: the DVD command blocks of the streaming reads (music), allocated at boot
 * from the low arena. shim_dvd completes those reads on its own schedule, so their state advances
 * with the disc, not with the simulation: they are neither compared nor rolled back. The default
 * is where the measured mismatches sat (0x801711A8..0x801711C9, the pstream header queue); it is
 * a boot-time allocation, so it is stable for one build/disc. MELEE_SNAP_ASYNC="hexva+hexlen,..."
 * overrides it. */
#define SN_FIXED_ASYNC_MAX 8
static struct {
    uint32_t va, len;
    uint8_t bytes[0x400];
} sn_fixed[SN_FIXED_ASYNC_MAX];
static double sn_t[8];
static long sn_tn[8];
static struct {
    uintptr_t cb;
    double ms;
    long n;
} sn_cb[64];
static int sn_ncb;
static double sn_cb_t0;
static int sn_cb_on = -1, sn_cb_window;

static int sn_nfixed;

static void sn_fixed_init(void) {
    const char *v = getenv("MELEE_SNAP_ASYNC");
    if (v == NULL) {
        v = "80171160+70";
    }
    while (*v != 0 && sn_nfixed < SN_FIXED_ASYNC_MAX) {
        char *e;
        unsigned long va = strtoul(v, &e, 16);
        unsigned long len;
        if (*e != '+') {
            break;
        }
        len = strtoul(e + 1, &e, 16);
        if (len > 0x400) {
            len = 0x400;
        }
        sn_fixed[sn_nfixed].va = (uint32_t) va;
        sn_fixed[sn_nfixed].len = (uint32_t) len;
        ++sn_nfixed;
        v = *e == ',' ? e + 1 : e;
    }
}

static void sn_fixed_collect(void) {
    int i;
    for (i = 0; i < sn_nfixed; ++i) {
        memcpy(sn_fixed[i].bytes, (const void *) (uintptr_t) sn_fixed[i].va, sn_fixed[i].len);
    }
}

static void sn_fixed_put_back(void) {
    int i;
    for (i = 0; i < sn_nfixed; ++i) {
        memcpy((void *) (uintptr_t) sn_fixed[i].va, sn_fixed[i].bytes, sn_fixed[i].len);
    }
}

static void sn_async_put_back(void) {
    int i;
    for (i = 0; i < sn_nasync; ++i) {
        memcpy((void *) (uintptr_t) sn_async[i].va, sn_async[i].bytes, SN_DEVCOM_NODE);
    }
}

static void sn_boundary_asserts(const char *what) {
    if (sn.deferred_count != NULL && *sn.deferred_count != 0) {
        gw_log("snap: %s with %d deferred callbacks pending", what, *sn.deferred_count);
    }
    if (sn.ppc_depth != NULL && *sn.ppc_depth != 0) {
        gw_log("snap: %s inside the PPC interpreter (depth %d)", what, *sn.ppc_depth);
    }
}

static GwSnapSlot *sn_slot_for(int frame, int create) {
    int i;
    GwSnapSlot *oldest = NULL;
    for (i = 0; i < sn.nslots; ++i) {
        if (sn.slot[i].frame == frame) {
            return &sn.slot[i];
        }
        if (oldest == NULL || sn.slot[i].frame < oldest->frame) {
            oldest = &sn.slot[i];
        }
    }
    return create ? oldest : NULL;
}

static void sn_verify_equal(const GwSnapSlot *s, const char *what) {
    /* MELEE_SNAP_VERIFY=1: after a dirty-mode save or load the live MEM1 must equal the slot on
       EVERY page - the cross-check that the dirty sets are complete. */
    const uint8_t *live = (const uint8_t *) (uintptr_t) 0x80000000u;
    uint32_t pg, bad = 0, first = 0;
    for (pg = 0; pg < sn.npages; ++pg) {
        if (memcmp(live + (size_t) pg * SN_PAGE, s->mem1 + (size_t) pg * SN_PAGE, SN_PAGE) != 0) {
            if (bad++ == 0) {
                first = pg;
            }
        }
    }
    if (bad != 0) {
        uint32_t o;
        const uint8_t *a = live + (size_t) first * SN_PAGE, *b = s->mem1 + (size_t) first * SN_PAGE;
        for (o = 0; o < SN_PAGE && a[o] == b[o]; ++o) {
        }
        sn.n_verify_fail++;
        if (sn.n_verify_fail <= 8) {
            gw_log("snap: VERIFY FAIL after %s (frame %d): %u page(s) differ, first 0x%08X +0x%X (live %02X slot %02X)",
                   what, s->frame, bad, 0x80000000u + first * SN_PAGE, o, a[o], b[o]);
        }
    }
}

void gw_snap_save(int frame) {
    double t0 = sn_ms();
    GwSnapSlot *s = sn_slot_for(frame, 1);
    sn_boundary_asserts("save");
    s->frame = frame;
    if (sn.dirty_mode) {
        sn_poll();
        sn.n_copy_pages += (long) sn_copy_pages(s->dirty, s->mem1, 0);
        memset(s->dirty, 0, SN_BM_WORDS(sn.npages) * 8);
        if (sn.verify) {
            sn_verify_equal(s, "save");
        }
    } else {
        memcpy(s->mem1, (const void *) (uintptr_t) 0x80000000u, gw_mem1_size);
    }
    sn_gather(s->globals);
    gw_Replay_GetCursor(s->replay_cursor);
    sn.ms_save += sn_ms() - t0;
    sn.n_save++;
    if (sn.hash_on) {
        s->hash = gw_snap_hash();
    }
}

int gw_snap_load(int frame) {
    double t0 = sn_ms();
    GwSnapSlot *s = sn_slot_for(frame, 0);
    if (s == NULL) {
        return -1;
    }
    sn_boundary_asserts("load");
    sn_async_collect();
    sn_fixed_collect();
    if (sn.dirty_mode) {
        int u;
        sn_poll();
        sn.n_copy_pages += (long) sn_copy_pages(s->dirty, s->mem1, 1);
        /* live now equals this slot: every other slot differs from live at most where it differed
           from live before, or where this slot did */
        for (u = 0; u < sn.nslots; ++u) {
            uint32_t w;
            if (&sn.slot[u] == s) {
                continue;
            }
            for (w = 0; w < SN_BM_WORDS(sn.npages); ++w) {
                sn.slot[u].dirty[w] |= s->dirty[w];
            }
        }
        if (sn.hash_dirty != NULL) {
            /* the pages the copy rewrote are the ones that were dirty: their hashes are stale */
            uint32_t w;
            for (w = 0; w < SN_BM_WORDS(sn.npages); ++w) {
                sn.hash_dirty[w] |= s->dirty[w];
            }
        }
        memset(s->dirty, 0, SN_BM_WORDS(sn.npages) * 8);
        sn_watch_reset(); /* the copy's own writes are not news */
        if (sn.verify) {
            sn_verify_equal(s, "load");
        }
    } else {
        memcpy((void *) (uintptr_t) 0x80000000u, s->mem1, gw_mem1_size);
    }
    sn_scatter(s->globals);
    sn_async_put_back();
    sn_fixed_put_back();
    gw_Replay_SetCursor(s->replay_cursor);
    sn.ms_load += sn_ms() - t0;
    sn.n_load++;
    return 0;
}

static const GwSnapSym *sn_sym_at(uint32_t va) {
    int lo = 0, hi = sn.nsyms - 1, best = -1;
    while (lo <= hi) {
        int mid = (lo + hi) / 2;
        if (sn.syms[mid].va <= va) {
            best = mid;
            lo = mid + 1;
        } else {
            hi = mid - 1;
        }
    }
    return best >= 0 ? &sn.syms[best] : NULL;
}

static int sn_in_render_pool_desc(uint32_t va);
static int sn_in_render_arena(uint32_t va, uint32_t *end);

static int sn_skip_global(uint32_t va) {
    int i;
    for (i = 0; i < sn.ncmp_skip; ++i) {
        if (va >= sn.cmp_skip[i].va && va < sn.cmp_skip[i].va + sn.cmp_skip[i].len) {
            return 1;
        }
    }
    /* a render-owned pool's own HSD_ObjAllocData (0x2C bytes): its free list is the render
       pass's, and a resimulated frame never touches it */
    return sn_in_render_pool_desc(va);
}

/* Particles and their AppSRTs are drawn - and written - by the render pass (psdisp.c: frame
 * stamps, sort links, matrices), so their bytes are render-owned too. They live in HSD ObjAlloc
 * pools anywhere on the heap, so mask them per compare by walking the display lists, in the live
 * memory and in the snapshot image (`img`, the snapshot's MEM1 copy, or NULL for live). */
static uint32_t sn_rd32(const uint8_t *img, uint32_t va) {
    const uint8_t *p;
    if (va < 0x80000000u || va + 4 > 0x80000000u + gw_mem1_size) {
        return 0;
    }
    p = img != NULL ? img + (va - 0x80000000u) : (const uint8_t *) (uintptr_t) va;
    return ((uint32_t) p[0] << 24) | ((uint32_t) p[1] << 16) | ((uint32_t) p[2] << 8) | p[3];
}

static void sn_add_mask(uint32_t va, uint32_t len) {
    if (va >= 0x80000000u && sn.nmask < (int) (sizeof sn.mask / sizeof sn.mask[0])) {
        sn.mask[sn.nmask].va = va;
        sn.mask[sn.nmask].len = len;
        ++sn.nmask;
    }
}

static void sn_mask_particles(const uint8_t *img, const uint8_t *globals_img) {
    int l, n;
    if (sn.particle_list == 0) {
        return;
    }
    for (l = 0; l < 17; ++l) {
        uint32_t head;
        if (globals_img != NULL) {
            /* the list head as the snapshot saved it: find it inside the gathered globals */
            uint32_t base = 0;
            int i;
            head = 0;
            for (i = 0; i < sn.nranges; ++i) {
                uint32_t a = sn.particle_list + 4u * (uint32_t) l;
                if (a >= sn.ranges[i].va && a + 4 <= sn.ranges[i].va + sn.ranges[i].len) {
                    const uint8_t *p = globals_img + base + (a - sn.ranges[i].va);
                    head = ((uint32_t) p[0] << 24) | ((uint32_t) p[1] << 16) |
                           ((uint32_t) p[2] << 8) | p[3];
                    break;
                }
                base += sn.ranges[i].len;
            }
        } else {
            const uint8_t *p = (const uint8_t *) (uintptr_t) (sn.particle_list + 4u * (uint32_t) l);
            head = ((uint32_t) p[0] << 24) | ((uint32_t) p[1] << 16) | ((uint32_t) p[2] << 8) | p[3];
        }
        for (n = 0; head != 0 && n < 4096; ++n) {
            uint32_t srt = sn_rd32(img, head + 0x8C);
            sn_add_mask(head, 0x98);
            if (srt != 0) {
                sn_add_mask(srt, 0xA4);
            }
            head = sn_rd32(img, head);
        }
    }
}

static int sn_masked(uint32_t va, uint32_t *end) {
    int i;
    for (i = 0; i < sn.nmask; ++i) {
        if (va >= sn.mask[i].va && va < sn.mask[i].va + sn.mask[i].len) {
            *end = sn.mask[i].va + sn.mask[i].len;
            return 1;
        }
    }
    return 0;
}

/* Does any page that could differ actually differ? (Full mode: any page at all.) */
static int sn_any_diff(const GwSnapSlot *s) {
    const uint8_t *live = (const uint8_t *) (uintptr_t) 0x80000000u;
    uint32_t pg = 0;
    if (!sn.dirty_mode) {
        return memcmp(s->mem1, live, gw_mem1_size) != 0;
    }
    while (pg < sn.npages) {
        uint32_t start, n;
        if (!sn_bm_test(s->dirty, pg)) {
            pg += ((pg & 63) == 0 && s->dirty[pg >> 6] == 0) ? 64 : 1;
            continue;
        }
        start = pg;
        while (pg < sn.npages && sn_bm_test(s->dirty, pg)) {
            ++pg;
        }
        n = pg - start;
        if (memcmp(s->mem1 + (size_t) start * SN_PAGE, live + (size_t) start * SN_PAGE,
                   (size_t) n * SN_PAGE) != 0) {
            return 1;
        }
    }
    return 0;
}

/* Compare the live state with the snapshot of `frame`; log up to a few differing runs. */
static int sn_compare(int frame) {
    GwSnapSlot *s = sn_slot_for(frame, 0);
    double t0 = sn_ms();
    int diffs = 0, logged = 0, i;
    const uint8_t *live = (const uint8_t *) (uintptr_t) 0x80000000u;
    if (s == NULL) {
        return 0;
    }
    sn.checked++;
    sn_poll();
    if (sn_any_diff(s)) {
        uint32_t off = 0;
        sn.nmask = 0;
        sn_mask_particles(NULL, NULL);
        sn_mask_particles(s->mem1, s->globals);
        while (off < gw_mem1_size) {
            uint32_t mend;
            if (sn.dirty_mode && !sn_bm_test(s->dirty, off / SN_PAGE)) {
                /* a clean page (nothing wrote it since this slot last matched live): skip to the
                   next dirty one */
                uint32_t pg = off / SN_PAGE;
                while (pg < sn.npages && !sn_bm_test(s->dirty, pg)) {
                    pg += ((pg & 63) == 0 && s->dirty[pg >> 6] == 0) ? 64 : 1;
                }
                off = pg >= sn.npages ? gw_mem1_size : pg * SN_PAGE;
                continue;
            }
            if (s->mem1[off] != live[off] && (sn.rmask_mem1[off >> 3] & (1u << (off & 7)))) {
                ++off;
                continue;
            }
            if (s->mem1[off] != live[off]) {
                int f;
                for (f = 0; f < sn_nfixed; ++f) {
                    if (0x80000000u + off >= sn_fixed[f].va && 0x80000000u + off < sn_fixed[f].va + sn_fixed[f].len) {
                        break;
                    }
                }
                if (f < sn_nfixed) {
                    off = sn_fixed[f].va + sn_fixed[f].len - 0x80000000u;
                    continue;
                }
            }
            if (s->mem1[off] != live[off] && sn_in_render_arena(0x80000000u + off, &mend)) {
                off = mend - 0x80000000u;
                continue;
            }
            if (s->mem1[off] != live[off] && sn_masked(0x80000000u + off, &mend)) {
                off = mend - 0x80000000u;
                continue;
            }
            if (s->mem1[off] != live[off]) {
                uint32_t start = off;
                while (off < gw_mem1_size && s->mem1[off] != live[off] &&
                       !(sn.rmask_mem1[off >> 3] & (1u << (off & 7))) && off - start < 4096) ++off;
                ++diffs;
                if (logged < 12) {
                    /* name the heap object: HSD objects begin with a pointer to their class info
                       in the native image, which the map names */
                    uint32_t back, owner = 0;
                    const GwSnapSym *cls = NULL;
                    for (back = start & ~3u; back + 0x400 > start && back >= 4; back -= 4) {
                        uint32_t w = sn_rd32(NULL, 0x80000000u + back);
                        if (w >= 0x10000000u && w < 0x11000000u) {
                            const GwSnapSym *c = sn_sym_at(w);
                            if (c != NULL && strstr(c->name, "Class") != NULL) {
                                cls = c;
                                owner = 0x80000000u + back;
                                break;
                            }
                        }
                        if (back == 0) break;
                    }
                    gw_log("snap:   MEM1 0x%08X +%u: first pass %02X.. now %02X..  [%s +0x%X]",
                           0x80000000u + start, off - start, s->mem1[start], live[start],
                           cls ? cls->name : "?", cls ? 0x80000000u + start - owner : 0);
                    if (sn.mismatches < 1 && logged < 2) {
                        /* context for the first mismatch: 32 words around it, both images */
                        uint32_t w0 = (start & ~0x3Fu) >= 0x40 ? (start & ~0x3Fu) - 0x40 : 0, w;
                        for (w = w0; w < w0 + 0x80 && w + 16 <= gw_mem1_size; w += 16) {
                            gw_log("snap:     %08X  pass1 %08X %08X %08X %08X  now %08X %08X %08X %08X",
                                   0x80000000u + w, sn_rd32(s->mem1, 0x80000000u + w),
                                   sn_rd32(s->mem1, 0x80000004u + w), sn_rd32(s->mem1, 0x80000008u + w),
                                   sn_rd32(s->mem1, 0x8000000Cu + w), sn_rd32(NULL, 0x80000000u + w),
                                   sn_rd32(NULL, 0x80000004u + w), sn_rd32(NULL, 0x80000008u + w),
                                   sn_rd32(NULL, 0x8000000Cu + w));
                        }
                    }
                    ++logged;
                }
            } else {
                ++off;
            }
        }
    }
    sn_gather(sn.cmp_globals);
    if (memcmp(s->globals, sn.cmp_globals, sn.globals_len) != 0) {
        uint32_t base = 0;
        for (i = 0; i < sn.nranges; ++i) {
            uint32_t j;
            for (j = 0; j < sn.ranges[i].len; ++j) {
                if (s->globals[base + j] != sn.cmp_globals[base + j] &&
                    !(sn.rmask_globals[(base + j) >> 3] & (1u << ((base + j) & 7))) &&
                    !sn_skip_global(sn.ranges[i].va + j)) {
                    uint32_t va = sn.ranges[i].va + j;
                    const GwSnapSym *sym = sn_sym_at(va);
                    ++diffs;
                    if (logged < 24) {
                        gw_log("snap:   global %s+0x%X (%s): first pass %02X now %02X",
                               sym ? sym->name : "?", sym ? va - sym->va : 0,
                               sym ? sym->obj : "?", s->globals[base + j],
                               sn.cmp_globals[base + j]);
                        ++logged;
                    }
                    /* skip to the end of this symbol: one line per symbol is enough */
                    if (sym != NULL) {
                        uint32_t skip = sym->va + sym->len - va;
                        j += skip > 0 ? skip - 1 : 0;
                    }
                }
            }
            base += sn.ranges[i].len;
        }
    }
    sn.ms_cmp += sn_ms() - t0;
    sn.n_cmp++;
    return diffs;
}

/* ---- the state hash -----------------------------------------------------------------------------
 *
 * gw_snap_hash(): a 64-bit hash of the LIVE game state - MEM1 (per-page hashes, only the pages
 * written since the last call are rehashed) combined with the game's globals (hashed each call, ~1 MB,
 * minus the render-owned/pad/rumble ranges SyncTest also does not compare). The streaming DVD block
 * and devcom's request nodes are hashed as zero: they are the disc's state, not the simulation's.
 * Two peers that ran the same inputs through the same code should agree on it frame for frame; the
 * render-owned bytes (object-pool cells the render pass takes, particle display state) are INCLUDED,
 * so it is a peer-to-peer checksum, not a way to compare a resimulation with its first pass -
 * SyncTest keeps the byte compare with its masks for that. */
static int sn_range_overlap(uint32_t a, uint32_t alen, uint32_t b, uint32_t blen) {
    return a < b + blen && b < a + alen;
}

uint64_t gw_snap_hash(void) {
    double t0 = sn_ms();
    const uint8_t *live = (const uint8_t *) (uintptr_t) 0x80000000u;
    uint8_t tmp[SN_PAGE];
    uint32_t pg, i;
    uint64_t total, g = 0x1234ABCDull;
    if (!sn.enabled || sn.page_h == NULL) {
        return 0;
    }
    sn_poll();
    /* async request nodes move around the heap: force their pages (and last time's) to be looked at */
    sn_async_collect();
    for (i = 0; i < (uint32_t) sn.n_hash_node_pg; ++i) {
        sn_bm_set(sn.hash_dirty, sn.hash_node_pg[i]);
    }
    sn.n_hash_node_pg = 0;
    for (i = 0; i < (uint32_t) sn_nasync; ++i) {
        uint32_t pgn = (sn_async[i].va - 0x80000000u) / SN_PAGE;
        uint32_t pge = (sn_async[i].va + SN_NODE_LEN - 1 - 0x80000000u) / SN_PAGE;
        for (; pgn <= pge && pgn < sn.npages; ++pgn) {
            sn_bm_set(sn.hash_dirty, pgn);
            if (sn.n_hash_node_pg < 64) {
                sn.hash_node_pg[sn.n_hash_node_pg++] = pgn;
            }
        }
    }
    for (pg = 0; pg < sn.npages; ++pg) {
        uint64_t h;
        int needs_zero = 0, f;
        uint32_t va = 0x80000000u + pg * SN_PAGE;
        if (!sn_bm_test(sn.hash_dirty, pg)) {
            if ((pg & 63) == 0 && sn.hash_dirty[pg >> 6] == 0) {
                pg += 63;
            }
            continue;
        }
        for (f = 0; f < sn_nfixed && !needs_zero; ++f) {
            needs_zero = sn_range_overlap(va, SN_PAGE, sn_fixed[f].va, sn_fixed[f].len);
        }
        for (i = 0; i < (uint32_t) sn_nasync && !needs_zero; ++i) {
            needs_zero = sn_range_overlap(va, SN_PAGE, sn_async[i].va, SN_NODE_LEN);
        }
        if (needs_zero) {
            memcpy(tmp, live + (size_t) pg * SN_PAGE, SN_PAGE);
            for (f = 0; f < sn_nfixed; ++f) {
                if (sn_range_overlap(va, SN_PAGE, sn_fixed[f].va, sn_fixed[f].len)) {
                    uint32_t lo = sn_fixed[f].va > va ? sn_fixed[f].va : va;
                    uint32_t hi = sn_fixed[f].va + sn_fixed[f].len < va + SN_PAGE ? sn_fixed[f].va + sn_fixed[f].len : va + SN_PAGE;
                    memset(tmp + (lo - va), 0, hi - lo);
                }
            }
            for (i = 0; i < (uint32_t) sn_nasync; ++i) {
                if (sn_range_overlap(va, SN_PAGE, sn_async[i].va, SN_NODE_LEN)) {
                    uint32_t lo = sn_async[i].va > va ? sn_async[i].va : va;
                    uint32_t hi = sn_async[i].va + SN_NODE_LEN < va + SN_PAGE ? sn_async[i].va + SN_NODE_LEN : va + SN_PAGE;
                    memset(tmp + (lo - va), 0, hi - lo);
                }
            }
            h = sn_h64(tmp, SN_PAGE, pg + 1);
        } else {
            h = sn_h64(live + (size_t) pg * SN_PAGE, SN_PAGE, pg + 1);
        }
        sn.hash_mem ^= sn.page_h[pg] ^ h;
        sn.page_h[pg] = h;
        sn.n_hash_pages++;
    }
    memset(sn.hash_dirty, 0, SN_BM_WORDS(sn.npages) * 8);
    /* globals: every game range, minus the ranges SyncTest does not compare */
    for (i = 0; i < (uint32_t) sn.nranges; ++i) {
        uint32_t a = sn.ranges[i].va, e = a + sn.ranges[i].len;
        while (a < e) {
            uint32_t next = e;
            int k, skipped = 0;
            for (k = 0; k < sn.ncmp_skip; ++k) {
                uint32_t sa = sn.cmp_skip[k].va, se = sa + sn.cmp_skip[k].len;
                if (a >= sa && a < se) {
                    next = se < e ? se : e;
                    skipped = 1;
                    break;
                }
                if (sa > a && sa < next) {
                    next = sa;
                }
            }
            if (!skipped) {
                g = sn_h64((const uint8_t *) (uintptr_t) a, next - a, g);
            }
            a = next;
        }
    }
    total = sn.hash_mem * 0x9E3779B97F4A7C15ull ^ (g << 17 | g >> 47);
    sn.n_hash_calls++;
    sn.ms_hash += sn_ms() - t0;
    return total;
}

uint64_t gw_snap_frame_hash(int frame) {
    GwSnapSlot *s = sn_slot_for(frame, 0);
    return s != NULL ? s->hash : 0;
}

/* ---- curated gameplay hash (MELEE_SYNCTEST_CURATED=1) --------------------------------------------
 *
 * The strict SyncTest compares every byte of MEM1 and the game's globals (with the render-owned
 * masks) and needs the resimulated frame to run the render pass too. This mode is the experiment the
 * coordinator asked for (_research/yampp-comparison.md): compare only a CURATED set of gameplay
 * fields - the RNG seed and, per fighter, position, velocity, action state, percent, hitlag, timers
 * and inputs - and resimulate WITHOUT any render calls. Game code feeds the fields through
 * gw_Snap_CuratedMix at the points it simulates (fighter.c's Fighter_procMap, gmscene.c's seed
 * check); the accumulator is a commutative sum of per-record hashes, so fighter order does not
 * matter. The first pass records each frame's value; a resimulated iteration compares its own. */
static int sn_curated = -1;
static uint64_t sn_cur_acc;
static struct {
    int frame;
    uint64_t h;
} sn_cur_ring[32];
static long sn_cur_mismatch, sn_cur_checked;

int gw_Snap_Curated(void) {
    if (sn_curated < 0) {
        const char *e = getenv("MELEE_SYNCTEST_CURATED");
        sn_curated = e != NULL && e[0] == '1';
    }
    return sn_curated && sn.enabled;
}

void gw_Snap_CuratedMix(const uint32_t *w, int n) {
    if (gw_Snap_Curated()) {
        static int poison = -1;
        sn_cur_acc += sn_h64((const uint8_t *) w, (size_t) n * 4, 0x5EED5EEDull);
        if (poison < 0) {
            const char *e = getenv("MELEE_SYNCTEST_CURATED_POISON");
            poison = e != NULL && e[0] == '1';
        }
        if (poison && sn.cur_is_resim) {
            sn_cur_acc += 1; /* NEGATIVE CONTROL: every resimulated frame must now mismatch */
        }
    }
}

/* Top of a logic iteration: the previous iteration simulated frame gw_Replay_Frame(); its
 * accumulator is the first pass's record (recorded) or a resimulation's (compared). */
static void sn_cur_take(void) {
    int f = gw_Replay_Frame();
    int slot = (f & 0x7FFFFFFF) % 32;
    uint64_t h = sn_cur_acc;
    sn_cur_acc = 0;
    if (f < -123) {
        return;
    }
    if (sn.cur_is_resim) {
        if (sn_cur_ring[slot].frame == f) {
            sn_cur_checked++;
            sn.checked++;
            if (sn_cur_ring[slot].h != h) {
                sn_cur_mismatch++;
                sn.mismatches++;
                if (sn_cur_mismatch <= 8) {
                    gw_log("snap: CURATED MISMATCH frame %d (resimulated %d back from %d): first pass %016llX resim %016llX",
                           f, sn.target - f, sn.target, (unsigned long long) sn_cur_ring[slot].h,
                           (unsigned long long) h);
                }
            }
        }
    } else {
        sn_cur_ring[slot].frame = f;
        sn_cur_ring[slot].h = h;
    }
}

/* ---- SyncTest --------------------------------------------------------------------------------- */

static void sn_sfx_rewind(int frame);
static void sn_mark_window(void);

/* gw_snap_open(k): set the snapshot machinery up with a ring of k+2 slots (enough to save the
 * frame about to run and roll back k frames). Idempotent; MELEE_SYNCTEST=<k> calls it at the first
 * scene-loop tick, and so can a rollback session. Returns 0 on success.
 *   MELEE_SNAP_MODE=full|dirty   dirty (default): write-watch dirty pages; full: copy all of MEM1
 *   MELEE_SNAP_VERIFY=1          after every dirty save/load, memcmp live against the slot
 *   MELEE_SNAP_HASH=0            do not hash at save time (gw_snap_hash() is still callable) */
int gw_snap_open(int k) {
    int i;
    const char *v;
    if (sn.enabled) {
        return 0;
    }
    if (k <= 0) {
        return -1;
    }
    sn.k = k > GW_SNAP_MAX_SLOTS - 2 ? GW_SNAP_MAX_SLOTS - 2 : k;
    if (sn_load_map() != 0) {
        return -1;
    }
    sn.nslots = sn.k + 2;
    sn.npages = gw_mem1_size / SN_PAGE;
    v = getenv("MELEE_SNAP_MODE");
    sn.dirty_mode = gw_mem1_watched && !(v != NULL && strcmp(v, "full") == 0);
    v = getenv("MELEE_SNAP_VERIFY");
    sn.verify = sn.dirty_mode && v != NULL && v[0] == '1';
    v = getenv("MELEE_SNAP_HASH");
    sn.hash_on = sn.dirty_mode && !(v != NULL && v[0] == '0');
    for (i = 0; i < sn.nslots; ++i) {
        sn.slot[i].frame = -0x7FFFFFFF - 1;
        sn.slot[i].mem1 = (uint8_t *) malloc(gw_mem1_size);
        sn.slot[i].globals = (uint8_t *) malloc(sn.globals_len);
        sn.slot[i].dirty = (uint64_t *) malloc(SN_BM_WORDS(sn.npages) * 8);
        if (sn.slot[i].mem1 == NULL || sn.slot[i].globals == NULL || sn.slot[i].dirty == NULL) {
            gw_log("snap: out of memory for %d slots", sn.nslots);
            return -1;
        }
        sn_bm_fill(sn.slot[i].dirty, sn.npages); /* never synced: every page differs */
    }
    sn.hash_dirty = (uint64_t *) malloc(SN_BM_WORDS(sn.npages) * 8);
    sn.page_h = (uint64_t *) calloc(sn.npages, 8);
    if (sn.hash_dirty == NULL || sn.page_h == NULL) {
        gw_log("snap: out of memory for the hash tables");
        return -1;
    }
    sn_bm_fill(sn.hash_dirty, sn.npages);
    sn.cmp_globals = (uint8_t *) malloc(sn.globals_len);
    sn.pre_mem1 = (uint8_t *) malloc(gw_mem1_size);
    sn.pre_globals = (uint8_t *) malloc(sn.globals_len);
    sn.rmask_mem1 = (uint8_t *) calloc(gw_mem1_size / 8 + 1, 1);
    sn_fixed_init();
    sn.rmask_globals = (uint8_t *) calloc(sn.globals_len / 8 + 1, 1);
    if (sn.pre_mem1 == NULL || sn.pre_globals == NULL || sn.rmask_mem1 == NULL ||
        sn.rmask_globals == NULL) {
        gw_log("snap: out of memory for the render mask");
        return -1;
    }
    sn.enabled = 1;
    sn_watch_reset(); /* start counting writes from here; every slot starts all-dirty */
    gw_log("snap: SyncTest k=%d (%d slots of %u bytes), %s mode%s%s", sn.k, sn.nslots,
           gw_mem1_size + sn.globals_len, sn.dirty_mode ? "dirty-page" : "full-copy",
           sn.verify ? ", verify" : "", sn.hash_on ? ", hash" : "");
    return 0;
}

static void sn_init(void) {
    const char *v;
    if (sn.tried) {
        return;
    }
    sn.tried = 1;
    v = getenv("MELEE_SYNCTEST");
    if (v == NULL || v[0] == '\0' || atoi(v) <= 0) {
        return;
    }
    gw_snap_open(atoi(v));
}

/* The replay has run at least one frame: the loading hold is over and the match is live. */
static int sn_live(void) {
    return gw_Replay_Frame() >= -123;
}

/* Before the scene loop's logic iterations for this render tick. */
static int sn_last_pool_frame = -1000000; /* frame a render pool was last discovered */
int gw_SyncTest_Iterations(int count) {
    sn_init();
    if (!sn.enabled || !sn_live()) {
        return count;
    }
    {
        static int ticks;
        if ((++ticks % 250) == 0) {
            gw_log("snap: heartbeat tick %d frame %d rollbacks %d mismatching %d slot(F-k)=%s", ticks, gw_Replay_Frame(),
                   sn.passes, sn.mismatches, sn_slot_for(gw_Replay_Frame() + 1 - sn.k, 0) != NULL ? "yes" : "no");
        }
    }
    sn.target = gw_Replay_Frame() + 1;
    /* A render pool discovered this recently has not been stocked by logic in the frames a
       rollback would resimulate, so the first pass and the resimulation would disagree about
       whether the heap grew (objalloc.c HSD_ObjAllocTopUp). Let the frames pass unrolled until
       every rollback window sees the same set of render pools. */
    if (sn.target - sn_last_pool_frame <= sn.k + 2) {
        return count;
    }
    if (sn_slot_for(sn.target - sn.k, 0) != NULL) {
        sn.plan_rollback = 1;
        return sn.k + 1;
    }
    return count;
}

/* At the top of each logic iteration. */
void gw_SyncTest_IterStart(void) {
    int next;
    if (!sn.enabled || !sn_live()) {
        return;
    }
    /* Close the "not written by logic" window opened when the last logic pass ended. Everything
       the render pass, the deferred queue and the asynchronous DVD/stream completions touched in
       between is state a resimulated frame never reproduces, so it stops being compared. */
    sn_mark_window();
    if (gw_Snap_Curated()) {
        if (sn.cur_is_resim) {
            gw_Snap_Time(2, 0); /* the resimulated iteration ends here: there is no render block */
        }
        sn_cur_take();
    }
    next = gw_Replay_Frame() + 1;
    if (sn.plan_rollback) {
        sn.plan_rollback = 0;
        gw_snap_save(sn.target);
        gw_snap_load(sn.target - sn.k);
        sn.resim = 1;
        sn.cur_is_resim = 1;
        sn_sfx_rewind(sn.target - sn.k);
        gw_Snap_Time(2, 1);
        return;
    }
    if (sn.resim) {
        int d = gw_Snap_Curated() ? 0 : sn_compare(next);
        if (d != 0) {
            sn.mismatches++;
            if (sn.mismatches == 1 && getenv("MELEE_SYNCTEST_DUMP") != NULL) {
                /* both MEM1 images of the first mismatch, for offline analysis */
                GwSnapSlot *sl = sn_slot_for(next, 0);
                char p[600];
                FILE *df;
                snprintf(p, sizeof p, "%s.pass1.bin", getenv("MELEE_SYNCTEST_DUMP"));
                if (sl != NULL && (df = fopen(p, "wb")) != NULL) {
                    fwrite(sl->mem1, 1, gw_mem1_size, df);
                    fclose(df);
                }
                snprintf(p, sizeof p, "%s.resim.bin", getenv("MELEE_SYNCTEST_DUMP"));
                if ((df = fopen(p, "wb")) != NULL) {
                    fwrite((const void *) (uintptr_t) 0x80000000u, 1, gw_mem1_size, df);
                    fclose(df);
                }
                gw_log("snap: dumped both MEM1 images of the first mismatch to %s.*.bin",
                       getenv("MELEE_SYNCTEST_DUMP"));
            }
            if (sn.mismatches <= 8) {
                gw_log("snap: SYNCTEST MISMATCH at the start of frame %d (resimulated %d back from "
                       "%d): %d differing runs",
                       next, sn.target - next, sn.target, d);
            }
        }
        sn.cur_is_resim = next < sn.target;
        if (sn.cur_is_resim) {
            sn_sfx_rewind(next);
            gw_Snap_Time(2, 1);
        }
        if (next >= sn.target) {
            sn.resim = 0;
            sn.passes++;
            if ((sn.passes % 100) == 0) {
                gw_log("snap: frame %d, %d rollbacks, %d checks, %d mismatching; per frame save "
                       "%.2f ms, load %.2f ms, compare %.2f ms",
                       next, sn.passes, sn.checked, sn.mismatches,
                       sn.ms_save / (sn.n_save ? sn.n_save : 1),
                       sn.ms_load / (sn.n_load ? sn.n_load : 1),
                       sn.ms_cmp / (sn.n_cmp ? sn.n_cmp : 1));
                gw_log("snap: costs (per call): poll %.3f ms (%.0f dirty pages), copy %.0f pages per "
                       "save/load, hash %.3f ms (%.0f pages hashed), resim render %.2f ms, real render "
                       "%.2f ms, resim logic %.2f ms [render parts: idle+inval %.3f, StartRender %.3f, GObjDraw %.3f, Init %.3f], verify fails %ld",
                       sn.ms_poll / (sn.n_poll ? sn.n_poll : 1),
                       (double) sn.n_dirty_pages / (sn.n_poll ? sn.n_poll : 1),
                       (double) sn.n_copy_pages / ((sn.n_save + sn.n_load) ? (sn.n_save + sn.n_load) : 1),
                       sn.ms_hash / (sn.n_hash_calls ? sn.n_hash_calls : 1),
                       (double) sn.n_hash_pages / (sn.n_hash_calls ? sn.n_hash_calls : 1),
                       sn_t[0] / (sn_tn[0] ? sn_tn[0] : 1), sn_t[1] / (sn_tn[1] ? sn_tn[1] : 1),
                       sn_t[2] / (sn_tn[2] ? sn_tn[2] : 1), sn_t[3] / (sn_tn[3] ? sn_tn[3] : 1),
                       sn_t[4] / (sn_tn[4] ? sn_tn[4] : 1), sn_t[5] / (sn_tn[5] ? sn_tn[5] : 1),
                       sn_t[6] / (sn_tn[6] ? sn_tn[6] : 1), sn.n_verify_fail);
                if (gw_Snap_Curated()) {
                    gw_log("snap: curated hash: %ld compared, %ld mismatching, resim iteration (logic only) %.2f ms",
                           sn_cur_checked, sn_cur_mismatch, sn_t[2] / (sn_tn[2] ? sn_tn[2] : 1));
                }
                if (sn_cb_on == 1) {
                    int a, b, top[8], nt = 0;
                    for (a = 0; a < 8 && a < sn_ncb; ++a) {
                        int best = -1;
                        for (b = 0; b < sn_ncb; ++b) {
                            int seen = 0, q;
                            for (q = 0; q < nt; ++q) {
                                if (top[q] == b) seen = 1;
                            }
                            if (!seen && (best < 0 || sn_cb[b].ms > sn_cb[best].ms)) best = b;
                        }
                        top[nt++] = best;
                        gw_log("snap:   render cb %p: %.3f ms per resim frame (%ld calls)",
                               (void *) sn_cb[best].cb,
                               sn_cb[best].ms / (sn_tn[0] ? sn_tn[0] : 1), sn_cb[best].n);
                    }
                }
            }
        }
        return;
    }
    sn.cur_is_resim = 0;
    gw_snap_save(next);
}

/* Per render-callback time inside a resimulated frame's GObj draw walk (gobj.c): while
 * MELEE_SNAP_CBTIME=1, the periodic log lists the costliest callbacks by native address (resolve
 * against melee-pc.map). Only counted while the resimulated render is running (sn_t[5] open). */

void gw_Snap_CbTime(void *cb, int begin) {
    int i;
    if (sn_cb_on < 0) {
        const char *e = getenv("MELEE_SNAP_CBTIME");
        sn_cb_on = e != NULL && e[0] == '1';
    }
    if (!sn_cb_on || !sn_cb_window) {
        return;
    }
    if (begin) {
        sn_cb_t0 = sn_ms();
        return;
    }
    for (i = 0; i < sn_ncb; ++i) {
        if (sn_cb[i].cb == (uintptr_t) cb) {
            break;
        }
    }
    if (i == sn_ncb) {
        if (sn_ncb >= 64) {
            return;
        }
        sn_cb[sn_ncb].cb = (uintptr_t) cb;
        ++sn_ncb;
    }
    sn_cb[i].ms += sn_ms() - sn_cb_t0;
    sn_cb[i].n++;
}

/* Render callbacks a RESIMULATED frame does not run at all (gobj.c). Only callbacks whose whole job
 * is the picture: Fountain's water reflection (grIzumi_801CCEA0) re-renders the scene from a mirrored
 * camera into a texture. ON by default: SyncTest k=3 passed 5600 rollbacks / 16800 checks with it
 * (resim render 0.76 -> 0.63 ms). MELEE_SNAP_SKIP_CB=none turns it off. A callback that also clears
 * dirty flags or fills matrix caches cannot be skipped - SyncTest is the test for adding another. */
extern void gw_grIzumi_801CCEA0(void *gobj, int pass);

int gw_Snap_SkipRenderCb(void *cb) {
    static int mode = -1;
    if (mode < 0) {
        const char *e = getenv("MELEE_SNAP_SKIP_CB");
        mode = (e != NULL && strstr(e, "none") != NULL) ? 0 : 1;
    }
    return mode && sn_cb_window && cb == (void *) (uintptr_t) gw_grIzumi_801CCEA0;
}

/* Should a resimulated frame's render pass skip submitting display lists (shim_gx.c)?
 * MELEE_SNAP_RESIM_DRAWS=1 keeps them, to tell a draw-owned difference apart. */
/* In curated mode a resimulated frame runs NO render calls at all (gmscene.c). */
int gw_Snap_CuratedNoRender(void) {
    return gw_Snap_Curated();
}

int gw_Snap_SuppressDraws(void) {
    static int v = -1;
    if (v < 0) {
        const char *e = getenv("MELEE_SNAP_RESIM_DRAWS");
        v = (e != NULL && e[0] == '1') ? 0 : 1;
    }
    return v;
}

/* gmscene.c times the render pass of a resimulated frame (what 0) and of the real one (what 1). */
void gw_Snap_Time(int what, int begin) {
    static double t0[8];
    if (!sn.enabled || what < 0 || what > 7) {
        return;
    }
    if (what == 5) {
        sn_cb_window = begin;
    }
    if (begin) {
        t0[what] = sn_ms();
    } else {
        sn_t[what] += sn_ms() - t0[what];
        sn_tn[what]++;
    }
}

/* The first pass's sound handles, per frame, for resimulated frames to get back (axdriver.c). */
#define GW_SNAP_SFX_FRAMES 16
#define GW_SNAP_SFX_PER_FRAME 64
static struct {
    int frame, n, taken;
    int id[GW_SNAP_SFX_PER_FRAME], res[GW_SNAP_SFX_PER_FRAME];
} sn_sfx[GW_SNAP_SFX_FRAMES];
static int sn_sfx_misses;

void gw_Snap_SfxPut(int sound_id, int result) {
    int f = gw_Replay_Frame();
    int s = (f & 0x7FFFFFFF) % GW_SNAP_SFX_FRAMES;
    if (!sn.enabled || !sn_live()) {
        return;
    }
    if (sn_sfx[s].frame != f) {
        sn_sfx[s].frame = f;
        sn_sfx[s].n = 0;
    }
    sn_sfx[s].taken = 0;
    if (sn_sfx[s].n < GW_SNAP_SFX_PER_FRAME) {
        sn_sfx[s].id[sn_sfx[s].n] = sound_id;
        sn_sfx[s].res[sn_sfx[s].n] = result;
        sn_sfx[s].n++;
    }
}

static void sn_sfx_rewind(int frame) {
    int s = (frame & 0x7FFFFFFF) % GW_SNAP_SFX_FRAMES;
    if (sn_sfx[s].frame == frame) {
        sn_sfx[s].taken = 0;
    }
}

int gw_Snap_SfxTake(int sound_id) {
    int f = gw_Replay_Frame();
    int s = (f & 0x7FFFFFFF) % GW_SNAP_SFX_FRAMES;
    if (sn_sfx[s].frame == f && sn_sfx[s].taken < sn_sfx[s].n &&
        sn_sfx[s].id[sn_sfx[s].taken] == sound_id) {
        return sn_sfx[s].res[sn_sfx[s].taken++];
    }
    if (sn_sfx_misses++ < 8) {
        gw_log("snap: resimulated frame %d played sound %d the first pass did not", f, sound_id);
    }
    return -1;
}

/* Around the render pass (gmscene.c): whatever it changes is render-owned. */
/* Does the render pass take cells out of the HSD object pools? A pool free list is LIFO, so a
 * render-time alloc/free pair reverses it and the NEXT logic allocation lands in a different
 * cell - which a resimulated frame (which never renders) does not reproduce. objalloc.c calls
 * this so the evidence is in the log rather than inferred from addresses. */
static int sn_in_render;
static int sn_obj_notes;

static int sn_alloc_trace(void) {
    static int trace = -1;
    if (trace < 0) {
        const char *v = getenv("MELEE_SYNCTEST_ALLOC");
        trace = (v != NULL && *v == '1') ? 1 : 0;
    }
    return trace;
}

/* The same question for the OSAlloc heap (memory.c), whose free list is what actually drifted. */
void gw_Snap_NoteMem(int size, void *ptr, unsigned caller, int freeing) {
    if (!sn_alloc_trace() || !sn.enabled || sn_obj_notes >= 400000) {
        return;
    }
    ++sn_obj_notes;
    gw_log("memtrace f=%d r=%d%s %s size %d ptr %p from %08X", gw_Replay_Frame(),
           sn.cur_is_resim ? 1 : 0, sn_in_render ? " R" : "", freeing ? "free " : "alloc", size,
           ptr, caller);
}

void gw_Snap_NoteObj(void *data, void *obj, int freeing) {
    int trace = sn_alloc_trace();
    /* MELEE_SYNCTEST_ALLOC=1: every pool operation, tagged with the frame, whether it happened
       inside the render pass and whether this is the first pass or a resimulation - diff the two
       sequences for one frame and the pool whose order drifted names itself. */
    if (trace && sn.enabled && sn_obj_notes < 400000) {
        ++sn_obj_notes;
        gw_log("objtrace f=%d r=%d%s %s pool %p cell %p", gw_Replay_Frame(),
               sn.cur_is_resim ? 1 : 0, sn_in_render ? " R" : "", freeing ? "free " : "alloc",
               data, obj);
        return;
    }
    if (!sn_in_render || sn_obj_notes >= 64) {
        return;
    }
    ++sn_obj_notes;
    gw_log("snap: render pass Obj%s pool %p cell %p (frame %d)", freeing ? "Free " : "Alloc",
           data, obj, gw_Replay_Frame());
}

/* objalloc.c asks, so it can bill a pool's cells to the render pass and refill them during logic
 * instead (HSD_ObjAllocTopUp). */
int gw_Snap_InRender(void) {
    return sn_in_render;
}

/* ---- render-owned object pools ---------------------------------------------------------------
 *
 * The HSD matrix pools (HSD_Mtx_804C2310/233C) and the display SList pool are drawn from by the
 * render pass and by nothing else: at the end of a render pass they still held 145 and 1 cells.
 * A resimulated frame renders nothing, so those cells - and the pool's own free list - are always
 * going to differ, and they say nothing about whether the simulation rolled back correctly.
 *
 * Masking the bytes the render wrote is not enough here, because the render takes DIFFERENT cells
 * every frame: an address-based mask never catches up with a rotating allocation. So the skip is
 * by POOL - every arena objalloc.c ever handed that pool, plus the pool descriptor itself. */
#define SN_MAX_ARENAS 512
static struct {
    uint32_t pool, va, len;
} sn_arena[SN_MAX_ARENAS];
static int sn_narenas;
static uint32_t sn_render_pool[16];

static int sn_nrender_pools;

void gw_Snap_NoteArena(void *data, void *start, unsigned size) {
    if (sn_narenas >= SN_MAX_ARENAS) {
        return;
    }
    sn_arena[sn_narenas].pool = (uint32_t) (uintptr_t) data;
    sn_arena[sn_narenas].va = (uint32_t) (uintptr_t) start;
    sn_arena[sn_narenas].len = size;
    ++sn_narenas;
}

void gw_Snap_NoteRenderPool(void *data) {
    int i;
    uint32_t p = (uint32_t) (uintptr_t) data;
    for (i = 0; i < sn_nrender_pools; ++i) {
        if (sn_render_pool[i] == p) {
            return;
        }
    }
    if (sn_nrender_pools < 16) {
        sn_last_pool_frame = gw_Replay_Frame();
        sn_render_pool[sn_nrender_pools++] = p;
        gw_log("snap: pool %p is render-owned (frame %d); its arenas stop being compared", data, gw_Replay_Frame());
    }
}

/* objalloc.c's top-up walks this list; it is native, so a load never un-registers a pool. */
int gw_Snap_RenderPoolCount(void) {
    return sn_nrender_pools;
}

void *gw_Snap_RenderPoolAt(int i) {
    return (void *) (uintptr_t) sn_render_pool[i];
}

static int sn_render_owned_pool(uint32_t pool) {
    int i;
    for (i = 0; i < sn_nrender_pools; ++i) {
        if (sn_render_pool[i] == pool) {
            return 1;
        }
    }
    return 0;
}

/* Inside a render-owned pool's HSD_ObjAllocData (0x2C bytes)? */
static int sn_in_render_pool_desc(uint32_t va) {
    int i;
    for (i = 0; i < sn_nrender_pools; ++i) {
        if (va >= sn_render_pool[i] && va < sn_render_pool[i] + 0x2Cu) {
            return 1;
        }
    }
    return 0;
}

/* MEM1 address inside an arena of a render-owned pool? *end gets the arena's end. */
static int sn_in_render_arena(uint32_t va, uint32_t *end) {
    int i;
    for (i = 0; i < sn_narenas; ++i) {
        if (va >= sn_arena[i].va && va < sn_arena[i].va + sn_arena[i].len &&
            sn_render_owned_pool(sn_arena[i].pool)) {
            *end = sn_arena[i].va + sn_arena[i].len;
            return 1;
        }
    }
    return 0;
}

static uint8_t *sn_async_mem1, *sn_async_globals, *sn_amask_mem1, *sn_amask_globals;

void gw_SyncTest_PreRender(void) {
    sn_in_render = 1;
    if (!sn.enabled || !sn_live() || sn.session) {
        return;
    }
    memcpy(sn.pre_mem1, (const void *) (uintptr_t) 0x80000000u, gw_mem1_size);
    sn_gather(sn.pre_globals);
    sn.pre_valid = 1;
    if (sn_amask_mem1 != NULL) {
        memset(sn_amask_mem1, 0, gw_mem1_size / 8 + 1);
        memset(sn_amask_globals, 0, sn.globals_len / 8 + 1);
    }
}

static uint32_t sn_mark(const uint8_t *before, const uint8_t *after, uint32_t len, uint8_t *mask) {
    uint32_t off, added = 0;
    for (off = 0; off < len; off += 4096) {
        uint32_t n = len - off < 4096 ? len - off : 4096, j;
        if (memcmp(before + off, after + off, n) == 0) {
            continue;
        }
        for (j = 0; j < n; ++j) {
            uint32_t b = off + j;
            if (before[b] != after[b] && !(mask[b >> 3] & (1u << (b & 7)))) {
                mask[b >> 3] |= (uint8_t) (1u << (b & 7));
                ++added;
            }
        }
    }
    return added;
}

/* THE RENDER PASS IS UNDONE. Everything the render pass writes to MEM1 or to the game's globals
 * is put back the way it was before it ran. That makes rendering a pure function of the
 * simulation state - which is what a resimulated frame, that never renders, needs it to be - and
 * it is far stronger than masking: the first design masked render-written bytes out of the
 * comparison, then chased the consequences (pools growing at different times, cell addresses
 * rotating, a pool the render drains being also drained by logic) for a dozen rounds. Undoing
 * the writes removes the whole class.
 *
 * What it costs: any state the render pass computes and LOGIC later reads is lost each frame, so
 * logic sees it as stale in the first pass and in the resimulation alike - deterministic, but
 * different from a normal run. Those are the render->logic couplings (x221F_b0, the camera
 * matrix cache, the magnifier flag ...); each is listed by MELEE_SYNCTEST_RENDER=report. This is a
 * SyncTest-only mode. MELEE_SYNCTEST_RENDER=keep leaves the render writes in place. */
static int sn_render_mode(void) {
    static int mode = -1; /* 0 undo, 1 keep, 2 report (undo + list what render wrote) */
    if (mode < 0) {
        const char *v = getenv("MELEE_SYNCTEST_RENDER");
        mode = v == NULL ? 1 : (strcmp(v, "keep") == 0 ? 1 : (strcmp(v, "report") == 0 ? 2 : (strcmp(v, "mem1") == 0 ? 3 : (strcmp(v, "globals") == 0 ? 4 : 0))));
    }
    return mode;
}

/* Deferred callbacks (DVD completions, alarms) run from inside the render window, in the
 * present call. What THEY write is the asynchronous world's and must survive the undo below, so
 * each run is bracketed and its writes are recorded in a second mask. */

void gw_Snap_AsyncBegin(void) {
    if (!sn.enabled || !sn.pre_valid || sn_render_mode() == 1) {
        return;
    }
    if (sn_async_mem1 == NULL) {
        sn_async_mem1 = (uint8_t *) malloc(gw_mem1_size);
        sn_async_globals = (uint8_t *) malloc(sn.globals_len);
        sn_amask_mem1 = (uint8_t *) calloc(gw_mem1_size / 8 + 1, 1);
        sn_amask_globals = (uint8_t *) calloc(sn.globals_len / 8 + 1, 1);
        if (!sn_async_mem1 || !sn_async_globals || !sn_amask_mem1 || !sn_amask_globals) {
            sn_async_mem1 = NULL;
            return;
        }
    }
    memcpy(sn_async_mem1, (const void *) (uintptr_t) 0x80000000u, gw_mem1_size);
    sn_gather(sn_async_globals);
}

void gw_Snap_AsyncEnd(void) {
    if (!sn.enabled || !sn.pre_valid || sn_render_mode() == 1 || sn_async_mem1 == NULL) {
        return;
    }
    sn_mark(sn_async_mem1, (const uint8_t *) (uintptr_t) 0x80000000u, gw_mem1_size, sn_amask_mem1);
    sn_gather(sn.cmp_globals);
    sn_mark(sn_async_globals, sn.cmp_globals, sn.globals_len, sn_amask_globals);
}

static int sn_rr, sn_rr_logged;
void gw_SyncTest_PostRender(void) {
    ++sn_rr;
    sn_in_render = 0;
    if (!sn.enabled || !sn.pre_valid || sn_render_mode() == 1) {
        return;
    }
    {
        uint32_t off, undone = 0;
        uint8_t *live = (uint8_t *) (uintptr_t) 0x80000000u;
        int i;
        uint32_t base = 0;
        for (off = 0; off < gw_mem1_size && sn_render_mode() != 4; off += 4096) {
            uint32_t n = gw_mem1_size - off < 4096 ? gw_mem1_size - off : 4096;
            if (memcmp(sn.pre_mem1 + off, live + off, n) != 0) {
                uint32_t j;
                for (j = 0; j < n; ++j) {
                    if (sn_render_mode() == 2 && sn_rr == 200 && sn_rr_logged < 60 &&
                        sn.pre_mem1[off + j] != live[off + j] && (j == 0 || sn.pre_mem1[off + j - 1] == live[off + j - 1])) {
                        const uint8_t *q = live + off + j;
                        gw_log("snap: render wrote MEM1 %08X: %02X%02X%02X%02X.. was %02X%02X%02X%02X", 0x80000000u + off + j,
                               q[0], q[1], q[2], q[3], sn.pre_mem1[off + j], sn.pre_mem1[off + j + 1], sn.pre_mem1[off + j + 2], sn.pre_mem1[off + j + 3]);
                        ++sn_rr_logged;
                    }
                    if (sn.pre_mem1[off + j] != live[off + j] &&
                        !(sn_amask_mem1 != NULL && (sn_amask_mem1[(off + j) >> 3] & (1u << ((off + j) & 7))))) {
                        live[off + j] = sn.pre_mem1[off + j];
                        ++undone;
                    }
                }
            }
        }
        for (i = 0; i < sn.nranges && sn_render_mode() != 3; ++i) {
            uint8_t *g = (uint8_t *) (uintptr_t) sn.ranges[i].va;
            const uint8_t *pre = sn.pre_globals + base;
            if (memcmp(pre, g, sn.ranges[i].len) != 0) {
                uint32_t j;
                for (j = 0; j < sn.ranges[i].len; ++j) {
                    uint32_t gb = base + j;
                    if (pre[j] != g[j] &&
                        !(sn_amask_globals != NULL && (sn_amask_globals[gb >> 3] & (1u << (gb & 7))))) {
                        if (sn_render_mode() == 2 && sn.passes < 3) {
                            const GwSnapSym *y = sn_sym_at(sn.ranges[i].va + j);
                            gw_log("snap: render wrote global %s+0x%X", y ? y->name : "?",
                                   y ? sn.ranges[i].va + j - y->va : 0);
                        }
                        g[j] = pre[j];
                        ++undone;
                    }
                }
            }
            base += sn.ranges[i].len;
        }
        if (undone != 0 && sn.passes < 4) {
            gw_log("snap: render pass undone: %u bytes put back", undone);
        }
    }
}

/* Called at the top of the next logic frame (gw_SyncTest_IterStart), not at the end of the render
 * pass: the render pass is not the only thing that runs between two logic frames. The deferred
 * callback queue and the asynchronous DVD/stream completions also write - the first mismatch this
 * chased down was the stream-header read queue (lbdvd + synth pstream), whose two entries swap
 * every other frame. All of it is state a resimulated frame cannot reproduce, because a
 * resimulated frame runs logic and nothing else. */
static void sn_mark_window(void) {
    uint32_t a, g;
    if (!sn.enabled || !sn.pre_valid) {
        return;
    }
    sn.pre_valid = 0;
    a = sn_mark(sn.pre_mem1, (const uint8_t *) (uintptr_t) 0x80000000u, gw_mem1_size, sn.rmask_mem1);
    sn_gather(sn.cmp_globals);
    g = sn_mark(sn.pre_globals, sn.cmp_globals, sn.globals_len, sn.rmask_globals);
    sn.rmask_bytes += a + g;
    if ((a + g) != 0 && sn.passes < 4) {
        gw_log("snap: between logic frames: %u new MEM1 bytes and %u global bytes (%u not-logic so far)",
               a, g, sn.rmask_bytes);
    }
}

/* Sound effects must not replay while resimulating (lbaudio_ax.c). MELEE_SYNCTEST_SFX=1 lets them
 * play again, to tell an audio-owned mismatch apart from a logic one. */
int gw_Snap_Resimulating(void) {
    return sn.enabled && sn.cur_is_resim;
}

/* The audio gate only (axdriver.c): separate from gw_Snap_Resimulating so that turning sound back
 * on for an experiment does not also re-enable the replay traces. */
int gw_Snap_SuppressSfx(void) {
    static int on = -1;
    if (on < 0) {
        const char *v = getenv("MELEE_SYNCTEST_SFX");
        on = (v != NULL && *v == '1') ? 0 : 1;
    }
    return on && sn.enabled && sn.cur_is_resim;
}

/* ---- for the rollback session (gw_rollback.c) --------------------------------------------------
 * The session drives save/load itself; it needs the slots allocated (sn_init reads
 * MELEE_SYNCTEST, so it is set here), the resimulation flag that gates sound, and a checksum. */
int gw_Snap_OpenSession(int k) {
    char b[16];
    snprintf(b, sizeof b, "%d", k);
    _putenv_s("MELEE_SYNCTEST", b);
    sn_init();
    sn.session = sn.enabled;
    return sn.enabled ? sn.nslots : 0;
}

void gw_Snap_SessionResim(int on, int frame) {
    sn.cur_is_resim = on;
    if (on) {
        sn_sfx_rewind(frame);
    }
}

int gw_Snap_HasFrame(int frame) {
    return sn_slot_for(frame, 0) != NULL;
}

/* A 32-bit checksum of the snapshot taken at the start of `frame`. 0 when there is no such
 * snapshot. In dirty-page mode (the default) it is a fold of gw_snap_hash() as computed at save time:
 * incremental (~0.25 ms per frame, only pages written since the last call are rehashed), with the
 * disc's asynchronous state (DVD stream blocks, devcom request nodes) hashed as zero and the pad /
 * rumble / render-display globals left out - state that legitimately differs between two peers'
 * machines. In full-copy mode (MELEE_SNAP_MODE=full, or MELEE_SNAP_HASH=0) it falls back to hashing
 * the whole slot, every byte, several milliseconds. Either way the render-owned bytes are included:
 * two peers running the same build see the same ones. */
uint32_t gw_Snap_Checksum(int frame) {
    GwSnapSlot *sl = sn_slot_for(frame, 0);
    uint64_t h = 0x9E3779B97F4A7C15ull;
    uint32_t i;
    const uint64_t *w;
    if (sl == NULL) {
        return 0;
    }
    if (sn.hash_on && sl->hash != 0) {
        return (uint32_t) (sl->hash ^ (sl->hash >> 32)) | 1u;
    }
    w = (const uint64_t *) sl->mem1;
    for (i = 0; i < gw_mem1_size / 8; ++i) {
        h = (h ^ w[i]) * 0x100000001B3ull;
        h ^= h >> 29;
    }
    w = (const uint64_t *) sl->globals;
    for (i = 0; i < sn.globals_len / 8; ++i) {
        h = (h ^ w[i]) * 0x100000001B3ull;
        h ^= h >> 29;
    }
    return (uint32_t) (h ^ (h >> 32)) | 1u;
}
