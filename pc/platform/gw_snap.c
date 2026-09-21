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
} GwSnapSlot;

static struct {
    int tried, enabled, k;
    int ready;
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
} sn = { 0 };

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
        "src_sysdolphin_baselib_devcom.c.obj",
        "src_melee_lb_lbcardgame.c.obj",          /* memory card */
        "src_melee_lb_lbcardnew.c.obj",
        "src_melee_lb_lbmthp.c.obj",              /* movies */
        "src_melee_lb_lbsnap.c.obj",
        "libs_dolphin_src_dolphin_thp_THPDec.c.obj",
        "src_sysdolphin_baselib_rumble.c.obj",
    };
    static const char *const excl_sym[] = {
        "_gw_HSD_PadLibData", /* the raw pad queue: filled by the pad alarm, not by logic */
        "_gmMain_8046B108",   /* its storage */
        "_gw_HSD_Rumble_804C22E0",
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
                   strcmp(s->name, "_gw_HSD_PadCopyStatus") == 0) {
            /* INPUTS, renewed from the live pad queue each logic frame: during playback fighters
               take the replay's inputs instead, so these are not state to compare */
            sn.cmp_skip[sn.ncmp_skip].va = s->va;
            sn.cmp_skip[sn.ncmp_skip].len = s->len;
            ++sn.ncmp_skip;
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

/* ---- save / load / compare -------------------------------------------------------------------- */

static void sn_gather(uint8_t *out) {
    int i;
    for (i = 0; i < sn.nranges; ++i) {
        memcpy(out, (const void *) (uintptr_t) sn.ranges[i].va, sn.ranges[i].len);
        out += sn.ranges[i].len;
    }
}

static void sn_scatter(const uint8_t *in) {
    int i;
    for (i = 0; i < sn.nranges; ++i) {
        memcpy((void *) (uintptr_t) sn.ranges[i].va, in, sn.ranges[i].len);
        in += sn.ranges[i].len;
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

void gw_snap_save(int frame) {
    double t0 = sn_ms();
    GwSnapSlot *s = sn_slot_for(frame, 1);
    sn_boundary_asserts("save");
    s->frame = frame;
    memcpy(s->mem1, (const void *) (uintptr_t) 0x80000000u, gw_mem1_size);
    sn_gather(s->globals);
    gw_Replay_GetCursor(s->replay_cursor);
    sn.ms_save += sn_ms() - t0;
    sn.n_save++;
}

int gw_snap_load(int frame) {
    double t0 = sn_ms();
    GwSnapSlot *s = sn_slot_for(frame, 0);
    if (s == NULL) {
        return -1;
    }
    sn_boundary_asserts("load");
    memcpy((void *) (uintptr_t) 0x80000000u, s->mem1, gw_mem1_size);
    sn_scatter(s->globals);
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

static int sn_skip_global(uint32_t va) {
    int i;
    for (i = 0; i < sn.ncmp_skip; ++i) {
        if (va >= sn.cmp_skip[i].va && va < sn.cmp_skip[i].va + sn.cmp_skip[i].len) {
            return 1;
        }
    }
    return 0;
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
    if (memcmp(s->mem1, live, gw_mem1_size) != 0) {
        uint32_t off = 0;
        sn.nmask = 0;
        sn_mask_particles(NULL, NULL);
        sn_mask_particles(s->mem1, s->globals);
        while (off < gw_mem1_size) {
            uint32_t mend;
            if (s->mem1[off] != live[off] && (sn.rmask_mem1[off >> 3] & (1u << (off & 7)))) {
                ++off;
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

/* ---- SyncTest --------------------------------------------------------------------------------- */

static void sn_sfx_rewind(int frame);

static void sn_init(void) {
    const char *v;
    int i;
    if (sn.tried) {
        return;
    }
    sn.tried = 1;
    v = getenv("MELEE_SYNCTEST");
    if (v == NULL || v[0] == '\0' || atoi(v) <= 0) {
        return;
    }
    sn.k = atoi(v);
    if (sn.k > GW_SNAP_MAX_SLOTS - 2) {
        sn.k = GW_SNAP_MAX_SLOTS - 2;
    }
    if (sn_load_map() != 0) {
        return;
    }
    sn.nslots = sn.k + 2;
    for (i = 0; i < sn.nslots; ++i) {
        sn.slot[i].frame = -0x7FFFFFFF - 1;
        sn.slot[i].mem1 = (uint8_t *) malloc(gw_mem1_size);
        sn.slot[i].globals = (uint8_t *) malloc(sn.globals_len);
        if (sn.slot[i].mem1 == NULL || sn.slot[i].globals == NULL) {
            gw_log("snap: out of memory for %d slots", sn.nslots);
            return;
        }
    }
    sn.cmp_globals = (uint8_t *) malloc(sn.globals_len);
    sn.pre_mem1 = (uint8_t *) malloc(gw_mem1_size);
    sn.pre_globals = (uint8_t *) malloc(sn.globals_len);
    sn.rmask_mem1 = (uint8_t *) calloc(gw_mem1_size / 8 + 1, 1);
    sn.rmask_globals = (uint8_t *) calloc(sn.globals_len / 8 + 1, 1);
    if (sn.pre_mem1 == NULL || sn.pre_globals == NULL || sn.rmask_mem1 == NULL ||
        sn.rmask_globals == NULL) {
        gw_log("snap: out of memory for the render mask");
        return;
    }
    sn.enabled = 1;
    gw_log("snap: SyncTest k=%d (%d slots of %u bytes)", sn.k, sn.nslots,
           gw_mem1_size + sn.globals_len);
}

/* The replay has run at least one frame: the loading hold is over and the match is live. */
static int sn_live(void) {
    return gw_Replay_Frame() >= -123;
}

/* Before the scene loop's logic iterations for this render tick. */
int gw_SyncTest_Iterations(int count) {
    sn_init();
    if (!sn.enabled || !sn_live()) {
        return count;
    }
    sn.target = gw_Replay_Frame() + 1;
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
    next = gw_Replay_Frame() + 1;
    if (sn.plan_rollback) {
        sn.plan_rollback = 0;
        gw_snap_save(sn.target);
        gw_snap_load(sn.target - sn.k);
        sn.resim = 1;
        sn.cur_is_resim = 1;
        sn_sfx_rewind(sn.target - sn.k);
        return;
    }
    if (sn.resim) {
        int d = sn_compare(next);
        if (d != 0) {
            sn.mismatches++;
            if (sn.mismatches <= 8) {
                gw_log("snap: SYNCTEST MISMATCH at the start of frame %d (resimulated %d back from "
                       "%d): %d differing runs",
                       next, sn.target - next, sn.target, d);
            }
        }
        sn.cur_is_resim = next < sn.target;
        if (sn.cur_is_resim) {
            sn_sfx_rewind(next);
        }
        if (next >= sn.target) {
            sn.resim = 0;
            sn.passes++;
            if ((sn.passes % 600) == 0) {
                gw_log("snap: frame %d, %d rollbacks, %d checks, %d mismatching; per frame save "
                       "%.2f ms, load %.2f ms, compare %.2f ms",
                       next, sn.passes, sn.checked, sn.mismatches,
                       sn.ms_save / (sn.n_save ? sn.n_save : 1),
                       sn.ms_load / (sn.n_load ? sn.n_load : 1),
                       sn.ms_cmp / (sn.n_cmp ? sn.n_cmp : 1));
            }
        }
        return;
    }
    sn.cur_is_resim = 0;
    gw_snap_save(next);
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
void gw_SyncTest_PreRender(void) {
    if (!sn.enabled || !sn_live()) {
        return;
    }
    memcpy(sn.pre_mem1, (const void *) (uintptr_t) 0x80000000u, gw_mem1_size);
    sn_gather(sn.pre_globals);
    sn.pre_valid = 1;
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

void gw_SyncTest_PostRender(void) {
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
        gw_log("snap: render pass wrote %u new MEM1 bytes and %u global bytes (%u render-owned so far)",
               a, g, sn.rmask_bytes);
    }
}

/* Sound effects must not replay while resimulating (lbaudio_ax.c). */
int gw_Snap_Resimulating(void) {
    return sn.enabled && sn.cur_is_resim;
}
