/* gw_mex_sss.h - m-ex STAGE SELECT SCREEN expansion: the icon table in mexData.
 *
 * m-ex's `Arch_Menu_SSS` (mexData +0x04 -> menu, menu +0x08) is the stage-select icon table, and
 * it maps one-to-one onto the port's own `mnStageSel_803F06D0[]` (melee/mn/mnstagesel.static.h)
 * with exactly one difference, verified against both discs:
 *
 *      | field              | port (vanilla) | m-ex                |
 *      |--------------------|----------------|---------------------|
 *      | stride             | 0x1C           | 0x20                |
 *      | external stage id  | u8 at +0x0B    | s32 at +0x1C        |
 *
 * 313 (Akaneia) and 372 (ACE) external stage ids do not fit in a byte, so m-ex widened the field
 * and moved it to its own word; word 2's byte 3 - where the port keeps `stkind` - is zero in
 * every m-ex row, which is the confirming detail. Everything else lines up: words 0..1 runtime
 * (HSD_JObj* and the random-picker cooldown), x8/x9/xA packed into word 2, four floats in words
 * 3..6 (cursor half-width/height, cursor scale x/y).
 *
 * So the port does NOT need a second layout: under TARGET_PC its own row struct is widened to
 * m-ex's 0x20 and the table is copied straight out of mexData by gwtool-compiled game code,
 * which byte-swaps every read for free. This file therefore exposes only the count and the
 * table's guest address; it deliberately caches nothing, so it cannot hold a stale pointer
 * across the test harness's MEM1 restore.
 *
 * NOT vendored from m-ex: this is original C written from the published behaviour of
 * SSS Expansion/MnSlMap References/ and SSS Expansion/mexMapData/Load mexMapData.asm.
 *
 * The icon MODELS come from the MnSlMap archive's own `mexMapData` public symbol, which game
 * code looks up with HSD_ArchiveGetPublicAddress - no shim is involved in that.
 */
#ifndef GW_MEX_SSS_H
#define GW_MEX_SSS_H

#ifdef __cplusplus
extern "C" {
#endif

/* Upper bound the port compiles its table for. Akaneia ships 67 icons, ACE 163; a build past
 * this is rejected (logged) rather than trusted. */
#define GW_MEX_SSS_MAX 256

/* metadata.sss_icon_count, or 0 when this disc has no usable m-ex SSS table. Zero is also
 * returned when the m-ex STAGE tables are off (gw_Mex_GrExternalCount() == 0), because an icon
 * whose external id the stage tables cannot resolve would load nothing. */
int gw_Mex_SssIconCount(void);

/* Guest address of the icon table (0x20 per row), or NULL. The rows are big-endian game memory:
 * only gwtool-compiled game code may dereference this. */
void *gw_Mex_SssTable(void);

/* Registers this module's self-contained tests with the in-engine suite. */
void gw_mex_sss_tests_register(void);

#ifdef __cplusplus
}
#endif
#endif /* GW_MEX_SSS_H */
