/* gw_mex_grfunction.c - m-ex custom STAGES: grFunction + the expanded stage tables.
 *
 * See gw_mex_grfunction.h for the shape. Reimplemented in original C from m-ex's published
 * behaviour (SSS Expansion/Init grFunction.asm, Standalone Functions/Get grFunction.asm and
 * Header.s's Arch_Map / Arch_grFunction offsets); none of m-ex's sources are vendored.
 *
 * WHAT WAS DUMPED, NOT ASSUMED (tools/mex_port + scratch dumpers, Akaneia.iso):
 *   - mexData root word 0x28 = Arch_Map, 0x2C = Arch_grFunction.
 *   - Arch_grFunction is `internal_stage_count` (96) pointers to 13-word rows whose layout is
 *     byte-for-byte the port's `struct StageData`: row[0] == the row index for every non-empty
 *     row, row[1] is a 0x803Exxxx StageCallbacks pointer for vanilla stages and 0xFFFFFFFF for
 *     added ones, row[2] is an in-archive "/GrXx.dat" string.
 *   - Rows 0..70 reproduce the port's compiled-in stage_datas[] exactly, including the two NULL
 *     rows at 23 and 26 and the grTe alias at 35. Rows 71..95 are the added stages; all 25 files
 *     are present on Akaneia and every one carries a grFunction symbol.
 *   - Arch_Map_StageIDs is 313 x 12 bytes and ends exactly where the mexData root begins, which
 *     is what pins the stride: it is the same {grkind, unk1, unk2} the port's stage_id_map[] is.
 *     External 0..283 reproduce the vanilla map; 288..312 -> internal 71..95.
 *
 * THE CLONE-BASE TRAP. Rows 71..86 ship with the on_* words of an existing stage (row 71 carries
 * grOldPupupu's, row 76 Meta Crystal's own set, and so on). A slot the blob does NOT override
 * therefore runs the clone base's handler rather than failing - silently, exactly as on hardware.
 * gw_Mex_GrVanillaFn resolves those words through the guest->native bridge so the port behaves
 * the same way, and every resolution is logged once so an unregistered slot is visible.
 */
#include "gw.h"
#include "gw_test.h"
#include "gw_ppc.h"
#include "gw_mex_bridge.h"
#include "gw_mex_ftfunction.h"
#include "gw_mex_grfunction.h"

#include <stdlib.h>
#include <string.h>

#define GW_HSD_HEADER_SIZE 0x20u

/* MexFunction header, data-relative - the same struct ftFunction uses. */
#define GRFUNC_OFF_CODE 0x00u
#define GRFUNC_OFF_INSTR_RELOC 0x04u
#define GRFUNC_OFF_INSTR_RELOC_COUNT 0x08u
#define GRFUNC_OFF_FUNC_RELOC 0x0Cu
#define GRFUNC_OFF_FUNC_RELOC_COUNT 0x10u
#define GRFUNC_OFF_CODE_SIZE 0x14u

/* mexData root words. */
#define GW_MEXDT_OFF_METADATA 0x00u
#define GW_MEXDT_OFF_MAP 0x28u  /* m-ex Arch_Map */
#define GW_MEXDT_OFF_DESC 0x2Cu /* m-ex Arch_grFunction */

/* MexMetaData: u8 major, u8 minor, u16 flags, then s32 counts. */
#define GW_MEXDT_META_INT_STAGES (4u + 3u * 4u)
#define GW_MEXDT_META_EXT_STAGES (4u + 4u * 4u)

#define GW_MEX_GR_STAGEIDS_STRIDE 12u

/* From gw_mex_ftfunction_runtime.c (the shared PPC runtime environment). */
extern void gw_Mex_RuntimeInit(void);
extern uint32_t gw_Mex_Rtoc(void);
extern uint32_t gw_Mex_StackTop(void);
extern uint32_t gw_Mex_Callable(uint32_t guest, const char *why);
extern uint32_t gw_Mex_MexData(uint32_t *base, uint32_t *size);
extern int gw_DVDConvertPathToEntrynum(const char *path);

