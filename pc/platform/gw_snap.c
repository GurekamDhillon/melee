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
#define GW_SNAP_MAX_SLOTS 48 /* slots are allocated on demand: gw_snap_open, gw_snap_reserve */

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

/* the Lab's long rewind (end of this file): its own "written since" page set, fed by sn_poll */
static uint64_t *sn_rw_dirty;
static void rw_render_pre(void);
static void rw_render_post(void);

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
               strncmp(obj, "pc_geno_", 8) == 0 || /* Geno's game half (pc/geno): per-fighter state */
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
        if (sn_rw_dirty != NULL) {
            sn_bm_fill(sn_rw_dirty, sn.npages);
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
        if (sn_rw_dirty != NULL) {
            sn_bm_set(sn_rw_dirty, pg);
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

static void sn_save_to(GwSnapSlot *s, int frame) {
    double t0 = sn_ms();
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

static int sn_load_from(GwSnapSlot *s) {
    double t0 = sn_ms();
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
        if (sn_rw_dirty != NULL) {
            uint32_t w; /* the Lab rewind's set: the copy's pages are written as far as it knows */
            for (w = 0; w < SN_BM_WORDS(sn.npages); ++w) {
                sn_rw_dirty[w] |= s->dirty[w];
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

void gw_snap_save(int frame) {
    GwSnapSlot *s = sn_slot_for(frame, 1);
    if (s != NULL) {
        sn_save_to(s, frame);
    }
}

int gw_snap_load(int frame) {
    GwSnapSlot *s = sn_slot_for(frame, 0);
    return s != NULL ? sn_load_from(s) : -1;
}

/* ---- slots by INDEX (the Lua savestates and the Geno Lab's history ring, gw_script.c) ----------
 * The frame-keyed calls above evict the lowest key when a new one arrives, which is right for
 * SyncTest and rollback but wrong for savestates the user named. These address a slot directly;
 * `tag` is only stored (it is the slot's "frame" for the keyed calls, so pick tags no session
 * will ask for - negative ones). gw_snap_reserve(n) makes sure n slots exist: it opens the
 * machinery with n slots, or grows an open one (up to GW_SNAP_MAX_SLOTS); each slot costs a MEM1
 * copy (24 MB) plus the game globals. Returns the number of slots now available (0 = failed). */
int gw_snap_open(int k);

int gw_snap_reserve(int n) {
    int i;
    if (n > GW_SNAP_MAX_SLOTS) {
        n = GW_SNAP_MAX_SLOTS;
    }
    if (!sn.enabled) {
        if (gw_snap_open(n < 3 ? 1 : n - 2) != 0) {
            return 0;
        }
    }
    for (i = sn.nslots; i < n; ++i) {
        GwSnapSlot *s = &sn.slot[i];
        s->frame = -0x7FFFFFFF - 1;
        s->mem1 = (uint8_t *) malloc(gw_mem1_size);
        s->globals = (uint8_t *) malloc(sn.globals_len);
        s->dirty = (uint64_t *) malloc(SN_BM_WORDS(sn.npages) * 8);
        if (s->mem1 == NULL || s->globals == NULL || s->dirty == NULL) {
            free(s->mem1);
            free(s->globals);
            free(s->dirty);
            s->mem1 = NULL;
            s->globals = NULL;
            s->dirty = NULL;
            gw_log("snap: out of memory growing to %d slots (have %d)", n, sn.nslots);
            break;
        }
        sn_bm_fill(s->dirty, sn.npages); /* never synced: every page differs */
        sn.nslots = i + 1;
    }
    return sn.nslots;
}

int gw_snap_slot_count(void) { return sn.enabled ? sn.nslots : 0; }

/* bytes one slot costs */
uint32_t gw_snap_slot_bytes(void) { return gw_mem1_size + sn.globals_len; }

void gw_snap_save_index(int idx, int tag) {
    if (sn.enabled && idx >= 0 && idx < sn.nslots) {
        sn_save_to(&sn.slot[idx], tag);
    }
}

int gw_snap_load_index(int idx) {
    if (!sn.enabled || idx < 0 || idx >= sn.nslots) {
        return -1;
    }
    return sn_load_from(&sn.slot[idx]);
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
        /* opened already - possibly bare (k = 0, the Lab's rewind): make sure k+2 slots exist */
        if (k > 0 && sn.nslots < k + 2) {
            if (sn.k < k) {
                sn.k = k > GW_SNAP_MAX_SLOTS - 2 ? GW_SNAP_MAX_SLOTS - 2 : k;
            }
            return gw_snap_reserve(sn.k + 2) >= sn.k + 2 ? 0 : -1;
        }
        return 0;
    }
    if (k < 0) {
        return -1;
    }
    /* k = 0: the machinery only (map, ranges, write-watch), no slots - the Lab's rewind keeps its
       own storage; gw_snap_reserve adds slots later if a savestate wants them */
    sn.k = k > GW_SNAP_MAX_SLOTS - 2 ? GW_SNAP_MAX_SLOTS - 2 : (k < 1 ? 1 : k);
    if (sn_load_map() != 0) {
        return -1;
    }
    sn.nslots = k == 0 ? 0 : sn.k + 2;
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
    if (!sn.enabled || sn.nslots < 2 || !sn_live()) {
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
    rw_render_post(); /* the Lab's render-owned measurement: the window closes at the next logic frame */
    if (!sn.enabled || sn.nslots < 2 || !sn_live()) {
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
    unsigned char claimed[GW_SNAP_SFX_PER_FRAME]; /* re-emitted by the resimulation now running */
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
        sn_sfx[s].claimed[sn_sfx[s].n] = 1;
        sn_sfx[s].n++;
    }
}

static void sn_sfx_rewind(int frame) {
    int s = (frame & 0x7FFFFFFF) % GW_SNAP_SFX_FRAMES;
    if (sn_sfx[s].frame == frame) {
        sn_sfx[s].taken = 0;
        memset(sn_sfx[s].claimed, 0, sizeof sn_sfx[s].claimed); /* all candidates again */
    }
}

/* A resimulated frame starts a sound: if the first pass (or an earlier resimulation) already
 * played one with this id in this frame and nothing has claimed it yet, this IS that sound - it is
 * already playing, so hand back its handle. Matching is by id, not by order: a corrected timeline
 * that emits the same sounds in a different order must not double-play them. GW_SFX_PLAY_NEW (-2):
 * a sound this timeline plays that the abandoned one did not - the caller plays it for real. */
int gw_Snap_SfxTake(int sound_id) {
    int f = gw_Replay_Frame();
    int s = (f & 0x7FFFFFFF) % GW_SNAP_SFX_FRAMES;
    if (sn_sfx[s].frame == f) {
        int i;
        for (i = 0; i < sn_sfx[s].n; ++i) {
            if (!sn_sfx[s].claimed[i] && sn_sfx[s].id[i] == sound_id) {
                sn_sfx[s].claimed[i] = 1;
                return sn_sfx[s].res[i];
            }
        }
    }
    if (sn_sfx_misses++ < 8) {
        gw_log("snap: resimulated frame %d starts sound %d the first pass did not - playing it", f, sound_id);
    }
    return -2;
}

/* The sound the caller just played for real during a resimulated frame: remembered (claimed), so a
 * later resimulation of the same frame finds it. */
void gw_Snap_SfxAdd(int sound_id, int result) {
    int f = gw_Replay_Frame();
    int s = (f & 0x7FFFFFFF) % GW_SNAP_SFX_FRAMES;
    if (sn_sfx[s].frame != f) {
        sn_sfx[s].frame = f;
        sn_sfx[s].n = 0;
    }
    if (sn_sfx[s].n < GW_SNAP_SFX_PER_FRAME) {
        sn_sfx[s].id[sn_sfx[s].n] = sound_id;
        sn_sfx[s].res[sn_sfx[s].n] = result;
        sn_sfx[s].claimed[sn_sfx[s].n] = 1;
        sn_sfx[s].n++;
    }
}

/* The end of a resimulated frame: sounds the abandoned timeline started in it that this one did
 * not are still playing - release them (the game's own key-off) and forget them. */
extern int gw_HSD_AudioSFXKeyOff(int vid);
int gw_Snap_SfxFrameEnd(int frame) {
    int s = (frame & 0x7FFFFFFF) % GW_SNAP_SFX_FRAMES, i, j = 0, killed = 0;
    if (sn_sfx[s].frame != frame) {
        return 0;
    }
    for (i = 0; i < sn_sfx[s].n; ++i) {
        if (!sn_sfx[s].claimed[i]) {
            if (sn_sfx[s].res[i] >= 0) {
                gw_HSD_AudioSFXKeyOff(sn_sfx[s].res[i]);
            }
            ++killed;
            continue;
        }
        sn_sfx[s].id[j] = sn_sfx[s].id[i];
        sn_sfx[s].res[j] = sn_sfx[s].res[i];
        sn_sfx[s].claimed[j] = 1;
        ++j;
    }
    sn_sfx[s].n = j;
    return killed;
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
    rw_render_pre();
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
    /* sn_init decides once, at the first scene-loop tick after boot. A session armed later (netplay
       from the online menu) finds it already decided "off": open directly. */
    if (!sn.enabled) {
        gw_snap_open(k);
    }
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

/* ==============================================================================================
 * THE GENO LAB'S LONG REWIND (gw_script.c drives it: gd.history / gd.step_back / gd.rewind_to).
 *
 * A full slot is 27 MB, so the old ring (one slot per frame) stopped at 40 frames. This keeps ONE
 * full image (the BASE, the oldest frame you can reach) and, every N frames, a DELTA KEYFRAME:
 *   - MEM1: the pages written since the previous keyframe (the write-watch set sn_poll feeds into
 *     sn_rw_dirty), copied as they are now. A keyframe's MEM1 = base + every earlier delta + its
 *     own, the newest copy of each page winning.
 *   - the game globals: the 4 KB chunks that differ from the previous keyframe's globals.
 * The caller (gw_script.c) keeps a per-frame INPUT LOG beside it; a frame between two keyframes is
 * reached by loading the keyframe before it and re-simulating forward on the logged inputs.
 *
 * Loading keyframe j: the live MEM1 can differ from state(j) only on the pages written since the
 * live state last WAS a keyframe (rw.dirty, relative to rw.dirty_ref) plus the pages of every
 * keyframe between that one and j. Only those pages are copied, each from the newest delta <= j
 * that has it, else from the base. The live-vs-keyframe relation (dirty_ref) is maintained through
 * saves, loads, drops and trims, so a delta never misses a page.
 * The window slides: a keyframe older than the window is folded into the base (its pages copied
 * over the base image) and freed.
 * ============================================================================================== */
/* The disc's streaming reads (music): four DVD command blocks of SN_DEVCOM_NODE bytes, allocated at
 * boot, that shim_dvd advances on its own schedule. The snapshot's fixed async range covers the first
 * three; the Lab's rewind measured the fourth moving too, so a rewind load leaves all four as they
 * are (like devcom's nodes) and the exactness test does not count them. */
#define SN_STREAM_VA 0x80171160u
#define SN_STREAM_LEN (4u * SN_DEVCOM_NODE)

#define RW_MAX_KEYS 512
#define RW_CH 4096u
#define RW_USER 128

typedef struct {
    int tag;
    uint32_t npg, ngc;
    uint32_t *pg;  /* page indices, ascending */
    uint8_t *pgd;  /* npg pages */
    uint32_t *gc;  /* globals chunk indices, ascending */
    uint8_t *gcd;  /* ngc chunks of RW_CH (the last chunk of the globals may be short) */
    int cursor[4];
    unsigned char user[RW_USER];
    size_t bytes;
} RwKey;

static struct {
    int on;
    uint8_t *base_mem1, *base_glob, *last_glob, *scratch;
    int base_tag, base_cursor[4];
    unsigned char base_user[RW_USER];
    RwKey key[RW_MAX_KEYS];
    int nkeys;
    uint64_t *dirty, *set;
    int dirty_ref;
    size_t delta_bytes;
    uint32_t nchunks;
    double ms_save, ms_load, ms_last_load;
    long n_save, n_load;
    uint32_t last_load_pages;
    /* the exactness self-test's full copy */
    uint8_t *cap_mem1, *cap_glob;
} rw;

static uint32_t rw_chunk_len(uint32_t c) {
    uint32_t off = c * RW_CH;
    return sn.globals_len - off < RW_CH ? sn.globals_len - off : RW_CH;
}

static void rw_free_key(RwKey *k) {
    rw.delta_bytes -= k->bytes;
    free(k->pg);
    free(k->pgd);
    free(k->gc);
    free(k->gcd);
    memset(k, 0, sizeof *k);
}

static int rw_latest_tag(void) { return rw.nkeys > 0 ? rw.key[rw.nkeys - 1].tag : rw.base_tag; }

/* index of the keyframe with this tag: -1 = the base, -2 = none */
static int rw_index(int tag) {
    int i;
    if (tag == rw.base_tag) {
        return -1;
    }
    for (i = 0; i < rw.nkeys; ++i) {
        if (rw.key[i].tag == tag) {
            return i;
        }
    }
    return -2;
}

static void rw_or_pages(uint64_t *bm, const RwKey *k) {
    uint32_t i;
    for (i = 0; i < k->npg; ++i) {
        sn_bm_set(bm, k->pg[i]);
    }
}

static const uint8_t *rw_page_src(uint32_t pg, int upto) {
    int i;
    for (i = upto; i >= 0; --i) {
        const RwKey *k = &rw.key[i];
        int lo = 0, hi = (int) k->npg - 1;
        while (lo <= hi) {
            int mid = (lo + hi) / 2;
            if (k->pg[mid] == pg) {
                return k->pgd + (size_t) mid * SN_PAGE;
            }
            if (k->pg[mid] < pg) {
                lo = mid + 1;
            } else {
                hi = mid - 1;
            }
        }
    }
    return rw.base_mem1 + (size_t) pg * SN_PAGE;
}

/* the globals as they were at keyframe index `upto` (-1 = the base) */
static void rw_globals_at(int upto, uint8_t *out) {
    int i;
    uint32_t j;
    memcpy(out, rw.base_glob, sn.globals_len);
    for (i = 0; i <= upto; ++i) {
        const RwKey *k = &rw.key[i];
        for (j = 0; j < k->ngc; ++j) {
            memcpy(out + (size_t) k->gc[j] * RW_CH, k->gcd + (size_t) j * RW_CH, rw_chunk_len(k->gc[j]));
        }
    }
}

/* set the pages of keyframes with index in (lo, hi] */
static void rw_or_range(uint64_t *bm, int lo, int hi) {
    int i;
    for (i = lo + 1; i <= hi; ++i) {
        rw_or_pages(bm, &rw.key[i]);
    }
}

static void rw_drop_all_keys(void) {
    while (rw.nkeys > 0) {
        rw_free_key(&rw.key[--rw.nkeys]);
    }
}

void gw_rw_end(void) {
    rw_drop_all_keys();
    free(rw.base_mem1);
    free(rw.base_glob);
    free(rw.last_glob);
    free(rw.scratch);
    free(rw.dirty);
    free(rw.set);
    sn_rw_dirty = NULL;
    rw.base_mem1 = rw.base_glob = rw.last_glob = rw.scratch = NULL;
    rw.dirty = rw.set = NULL;
    rw.on = 0;
    rw.delta_bytes = 0;
}

int gw_rw_active(void) { return rw.on; }

/* Start (or restart) the rewind with the live state as the base, tagged `tag`. 0 = ok. */
int gw_rw_begin(int tag, const void *user, int ulen) {
    size_t bmb;
    if (!sn.enabled && gw_snap_open(0) != 0) {
        return -1;
    }
    bmb = SN_BM_WORDS(sn.npages) * 8;
    rw_drop_all_keys();
    if (rw.base_mem1 == NULL) {
        rw.base_mem1 = (uint8_t *) malloc(gw_mem1_size);
        rw.base_glob = (uint8_t *) malloc(sn.globals_len);
        rw.last_glob = (uint8_t *) malloc(sn.globals_len);
        rw.scratch = (uint8_t *) malloc(sn.globals_len);
        rw.dirty = (uint64_t *) malloc(bmb);
        rw.set = (uint64_t *) malloc(bmb);
        if (rw.base_mem1 == NULL || rw.base_glob == NULL || rw.last_glob == NULL ||
            rw.scratch == NULL || rw.dirty == NULL || rw.set == NULL) {
            gw_log("rewind: out of memory for the base image");
            gw_rw_end();
            return -1;
        }
    }
    rw.nchunks = (sn.globals_len + RW_CH - 1) / RW_CH;
    sn_poll();
    memcpy(rw.base_mem1, (const void *) (uintptr_t) 0x80000000u, gw_mem1_size);
    sn_gather(rw.base_glob);
    memcpy(rw.last_glob, rw.base_glob, sn.globals_len);
    memset(rw.dirty, 0, bmb);
    sn_rw_dirty = rw.dirty;
    rw.base_tag = tag;
    rw.dirty_ref = tag;
    gw_Replay_GetCursor(rw.base_cursor);
    memset(rw.base_user, 0, RW_USER);
    if (user != NULL && ulen > 0) {
        memcpy(rw.base_user, user, ulen > (int) RW_USER ? RW_USER : (size_t) ulen);
    }
    rw.on = 1;
    return 0;
}

/* Save a keyframe of the live state, tagged `tag` (> every stored tag). 0 = ok. */
int gw_rw_save(int tag, const void *user, int ulen) {
    double t0 = sn_ms();
    size_t bmb;
    uint32_t pg, n = 0, c;
    RwKey *k;
    int latest, ref;
    if (!rw.on || tag <= rw_latest_tag() || rw.nkeys >= RW_MAX_KEYS) {
        return -1;
    }
    bmb = SN_BM_WORDS(sn.npages) * 8;
    latest = rw_latest_tag();
    sn_poll();
    if (!sn.dirty_mode) {
        sn_bm_fill(rw.dirty, sn.npages);
    }
    memcpy(rw.set, rw.dirty, bmb);
    ref = rw_index(rw.dirty_ref);
    if (rw.dirty_ref != latest) {
        /* live = state(ref) + dirty; state(latest) = state(ref) + the keyframes after ref */
        if (ref == -2) {
            sn_bm_fill(rw.set, sn.npages);
        } else {
            rw_or_range(rw.set, ref, rw.nkeys - 1);
        }
    }
    for (pg = 0; pg < sn.npages; ++pg) {
        if (sn_bm_test(rw.set, pg)) {
            ++n;
        }
    }
    k = &rw.key[rw.nkeys];
    memset(k, 0, sizeof *k);
    k->tag = tag;
    k->pg = (uint32_t *) malloc((size_t) (n ? n : 1) * 4);
    k->pgd = (uint8_t *) malloc((size_t) (n ? n : 1) * SN_PAGE);
    if (k->pg == NULL || k->pgd == NULL) {
        free(k->pg);
        free(k->pgd);
        memset(k, 0, sizeof *k);
        gw_log("rewind: out of memory for a keyframe (%u pages)", n);
        return -1;
    }
    n = 0;
    for (pg = 0; pg < sn.npages; ++pg) {
        if (sn_bm_test(rw.set, pg)) {
            k->pg[n] = pg;
            memcpy(k->pgd + (size_t) n * SN_PAGE,
                   (const uint8_t *) (uintptr_t) 0x80000000u + (size_t) pg * SN_PAGE, SN_PAGE);
            ++n;
        }
    }
    k->npg = n;
    /* the globals: the chunks that changed since the previous keyframe */
    sn_gather(rw.scratch);
    n = 0;
    for (c = 0; c < rw.nchunks; ++c) {
        if (memcmp(rw.scratch + (size_t) c * RW_CH, rw.last_glob + (size_t) c * RW_CH, rw_chunk_len(c)) != 0) {
            ++n;
        }
    }
    k->gc = (uint32_t *) malloc((size_t) (n ? n : 1) * 4);
    k->gcd = (uint8_t *) malloc((size_t) (n ? n : 1) * RW_CH);
    if (k->gc == NULL || k->gcd == NULL) {
        free(k->pg);
        free(k->pgd);
        free(k->gc);
        free(k->gcd);
        memset(k, 0, sizeof *k);
        gw_log("rewind: out of memory for a keyframe's globals");
        return -1;
    }
    n = 0;
    for (c = 0; c < rw.nchunks; ++c) {
        uint32_t len = rw_chunk_len(c);
        if (memcmp(rw.scratch + (size_t) c * RW_CH, rw.last_glob + (size_t) c * RW_CH, len) != 0) {
            k->gc[n] = c;
            memcpy(k->gcd + (size_t) n * RW_CH, rw.scratch + (size_t) c * RW_CH, len);
            memcpy(rw.last_glob + (size_t) c * RW_CH, rw.scratch + (size_t) c * RW_CH, len);
            ++n;
        }
    }
    k->ngc = n;
    gw_Replay_GetCursor(k->cursor);
    if (user != NULL && ulen > 0) {
        memcpy(k->user, user, ulen > (int) RW_USER ? RW_USER : (size_t) ulen);
    }
    k->bytes = (size_t) k->npg * (SN_PAGE + 4) + (size_t) k->ngc * (RW_CH + 4);
    rw.delta_bytes += k->bytes;
    rw.nkeys++;
    memset(rw.dirty, 0, bmb);
    rw.dirty_ref = tag;
    rw.ms_save += sn_ms() - t0;
    rw.n_save++;
    return 0;
}

/* Load the keyframe tagged `tag` (the base included) into the live state. 0 = ok. */
int gw_rw_load(int tag, void *user_out, int ulen) {
    double t0 = sn_ms();
    uint8_t stream[4 * 0x24];
    size_t bmb;
    int idx, ref, lo, hi, u;
    uint32_t pg, copied = 0, w;
    uint8_t *live = (uint8_t *) (uintptr_t) 0x80000000u;
    if (!rw.on || (idx = rw_index(tag)) == -2) {
        return -1;
    }
    bmb = SN_BM_WORDS(sn.npages) * 8;
    sn_boundary_asserts("rewind load");
    sn_async_collect();
    sn_fixed_collect();
    memcpy(stream, (const void *) (uintptr_t) SN_STREAM_VA, SN_STREAM_LEN);
    sn_poll();
    if (!sn.dirty_mode) {
        sn_bm_fill(rw.dirty, sn.npages);
    }
    memcpy(rw.set, rw.dirty, bmb);
    ref = rw_index(rw.dirty_ref);
    if (ref == -2) {
        sn_bm_fill(rw.set, sn.npages);
    } else {
        lo = ref < idx ? ref : idx;
        hi = ref < idx ? idx : ref;
        rw_or_range(rw.set, lo, hi);
    }
    for (pg = 0; pg < sn.npages; ++pg) {
        if (!sn_bm_test(rw.set, pg)) {
            continue;
        }
        memcpy(live + (size_t) pg * SN_PAGE, rw_page_src(pg, idx), SN_PAGE);
        ++copied;
    }
    rw_globals_at(idx, rw.scratch);
    sn_scatter(rw.scratch);
    sn_async_put_back();
    sn_fixed_put_back();
    memcpy((void *) (uintptr_t) SN_STREAM_VA, stream, SN_STREAM_LEN); /* all four stream blocks */
    /* every other consumer of the write-watch: those pages changed under them */
    for (u = 0; u < sn.nslots; ++u) {
        for (w = 0; w < SN_BM_WORDS(sn.npages); ++w) {
            sn.slot[u].dirty[w] |= rw.set[w];
        }
    }
    if (sn.hash_dirty != NULL) {
        for (w = 0; w < SN_BM_WORDS(sn.npages); ++w) {
            sn.hash_dirty[w] |= rw.set[w];
        }
    }
    sn_watch_reset();
    memset(rw.dirty, 0, bmb);
    rw.dirty_ref = tag;
    gw_Replay_SetCursor(idx < 0 ? rw.base_cursor : rw.key[idx].cursor);
    if (user_out != NULL && ulen > 0) {
        memcpy(user_out, idx < 0 ? rw.base_user : rw.key[idx].user,
               ulen > (int) RW_USER ? RW_USER : (size_t) ulen);
    }
    rw.last_load_pages = copied;
    rw.ms_last_load = sn_ms() - t0;
    rw.ms_load += rw.ms_last_load;
    rw.n_load++;
    return 0;
}

/* Drop every keyframe tagged >= tag (a new timeline starts there). -1 when the base itself would
 * go (the caller restarts the rewind). */
int gw_rw_drop_from(int tag) {
    int keep, ref;
    if (!rw.on || rw.base_tag >= tag) {
        return -1;
    }
    for (keep = rw.nkeys; keep > 0 && rw.key[keep - 1].tag >= tag; --keep) {
    }
    if (keep == rw.nkeys) {
        return 0;
    }
    ref = rw_index(rw.dirty_ref);
    if (ref >= keep) {
        /* live = state(ref) + dirty; state(ref) = state(the newest kept) + the dropped deltas to ref */
        rw_or_range(rw.dirty, keep - 1, ref);
        rw.dirty_ref = keep > 0 ? rw.key[keep - 1].tag : rw.base_tag;
    } else if (ref == -2) {
        sn_bm_fill(rw.dirty, sn.npages);
        rw.dirty_ref = keep > 0 ? rw.key[keep - 1].tag : rw.base_tag;
    }
    while (rw.nkeys > keep) {
        rw_free_key(&rw.key[--rw.nkeys]);
    }
    rw_globals_at(rw.nkeys - 1, rw.last_glob);
    return 0;
}

/* Fold keyframes tagged <= limit into the base, so the base is the newest keyframe <= limit. */
void gw_rw_trim(int limit) {
    while (rw.on && rw.nkeys > 0 && rw.key[0].tag <= limit) {
        RwKey *k = &rw.key[0];
        uint32_t j;
        for (j = 0; j < k->npg; ++j) {
            memcpy(rw.base_mem1 + (size_t) k->pg[j] * SN_PAGE, k->pgd + (size_t) j * SN_PAGE, SN_PAGE);
        }
        for (j = 0; j < k->ngc; ++j) {
            memcpy(rw.base_glob + (size_t) k->gc[j] * RW_CH, k->gcd + (size_t) j * RW_CH,
                   rw_chunk_len(k->gc[j]));
        }
        if (rw.dirty_ref == rw.base_tag) {
            rw_or_pages(rw.dirty, k);
            rw.dirty_ref = k->tag;
        }
        rw.base_tag = k->tag;
        memcpy(rw.base_cursor, k->cursor, sizeof rw.base_cursor);
        memcpy(rw.base_user, k->user, RW_USER);
        rw_free_key(k);
        memmove(&rw.key[0], &rw.key[1], (size_t) (rw.nkeys - 1) * sizeof rw.key[0]);
        rw.nkeys--;
        memset(&rw.key[rw.nkeys], 0, sizeof rw.key[0]);
    }
}

/* the newest keyframe tag <= tag (the base included), INT_MIN when there is none */
int gw_rw_find_le(int tag) {
    int i;
    if (!rw.on || rw.base_tag > tag) {
        return -0x7FFFFFFF - 1;
    }
    for (i = rw.nkeys - 1; i >= 0; --i) {
        if (rw.key[i].tag <= tag) {
            return rw.key[i].tag;
        }
    }
    return rw.base_tag;
}

int gw_rw_base_tag(void) { return rw.on ? rw.base_tag : 0; }
int gw_rw_latest(void) { return rw.on ? rw_latest_tag() : 0; }
int gw_rw_count(void) { return rw.on ? rw.nkeys + 1 : 0; }

/* bytes held: the base image, the three globals buffers, the bitmaps and every delta */
double gw_rw_bytes(void) {
    double b;
    if (!rw.on) {
        return 0;
    }
    b = (double) gw_mem1_size + 3.0 * sn.globals_len + 2.0 * SN_BM_WORDS(sn.npages) * 8;
    return b + (double) rw.delta_bytes;
}
double gw_rw_delta_bytes(void) { return rw.on ? (double) rw.delta_bytes : 0; }
double gw_rw_ms_last_load(void) { return rw.ms_last_load; }
double gw_rw_ms_save_avg(void) { return rw.n_save ? rw.ms_save / rw.n_save : 0; }
uint32_t gw_rw_last_load_pages(void) { return rw.last_load_pages; }

/* ---- the exactness self-test: a full copy now, a byte compare later ----------------------------- */
int gw_rw_capture(void) {
    if (!sn.enabled) {
        return -1;
    }
    if (rw.cap_mem1 == NULL) {
        rw.cap_mem1 = (uint8_t *) malloc(gw_mem1_size);
        rw.cap_glob = (uint8_t *) malloc(sn.globals_len);
        if (rw.cap_mem1 == NULL || rw.cap_glob == NULL) {
            free(rw.cap_mem1);
            free(rw.cap_glob);
            rw.cap_mem1 = rw.cap_glob = NULL;
            return -1;
        }
    }
    memcpy(rw.cap_mem1, (const void *) (uintptr_t) 0x80000000u, gw_mem1_size);
    sn_gather(rw.cap_glob);
    return 0;
}

static int sn_in_cmp_skip(uint32_t va) {
    int i;
    for (i = 0; i < sn.ncmp_skip; ++i) {
        if (va >= sn.cmp_skip[i].va && va < sn.cmp_skip[i].va + sn.cmp_skip[i].len) {
            return 1;
        }
    }
    return 0;
}


/* ---- the self-test's render-owned measurement ---------------------------------------------------
 * While gd.rewind_test runs, every byte that changes between the start of a render pass and the
 * next logic frame (the render, the deferred callbacks, the tick's non-logic work) is marked, as
 * SyncTest marks its "not written by logic" window. Such a byte is render-owned: the render pass
 * advances it per render, not per logic frame (the particle display, the light list, the object
 * pools it takes cells from), so a paused Lab - which renders without running logic - and a
 * re-simulated frame - which skips render callbacks (gw_Snap_SkipRenderCb) - move it differently
 * from the first run. The compare leaves marked bytes out and reports how many there were. */
static struct {
    int on, pre_valid;
    uint8_t *pre_mem1, *pre_glob, *tmp_glob, *mask_mem1, *mask_glob;
} rwm;

void gw_rw_measure(int on) {
    if (on && !rwm.on && sn.enabled) {
        rwm.pre_mem1 = (uint8_t *) malloc(gw_mem1_size);
        rwm.pre_glob = (uint8_t *) malloc(sn.globals_len);
        rwm.tmp_glob = (uint8_t *) malloc(sn.globals_len);
        rwm.mask_mem1 = (uint8_t *) calloc(gw_mem1_size / 8 + 1, 1);
        rwm.mask_glob = (uint8_t *) calloc(sn.globals_len / 8 + 1, 1);
        rwm.on = rwm.pre_mem1 && rwm.pre_glob && rwm.tmp_glob && rwm.mask_mem1 && rwm.mask_glob;
        rwm.pre_valid = 0;
        if (!rwm.on) {
            on = 0;
        }
    }
    if (!on) {
        free(rwm.pre_mem1);
        free(rwm.pre_glob);
        free(rwm.tmp_glob);
        free(rwm.mask_mem1);
        free(rwm.mask_glob);
        memset(&rwm, 0, sizeof rwm);
    }
}

static void rw_render_post(void) {
    if (!rwm.on || !rwm.pre_valid) {
        return;
    }
    rwm.pre_valid = 0;
    (void) sn_mark(rwm.pre_mem1, (const uint8_t *) (uintptr_t) 0x80000000u, gw_mem1_size, rwm.mask_mem1);
    sn_gather(rwm.tmp_glob);
    (void) sn_mark(rwm.pre_glob, rwm.tmp_glob, sn.globals_len, rwm.mask_glob);
}

static void rw_render_pre(void) {
    if (!rwm.on) {
        return;
    }
    rw_render_post(); /* two renders with no logic frame between (a paused Lab) */
    memcpy(rwm.pre_mem1, (const void *) (uintptr_t) 0x80000000u, gw_mem1_size);
    sn_gather(rwm.pre_glob);
    rwm.pre_valid = 1;
}

/* word-granular: game data is 32-bit words, and a render-written float can keep its top byte over
   the renders measured */
static int rw_render_owned_mem1(uint32_t off) {
    return rwm.on && ((rwm.mask_mem1[off >> 3] >> (off & 4)) & 0xFu) != 0;
}

static int rw_render_owned_glob(uint32_t off) {
    return rwm.on && ((rwm.mask_glob[off >> 3] >> (off & 4)) & 0xFu) != 0;
}

/* ---- what the exactness compare does not count ------------------------------------------------
 * 1. the disc's asynchronous state: the stream command blocks and devcom's request nodes;
 * 2. the pad-side globals SyncTest never compares either (pad statuses, rumble, the render counter);
 * 3. THE LIGHT LIST. Every render pass builds the current-lights list (lobj.c `current_lights`) out
 *    of HSD_SList nodes from list.c's `slist_alloc_data` and frees it again; the free list is LIFO,
 *    so which node holds which light - and both list heads - flip with the number of renders since
 *    the match began. A paused Lab renders without running logic, so a replay of a timeline after a
 *    pause meets the same lights in swapped nodes. Logic never reads the list: render-owned, like
 *    the particle display SyncTest masks. The nodes are found by walking both lists in both
 *    images. */
#define RW_EX_MAX 1200
static struct {
    uint32_t va, len;
} rw_ex[RW_EX_MAX];
static int rw_nex;
static uint32_t rw_lights_va, rw_slist_va;

static void rw_ex_add(uint32_t va, uint32_t len) {
    if (rw_nex < RW_EX_MAX) {
        rw_ex[rw_nex].va = va;
        rw_ex[rw_nex].len = len;
        rw_nex++;
    }
}

static int rw_ex_cmp(const void *a, const void *b) {
    uint32_t x = ((const uint32_t *) a)[0], y = ((const uint32_t *) b)[0];
    return x < y ? -1 : x > y;
}

/* a global's bytes inside a gathered globals image */
static const uint8_t *rw_glob_at(const uint8_t *img, uint32_t va) {
    uint32_t base = 0;
    int r;
    for (r = 0; r < sn.nranges; ++r) {
        if (va >= sn.ranges[r].va && va + 4 <= sn.ranges[r].va + sn.ranges[r].len) {
            return img + base + (va - sn.ranges[r].va);
        }
        base += sn.ranges[r].len;
    }
    return NULL;
}

static uint32_t rw_be(const uint8_t *p) {
    return ((uint32_t) p[0] << 24) | ((uint32_t) p[1] << 16) | ((uint32_t) p[2] << 8) | p[3];
}

/* walk a singly linked list of MEM1 nodes (next at +0) and exempt `len` bytes of each */
static void rw_ex_walk(const uint8_t *mem1, uint32_t head, uint32_t len, int max) {
    int n;
    for (n = 0; n < max && head >= 0x80000000u && head + len <= 0x80000000u + gw_mem1_size; ++n) {
        rw_ex_add(head, len);
        head = rw_be(mem1 + (head - 0x80000000u));
    }
}

static void rw_ex_build(const uint8_t *mem1, const uint8_t *glob, int live) {
    const uint8_t *p;
    int k;
    if (live) {
        rw_ex_add(SN_STREAM_VA, SN_STREAM_LEN);
        for (k = 0; k < sn_nasync; ++k) {
            rw_ex_add(sn_async[k].va, SN_DEVCOM_NODE);
        }
    }
    if (rw_lights_va == 0) {
        for (k = 0; k < sn.nsyms; ++k) {
            if (strcmp(sn.syms[k].name, "_current_lights") == 0) rw_lights_va = sn.syms[k].va;
            if (strcmp(sn.syms[k].name, "_gw_slist_alloc_data") == 0) rw_slist_va = sn.syms[k].va;
        }
    }
    if (rw_lights_va != 0 && (p = rw_glob_at(glob, rw_lights_va)) != NULL) {
        rw_ex_walk(mem1, rw_be(p), 8, 64);
    }
    if (rw_slist_va != 0 && (p = rw_glob_at(glob, rw_slist_va + 4)) != NULL) {
        rw_ex_walk(mem1, rw_be(p), 8, 512);
    }
}

static int rw_mem1_exempt(uint32_t va) {
    int lo = 0, hi = rw_nex - 1;
    /* sorted by va; ranges may overlap, so check the neighbours of the last start <= va */
    while (lo <= hi) {
        int mid = (lo + hi) / 2;
        if (rw_ex[mid].va <= va) lo = mid + 1; else hi = mid - 1;
    }
    for (lo = hi; lo >= 0 && lo > hi - 4; --lo) {
        if (va >= rw_ex[lo].va && va < rw_ex[lo].va + rw_ex[lo].len) {
            return 1;
        }
    }
    return 0;
}

static int rw_glob_exempt(uint32_t va) {
    if (sn_in_cmp_skip(va)) {
        return 1;
    }
    if (rw_lights_va != 0 && va >= rw_lights_va && va < rw_lights_va + 4) {
        return 1;
    }
    if (rw_slist_va != 0 && va >= rw_slist_va + 4 && va < rw_slist_va + 8) {
        return 1; /* the free list's head */
    }
    return 0;
}

/* a hash of a MEM1 image without the exempt ranges (each is hashed as zeros) */
static uint64_t rw_hash_mem1(const uint8_t *m) {
    uint64_t h = 1;
    uint32_t at = 0x80000000u;
    int i;
    static const uint8_t zero[64];
    for (i = 0; i < rw_nex; ++i) {
        uint32_t s = rw_ex[i].va, e = rw_ex[i].va + rw_ex[i].len;
        if (e <= at) {
            continue;
        }
        if (s > at) {
            h = sn_h64(m + (at - 0x80000000u), s - at, h);
            at = s;
        }
        while (at < e) {
            uint32_t n = e - at > sizeof zero ? (uint32_t) sizeof zero : e - at;
            h = sn_h64(zero, n, h);
            at += n;
        }
    }
    return sn_h64(m + (at - 0x80000000u), 0x80000000u + gw_mem1_size - at, h);
}

/* Compare the live state with the capture. out[0]: every differing byte (MEM1 + globals); out[1]:
 * the SIMULATION's differing bytes - everything but what is listed above. out_h: a 64-bit hash of
 * each side's simulation bytes (capture, live). Logs up to 40 simulation differences, named.
 * Frees the capture. -1 when there is none. */
int gw_rw_compare(double out[2], uint64_t out_h[2]) {
    const uint8_t *live = (const uint8_t *) (uintptr_t) 0x80000000u;
    uint32_t i, logged = 0, base = 0;
    double all = 0, sim = 0, ex_bytes = 0, ro_bytes = 0;
    uint64_t hc = 1, hl = 1;
    uint8_t *g;
    int r;
    if (rw.cap_mem1 == NULL) {
        return -1;
    }
    g = (uint8_t *) malloc(sn.globals_len);
    if (g == NULL) {
        return -1;
    }
    sn_gather(g);
    sn_async_collect();
    rw_nex = 0;
    rw_ex_build(live, g, 1);
    rw_ex_build(rw.cap_mem1, rw.cap_glob, 0);
    qsort(rw_ex, (size_t) rw_nex, sizeof rw_ex[0], rw_ex_cmp);
    for (i = 0; i < gw_mem1_size; ++i) {
        if (live[i] != rw.cap_mem1[i]) {
            all += 1;
            if (rw_mem1_exempt(0x80000000u + i)) {
                ex_bytes += 1;
                continue;
            }
            if (rw_render_owned_mem1(i)) {
                ro_bytes += 1;
                continue;
            }
            sim += 1;
            if (logged++ < 40) {
                uint32_t a = i & ~15u, k2;
                char was[40], now[40];
                for (k2 = 0; k2 < 16 && a + k2 < gw_mem1_size; ++k2) {
                    snprintf(was + k2 * 2, 3, "%02X", rw.cap_mem1[a + k2]);
                    snprintf(now + k2 * 2, 3, "%02X", live[a + k2]);
                }
                gw_log("rewind test: MEM1 0x%08X differs; the 16 at 0x%08X were %s, now %s",
                       0x80000000u + i, 0x80000000u + a, was, now);
            }
        }
    }
    if (rwm.on) {
        /* zero the render-owned bytes in both images for the hash (the capture is ours to change;
           the live side goes through a scratch copy) */
        uint8_t *lc = (uint8_t *) malloc(gw_mem1_size);
        uint32_t w;
        if (lc != NULL) {
            memcpy(lc, live, gw_mem1_size);
            for (w = 0; w < gw_mem1_size; ++w) {
                if (rwm.mask_mem1[w >> 3] == 0) {
                    w |= 7;
                    continue;
                }
                if (rw_render_owned_mem1(w)) {
                    lc[w] = 0;
                    rw.cap_mem1[w] = 0;
                }
            }
            hc = rw_hash_mem1(rw.cap_mem1);
            hl = rw_hash_mem1(lc);
            free(lc);
        }
    } else {
        hc = rw_hash_mem1(rw.cap_mem1);
        hl = rw_hash_mem1(live);
    }
    for (r = 0; r < sn.nranges; ++r) {
        uint32_t o;
        for (o = 0; o < sn.ranges[r].len; ++o) {
            uint32_t va = sn.ranges[r].va + o;
            int skip = rw_glob_exempt(va) || rw_render_owned_glob(base + o);
            if (!skip) {
                hc = (hc ^ rw.cap_glob[base + o]) * 0x100000001B3ull;
                hl = (hl ^ g[base + o]) * 0x100000001B3ull;
            }
            if (g[base + o] != rw.cap_glob[base + o]) {
                const GwSnapSym *sym = sn_sym_at(va);
                all += 1;
                if (skip) {
                    ex_bytes += 1;
                    continue;
                }
                sim += 1;
                if (logged++ < 40) {
                    gw_log("rewind test: global %s+0x%X differs (was %02X, now %02X)",
                           sym ? sym->name : "?", sym ? va - sym->va : 0, rw.cap_glob[base + o], g[base + o]);
                }
            }
        }
        base += sn.ranges[r].len;
    }
    gw_log("rewind test: %.0f bytes differ: %.0f simulation, %.0f render-owned (measured), %.0f disc "
           "async / pad-side / light list (%d ranges walked)", all, sim, ro_bytes, ex_bytes, rw_nex);
    out[0] = all;
    out[1] = sim;
    out_h[0] = hc;
    out_h[1] = hl;
    free(g);
    free(rw.cap_mem1);
    free(rw.cap_glob);
    rw.cap_mem1 = rw.cap_glob = NULL;
    return 0;
}

/* The Lab's re-simulation flag: sound effects are not played again and the scene loop renders the
 * frame without presenting it (gmscene.c), exactly as for a rollback's resimulated frame. */
void gw_Snap_LabResim(int on) {
    if (sn.enabled) {
        sn.cur_is_resim = on;
    }
}

/* ==============================================================================================
 * SAVESTATE FILES (the Lab's persistent library). The whole state: every non-zero MEM1 page plus
 * the game globals, after a caller header (gw_script.c checks it before anything is loaded).
 *   "GDSTATE1" | u32 hdr_len | hdr | u32 mem1 | u32 globals_len | u32 npages | page bitmap
 *   | the non-zero pages | globals | u64 hash of everything after the sizes
 * A load reads and verifies the whole file before it touches the game: never half a state.
 * ============================================================================================== */
static const char rw_magic[8] = {0x47, 0x44, 0x53, 0x54, 0x41, 0x54, 0x45, 0x31}; /* GDSTATE1 */

int gw_snap_file_save(const char *path, const void *hdr, uint32_t hdr_len) {
    const uint8_t *live = (const uint8_t *) (uintptr_t) 0x80000000u;
    uint32_t pg, npages, bml;
    uint8_t *bm, *g;
    uint64_t h = 1;
    FILE *f;
    char tmp[MAX_PATH + 8];
    static const uint8_t zero[SN_PAGE];
    if (!sn.enabled && gw_snap_open(0) != 0) {
        return -1;
    }
    npages = gw_mem1_size / SN_PAGE;
    bml = (npages + 7) / 8;
    bm = (uint8_t *) calloc(bml, 1);
    g = (uint8_t *) malloc(sn.globals_len);
    if (bm == NULL || g == NULL) {
        free(bm);
        free(g);
        return -1;
    }
    for (pg = 0; pg < npages; ++pg) {
        if (memcmp(live + (size_t) pg * SN_PAGE, zero, SN_PAGE) != 0) {
            bm[pg >> 3] |= (uint8_t) (1u << (pg & 7));
        }
    }
    sn_gather(g);
    snprintf(tmp, sizeof tmp, "%s.tmp", path);
    f = fopen(tmp, "wb");
    if (f == NULL) {
        free(bm);
        free(g);
        return -1;
    }
    fwrite(rw_magic, 1, 8, f);
    fwrite(&hdr_len, 4, 1, f);
    fwrite(hdr, 1, hdr_len, f);
    fwrite(&gw_mem1_size, 4, 1, f);
    fwrite(&sn.globals_len, 4, 1, f);
    fwrite(&npages, 4, 1, f);
    fwrite(bm, 1, bml, f);
    h = sn_h64(bm, bml, h);
    for (pg = 0; pg < npages; ++pg) {
        if (bm[pg >> 3] & (1u << (pg & 7))) {
            fwrite(live + (size_t) pg * SN_PAGE, 1, SN_PAGE, f);
            h = sn_h64(live + (size_t) pg * SN_PAGE, SN_PAGE, h);
        }
    }
    fwrite(g, 1, sn.globals_len, f);
    h = sn_h64(g, sn.globals_len, h);
    fwrite(&h, 8, 1, f);
    free(bm);
    free(g);
    if (fclose(f) != 0) {
        DeleteFileA(tmp);
        return -1;
    }
    if (!MoveFileExA(tmp, path, MOVEFILE_REPLACE_EXISTING)) {
        DeleteFileA(tmp);
        return -1;
    }
    return 0;
}

/* Read only the caller header. Returns its length, -1 when the file is not a state. */
int gw_snap_file_header(const char *path, void *hdr, uint32_t cap) {
    char m[8];
    uint32_t len;
    FILE *f = fopen(path, "rb");
    if (f == NULL) {
        return -1;
    }
    if (fread(m, 1, 8, f) != 8 || memcmp(m, rw_magic, 8) != 0 || fread(&len, 4, 1, f) != 1 ||
        len > cap || fread(hdr, 1, len, f) != len) {
        fclose(f);
        return -1;
    }
    fclose(f);
    return (int) len;
}

/* Rewrite the caller header in place (same length): renaming a state. */
int gw_snap_file_rewrite_header(const char *path, const void *hdr, uint32_t hdr_len) {
    char m[8];
    uint32_t len;
    FILE *f = fopen(path, "r+b");
    if (f == NULL) {
        return -1;
    }
    if (fread(m, 1, 8, f) != 8 || memcmp(m, rw_magic, 8) != 0 || fread(&len, 4, 1, f) != 1 ||
        len != hdr_len) {
        fclose(f);
        return -1;
    }
    fseek(f, 12, SEEK_SET);
    fwrite(hdr, 1, hdr_len, f);
    return fclose(f) == 0 ? 0 : -1;
}

/* Load a state file into the live game. Everything is read and verified first. 0 = loaded;
 * -1 unreadable, -2 made for a different memory layout, -3 damaged. */
int gw_snap_file_load(const char *path) {
    char m[8];
    uint32_t hl, mem1, gl, npages, bml, pg;
    int u;
    uint8_t *bm = NULL, *img = NULL, *g = NULL;
    uint64_t h = 1, want = 0;
    uint8_t *live = (uint8_t *) (uintptr_t) 0x80000000u;
    int rc = -1;
    FILE *f;
    if (!sn.enabled && gw_snap_open(0) != 0) {
        return -1;
    }
    f = fopen(path, "rb");
    if (f == NULL) {
        return -1;
    }
    if (fread(m, 1, 8, f) != 8 || memcmp(m, rw_magic, 8) != 0 || fread(&hl, 4, 1, f) != 1 ||
        fseek(f, (long) hl, SEEK_CUR) != 0 || fread(&mem1, 4, 1, f) != 1 || fread(&gl, 4, 1, f) != 1 ||
        fread(&npages, 4, 1, f) != 1) {
        goto out;
    }
    if (mem1 != gw_mem1_size || gl != sn.globals_len || npages != gw_mem1_size / SN_PAGE) {
        rc = -2;
        goto out;
    }
    bml = (npages + 7) / 8;
    bm = (uint8_t *) malloc(bml);
    img = (uint8_t *) calloc(gw_mem1_size, 1);
    g = (uint8_t *) malloc(gl);
    rc = -3;
    if (bm == NULL || img == NULL || g == NULL || fread(bm, 1, bml, f) != bml) {
        goto out;
    }
    h = sn_h64(bm, bml, h);
    for (pg = 0; pg < npages; ++pg) {
        if (bm[pg >> 3] & (1u << (pg & 7))) {
            if (fread(img + (size_t) pg * SN_PAGE, 1, SN_PAGE, f) != SN_PAGE) {
                goto out;
            }
            h = sn_h64(img + (size_t) pg * SN_PAGE, SN_PAGE, h);
        }
    }
    if (fread(g, 1, gl, f) != gl || fread(&want, 8, 1, f) != 1) {
        goto out;
    }
    h = sn_h64(g, gl, h);
    if (h != want) {
        goto out;
    }
    /* verified: now it goes in, whole (the disc's asynchronous blocks stay as they are) */
    sn_boundary_asserts("state file load");
    sn_async_collect();
    sn_fixed_collect();
    sn_poll();
    memcpy(img + (SN_STREAM_VA - 0x80000000u), live + (SN_STREAM_VA - 0x80000000u), SN_STREAM_LEN);
    memcpy(live, img, gw_mem1_size);
    sn_scatter(g);
    sn_async_put_back();
    sn_fixed_put_back();
    for (u = 0; u < sn.nslots; ++u) {
        sn_bm_fill(sn.slot[u].dirty, sn.npages);
    }
    if (sn.hash_dirty != NULL) {
        sn_bm_fill(sn.hash_dirty, sn.npages);
    }
    if (sn_rw_dirty != NULL) {
        sn_bm_fill(sn_rw_dirty, sn.npages);
    }
    sn_watch_reset();
    rc = 0;
out:
    fclose(f);
    free(bm);
    free(img);
    free(g);
    return rc;
}
