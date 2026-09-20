/* gw_mex_graudio.h - m-ex custom STAGES: the per-stage audio and line-type tables.
 *
 * Three vanilla structures are indexed straight by `stage_info.grkind` / `Stage_8022519C()` and
 * are sized for the 71 vanilla stages only. An m-ex added stage is internal id >= 71 (up to 95 on
 * Akaneia, 154 on ACE) and falls outside every one of them. This file supplies the real data for
 * those rows, read from `mexData`'s `Arch_Map`:
 *
 *   Arch_Map +0x04  Audio         stride 3, INTERNAL-indexed, {u8 ssm_id; u8 echo; u8 echo2}
 *   Arch_Map +0x08  LineTypeData  stride 8, INTERNAL-indexed, {s32 index; void* rows}
 *
 * DUMPED, NOT ASSUMED (Akaneia.iso and SSBM ACE Build v2.0.0.iso, scratch dumper over
 * tools/mex_port/mex_hsd.py):
 *
 *  - `Audio[0..70]` is BYTE-IDENTICAL to the port's compiled `s32_arr_803BB6B0[0..70]` on BOTH
 *    discs - 0 mismatching rows - which is what pins the stride and the index space. Rows 71.. are
 *    the added stages. SSM ids there run 55..77 on Akaneia (9 of 25 above 63) and 38..100 on ACE
 *    (11 of 84 above 63); id 55 is m-ex's empty null.ssm, the same "no bank" value retail uses.
 *
 *  - `LineTypeData[i].index == i` for every row on both discs, and `LineTypeData[i].rows` is an
 *    absolute VANILLA guest address in 0x803BDC18..0x803BF1F8 - the port's own
 *    `mpLib_803BF248[0..70].x4` arrays. It is not relocated and no added stage introduces a new
 *    one: all 155 ACE rows draw on exactly the same 71 distinct pointers as the 71 vanilla rows.
 *    So an added stage's line-type data is always some vanilla stage's, and the port can express
 *    that as a ROW INDEX rather than a pointer - which is why gw_Mex_GrLineTypeRow returns an
 *    index into the table game code already has, instead of an address.
 *
 * Everything here RETURNS values. Nothing writes through a pointer into game memory: gwtool-
 * compiled game code reads its locals big-endian while a native shim writes host order, so an
 * out-parameter needs a byte swap and a return value does not (this exact mistake sent `*bgm = 98`
 * into game code as 0x62000000 and faulted inside the music player).
 *
 * This is NATIVE platform code (compiled directly with the i686 clang, not through gwtool).
 */
#ifndef GW_MEX_GRAUDIO_H
#define GW_MEX_GRAUDIO_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* mexData's internal_stage_count, or 0 on a disc with no mexData (so every accessor below is
 * safe to call unconditionally). */
int gw_Mex_GrAudioCount(void);

/* Byte `which` (0 = ssm bank id, 1 = echo, 2 = echo2) of Arch_Map_Audio[grkind].
 * Returns -1 when there is no mexData, the index is out of range, or the byte is not plausible -
 * never a half-validated number, because a bad bank id faults inside the mixer rather than here. */
int gw_Mex_GrAudioByte(int grkind, int which);

/* The VANILLA internal stage id (0..70) whose line-type rows this stage shares, or -1.
 * For a vanilla grkind this is grkind itself. For an m-ex added stage it is the row its
 * Arch_Map_LineTypeData pointer names - see the header comment for why that is always a vanilla
 * row. Callers use it to index a table they already hold; nothing here hands out an address. */
int gw_Mex_GrLineTypeRow(int grkind);

/* Registered from gw_tests_core.c's gw_tests_register_all(). */
void gw_mex_graudio_tests_register(void);

#ifdef __cplusplus
}
#endif
#endif /* GW_MEX_GRAUDIO_H */