/* ---- mexData reads, bounds-checked ---------------------------------------------------- */

static uint32_t gr_root, gr_base, gr_size, gr_map, gr_desc;
static int gr_int_count, gr_ext_count;
static int gr_tables_ready; /* 1 = usable, -1 = tried and unusable */

static int gr_in(uint32_t a, uint32_t len) {
    return gr_size != 0u && a >= gr_base && (uint64_t) a + len <= (uint64_t) gr_base + gr_size;
}

static uint32_t gr_rd(uint32_t a) {
    return gr_in(a, 4u) ? gw_r32((const void *) (uintptr_t) a) : 0u;
}

/* Resolve the stage tables once. Safe on a vanilla disc: leaves everything zero. */
static int gr_tables(void) {
    uint32_t meta;
    if (gr_tables_ready != 0) {
        return gr_tables_ready > 0;
    }
    gr_tables_ready = -1;
    gw_Mex_RuntimeInit();
    gr_root = gw_Mex_MexData(&gr_base, &gr_size);
    if (gr_root == 0u) {
        return 0;
    }
    meta = gr_rd(gr_root + GW_MEXDT_OFF_METADATA);
    gr_map = gr_rd(gr_root + GW_MEXDT_OFF_MAP);
    gr_desc = gr_rd(gr_root + GW_MEXDT_OFF_DESC);
    if (!gr_in(meta, 0x3Cu) || !gr_in(gr_map, 0x18u) || !gr_in(gr_desc, 4u)) {
        gw_log("grfunction: mexData has no usable stage tables (metadata 0x%08X map 0x%08X "
               "desc 0x%08X) - m-ex stages stay off",
               meta, gr_map, gr_desc);
        return 0;
    }
    gr_int_count = (int) gr_rd(meta + GW_MEXDT_META_INT_STAGES);
    gr_ext_count = (int) gr_rd(meta + GW_MEXDT_META_EXT_STAGES);
    if (gr_int_count <= 0 || gr_int_count > GW_MEX_GR_MAX || gr_ext_count <= 0 ||
        gr_ext_count > 4096)
    {
        gw_log("grfunction: implausible stage counts (internal %d, external %d; the port's "
               "stage_datas[] holds %d rows) - m-ex stages stay off",
               gr_int_count, gr_ext_count, GW_MEX_GR_MAX);
        gr_int_count = gr_ext_count = 0;
        return 0;
    }
    /* The external->internal table is the port's own stage_id_map entry type; prove the stride by
     * checking the table really spans `count` entries inside the archive. */
    {
        uint32_t ids = gr_rd(gr_map);
        if (!gr_in(ids, (uint32_t) gr_ext_count * GW_MEX_GR_STAGEIDS_STRIDE)) {
            gw_log("grfunction: Arch_Map_StageIDs 0x%08X does not hold %d x %u bytes - m-ex "
                   "stages stay off",
                   ids, gr_ext_count, GW_MEX_GR_STAGEIDS_STRIDE);
            gr_int_count = gr_ext_count = 0;
            return 0;
        }
    }
    gr_tables_ready = 1;
    gw_log("grfunction: mexData stage tables ready: %d internal, %d external (map 0x%08X, "
           "grFunction 0x%08X)",
           gr_int_count, gr_ext_count, gr_map, gr_desc);
    return 1;
}

/* Guest address of internal stage `grkind`'s 13-word StageData row, or 0. */
static uint32_t gr_row(int grkind) {
    uint32_t p;
    if (!gr_tables() || grkind < 0 || grkind >= gr_int_count) {
        return 0u;
    }
    p = gr_rd(gr_desc + (uint32_t) grkind * 4u);
    return gr_in(p, GW_MEX_GR_SLOT_COUNT * 4u) ? p : 0u;
}

