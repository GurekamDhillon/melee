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

#include <stdio.h>
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
#define GRFUNC_OFF_SYMBOL_COUNT 0x18u
#define GRFUNC_OFF_SYMBOL_TABLE 0x1Cu

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
extern void gw_Mex_NoteGuestCodeInstalled(void);
extern void gw_Mex_ReleaseThunks(uint32_t lo, uint32_t hi);
extern uint32_t gw_Mex_MexData(uint32_t *base, uint32_t *size);
extern int gw_DVDConvertPathToEntrynum(const char *path);
extern int gw_DVDFileExists(const char *path);

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
    /* The external bound was a loose 4096 and should not have been: stage_id_map[] is indexed by
     * a raw StKind out of mexData at several sites with no check of their own, so the real limit
     * is the port's own table. Both bounds are now the compiled-in table sizes. */
    if (gr_int_count <= 0 || gr_int_count > GW_MEX_GR_MAX || gr_ext_count <= 0 ||
        gr_ext_count > GW_MEX_GR_EXT_MAX)
    {
        gw_log("grfunction: implausible stage counts (internal %d, external %d; the port's "
               "stage_datas[] holds %d rows and stage_id_map[] %d) - m-ex stages stay off",
               gr_int_count, gr_ext_count, GW_MEX_GR_MAX, GW_MEX_GR_EXT_MAX);
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

void gw_Mex_GrInvalidate(void) {
    gr_tables_ready = 0;
    gr_root = gr_base = gr_size = gr_map = gr_desc = 0u;
    gr_int_count = gr_ext_count = 0;
}

/* ---- stage BGM (Arch_Map_Playlists) ---------------------------------------------------- */

#define GW_MEX_GR_MAP_PLAYLISTS 0x14u
/* mexData's bgm_count is 139 on Akaneia and 231 on ACE; this is a loose upper bound whose
 * only job is to stop a misread word reaching the audio path. */
#define GW_MEX_GR_BGM_MAX 1024

/* Guest address of this stage's playlist row {s32 count; entries*}, or 0. */
static uint32_t gr_playlist_row(int grkind, int *out_count) {
    uint32_t tbl, row;
    int count;
    if (out_count != NULL) {
        *out_count = 0;
    }
    if (!gr_tables() || grkind < 0 || grkind >= gr_int_count) {
        return 0u;
    }
    tbl = gr_rd(gr_map + GW_MEX_GR_MAP_PLAYLISTS);
    if (tbl == 0u) {
        return 0u;
    }
    row = tbl + (uint32_t) grkind * 8u;
    if (!gr_in(row, 8u)) {
        return 0u;
    }
    count = (int) gr_rd(row);
    if (count <= 0 || count > 64) {
        return 0u;
    }
    if (out_count != NULL) {
        *out_count = count;
    }
    return row;
}

int gw_Mex_GrBgmCount(int grkind) {
    int count = 0;
    (void) gr_playlist_row(grkind, &count);
    return count;
}

/* Entry `i` of this stage's playlist, as {u16 bgm_id; u16 chance}. Returns -1 / 0 when there is
 * no such entry or the bytes are not plausible.
 *
 * These return values rather than filling out-parameters, and that is deliberate. An out-param
 * points at GAME memory, and gwtool-compiled game code reads its locals BIG-ENDIAN while a native
 * shim writes host order - so `*bgm = 98` came back to the game as 0x62000000 and faulted inside
 * mpLib_800569EC. A return value travels in a register and has no byte order. Any future shim
 * that writes through a pointer into game memory has to byte-swap; returning is safer. */
static int gr_bgm_word(int grkind, int i, int want_chance) {
    int count = 0;
    uint32_t row = gr_playlist_row(grkind, &count);
    uint32_t entries, e;
    const uint8_t *p;
    int id, ch;

    if (row == 0u || i < 0 || i >= count) {
        return want_chance ? 0 : -1;
    }
    entries = gr_rd(row + 4u);
    e = entries + (uint32_t) i * 4u;
    if (entries == 0u || !gr_in(e, 4u)) {
        return want_chance ? 0 : -1;
    }
    /* Dumped from Akaneia: internal 76 is `00 62 00 64` = bgm 0x62 chance 100; internal 7 is
     * `00 4B 00 5A` / `00 82 00 32` / `00 01 00 10`. Byte 2 is the HIGH byte of a 16-bit chance,
     * not padding - reading it as {u16; u8; u8} only works because it is always zero here. */
    p = (const uint8_t *) (uintptr_t) e;
    id = (int) (((uint32_t) p[0] << 8) | p[1]);
    ch = (int) (((uint32_t) p[2] << 8) | p[3]);

    if (id <= 0 || id >= GW_MEX_GR_BGM_MAX) {
        gw_log("grfunction: stage %d playlist entry %d has implausible bgm id %d "
               "(bytes %02X %02X %02X %02X) - ignoring it",
               grkind, i, id, p[0], p[1], p[2], p[3]);
        return want_chance ? 0 : -1;
    }
    if (ch < 0 || ch > 10000) {
        ch = 0;
    }
    return want_chance ? ch : id;
}

int gw_Mex_GrBgmId(int grkind, int i) { return gr_bgm_word(grkind, i, 0); }
int gw_Mex_GrBgmChance(int grkind, int i) { return gr_bgm_word(grkind, i, 1); }

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
            if (i <= 1u) {
                return NULL;
            }
            /* An ADDED stage whose file is not on the disc or in a mounted mod has no file: the
             * row is in MxDt.dat (the base mod ships every row), but its per-stage mod is off.
             * Returning NULL here is what hides it from the SSS (mnStageSel_MexScanUnlocked)
             * and keeps it out of the stage tables (gw_Mex_GrIsMex). Vanilla rows are always
             * on the disc and are not checked. */
            if (grkind >= GW_MEX_GR_FIRST_NEW && !gw_DVDFileExists(s)) {
                return NULL;
            }
            return s;
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
     * would walk into a missing file. This is the stage side of the fighter path's dense slots.
     * gw_Mex_GrFile already returns NULL for an added stage whose file is absent. */
    return f != NULL;
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
        /* The thunks bound to this stage's StageCallbacks outlive the code they point at: the
         * blob was relocated inside the stage's archive, and that archive is freed with the
         * scene. Releasing them is the stage half of the discipline the fighter path already
         * keeps on reinstall - without it, a stale gobj proc from the previous stage would
         * interpret whatever now occupies that memory. */
        gw_Mex_ReleaseThunks(gr_code_lo, gr_code_hi);
    }
    gr_code_lo = gr_code_hi = 0u;
    gr_loaded = -1;
    /* The stage's item articles were relocated into the same archive and die with it. */
    gw_Mex_UnloadStageItems();
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

    /* 10 of ACE's 42 added-stage blobs declare codeSize 0 - the same older MexTK that produced
     * the fighter-side blobs with no size. Recover it from the archive layout; the derivation and
     * the measurements behind it are in gw_mex_ftfunction.h / .c. */
    if (code_size == 0u && code_off < arch_data_size) {
        uint32_t others[4], bound, need;
        int on = 0;
        others[on++] = (uint32_t) pub;
        if (irt_count != 0u) {
            others[on++] = irt_off;
        }
        if (frt_count != 0u) {
            others[on++] = frt_off;
        }
        /* The struct is only guaranteed to 0x18 above; the symbol words are two more. */
        if ((uint32_t) pub + 0x20u <= arch_data_size &&
            gw_r32(dat + sym + GRFUNC_OFF_SYMBOL_COUNT) != 0u)
        {
            others[on++] = gw_r32(dat + sym + GRFUNC_OFF_SYMBOL_TABLE);
        }
        bound = gw_ftfunction_code_bound(dat, dat_size, code_off, others, on);
        need = gw_ftfunction_reloc_extent(dat, dat_size, irt_off, irt_count);
        if (bound > code_off && need != 0u && bound - code_off >= need) {
            code_size = bound - code_off;
            gw_log("grfunction: %s declares codeSize 0 (a MexTK older than the field); recovered "
                   "0x%X from the archive layout - code 0x%X..0x%X, instruction relocs reach 0x%X",
                   file, code_size, code_off, bound, need);
        } else {
            gw_log("grfunction: %s declares codeSize 0 and the archive layout does not recover it "
                   "(code 0x%X, next structure 0x%X, instruction relocs need 0x%X) - clone-base "
                   "handlers only",
                   file, code_off, bound, need);
        }
    }

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
    /* Arm the interpreter's execute trap. A stage blob hands code addresses to native engine
     * code the same way a fighter blob does - a map gobj's callbacks, or an address stored
     * straight into engine data - and until the trap is armed the first such call is a fatal
     * access violation inside MEM1 instead of an interpreted call. gw_Mex_FtFunctionInstall()
     * used to be the only thing that armed it, so a stage run with a vanilla fighter never did.
     */
    /* MELEE_GR_DUMP_CODE=<path> writes the RELOCATED stage blob, the same way
     * MELEE_MEX_DUMP_CODE does for a fighter blob, so a call site the trace names can be
     * disassembled offline:
     *   python tools/mex_port/ppc_disasm.py --raw <path> --base <gr_code_lo> --start <va> --count N
     * The disc bytes are no use for this - their branch and address operands are unrelocated. */
    {
        const char *gr_dump = getenv("MELEE_GR_DUMP_CODE");
        if (gr_dump != NULL && gr_dump[0] != 0) {
            FILE *gf = fopen(gr_dump, "wb");
            if (gf == NULL) {
                gw_log("grfunction: MELEE_GR_DUMP_CODE: cannot open %s", gr_dump);
            } else {
                size_t wrote = fwrite((const void *)(uintptr_t)code_base, 1, code_size, gf);
                fclose(gf);
                gw_log("grfunction: dumped %u bytes of relocated stage code (base 0x%08X) to %s",
                       (unsigned)wrote, code_base, gr_dump);
            }
        }
    }
    gw_Mex_NoteGuestCodeInstalled();
    gw_log("grfunction: %s (internal stage %d) installed: code 0x%08X..0x%08X (%u bytes), %u "
           "instruction relocs, %d of %u overloads",
           file, grkind, gr_code_lo, gr_code_hi, code_size, irt_count, n, frt_count);
    for (i = 0; i < GW_MEX_GR_SLOT_COUNT; ++i) {
        if (gr_slot[i] != 0u) {
            gw_log("grfunction:   StageData word %u -> %s", i, gw_ppc_describe(gr_slot[i]));
        }
    }
    /* A stage's own item articles ship in the same archive under `itFunction`, exactly as a
     * fighter's do, and fill item.Custom for the global kinds MEX_GetGrItemID hands out. */
    gw_Mex_LoadStageItems(file, grkind, (void *) (uintptr_t) arch_data, arch_data_size);
}

