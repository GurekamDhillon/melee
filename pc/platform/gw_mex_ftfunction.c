/* gw_mex_ftfunction.c - m-ex ftFunction blob loader (phase 2). See gw_mex_ftfunction.h.
 *
 * Reads a fighter .dat (HSD archive, big-endian) from the disc, finds the "ftFunction" public
 * symbol, copies the relocatable PPC code into guest MEM1, and applies the m-ex `Reloc` and
 * `Overload` semantics (reimplemented from the spec in _build/m-ex/asm/m-ex/ "Init ftFunction.asm"
 * and "Standalone Functions/Reloc.asm" - no m-ex source is vendored).
 *
 * Verified struct layout (PlSn.dat on the Akaneia disc, data-relative):
 *   0x00 code, 0x04 instructionRelocTable, 0x08 instructionRelocTableCount,
 *   0x0C functionRelocTable, 0x10 functionRelocTableCount, 0x14 codeSize.
 * All three pointer fields are offsets from the HSD data section base (they are relocated by the
 * archive's own relocation pass, which adds the data base to each listed pointer).
 *
 * Corruption never faults natively: every parse step is bounds-checked and returns a distinct
 * error code with a logged reason, so a deliberately bad blob fails clean.
 */
#include "gw.h"
#include "gw_test.h"
#include "gw_ppc.h"
#include "gw_mex_ftfunction.h"

#include <stdlib.h>
#include <string.h>

/* Guest placement (MEM1). Phase 2 uses a fixed region because the loader is only reached from the
 * test suite (the game main never runs under --test). Phase 3 will allocate from the fighter heap
 * instead. Both addresses sit in free MEM1 well below the phase-1 test scratch (0x80300000+). */
#define GW_FTFUNC_CODE_BASE 0x802F0000u    /* relocated code */
#define GW_FTFUNC_MEXDATA_BASE 0x802E0000u /* r2 base: Arch_FighterFunc + per-kind arrays */
#define GW_FTFUNC_KIND_MAX 64              /* per-kind array width (m-ex internal ids) */
#define GW_FTFUNC_SLOT_COUNT 46            /* Arch_FighterFunc word slots (Header.s) */
#define GW_FTFUNC_PERKIND_STRIDE (GW_FTFUNC_KIND_MAX * 4u)

#define GW_HSD_HEADER_SIZE 0x20u

#define FTFUNC_OFF_CODE 0x00u
#define FTFUNC_OFF_INSTR_RELOC 0x04u
#define FTFUNC_OFF_INSTR_RELOC_COUNT 0x08u
#define FTFUNC_OFF_FUNC_RELOC 0x0Cu
#define FTFUNC_OFF_FUNC_RELOC_COUNT 0x10u
#define FTFUNC_OFF_CODE_SIZE 0x14u

/* Arch_FighterFunc slot names, word-indexed (Header.s: onLoad 0x0 ... GetTrailData 0xB4). */
static const char *const gw_ftfunction_slot_names[GW_FTFUNC_SLOT_COUNT] = {
    "onLoad",         "onDeath",        "onDestroy",       "MoveLogic",
    "SpecialN",       "SpecialNAir",    "SpecialS",        "SpecialSAir",
    "SpecialHi",      "SpecialHiAir",   "SpecialLw",       "SpecialLwAir",
    "onAbsorb",       "OnItemPickup",   "onMakeItemInvisible", "onMakeItemVisible",
    "OnItemRelease",  "OnItemPickup2",  "onUnknownItemRelated", "onApplyHeadItem",
    "onRemoveHeadItem", "onKnockbackEnter", "onKnockbackExit", "onFrame",
    "onActionStateChange", "onReapplyAttr", "onModelRender", "onShadowRender",
    "onUnknownMultijump", "onActionStateChangeWhileEyeTextureIsChanged", "onTwoEntryTable", "onFloat",
    "onDoubleJump",   "onZair",         "onLanding",       "onFSmash",
    "onUSmash",       "onDSmash",       "onGetExtResultAnim", "onIndexExtResultAnim",
    "MoveLogicDemo",  "onIntroL",       "onIntroR",        "onTaunt",
    "onCatch",        "GetTrailData",
};