int gw_Mex_GrInternalCount(void) { return gr_tables() ? gr_int_count : 0; }
int gw_Mex_GrExternalCount(void) { return gr_tables() ? gr_ext_count : 0; }

int gw_Mex_GrKindForExt(int ext) {
    uint32_t ids;
    int k;
    if (!gr_tables() || ext < 0 || ext >= gr_ext_count) {
        return -1;
    }
    ids = gr_rd(gr_map);
    k = (int) gr_rd(ids + (uint32_t) ext * GW_MEX_GR_STAGEIDS_STRIDE);
    return (k >= 0 && k < gr_int_count) ? k : -1;
}

const char *gw_Mex_GrFile(int grkind) {
    uint32_t row = gr_row(grkind);
    uint32_t p = row != 0u ? gr_rd(row + GW_MEX_GR_SLOT_FILE * 4u) : 0u;
    const char *s;
    uint32_t i;
    if (p == 0u || !gr_in(p, 2u)) {
        return NULL;
    }
    s = (const char *) (uintptr_t) p;
    for (i = 0; gr_in(p + i, 1u) && i < 64u; ++i) {
        if (s[i] == '\0') {
            return i > 1u ? s : NULL;
        }
    }
    return NULL;
}

int gw_Mex_GrIsMex(int grkind) {
    const char *f;
    if (grkind < GW_MEX_GR_FIRST_NEW) {
        return 0;
    }
    f = gw_Mex_GrFile(grkind);
    /* A declared stage whose file this disc does not carry must NOT get a row: the first load
     * would walk into a missing file. This is the stage side of the fighter path's dense slots. */
    return f != NULL && gw_DVDConvertPathToEntrynum(f) >= 0;
}

uint32_t gw_Mex_GrFlags2(int grkind) {
    uint32_t row = gr_row(grkind);
    return row != 0u ? gr_rd(row + GW_MEX_GR_SLOT_FLAGS2 * 4u) : 0u;
}

void *gw_Mex_GrVanillaFn(int grkind, int slot) {
    uint32_t row = gr_row(grkind), g, native;
    int kind = -1;
    static uint32_t moaned[32];
    static int nmoaned;
    int i;

    if (row == 0u || slot < GW_MEX_GR_SLOT_ON_INIT || slot > GW_MEX_GR_SLOT_ON_CHECK_SHADOW) {
        return NULL;
    }
    g = gr_rd(row + (uint32_t) slot * 4u);
    if (g == 0u || g == 0xFFFFFFFFu) {
        return NULL;
    }
    native = gw_mex_bridge_lookup(g, &kind);
    if (native != 0u && kind == 1) {
        return (void *) (uintptr_t) native;
    }
    for (i = 0; i < nmoaned; ++i) {
        if (moaned[i] == g) {
            return NULL;
        }
    }
    if (nmoaned < (int) (sizeof moaned / sizeof moaned[0])) {
        moaned[nmoaned++] = g;
    }
    gw_log("grfunction: stage %d slot %d clone-base function 0x%08X has no native counterpart - "
           "left empty",
           grkind, slot, g);
    return NULL;
}

/* ---- the loaded blob ------------------------------------------------------------------- */

static int gr_cur = -1;                              /* stage being brought up, or -1 */
static int gr_loaded = -1;                           /* stage whose blob is installed, or -1 */
static uint32_t gr_code_lo, gr_code_hi;              /* installed blob's guest code range */
static uint32_t gr_slot[GW_MEX_GR_SLOT_COUNT];       /* Overload results, guest addresses */

static void gr_unload(void) {
    if (gr_code_hi != 0u) {
        gw_ppc_remove_code_range(gr_code_lo, gr_code_hi);
    }
    gr_code_lo = gr_code_hi = 0u;
    gr_loaded = -1;
    memset(gr_slot, 0, sizeof gr_slot);
}

