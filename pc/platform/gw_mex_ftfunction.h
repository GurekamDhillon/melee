/* gw_mex_ftfunction.h - m-ex ftFunction blob loader (phase 2 of the PPC blob executor).
 *
 * Loads a fighter .dat from the disc, extracts the HSD public symbol "ftFunction", copies its
 * relocatable PPC code into guest MEM1, applies the m-ex `Reloc` and `Overload` semantics, and
 * reports the parsed structure. It does NOT wire the overrides into the fighter runtime dispatch
 * (that is phase 3). See _research/mex-ppc-interpreter.md.
 *
 * This is NATIVE platform code (compiled directly with the i686 clang, not through gwtool). It
 * reads the disc file big-endian with gw_r32/... and writes guest memory the same way.
 */
#ifndef GW_MEX_FTFUNCTION_H
#define GW_MEX_FTFUNCTION_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* One resolved function-reloc entry (the m-ex `Overload` walk). */
typedef struct gw_ftfunction_override {
    uint32_t slot;    /* table-index case: Arch_FighterFunc word index (0..45);
                       * func-address case: the absolute guest hook site */
    uint32_t target;  /* resolved guest address (code_base + ReplaceWith) */
    int is_func_addr; /* non-zero => func-address case (must become a native hook, phase 3) */
} gw_ftfunction_override;

/* A loaded, relocated ftFunction blob. */
typedef struct gw_ftfunction {
    uint32_t code_base;         /* guest address of the relocated code */
    uint32_t code_size;         /* bytes (the struct's codeSize field) */
    uint32_t instr_reloc_count; /* instructionRelocTableCount */
    uint32_t func_reloc_count;  /* functionRelocTableCount */
    uint32_t mexdata_base;      /* guest address of the Arch_FighterFunc holder (r2 base) */
    int override_count;
    gw_ftfunction_override overrides[64]; /* capped; func_reloc_count is authoritative */
} gw_ftfunction;

/* Error codes. 0 = success, negative = a clean, logged failure (never a native fault). */
enum {
    GW_FTFUNC_OK = 0,
    GW_FTFUNC_ERR_NO_FILE = -1,      /* disc file not found / unreadable */
    GW_FTFUNC_ERR_NO_SYMBOL = -2,    /* archive has no ftFunction public symbol */
    GW_FTFUNC_ERR_BAD_ARCHIVE = -3,  /* HSD header/file-size mismatch */
    GW_FTFUNC_ERR_BAD_STRUCT = -4,   /* a struct pointer/count runs past the file */
    GW_FTFUNC_ERR_BAD_INSTR_RELOC = -5, /* an instruction-reloc entry is out of bounds */
    GW_FTFUNC_ERR_BAD_FUNC_RELOC = -6,  /* a function-reloc entry is out of bounds */
    GW_FTFUNC_ERR_BAD_SLOT = -7,     /* table-index case names an out-of-range slot */
};

/* Load + relocate the ftFunction blob from `dat_path` (e.g. "PlSn.dat") on the disc image. The
 * code is placed at the fixed guest address GW_FTFUNC_CODE_BASE; Arch_FighterFunc and its
 * per-kind tables are placed at GW_FTFUNC_MEXDATA_BASE. `internal_id` is the fighter's m-ex
 * internal character id (31 for Sonic) used to index the per-kind tables. Returns 0 on success
 * (out is filled and the structure is logged), or a GW_FTFUNC_ERR_* code with the reason logged. */
int gw_ftfunction_load(const char *dat_path, uint32_t internal_id, gw_ftfunction *out);

/* Like gw_ftfunction_load, but relocates the code to an explicit `code_base` and the
 * Arch_FighterFunc tables to an explicit `mexdata_base` (both guest addresses the caller has
 * already allocated from the fighter heap). The phase-3 runtime path. */
int gw_ftfunction_load_at(const char *dat_path, uint32_t internal_id, uint32_t code_base,
                          uint32_t mexdata_base, gw_ftfunction *out);

/* Logs the parsed structure (code size, reloc counts, resolved per-slot overrides). */
void gw_ftfunction_report(const gw_ftfunction *ff);

/* Registers the module's self-contained tests with the in-engine suite. */
void gw_ftfunction_tests_register(void);

#ifdef __cplusplus
}
#endif
#endif /* GW_MEX_FTFUNCTION_H */