static const char *gw_ftfunction_slot_name(uint32_t slot) {
    return slot < GW_FTFUNC_SLOT_COUNT ? gw_ftfunction_slot_names[slot] : "?";
}
/* ---- HSD archive public-symbol lookup ------------------------------------------------ */

static int32_t gw_ftfunction_find_public(const unsigned char *dat, size_t dat_size,
                                         const char *symbol) {
    uint32_t data_size, nb_reloc, nb_public, nb_extern;
    uint32_t o_public, o_symbols;
    uint32_t i;

    if (dat_size < GW_HSD_HEADER_SIZE) {
        return -1;
    }
    data_size = gw_r32(dat + 0x04);
    nb_reloc = gw_r32(dat + 0x08);
    nb_public = gw_r32(dat + 0x0C);
    nb_extern = gw_r32(dat + 0x10);
    o_public = GW_HSD_HEADER_SIZE + data_size + nb_reloc * 4u;
    o_symbols = o_public + nb_public * 8u + nb_extern * 8u;

    for (i = 0; i < nb_public; ++i) {
        uint32_t p = o_public + i * 8u;
        uint32_t sym_off;
        if (p + 8u > dat_size) {
            return -1;
        }
        sym_off = gw_r32(dat + p + 4);
        if (o_symbols + sym_off < dat_size && strcmp((const char *)(dat + o_symbols + sym_off),
                                                     symbol) == 0) {
            return (int32_t)gw_r32(dat + p);
        }
    }
    return -1;
}

/* ---- Reloc (reimplemented from Standalone Functions/Reloc.asm) ------------------------
 * Each 8-byte entry is { flag:code_offset (u32), target (u32) }. The flag is the top byte; the
 * code offset is the low 24 bits. A target whose top nibble is 0x8 is an absolute guest address
 * (the asm tests `(target << 8) & 0xF0 == 0x80`); otherwise it is an offset from the code base. */

static int gw_ftfunction_reloc(const unsigned char *dat, size_t dat_size, uint32_t irt_data_off,
                               uint32_t irt_count, uint32_t code_base, uint32_t code_size) {
    uint32_t i;

    for (i = 0; i < irt_count; ++i) {
        uint32_t e = GW_HSD_HEADER_SIZE + irt_data_off + i * 8u;
        uint32_t w0, w1, flag, code_off, code_ptr, func_ptr;

        if (e + 8u > dat_size) {
            gw_log("ftfunction: instruction-reloc entry %u past end of file", i);
            return GW_FTFUNC_ERR_BAD_INSTR_RELOC;
        }
        w0 = gw_r32(dat + e);
        w1 = gw_r32(dat + e + 4);
        flag = w0 >> 24;
        code_off = w0 & 0x00FFFFFFu;
        if (code_off >= code_size) {
            gw_log("ftfunction: instruction-reloc entry %u code offset 0x%X >= codeSize 0x%X", i,
                   code_off, code_size);
            return GW_FTFUNC_ERR_BAD_INSTR_RELOC;
        }
        code_ptr = code_base + code_off;
        func_ptr = ((w1 & 0xF0000000u) == 0x80000000u) ? w1 : code_base + w1;

        switch (flag) {
        case 0x01: /* static address: store a 32-bit absolute value */
            gw_w32((void *)(uintptr_t)code_ptr, func_ptr);
            break;
        case 0x04: /* low 16 of an address (lis/addi pair) */
        case 0x06: /* high 16 of an address */
        {
            uint32_t v = func_ptr;
            if (v & 0x8000u) {
                uint32_t high = (v >> 16) + 1u;
                v = (high << 16) | ((v - (high << 16)) & 0xFFFFu);
            }
            gw_w16((void *)(uintptr_t)code_ptr,
                   (uint16_t)((flag == 0x04) ? (v & 0xFFFFu) : ((v >> 16) & 0xFFFFu)));
            break;
        }
        case 0x0A: /* branch: OR the displacement into the LI field of the existing word */
        {
            uint32_t old = gw_r32((void *)(uintptr_t)code_ptr);
            gw_w32((void *)(uintptr_t)code_ptr, old | ((func_ptr - code_ptr) & 0x03FFFFFCu));
            break;
        }
        case 0x1A: /* relative 32-bit offset */
            gw_w32((void *)(uintptr_t)code_ptr, func_ptr - code_ptr);
            break;
        default: /* unknown flag: m-ex skips it */
            break;
        }
    }
    return GW_FTFUNC_OK;
}