void gw_Mex_GrSelect(int grkind) {
    if (grkind == gr_cur) {
        return;
    }
    gr_cur = grkind;
    /* The previous stage's archive is gone, and with it its code. Anything still pointing into
     * that range would be a dangling interpret target. */
    if (gr_loaded >= 0 && gr_loaded != grkind) {
        gw_log("grfunction: stage %d replaces stage %d; dropping its code range 0x%08X..0x%08X",
               grkind, gr_loaded, gr_code_lo, gr_code_hi);
        gr_unload();
    }
}

void gw_Mex_GrFunctionInit(void *archive, int grkind) {
    unsigned char *dat = NULL;
    uint32_t dat_size = 0, arch_data, arch_data_size;
    uint32_t sym, code_off, code_size, irt_off, irt_count, frt_off, frt_count, code_base, i;
    const char *file;
    int32_t pub;
    int n = 0;

    gw_Mex_GrSelect(grkind);
    if (archive == NULL || !gw_Mex_GrIsMex(grkind)) {
        return;
    }
    file = gw_Mex_GrFile(grkind);
    arch_data_size = gw_r32((const void *) (uintptr_t) ((uintptr_t) archive + 0x04u));
    arch_data = gw_r32((const void *) (uintptr_t) ((uintptr_t) archive + 0x20u));
    if (arch_data == 0u || arch_data_size == 0u) {
        gw_log("grfunction: %s: loaded archive has no data section (0x%08X, %u bytes)", file,
               arch_data, arch_data_size);
        return;
    }

    dat = (unsigned char *) gw_DVDReadFileAlloc(file, &dat_size);
    if (dat == NULL) {
        gw_log("grfunction: %s could not be read from the disc", file);
        return;
    }
    /* The disc copy supplies the PRISTINE tables: HSD relocation has already rewritten pointer
     * words in the loaded archive, and the reloc/overload tables are raw offsets that must be
     * read before that happened. Proving the two are the same file is what makes that safe. */
    if (dat_size < GW_HSD_HEADER_SIZE || gw_r32(dat + 0x04) != arch_data_size) {
        gw_log("grfunction: %s on disc (data 0x%X) is not the loaded archive (data 0x%X)", file,
               dat_size >= GW_HSD_HEADER_SIZE ? gw_r32(dat + 0x04) : 0u, arch_data_size);
        free(dat);
        return;
    }
    pub = gw_ftfunction_find_public(dat, dat_size, "grFunction");
    if (pub < 0 || (uint32_t) pub + 0x18u > arch_data_size) {
        gw_log("grfunction: %s has no grFunction symbol - clone-base handlers only", file);
        free(dat);
        return;
    }
    sym = GW_HSD_HEADER_SIZE + (uint32_t) pub;
    code_off = gw_r32(dat + sym + GRFUNC_OFF_CODE);
    irt_off = gw_r32(dat + sym + GRFUNC_OFF_INSTR_RELOC);
    irt_count = gw_r32(dat + sym + GRFUNC_OFF_INSTR_RELOC_COUNT);
    frt_off = gw_r32(dat + sym + GRFUNC_OFF_FUNC_RELOC);
    frt_count = gw_r32(dat + sym + GRFUNC_OFF_FUNC_RELOC_COUNT);
    code_size = gw_r32(dat + sym + GRFUNC_OFF_CODE_SIZE);

    if (code_size == 0u || code_off > arch_data_size || code_size > arch_data_size - code_off ||
        (irt_count != 0u &&
         (irt_off > arch_data_size || irt_count * 8u > arch_data_size - irt_off)) ||
        (frt_count != 0u && (frt_off > arch_data_size || frt_count * 8u > arch_data_size - frt_off)))
    {
        gw_log("grfunction: %s grFunction struct is out of bounds (code 0x%X+0x%X, irt 0x%X x%u, "
               "frt 0x%X x%u, archive data 0x%X)",
               file, code_off, code_size, irt_off, irt_count, frt_off, frt_count, arch_data_size);
        free(dat);
        return;
    }

    gr_unload();
    code_base = arch_data + code_off;
    /* Relocate IN PLACE inside the stage's own loaded archive, as m-ex does on hardware, so the
     * code lives exactly as long as the file and costs no persistent memory. */
    if (gw_ftfunction_reloc(dat, dat_size, irt_off, irt_count, code_base, code_size) !=
        GW_FTFUNC_OK) {
        gw_log("grfunction: %s instruction relocation failed - clone-base handlers only", file);
        free(dat);
        return;
    }

    /* Overload: each entry names a WORD INDEX INTO StageData, not a separate table. */
    for (i = 0; i < frt_count; ++i) {
        uint32_t e = GW_HSD_HEADER_SIZE + frt_off + i * 8u;
        uint32_t replace_this = gw_r32(dat + e);
        uint32_t replace_with = gw_r32(dat + e + 4u);
        if (replace_this & 0x80000000u) {
            /* The func-address case patches a guest instruction word. In the port that address is
             * native code we must not write; no shipped stage blob uses it. */
            gw_log("grfunction: %s: func-address hook guest 0x%08X -> 0x%08X is NOT applied",
                   file, replace_this & 0x7FFFFFFFu, code_base + replace_with);
            continue;
        }
        if (replace_this >= GW_MEX_GR_SLOT_COUNT) {
            gw_log("grfunction: %s: overload entry %u names StageData word %u, past the %u-word "
                   "struct - skipped",
                   file, i, replace_this, GW_MEX_GR_SLOT_COUNT);
            continue;
        }
        gr_slot[replace_this] = code_base + replace_with;
        ++n;
    }
    free(dat);

    gr_code_lo = code_base;
    gr_code_hi = code_base + code_size;
    gr_loaded = grkind;
    gw_ppc_add_code_range(gr_code_lo, gr_code_hi);
    gw_log("grfunction: %s (internal stage %d) installed: code 0x%08X..0x%08X (%u bytes), %u "
           "instruction relocs, %d of %u overloads",
           file, grkind, gr_code_lo, gr_code_hi, code_size, irt_count, n, frt_count);
    for (i = 0; i < GW_MEX_GR_SLOT_COUNT; ++i) {
        if (gr_slot[i] != 0u) {
            gw_log("grfunction:   StageData word %u -> %s", i, gw_ppc_describe(gr_slot[i]));
        }
    }
}

