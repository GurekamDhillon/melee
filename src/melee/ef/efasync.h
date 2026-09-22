#ifndef MELEE_EF_EFASYNC_H
#define MELEE_EF_EFASYNC_H

#include <melee/ef/forward.h>
#include <sysdolphin/baselib/forward.h>

#include <stdarg.h>
#if defined(TARGET_PC)
#include <dolphin/mtx.h>
#endif

/* 063930 */ void* efAsync_Dispatch(s32 gfx_id, HSD_GObj* gobj, va_list vlist);
/* 06729C */ void efAsync_LoadAsync(int index);
/* 06730C */ void efAsync_OnLoad(HSD_Archive* archive, u8* data, u32 length,
                                 int index);
/* 06737C */ void efAsync_LoadSync(int index);
/* 06744C */ void efAsync_QueueProcessDeferred(HSD_GObj* gobj,
                                               EF_QueuedEffect* queued_effect);
/* 067624 */ void efAsync_QueueFlush(HSD_GObj* gobj, void* arg_struct);
/* 067688 */ void efAsync_QueueClear(void* arg_struct);
/* 0676F0 */ void efAsync_Spawn(HSD_GObj* gobj, void* queue_head,
                                u32 spawn_kind, u32 gfx_id, HSD_JObj* jobj,
                                ...);
/* 0676F0 */ void efAsync_QueueInit(void);

#if defined(TARGET_PC)
/* Ported from m-ex (https://github.com/akaneia/m-ex).
 * Source patches: asm/m-ex/Effect Expansion/{SyncEffect,AsyncEffect,AsyncToSync,Item/AsyncEffect}.asm,
 * asm/m-ex/Standalone Functions/Effect_CreateAsyncObject.asm,
 * asm/m-ex/MnSlChrData - Effect File Names/ (incl. Index effBehaviorTable).
 * Behaviour: effect ids 5000..8999 are a fighter's OWN effects, relative to its effect bank. */
/* The effect bank table grows from retail's 51 rows to the particle system's 65 banks
 * (psInitDataBank's arrays), filled from MxDt.dat's effect table on an m-ex disc. */
#define EF_BANK_MAX 65
/* m-ex custom effect ids: [5000,6000) model of this fighter's bank, [6000,7000) particle
 * generator, [7000,8000) / [8000,9000) the same for the fighter Kirby has copied. */
#define EF_MEX_MDL_START 5000
#define EF_MEX_PTCL_START 6000
#define EF_MEX_CPMDL_START 7000
#define EF_MEX_CPPTCL_START 8000
#define EF_MEX_END 9000
#define EF_MEX_IS_CUSTOM(id) ((id) >= EF_MEX_MDL_START && (id) < EF_MEX_END)
/* EF_QueuedEffect.spawn_kind for an m-ex effect: params = offset from the bone, extra1 =
 * facing direction, extra2 = ground orientation (Effect_CreateAsyncObject's layout). */
#define EF_SPAWN_MEX 9
struct EF_DAT_Entry;
void efAsync_MexInit(void);
/* Resolve a custom id spawned by `gobj` (a fighter, or an item with an original owner): the final
 * gfx id (bank * 1000 + index), whether it is a particle generator, the owning fighter's gobj,
 * and the m-ex behaviour type. Returns the type (>= 0), or -1 when it cannot be spawned (logged
 * once per fighter kind and id). */
int efAsync_MexResolve(HSD_GObj* gobj, s32 gfx_id, s32* final_id, int* is_ptcl,
                       HSD_GObj** owner);
/* The fighter-side spawn of a custom id from a subaction or item script: queue it (or spawn it
 * now) like efAsync_Spawn does, with the bone, the offset, facing direction and orientation. */
void efAsync_MexSpawn(HSD_GObj* gobj, void* queue_head, s32 gfx_id, HSD_JObj* jobj,
                      Vec3* offset, f32 facing, f32 orientation);
#endif

#endif