/* ---- Overload (reimplemented from Init ftFunction.asm) --------------------------------
 * Each 8-byte entry is { ReplaceThis, ReplaceWith }. Top bit of ReplaceThis clear => table index:
 * Arch_FighterFunc[ReplaceThis][internal_id] = code_base + ReplaceWith. Top bit set => the entry
 * is an absolute guest address to patch with a `b` branch; in the port that is native code we
 * must not write, so it is re-expressed as a native-hook registration (recorded, not applied). */

static int gw_ftfunction_overload(const unsigned char *dat, size_t dat_size, uint32_t frt_data_off,
                                  uint32_t frt_count, uint32_t code_base, uint32_t mexdata_base,
                                  uint32_t internal_id, gw_ftfunction *out) {
    uint32_t next_perkind = mexdata_base + GW_FTFUNC_SLOT_COUNT * 4u;
    uint32_t i;

    for (i = 0; i < frt_count; ++i) {
        uint32_t e = GW_HSD_HEADER_SIZE + frt_data_off + i * 8u;
        uint32_t replace_this, replace_with;
        uint32_t target;

        if (e + 8u > dat_size) {
            gw_log("ftfunction: function-reloc entry %u past end of file", i);
            return GW_FTFUNC_ERR_BAD_FUNC_RELOC;
        }
        replace_this = gw_r32(dat + e);
        replace_with = gw_r32(dat + e + 4);
        target = code_base + replace_with;

        if ((replace_this & 0x80000000u) == 0) {
            uint32_t slot = replace_this;
            uint32_t perkind;
            if (slot >= GW_FTFUNC_SLOT_COUNT) {
                gw_log("ftfunction: function-reloc entry %u slot %u out of range", i, slot);
                return GW_FTFUNC_ERR_BAD_SLOT;
            }
            perkind = gw_r32((void *)(uintptr_t)(mexdata_base + slot * 4u));
            if (perkind == 0) {
                perkind = next_perkind;
                next_perkind += GW_FTFUNC_PERKIND_STRIDE;
                gw_w32((void *)(uintptr_t)(mexdata_base + slot * 4u), perkind);
            }
            gw_w32((void *)(uintptr_t)(perkind + internal_id * 4u), target);

            if (out->override_count < (int)(sizeof out->overrides / sizeof out->overrides[0])) {
                out->overrides[out->override_count].slot = slot;
                out->overrides[out->override_count].target = target;
                out->overrides[out->override_count].is_func_addr = 0;
                out->override_count++;
            }
        } else {
            gw_log("ftfunction: func-addr hook: guest 0x%08X -> code 0x%08X (native hook, phase 3)",
                   replace_this, target);
            if (out->override_count < (int)(sizeof out->overrides / sizeof out->overrides[0])) {
                out->overrides[out->override_count].slot = replace_this;
                out->overrides[out->override_count].target = target;
                out->overrides[out->override_count].is_func_addr = 1;
                out->override_count++;
            }
        }
    }
    return GW_FTFUNC_OK;
}

/* ---- core: parse + relocate a whole .dat buffer in memory ---------------------------- */