void *gw_Mex_GrCallbacks(void) {
    if (gr_loaded < 0 || gr_loaded != gr_cur) {
        return NULL;
    }
    return (void *) (uintptr_t) gr_slot[GW_MEX_GR_SLOT_CALLBACKS];
}

void *gw_Mex_GrBind(void *fn) {
    uint32_t a = (uint32_t) (uintptr_t) fn;
    if (a == 0u || !gw_ppc_is_guest_code(a)) {
        return fn;
    }
    return (void *) (uintptr_t) gw_Mex_Callable(a, "m-ex stage callback");
}

/* ---- the seven StageData trampolines ---------------------------------------------------- */

/* The blob override for `slot`, but only while the stage it came from is the one being run. */
static uint32_t gr_target(int slot) {
    if (gr_loaded < 0 || gr_loaded != gr_cur) {
        return 0u;
    }
    return gr_slot[slot];
}

static uint32_t gr_run(int slot, const uint32_t *args, int nargs) {
    uint32_t t = gr_target(slot);
    if (t == 0u) {
        return 0u;
    }
    return gw_ppc_call(t, args, nargs, gw_Mex_Rtoc(), gw_Mex_StackTop());
}

typedef void (*gr_void_fn)(void);
typedef void (*gr_int_fn)(int);
typedef int (*gr_pred_fn)(void);
typedef void *(*gr_touch_fn)(int);
typedef int (*gr_shadow_fn)(void *, int, void *);