void *gw_Mex_GrRowCallbacks(int grkind) {
    uint32_t row = gr_row(grkind), g, native;
    int kind = -1;
    static uint32_t moaned[16];
    static int nmoaned;
    int i;

    if (row == 0u) {
        return NULL;
    }
    g = gr_rd(row + (uint32_t) GW_MEX_GR_SLOT_CALLBACKS * 4u);
    if (g == 0u || g == 0xFFFFFFFFu) {
        return NULL;
    }
    /* The word is a GUEST address. A vanilla StageCallbacks[] is a static in the retargeted
     * exe, not in MEM1, so it has to come back through the bridge as a DATA object (kind 0) -
     * the same translation gw_ppc_static_native() does for an interpreted load. A guest
     * address that is not an exact bridge base cannot be translated, and a raw guest value
     * stored into a StageData row would be dereferenced by native game code as if it were a
     * host pointer, so refuse rather than guess. */
    native = gw_mex_bridge_lookup(g, &kind);
    if (native != 0u && kind == 0) {
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
    gw_log("grfunction: internal stage %d names StageCallbacks[] at guest 0x%08X, which has "
           "no data entry in the bridge - this stage gets no fog, lighting or per-model "
           "camera",
           grkind, g);
    return NULL;
}

void *gw_Mex_GrCallbacks(void) {
    if (gr_loaded < 0 || gr_loaded != gr_cur) {
        return NULL;
    }
    return (void *) (uintptr_t) gr_slot[GW_MEX_GR_SLOT_CALLBACKS];
}

/* MELEE_GR_TRACE=1: narrate the m-ex stage bring-up (which map gobjs get created and set up,
 * which StageCallbacks table each one used, and what the lighting and fog scans selected).
 * Off by default and read once - these sit on paths every stage walks. */
/* The blob call site that reached the native function now running, for tracing a stage whose own
 * code drives the call. A map gobj created 300 times says the loop is in the blob, not the
 * engine; this says WHERE in the blob. Zero when the engine itself is the caller. */
unsigned int gw_Mex_GuestLr(void) { return gw_ppc_guest_lr(); }

int gw_Mex_GrTrace(void) {
    static int v = -1;
    if (v < 0) {
        const char *e = getenv("MELEE_GR_TRACE");
        v = (e != NULL && e[0] != 0 && e[0] != '0') ? 1 : 0;
    }
    return v;
}

void *gw_Mex_GrBind(void *fn) {
    uint32_t a = (uint32_t) (uintptr_t) fn;
    if (a == 0u) {
        return fn;
    }
    /* A STAGE'S CALLBACKS TABLE IS NOT ALL BLOB CODE. gw_ppc_is_guest_code answers only
     * "is this inside a registered blob range", so guarding on it let a VANILLA guest
     * address through untouched - and the engine then called it as a native function
     * pointer. Akaneia's GrOPz.dat overloads three StageData words with blob code and
     * leaves the rest of its StageCallbacks pointing at the retail engine, so
     * stageGObj0_OnInit arrived as guest 0x802239F0 and the process jumped into MEM1.
     * It killed six stages on Akaneia alone, every one of them before a fighter loaded.
     *
     * The real question is "is this a GUEST address", not "is this blob code":
     * anything inside MEM1 needs binding, and gw_Mex_Callable already knows how to
     * answer both halves - a thunk for blob code, the bridged native for an engine
     * address. A native pointer is outside MEM1 and passes straight through. */
    if (a < 0x80000000u || a >= 0x80000000u + gw_mem1_size) {
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
    uint32_t r;
    if (t == 0u) {
        return 0u;
    }
    if (!gw_Mex_GrTrace()) {
        return gw_ppc_call(t, args, nargs, gw_Mex_Rtoc(), gw_Mex_StackTop());
    }
    /* Bracket the call. If an interpreted StageData handler stops early - a bridge gap, the
     * interpreter's depth cap, a guarded load - the "enter" line is there and "leave" is not,
     * which is exactly the question map-gobj 1 and 2 never being set up poses. */
    gw_log("grtrace: enter StageData word %d -> %s (stage %d)", slot, gw_ppc_describe(t), gr_cur);
    r = gw_ppc_call(t, args, nargs, gw_Mex_Rtoc(), gw_Mex_StackTop());
    gw_log("grtrace: leave StageData word %d, r3=0x%08X", slot, r);
    return r;
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
    /* A row naming a file is not the same as the file being here: a vanilla disc under a
     * base mod has the full table and only the stage files whose mods are enabled, and then
     * gw_Mex_GrFile correctly answers NULL for the rest (gw_mods.h). Assert only when the
     * stage's file is actually present. */
    if (!gw_DVDFileExists("/GrOMc.dat")) {
        gw_log("grfunction: /GrOMc.dat not on this disc or in a mounted mod - skipping the row "
               "assertions");
        return 0;
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

/* The shared thunk pool, past the first eight.
 *
 * A stage needs nine thunks at once - three map gobjs, each with an on_init, a gobj_proc and a
 * callback3 - and that is the first content in the port to need more than eight. It found a
 * numbering bug in gw_mex_ftfunction_runtime.c's thunk table: entry N called
 * gw_mex_thunk_run with a DECIMAL two-digit index instead of an octal one, so thunk 8 read
 * gw_mex_thunk_guest[10] (never assigned, hence pc=0) and thunk 63 read past the array.
 *
 * This binds more than eight distinct guest addresses and checks each thunk reaches ITS OWN one,
 * which is the invariant the numbering broke. Pure interpreter + pool: no disc, no mexData, so it
 * runs on every disc. */
#define GW_MEX_GR_THUNK_TEST_BASE 0x81000000u
#define GW_MEX_GR_THUNK_TEST_N 12

typedef uint32_t (*gw_gr_thunk_fn)(uint32_t, uint32_t, uint32_t, uint32_t);

/* THE ROW'S OWN StageCallbacks[] WORD MUST SURVIVE THE GUEST->NATIVE BOUNDARY.
 *
 * An m-ex row that CLONES a vanilla stage names that stage's compiled-in StageCallbacks[] in
 * StageData word 1, as a GUEST address. The port dropped that word and set the synthesised row's
 * `callbacks` to NULL, which ground.c's fog scan then dereferenced - `foo()`/Ground_801C1E94 read
 * 0x10 and killed every ACE run that reached internal stage 96 or above.
 *
 * Akaneia cannot see this: all 25 of its added rows ship word 1 as 0 and let the blob supply the
 * table. ACE's added rows mostly do the opposite. So the test is written to say which disc it is
 * looking at rather than to assert one disc's shape: it fails only if rows DO carry the word and
 * not one of them translates, which is the mechanism being broken rather than the disc being
 * different. Anything that does not translate is named, because each one is a stage that silently
 * loses fog, lighting and its per-model camera. */
static int test_grfunction_row_callbacks(void) {
    int k, n, with_word = 0, resolved = 0;

    if (gw_iso_path() == NULL || (n = gw_Mex_GrInternalCount()) == 0) {
        gw_log("grfunction: no m-ex stage tables on this disc - skipping");
        return 0;
    }
    for (k = GW_MEX_GR_FIRST_NEW; k < n && k < GW_MEX_GR_MAX; k++) {
        uint32_t row = gr_row(k), g;
        void *native;
        if (row == 0u) {
            continue;
        }
        g = gr_rd(row + (uint32_t) GW_MEX_GR_SLOT_CALLBACKS * 4u);
        if (g == 0u || g == 0xFFFFFFFFu) {
            continue;
        }
        ++with_word;
        native = gw_Mex_GrRowCallbacks(k);
        if (native == NULL) {
            continue;
        }
        ++resolved;
        /* A raw guest value stored into a StageData row would be dereferenced by native game
         * code as a host pointer. Translation must actually have happened. */
        if ((uint32_t) (uintptr_t) native == g) {
            gw_test_fail("internal stage %d returned its raw guest StageCallbacks[] 0x%08X "
                         "instead of a native address",
                         k, g);
            return 1;
        }
    }
    gw_log("grfunction: %d of %d added rows name a StageCallbacks[] of their own; %d translated",
           with_word, n - GW_MEX_GR_FIRST_NEW, resolved);
    if (with_word > 0 && resolved == 0) {
        gw_test_fail("%d added rows name a StageCallbacks[] and none translated - the guest->native "
                     "data bridge lookup is not working",
                     with_word);
        return 1;
    }
    return 0;
}

static int test_grfunction_thunk_pool_beyond_8(void) {
    uint32_t base = GW_MEX_GR_THUNK_TEST_BASE;
    uint32_t bound[GW_MEX_GR_THUNK_TEST_N];
    int i, rc = 0;

    gw_Mex_RuntimeInit();
    if (gw_Mex_StackTop() == 0u) {
        gw_test_fail("m-ex PPC runtime did not initialise (stack top 0)");
        return 1;
    }
    /* Each stub is `li r3, 0x100+i ; blr` - eight bytes, distinct return value. */
    for (i = 0; i < GW_MEX_GR_THUNK_TEST_N; ++i) {
        gw_w32((void *) (uintptr_t) (base + (uint32_t) i * 8u), 0x38600000u | (0x100u + (uint32_t) i));
        gw_w32((void *) (uintptr_t) (base + (uint32_t) i * 8u + 4u), 0x4E800020u);
    }
    gw_ppc_add_code_range(base, base + GW_MEX_GR_THUNK_TEST_N * 8u);

    for (i = 0; i < GW_MEX_GR_THUNK_TEST_N; ++i) {
        bound[i] = gw_Mex_Callable(base + (uint32_t) i * 8u, "grfunction thunk pool test");
        if (bound[i] == 0u) {
            gw_test_fail("thunk %d did not bind", i);
            rc = 1;
            goto done;
        }
    }
    for (i = 0; i < GW_MEX_GR_THUNK_TEST_N; ++i) {
        uint32_t got = ((gw_gr_thunk_fn) (uintptr_t) bound[i])(0u, 0u, 0u, 0u);
        if (got != 0x100u + (uint32_t) i) {
            gw_test_fail("thunk %d ran the wrong guest function: returned 0x%X, expected 0x%X "
                         "(the pool index it passes does not match its table position)",
                         i, got, 0x100u + (uint32_t) i);
            rc = 1;
            goto done;
        }
    }
done:
    gw_ppc_remove_code_range(base, base + GW_MEX_GR_THUNK_TEST_N * 8u);
    return rc;
}

/* Every m-ex-ADDED stage this disc actually ships must parse. GrOMc (above) is one stage on one
 * disc; this walks the whole added range, which is 25 rows on Akaneia and 84 on ACE - and ACE's
 * 96..154 were unreachable until the port's tables were sized for them, so nothing had ever
 * looked at them. Header-level only: the file is read, grFunction is located, and its code and
 * both reloc tables are required to lie inside the data section with every Overload slot naming
 * a real StageData word (0..12). That is exactly what gw_Mex_GrFunctionInit() will trust at load
 * time, and it is cheap enough to run over every stage on the disc. */
static int test_grfunction_added_rows_parse(void) {
    int k, n, checked = 0, nofile = 0, nosym = 0, recovered = 0;

    if (gw_iso_path() == NULL || (n = gw_Mex_GrInternalCount()) == 0) {
        gw_log("grfunction: no m-ex stage tables on this disc - skipping");
        return 0;
    }
    for (k = GW_MEX_GR_FIRST_NEW; k < n; k++) {
        const char *path = gw_Mex_GrFile(k);
        unsigned char *dat;
        uint32_t dat_size = 0, data_size, sym, code_off, code_size, irt_off, irt_count;
        uint32_t frt_off, frt_count, i;
        int32_t pub;
        int bad = 0;

        /* gw_Mex_GrFile() answers from the table alone - it does NOT check the disc; only
         * gw_Mex_GrIsMex() does, and that is what decides whether a row is synthesised. A
         * vanilla disc run with the mods folder on sees Akaneia's full 96-row table out of the
         * sonic mod and none of the stage files, so this is the normal case there, not an edge. */
        if (path == NULL || !gw_Mex_GrIsMex(k)) {
            ++nofile;
            continue;
        }
        dat = (unsigned char *) gw_DVDReadFileAlloc(path, &dat_size);
        if (dat == NULL) {
            gw_test_fail("internal stage %d names %s, which gw_Mex_GrFile() said is on this disc "
                         "but could not be read",
                         k, path);
            return 1;
        }
        data_size = dat_size >= 0x20u ? gw_r32(dat + 0x04) : 0u;
        pub = gw_ftfunction_find_public(dat, dat_size, "grFunction");
        if (pub < 0) {
            /* Legal: a row can be a plain stage file with no code of its own, running entirely
             * on its clone base's handlers. Counted so it is visible rather than assumed. */
            ++nosym;
            free(dat);
            continue;
        }
        sym = 0x20u + (uint32_t) pub;
        if ((uint32_t) pub + 0x18u > data_size) {
            gw_test_fail("%s (internal %d): grFunction struct past the data section", path, k);
            free(dat);
            return 1;
        }
        code_off = gw_r32(dat + sym + GRFUNC_OFF_CODE);
        code_size = gw_r32(dat + sym + GRFUNC_OFF_CODE_SIZE);
        irt_off = gw_r32(dat + sym + GRFUNC_OFF_INSTR_RELOC);
        irt_count = gw_r32(dat + sym + GRFUNC_OFF_INSTR_RELOC_COUNT);
        frt_off = gw_r32(dat + sym + GRFUNC_OFF_FUNC_RELOC);
        frt_count = gw_r32(dat + sym + GRFUNC_OFF_FUNC_RELOC_COUNT);
        if (code_size == 0u && code_off < data_size) {
            uint32_t others[4], b, need;
            int on = 0;
            others[on++] = (uint32_t) pub;
            if (irt_count != 0u) {
                others[on++] = irt_off;
            }
            if (frt_count != 0u) {
                others[on++] = frt_off;
            }
            if ((uint32_t) pub + 0x20u <= data_size &&
                gw_r32(dat + sym + GRFUNC_OFF_SYMBOL_COUNT) != 0u)
            {
                others[on++] = gw_r32(dat + sym + GRFUNC_OFF_SYMBOL_TABLE);
            }
            b = gw_ftfunction_code_bound(dat, dat_size, code_off, others, on);
            need = gw_ftfunction_reloc_extent(dat, dat_size, irt_off, irt_count);
            if (b <= code_off || need == 0u || b - code_off < need) {
                gw_test_fail("%s (internal %d): declares codeSize 0 and the archive layout does "
                             "not recover it (code 0x%X, next structure 0x%X, relocs need 0x%X)",
                             path, k, code_off, b, need);
                free(dat);
                return 1;
            }
            code_size = b - code_off;
            ++recovered;
        }

        if (code_size == 0u || code_off > data_size || code_size > data_size - code_off) {
            gw_test_fail("%s (internal %d): code [0x%X,+0x%X) outside the 0x%X-byte data section",
                         path, k, code_off, code_size, data_size);
            bad = 1;
        } else if (irt_count != 0u &&
                   (irt_off > data_size || irt_count * 8u > data_size - irt_off)) {
            gw_test_fail("%s (internal %d): instruction reloc table outside the data section",
                         path, k);
            bad = 1;
        } else if (frt_count == 0u || frt_off > data_size ||
                   frt_count * 8u > data_size - frt_off) {
            gw_test_fail("%s (internal %d): function reloc table (%u entries) outside the data "
                         "section, or empty",
                         path, k, frt_count);
            bad = 1;
        }
        for (i = 0; bad == 0 && i < frt_count; ++i) {
            uint32_t slot = gw_r32(dat + 0x20u + frt_off + i * 8u);
            uint32_t with = gw_r32(dat + 0x20u + frt_off + i * 8u + 4u);
            if (slot >= GW_MEX_GR_SLOT_COUNT) {
                gw_test_fail("%s (internal %d): Overload entry %u names StageData word %u, but a "
                             "row is only %d words",
                             path, k, i, slot, GW_MEX_GR_SLOT_COUNT);
                bad = 1;
            } else if (with >= code_size) {
                gw_test_fail("%s (internal %d): Overload entry %u points at code+0x%X, past the "
                             "0x%X-byte blob",
                             path, k, i, with, code_size);
                bad = 1;
            }
        }
        free(dat);
        if (bad) {
            return 1;
        }
        ++checked;
    }
    gw_log("test grfunction_added_rows_parse: %d added stage blob(s) parse (%d with a recovered "
           "codeSize); %d row(s) not on this disc, %d file(s) with no grFunction of their own",
           checked, recovered, nofile, nosym);
    return 0;
}

void gw_mex_grfunction_tests_register(void) {
    gw_test_register("grfunction_tables", test_grfunction_tables);
    gw_test_register("grfunction_rows", test_grfunction_rows);
    gw_test_register("grfunction_row_callbacks", test_grfunction_row_callbacks);
    gw_test_register("grfunction_load_gromc", test_grfunction_load_gromc);
    gw_test_register("grfunction_thunk_pool_beyond_8", test_grfunction_thunk_pool_beyond_8);
    gw_test_register("grfunction_added_rows_parse", test_grfunction_added_rows_parse);
}