static int gw_ftfunction_load_from_memory_at(const unsigned char *dat, size_t dat_size,
                                             uint32_t internal_id, uint32_t code_base,
                                             uint32_t mexdata_base, gw_ftfunction *out) {
    uint32_t data_size, file_size;
    int32_t pub_off;
    uint32_t struct_off, code_off, irt_off, frt_off, irt_count, frt_count, code_size;
    int rc;

    memset(out, 0, sizeof *out);

    if (dat_size < GW_HSD_HEADER_SIZE) {
        gw_log("ftfunction: file too small (%u bytes) for an HSD header", (unsigned)dat_size);
        return GW_FTFUNC_ERR_BAD_ARCHIVE;
    }
    file_size = gw_r32(dat + 0x00);
    data_size = gw_r32(dat + 0x04);
    if (file_size != dat_size) {
        gw_log("ftfunction: HSD file_size 0x%X != actual 0x%X", file_size, (unsigned)dat_size);
        return GW_FTFUNC_ERR_BAD_ARCHIVE;
    }
    if (data_size > dat_size - GW_HSD_HEADER_SIZE) {
        gw_log("ftfunction: data_size 0x%X overruns the file", data_size);
        return GW_FTFUNC_ERR_BAD_ARCHIVE;
    }

    pub_off = gw_ftfunction_find_public(dat, dat_size, "ftFunction");
    if (pub_off < 0) {
        gw_log("ftfunction: no ftFunction public symbol in the archive");
        return GW_FTFUNC_ERR_NO_SYMBOL;
    }
    struct_off = GW_HSD_HEADER_SIZE + (uint32_t)pub_off;
    if ((uint32_t)pub_off + 0x18u > data_size) {
        gw_log("ftfunction: ftFunction struct past the data section");
        return GW_FTFUNC_ERR_BAD_STRUCT;
    }

    code_off = gw_r32(dat + struct_off + FTFUNC_OFF_CODE);
    irt_off = gw_r32(dat + struct_off + FTFUNC_OFF_INSTR_RELOC);
    irt_count = gw_r32(dat + struct_off + FTFUNC_OFF_INSTR_RELOC_COUNT);
    frt_off = gw_r32(dat + struct_off + FTFUNC_OFF_FUNC_RELOC);
    frt_count = gw_r32(dat + struct_off + FTFUNC_OFF_FUNC_RELOC_COUNT);
    code_size = gw_r32(dat + struct_off + FTFUNC_OFF_CODE_SIZE);

    if (code_size == 0 || code_off > data_size || code_size > data_size - code_off) {
        gw_log("ftfunction: code [0x%X,+0x%X) outside the data section", code_off, code_size);
        return GW_FTFUNC_ERR_BAD_STRUCT;
    }
    if (irt_count != 0 && (irt_off > data_size || irt_count * 8u > data_size - irt_off)) {
        gw_log("ftfunction: instruction reloc table [0x%X,+%u*8) outside the data section", irt_off,
               irt_count);
        return GW_FTFUNC_ERR_BAD_STRUCT;
    }
    if (frt_count != 0 && (frt_off > data_size || frt_count * 8u > data_size - frt_off)) {
        gw_log("ftfunction: function reloc table [0x%X,+%u*8) outside the data section", frt_off,
               frt_count);
        return GW_FTFUNC_ERR_BAD_STRUCT;
    }
    if (code_size > gw_mem1_size || code_base + code_size > 0x80000000u + gw_mem1_size) {
        gw_log("ftfunction: code size 0x%X does not fit in guest MEM1", code_size);
        return GW_FTFUNC_ERR_BAD_STRUCT;
    }

    memcpy((void *)(uintptr_t)code_base, dat + GW_HSD_HEADER_SIZE + code_off, code_size);

    rc = gw_ftfunction_reloc(dat, dat_size, irt_off, irt_count, code_base, code_size);
    if (rc != GW_FTFUNC_OK) {
        return rc;
    }
    rc = gw_ftfunction_overload(dat, dat_size, frt_off, frt_count, code_base, mexdata_base,
                                internal_id, out);
    if (rc != GW_FTFUNC_OK) {
        return rc;
    }

    out->code_base = code_base;
    out->code_size = code_size;
    out->instr_reloc_count = irt_count;
    out->func_reloc_count = frt_count;
    out->mexdata_base = mexdata_base;
    return GW_FTFUNC_OK;
}

int gw_ftfunction_load_at(const char *dat_path, uint32_t internal_id, uint32_t code_base,
                          uint32_t mexdata_base, gw_ftfunction *out) {
    unsigned char *dat;
    uint32_t dat_size = 0;
    int rc;

    memset(out, 0, sizeof *out);
    dat = (unsigned char *)gw_DVDReadFileAlloc(dat_path, &dat_size);
    if (dat == NULL) {
        return GW_FTFUNC_ERR_NO_FILE;
    }
    rc = gw_ftfunction_load_from_memory_at(dat, dat_size, internal_id, code_base, mexdata_base,
                                           out);
    free(dat);
    if (rc == GW_FTFUNC_OK) {
        gw_ftfunction_report(out);
    }
    return rc;
}

int gw_ftfunction_load(const char *dat_path, uint32_t internal_id, gw_ftfunction *out) {
    return gw_ftfunction_load_at(dat_path, internal_id, GW_FTFUNC_CODE_BASE,
                                 GW_FTFUNC_MEXDATA_BASE, out);
}