void gw_Mex_GrOnInit(void) {
    if (gr_target(GW_MEX_GR_SLOT_ON_INIT) != 0u) {
        gr_run(GW_MEX_GR_SLOT_ON_INIT, NULL, 0);
        return;
    }
    {
        gr_void_fn f = (gr_void_fn) gw_Mex_GrVanillaFn(gr_cur, GW_MEX_GR_SLOT_ON_INIT);
        if (f != NULL) {
            f();
        }
    }
}

void gw_Mex_GrOnDemoInit(int arg) {
    uint32_t a = (uint32_t) arg;
    if (gr_target(GW_MEX_GR_SLOT_ON_DEMO_INIT) != 0u) {
        gr_run(GW_MEX_GR_SLOT_ON_DEMO_INIT, &a, 1);
        return;
    }
    {
        gr_int_fn f = (gr_int_fn) gw_Mex_GrVanillaFn(gr_cur, GW_MEX_GR_SLOT_ON_DEMO_INIT);
        if (f != NULL) {
            f(arg);
        }
    }
}

void gw_Mex_GrOnLoad(void) {
    if (gr_target(GW_MEX_GR_SLOT_ON_LOAD) != 0u) {
        gr_run(GW_MEX_GR_SLOT_ON_LOAD, NULL, 0);
        return;
    }
    {
        gr_void_fn f = (gr_void_fn) gw_Mex_GrVanillaFn(gr_cur, GW_MEX_GR_SLOT_ON_LOAD);
        if (f != NULL) {
            f();
        }
    }
}

void gw_Mex_GrOnStart(void) {
    if (gr_target(GW_MEX_GR_SLOT_ON_START) != 0u) {
        gr_run(GW_MEX_GR_SLOT_ON_START, NULL, 0);
        return;
    }
    {
        gr_void_fn f = (gr_void_fn) gw_Mex_GrVanillaFn(gr_cur, GW_MEX_GR_SLOT_ON_START);
        if (f != NULL) {
            f();
        }
    }
}

int gw_Mex_GrCallback4(void) {
    if (gr_target(GW_MEX_GR_SLOT_CALLBACK4) != 0u) {
        return (int) gr_run(GW_MEX_GR_SLOT_CALLBACK4, NULL, 0);
    }
    {
        gr_pred_fn f = (gr_pred_fn) gw_Mex_GrVanillaFn(gr_cur, GW_MEX_GR_SLOT_CALLBACK4);
        return f != NULL ? f() : 0;
    }
}

void *gw_Mex_GrOnTouchLine(int index) {
    uint32_t a = (uint32_t) index;
    if (gr_target(GW_MEX_GR_SLOT_ON_TOUCH_LINE) != 0u) {
        return (void *) (uintptr_t) gr_run(GW_MEX_GR_SLOT_ON_TOUCH_LINE, &a, 1);
    }
    {
        gr_touch_fn f = (gr_touch_fn) gw_Mex_GrVanillaFn(gr_cur, GW_MEX_GR_SLOT_ON_TOUCH_LINE);
        return f != NULL ? f(index) : NULL;
    }
}

int gw_Mex_GrOnCheckShadowRender(void *pos, int arg1, void *jobj) {
    uint32_t a[3];
    a[0] = (uint32_t) (uintptr_t) pos;
    a[1] = (uint32_t) arg1;
    a[2] = (uint32_t) (uintptr_t) jobj;
    if (gr_target(GW_MEX_GR_SLOT_ON_CHECK_SHADOW) != 0u) {
        return (int) gr_run(GW_MEX_GR_SLOT_ON_CHECK_SHADOW, a, 3);
    }
    {
        gr_shadow_fn f = (gr_shadow_fn) gw_Mex_GrVanillaFn(gr_cur, GW_MEX_GR_SLOT_ON_CHECK_SHADOW);
        return f != NULL ? f(pos, arg1, jobj) : 0;
    }
}

/* ---- tests ------------------------------------------------------------------------------
 * These run headless under --test against whatever disc was passed. On a vanilla disc there is no
 * MxDt.dat and they skip; on Akaneia they assert the dumped layout, because a silent layout drift
 * is exactly the failure mode that costs a day. */

#define GW_MEX_GR_TEST_STAGE 76 /* GrOMc, Meta Crystal - the smallest complete added stage */

static int test_grfunction_tables(void) {
    int k;
    if (gw_iso_path() == NULL || gw_Mex_GrInternalCount() == 0) {
        gw_log("grfunction: no m-ex stage tables on this disc - skipping");
        return 0;
    }
    if (gw_Mex_GrInternalCount() > GW_MEX_GR_MAX) {
        gw_test_fail("internal stage count %d exceeds stage_datas[] (%d rows)",
                     gw_Mex_GrInternalCount(), GW_MEX_GR_MAX);
        return 1;
    }
    /* Rows 0..70 must reproduce the vanilla numbering, or every index below is a different
     * stage. Row 1 is /GrTe.dat and row 35 aliases it; 23 and 26 are the vanilla holes. */
    if (gw_Mex_GrFile(1) == NULL || strcmp(gw_Mex_GrFile(1), "/GrTe.dat") != 0) {
        gw_test_fail("internal stage 1 is %s, expected /GrTe.dat",
                     gw_Mex_GrFile(1) ? gw_Mex_GrFile(1) : "(none)");
        return 1;
    }
    if (gw_Mex_GrFile(23) != NULL || gw_Mex_GrFile(26) != NULL) {
        gw_test_fail("internal stages 23/26 should be the vanilla empty rows");
        return 1;
    }
    /* External -> internal: the added stages sit at the end of the external space. */
    k = gw_Mex_GrKindForExt(293);
    if (k != GW_MEX_GR_TEST_STAGE) {
        gw_test_fail("external stage 293 maps to internal %d, expected %d", k,
                     GW_MEX_GR_TEST_STAGE);
        return 1;
    }
    if (gw_Mex_GrKindForExt(gw_Mex_GrExternalCount()) != -1 || gw_Mex_GrKindForExt(-1) != -1) {
        gw_test_fail("out-of-range external ids must map to -1");
        return 1;
    }
    return 0;
}

static int test_grfunction_rows(void) {
    const char *f;
    if (gw_iso_path() == NULL || gw_Mex_GrInternalCount() == 0) {
        return 0;
    }
    /* A vanilla row is never "m-ex", whatever its file says. */
    if (gw_Mex_GrIsMex(1) || gw_Mex_GrIsMex(GW_MEX_GR_FIRST_NEW - 1)) {
        gw_test_fail("a vanilla internal stage must not be reported as m-ex");
        return 1;
    }
    f = gw_Mex_GrFile(GW_MEX_GR_TEST_STAGE);
    if (f == NULL || strcmp(f, "/GrOMc.dat") != 0) {
        gw_test_fail("internal stage %d is %s, expected /GrOMc.dat", GW_MEX_GR_TEST_STAGE,
                     f ? f : "(none)");
        return 1;
    }
    if (!gw_Mex_GrIsMex(GW_MEX_GR_TEST_STAGE)) {
        gw_test_fail("internal stage %d (%s) is on this disc but was not reported as m-ex",
                     GW_MEX_GR_TEST_STAGE, f);
        return 1;
    }
    return 0;
}

/* End to end without the engine: relocate GrOMc's grFunction into a scratch guest buffer and
 * check the Overload result and one relocated branch. This is the same code path the real load
 * takes, with the archive replaced by a copy of the disc file's data section. */