/* ---- report -------------------------------------------------------------------------- */

void gw_ftfunction_report(const gw_ftfunction *ff) {
    int i;
    gw_log("ftfunction: code @ guest 0x%08X, %u bytes", ff->code_base, ff->code_size);
    gw_log("ftfunction: instruction relocs=%u, function relocs=%u", ff->instr_reloc_count,
           ff->func_reloc_count);
    gw_log("ftfunction: Arch_FighterFunc @ guest 0x%08X", ff->mexdata_base);
    for (i = 0; i < ff->override_count; ++i) {
        const gw_ftfunction_override *o = &ff->overrides[i];
        if (o->is_func_addr) {
            gw_log("ftfunction:   hook guest 0x%08X -> 0x%08X (native hook, phase 3)", o->slot,
                   o->target);
        } else {
            gw_log("ftfunction:   slot %02u (%-40s) -> 0x%08X", o->slot,
                   gw_ftfunction_slot_name(o->slot), o->target);
        }
    }
}

/* ---- tests --------------------------------------------------------------------------- */

#define GW_FTFUNC_TEST_STACK 0x80400000u
#define GW_FTFUNC_TEST_THUNK 0x80301000u

static int test_ftfunction_load_plsn(void) {
    gw_ftfunction ff;
    uint32_t word;
    int rc;

    if (gw_iso_path() == NULL) {
        gw_log("ftfunction: no disc image - skipping");
        return 0;
    }
    rc = gw_ftfunction_load("PlSn.dat", 31 /* Sonic internal id */, &ff);
    if (rc == GW_FTFUNC_ERR_NO_FILE) {
        gw_log("ftfunction: PlSn.dat not on this disc (vanilla ISO) - skipping");
        return 0;
    }
    if (rc != GW_FTFUNC_OK) {
        gw_test_fail("gw_ftfunction_load(PlSn.dat) returned %d", rc);
        return 1;
    }

    if (ff.code_size != 0x5778u) {
        gw_test_fail("code_size 0x%X, expected 0x5778", ff.code_size);
        return 1;
    }
    if (ff.instr_reloc_count != 944u) {
        gw_test_fail("instruction reloc count %u, expected 944", ff.instr_reloc_count);
        return 1;
    }
    if (ff.func_reloc_count != 25u) {
        gw_test_fail("function reloc count %u, expected 25", ff.func_reloc_count);
        return 1;
    }
    if (ff.override_count != 25) {
        gw_test_fail("override count %d, expected 25", ff.override_count);
        return 1;
    }

    /* The prologue is not relocated, so it must be byte-identical to the source. */
    word = gw_r32((const void *)(uintptr_t)ff.code_base);
    if (word != 0x9421FFF0u) {
        gw_test_fail("relocated entry word 0x%08X, expected stwu r1,-0x10(r1) 0x9421FFF0", word);
        return 1;
    }

    /* The first relocation is a branch to 0x800031F4 at code offset 0x2C; decode it back. */
    {
        uint32_t insn = gw_r32((const void *)(uintptr_t)(ff.code_base + 0x2Cu));
        int32_t li = ((int32_t)(insn << 6)) >> 8;
        uint32_t target = (ff.code_base + 0x2Cu) + (uint32_t)(li << 2);
        if ((insn >> 26) != 18 || target != 0x800031F4u) {
            gw_test_fail("relocated branch @+0x2C = 0x%08X -> 0x%08X, expected bl 0x800031F4", insn,
                         target);
            return 1;
        }
    }

    /* Spot-check resolved overrides (slot, code offset) against the real disc table. */
    {
        static const struct {
            uint32_t slot;
            uint32_t off;
        } expect[] = {
            {0x0, 0x0},   {0x1, 0x124}, {0x2, 0x194}, {0xB, 0x694}, {0x24, 0xB64},
        };
        unsigned i;
        for (i = 0; i < sizeof expect / sizeof expect[0]; ++i) {
            int j;
            int found = 0;
            for (j = 0; j < ff.override_count; ++j) {
                if (ff.overrides[j].slot != expect[i].slot) {
                    continue;
                }
                found = 1;
                if (ff.overrides[j].is_func_addr ||
                    ff.overrides[j].target != ff.code_base + expect[i].off) {
                    gw_test_fail("override slot %u resolved to 0x%08X, expected 0x%08X",
                                 expect[i].slot, ff.overrides[j].target,
                                 ff.code_base + expect[i].off);
                    return 1;
                }
                break;
            }
            if (!found) {
                gw_test_fail("override slot %u missing", expect[i].slot);
                return 1;
            }
        }
        for (i = 0; i < (unsigned)ff.override_count; ++i) {
            if (ff.overrides[i].is_func_addr) {
                gw_test_fail("unexpected func-addr override in PlSn.dat");
                return 1;
            }
        }
    }

    /* Prove the phase-1 interpreter executes code the loader placed: a 3-instruction thunk reads
     * the relocated entry word back through guest memory and returns it. */
    {
        static const uint32_t thunk[] = {
            0x3C60802Fu, /* lis r3, 0x802F (code base high) */
            0x80630000u, /* lwz r3, 0(r3) */
            0x4E800020u, /* blr */
        };
        uint32_t r3;
        unsigned i;
        for (i = 0; i < sizeof thunk / sizeof thunk[0]; ++i) {
            gw_w32((void *)(uintptr_t)(GW_FTFUNC_TEST_THUNK + 4 * i), thunk[i]);
        }
        gw_ppc_set_bridge(NULL, NULL, GW_FTFUNC_TEST_THUNK,
                          GW_FTFUNC_TEST_THUNK + (uint32_t)sizeof thunk);
        r3 = gw_ppc_call(GW_FTFUNC_TEST_THUNK, NULL, 0, 0, GW_FTFUNC_TEST_STACK);
        if (r3 != 0x9421FFF0u) {
            gw_test_fail("thunk returned 0x%08X, expected relocated entry word 0x9421FFF0", r3);
            return 1;
        }
    }

    return 0;
}

/* Build a minimal, valid in-memory HSD archive carrying one ftFunction public symbol: a 1-word
 * code (blr), no instruction relocs, and one table-index function reloc {slot 0, off 0}. Returns
 * a malloc'd buffer (caller frees) and its size in *out_size. */
static unsigned char *gw_ftfunction_build_archive(size_t *out_size) {
    /* data section: struct(0x18) + code(0x4) + func-reloc entry(0x8) = 0x24 */
    const uint32_t data_size = 0x24;
    const uint32_t nb_public = 1;
    const char symbols[] = "ftFunction";
    const size_t symbols_len = sizeof symbols; /* includes NUL */
    const size_t file_size = GW_HSD_HEADER_SIZE + data_size + nb_public * 8u + symbols_len;
    unsigned char *buf = (unsigned char *)calloc(1, file_size);

    gw_w32(buf + 0x00, (uint32_t)file_size);
    gw_w32(buf + 0x04, data_size);
    gw_w32(buf + 0x08, 0);       /* nb_reloc */
    gw_w32(buf + 0x0C, nb_public);
    gw_w32(buf + 0x10, 0);       /* nb_extern */
    buf[0x14] = '0';
    buf[0x15] = '0';
    buf[0x16] = '1';
    buf[0x17] = 'B';             /* version */
    gw_w32(buf + 0x18, 0);
    gw_w32(buf + 0x1C, 0);       /* pad */

    /* ftFunction struct at data offset 0x00 */
    gw_w32(buf + 0x20 + 0x00, 0x18); /* code */
    gw_w32(buf + 0x20 + 0x04, 0x1C); /* instructionRelocTable (unused, count 0) */
    gw_w32(buf + 0x20 + 0x08, 0);    /* instructionRelocTableCount */
    gw_w32(buf + 0x20 + 0x0C, 0x1C); /* functionRelocTable */
    gw_w32(buf + 0x20 + 0x10, 1);    /* functionRelocTableCount */
    gw_w32(buf + 0x20 + 0x14, 4);    /* codeSize */

    /* code at data offset 0x18: blr */
    gw_w32(buf + 0x20 + 0x18, 0x4E800020u);
    /* function reloc entry at data offset 0x1C: {ReplaceThis=0, ReplaceWith=0} */
    gw_w32(buf + 0x20 + 0x1C, 0);
    gw_w32(buf + 0x20 + 0x20, 0);

    /* public_info {offset=0, symbol=0}, then the string table */
    gw_w32(buf + 0x20 + data_size + 0, 0);
    gw_w32(buf + 0x20 + data_size + 4, 0);
    memcpy(buf + 0x20 + data_size + 8, symbols, symbols_len);

    *out_size = file_size;
    return buf;
}