static int test_grfunction_load_gromc(void) {
    unsigned char *dat;
    uint32_t dat_size = 0, data_size, scratch, code_off, code_size, insn, target;
    int32_t pub;
    int rc = 0;

    if (gw_iso_path() == NULL || gw_Mex_GrInternalCount() == 0) {
        return 0;
    }
    dat = (unsigned char *) gw_DVDReadFileAlloc("/GrOMc.dat", &dat_size);
    if (dat == NULL) {
        gw_log("grfunction: /GrOMc.dat not on this disc - skipping");
        return 0;
    }
    data_size = gw_r32(dat + 0x04);
    pub = gw_ftfunction_find_public(dat, dat_size, "grFunction");
    if (pub < 0) {
        gw_test_fail("/GrOMc.dat has no grFunction public symbol");
        free(dat);
        return 1;
    }
    code_off = gw_r32(dat + GW_HSD_HEADER_SIZE + (uint32_t) pub + GRFUNC_OFF_CODE);
    code_size = gw_r32(dat + GW_HSD_HEADER_SIZE + (uint32_t) pub + GRFUNC_OFF_CODE_SIZE);
    if (code_size != 0x1E8u) {
        gw_test_fail("GrOMc grFunction codeSize 0x%X, expected 0x1E8", code_size);
        free(dat);
        return 1;
    }
    if (gw_r32(dat + GW_HSD_HEADER_SIZE + (uint32_t) pub + GRFUNC_OFF_FUNC_RELOC_COUNT) != 8u) {
        gw_test_fail("GrOMc grFunction has %u overloads, expected 8",
                     gw_r32(dat + GW_HSD_HEADER_SIZE + (uint32_t) pub + GRFUNC_OFF_FUNC_RELOC_COUNT));
        free(dat);
        return 1;
    }

    /* A scratch "archive" high in MEM1, below the test stack the interpreter tests use. */
    scratch = 0x80500000u;
    if (data_size > 0x00800000u) {
        gw_test_fail("GrOMc data section 0x%X is too large for the scratch region", data_size);
        free(dat);
        return 1;
    }
    memcpy((void *) (uintptr_t) scratch, dat + GW_HSD_HEADER_SIZE, data_size);
    if (gw_ftfunction_reloc(dat, dat_size,
                            gw_r32(dat + GW_HSD_HEADER_SIZE + (uint32_t) pub + GRFUNC_OFF_INSTR_RELOC),
                            gw_r32(dat + GW_HSD_HEADER_SIZE + (uint32_t) pub +
                                   GRFUNC_OFF_INSTR_RELOC_COUNT),
                            scratch + code_off, code_size) != GW_FTFUNC_OK) {
        gw_test_fail("GrOMc grFunction instruction relocation failed");
        free(dat);
        return 1;
    }

    /* OnInit is at code+0x3C and its first call is `bl grTSeak_80223908` at +0x4C; decode the
     * relocated branch back to prove the reloc landed. */
    insn = gw_r32((const void *) (uintptr_t) (scratch + code_off + 0x4Cu));
    target = (scratch + code_off + 0x4Cu) + (uint32_t) ((((int32_t) (insn << 6)) >> 8) << 2);
    if ((insn >> 26) != 18u || target != 0x80223908u) {
        gw_test_fail("GrOMc OnInit branch @+0x4C = 0x%08X -> 0x%08X, expected bl 0x80223908", insn,
                     target);
        rc = 1;
    }
    /* map_gobjs (slot 1) is the stage's StageCallbacks[]; its third entry's flags word carries
     * the lighting + fog bits the engine scans for. */
    if (rc == 0 && gw_r32((const void *) (uintptr_t) (scratch + code_off + 0x38u)) != 0xC0000000u) {
        gw_test_fail("GrOMc map_gobjs[2].flags = 0x%08X, expected 0xC0000000",
                     gw_r32((const void *) (uintptr_t) (scratch + code_off + 0x38u)));
        rc = 1;
    }
    free(dat);
    return rc;
}

void gw_mex_grfunction_tests_register(void) {
    gw_test_register("grfunction_tables", test_grfunction_tables);
    gw_test_register("grfunction_rows", test_grfunction_rows);
    gw_test_register("grfunction_load_gromc", test_grfunction_load_gromc);
}