static int test_ftfunction_corrupt_rejected(void) {
    /* Each case corrupts one field of a fresh archive and must fail cleanly with a distinct code,
     * never a native fault. */
    struct {
        size_t off;
        uint32_t value;
        int expect;
        const char *what;
    } cases[] = {
        {0x00, 0xDEADBEEFu, GW_FTFUNC_ERR_BAD_ARCHIVE, "file_size mismatch"},
        {0x34, 0xFFFFFFFu, GW_FTFUNC_ERR_BAD_STRUCT, "codeSize overruns data"},
        {0x28, 99999u, GW_FTFUNC_ERR_BAD_STRUCT, "instructionRelocTableCount overruns"},
        {0x2C, 0xFFFFFFFu, GW_FTFUNC_ERR_BAD_STRUCT, "functionRelocTable overruns"},
        {0x3C, 46u, GW_FTFUNC_ERR_BAD_SLOT, "slot out of range"},
    };
    unsigned i;
    gw_ftfunction ff;

    /* The valid archive must load first. */
    {
        size_t size;
        unsigned char *buf = gw_ftfunction_build_archive(&size);
        int rc = gw_ftfunction_load_from_memory_at(buf, size, 0, GW_FTFUNC_CODE_BASE, GW_FTFUNC_MEXDATA_BASE, &ff);
        free(buf);
        if (rc != GW_FTFUNC_OK) {
            gw_test_fail("valid archive rejected with %d", rc);
            return 1;
        }
    }

    for (i = 0; i < sizeof cases / sizeof cases[0]; ++i) {
        size_t size;
        unsigned char *buf = gw_ftfunction_build_archive(&size);
        int rc;
        gw_w32(buf + cases[i].off, cases[i].value);
        rc = gw_ftfunction_load_from_memory_at(buf, size, 0, GW_FTFUNC_CODE_BASE, GW_FTFUNC_MEXDATA_BASE, &ff);
        free(buf);
        if (rc == GW_FTFUNC_OK) {
            gw_test_fail("corrupt (%s) accepted", cases[i].what);
            return 1;
        }
        if (rc != cases[i].expect) {
            gw_test_fail("corrupt (%s) -> %d, expected %d", cases[i].what, rc, cases[i].expect);
            return 1;
        }
    }
    return 0;
}

static int test_ftfunction_funcaddr_hook(void) {
    size_t size;
    unsigned char *buf = gw_ftfunction_build_archive(&size);
    gw_ftfunction ff;
    int rc;

    /* Rewrite the single function-reloc entry to the func-address case: ReplaceThis = an absolute
     * guest address (0x80068B40, the m-ex init site) with the top bit set. */
    gw_w32(buf + 0x3C, 0x80068B40u);
    gw_w32(buf + 0x40, 0x0u);
    rc = gw_ftfunction_load_from_memory_at(buf, size, 0, GW_FTFUNC_CODE_BASE, GW_FTFUNC_MEXDATA_BASE, &ff);
    free(buf);
    if (rc != GW_FTFUNC_OK) {
        gw_test_fail("func-addr archive rejected with %d", rc);
        return 1;
    }
    if (ff.override_count != 1 || !ff.overrides[0].is_func_addr ||
        ff.overrides[0].slot != 0x80068B40u || ff.overrides[0].target != GW_FTFUNC_CODE_BASE) {
        gw_test_fail("func-addr override not recorded (hook site, target)");
        return 1;
    }
    /* The hook site must NOT have been written (a `b` opcode has top bits 0x12). */
    if ((gw_r32((const void *)(uintptr_t)0x80068B40u) >> 26) == 18u) {
        gw_test_fail("func-addr case wrote a branch into the hook site");
        return 1;
    }
    return 0;
}

void gw_ftfunction_tests_register(void) {
    gw_test_register("ftfunction_load_plsn", test_ftfunction_load_plsn);
    gw_test_register("ftfunction_corrupt_rejected", test_ftfunction_corrupt_rejected);
    gw_test_register("ftfunction_funcaddr_hook", test_ftfunction_funcaddr_hook);
}
